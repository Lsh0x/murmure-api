//! Toggle Recording Client for Murmure gRPC Server
//!
//! This client uses toggle mode: press SPACE to start recording,
//! press SPACE again to stop and transcribe. Simple and intuitive.
//!
//! ## Usage
//!
//! First, ensure the server is running:
//! ```bash
//! cd ../src-tauri
//! cargo run --bin murmure-server
//! ```
//!
//! Then run this client:
//! ```bash
//! cd examples
//! cargo run --example rust_streaming_client
//! ```
//!
//! Controls:
//! - Press SPACE to start/stop recording (toggle)
//! - Press Ctrl+C to exit
//!
//! Options:
//! - `--server <address>` - Server address (default: http://localhost:50051)

use std::fs::File;
use std::io::{self, BufWriter, Cursor, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SupportedStreamConfig};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use hound::{WavReader, WavSpec, WavWriter};
use murmure_core::tts::{SynthesisService, TtsConfig, TtsModel};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

// Include generated proto code from build script
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::{TranscribeStreamRequest, TranscribeStreamResponse};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type SendResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ============================================================================
// Configuration and State
// ============================================================================

struct AudioConfig {
    device: cpal::Device,
    config: SupportedStreamConfig,
}

struct RecordingState {
    is_recording: bool,
    count: usize,
    stop_flag: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<SendResult<Vec<u8>>>>,
}

impl RecordingState {
    fn new() -> Self {
        Self {
            is_recording: false,
            count: 0,
            stop_flag: None,
            handle: None,
        }
    }

    fn start(&mut self, device: &cpal::Device, config: &SupportedStreamConfig) {
        self.count += 1;
        self.is_recording = true;

        let stop_flag = Arc::new(AtomicBool::new(false));
        self.stop_flag = Some(stop_flag.clone());

        let device_clone = device.clone();
        let config_clone = config.clone();

        self.handle = Some(tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                record_audio(&device_clone, &config_clone, stop_flag)
            })
            .await
            .map_err(|e| {
                Box::new(io::Error::other(format!("Task error: {}", e)))
                    as Box<dyn std::error::Error + Send + Sync>
            })?
        }));
    }

    async fn stop(&mut self) -> Option<SendResult<Vec<u8>>> {
        self.is_recording = false;

        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, Ordering::Relaxed);
        }

        if let Some(handle) = self.handle.take() {
            let result = match handle.await {
                Ok(inner_result) => inner_result,
                Err(e) => Err(Box::new(io::Error::other(format!("Join error: {}", e)))
                    as Box<dyn std::error::Error + Send + Sync>),
            };
            Some(result)
        } else {
            None
        }
    }
}

// ============================================================================
// Main Application
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let server_address = parse_server_address();
    print_welcome(&server_address);

    let audio_config = setup_audio()?;
    let mut client = connect_to_server(&server_address).await?;

    print_instructions();

    enable_raw_mode()?;
    let shutdown_flag = setup_shutdown_handler();

    let result = run_recording_loop(&mut client, &audio_config, shutdown_flag).await;

    disable_raw_mode()?;
    result
}

async fn run_recording_loop(
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    audio_config: &AudioConfig,
    shutdown_flag: Arc<AtomicBool>,
) -> Result<()> {
    let mut conversation_text = String::new();
    let mut recording_state = RecordingState::new();
    
    // Initialize TTS service (optional, will fail gracefully if not available)
    let tts_service = init_tts_service().ok();

    loop {
        if shutdown_flag.load(Ordering::Relaxed) {
            handle_shutdown(&mut recording_state, &conversation_text).await?;
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if should_exit(&key_event) {
                    handle_shutdown(&mut recording_state, &conversation_text).await?;
                    break;
                }

                match key_event.code {
                    KeyCode::Char(' ') if key_event.kind == KeyEventKind::Press => {
                        handle_space_press(
                            &mut recording_state,
                            audio_config,
                            client,
                            &mut conversation_text,
                            &tts_service,
                        )
                        .await?;
                    }
                    KeyCode::Esc => {
                        handle_shutdown(&mut recording_state, &conversation_text).await?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn handle_space_press(
    state: &mut RecordingState,
    audio_config: &AudioConfig,
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    conversation_text: &mut String,
    tts_service: &Option<Arc<SynthesisService>>,
) -> Result<()> {
    disable_raw_mode()?;

    if !state.is_recording {
        start_recording(state, audio_config)?;
    } else {
        stop_and_transcribe(state, client, conversation_text, tts_service).await?;
    }

    enable_raw_mode()?;
    Ok(())
}

fn start_recording(state: &mut RecordingState, audio_config: &AudioConfig) -> Result<()> {
    println!(
        "\n🎙️  Recording #{} started (press SPACE again to stop)...",
        state.count + 1
    );
    io::stdout().flush()?;
    state.start(&audio_config.device, &audio_config.config);
    Ok(())
}

async fn stop_and_transcribe(
    state: &mut RecordingState,
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    conversation_text: &mut String,
    tts_service: &Option<Arc<SynthesisService>>,
) -> Result<()> {
    println!("\n   ⏹️  Stopping recording...");
    io::stdout().flush()?;

    let audio_result = state.stop().await;

    let audio_data = match audio_result {
        Some(Ok(data)) => data,
        Some(Err(e)) => {
            eprintln!("\n❌ Recording error: {}", e);
            return Ok(());
        }
        None => {
            eprintln!("\n❌ No recording handle found");
            return Ok(());
        }
    };

    if audio_data.is_empty() {
        println!("⚠️  No audio recorded (too short or silent)\n");
        return Ok(());
    }

    print!("   📤 Sending to server for transcription...");
    io::stdout().flush()?;

    match transcribe_audio(client, audio_data).await {
        Ok(text) if !text.trim().is_empty() => {
            println!("\r   ✅ Transcription #{}: {}", state.count, text);
            conversation_text.push_str(&text);
            conversation_text.push(' ');
            
            // Synthesize and play using TTS
            if let Some(service) = tts_service {
                print!("   🔊 Synthesizing and playing...");
                io::stdout().flush()?;
                if let Err(e) = synthesize_and_play(service, &text).await {
                    println!("\r   ⚠️  TTS error: {} (continuing anyway)", e);
                } else {
                    println!("\r   ✅ TTS playback complete");
                }
            }
            println!();
        }
        Ok(_) => {
            println!("\r   ⚠️  Empty transcription\n");
        }
        Err(e) => {
            println!("\r   ❌ Transcription error: {}\n", e);
        }
    }

    io::stdout().flush()?;
    Ok(())
}

async fn handle_shutdown(state: &mut RecordingState, conversation_text: &str) -> Result<()> {
    if state.is_recording {
        println!("\n🛑 Stopping recording...");
        state.stop().await;
    }

    println!("\n📝 Conversation transcript:\n{}", conversation_text);
    Ok(())
}

// ============================================================================
// Setup and Initialization
// ============================================================================

fn parse_server_address() -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "http://localhost:50051".to_string())
}

fn print_welcome(server_address: &str) {
    println!("🎙️  Murmure Toggle Recording Client");
    println!("Server: {}\n", server_address);
}

fn print_instructions() {
    println!("🎤 Toggle Recording Mode");
    println!("   Press SPACE to start recording");
    println!("   Press SPACE again to stop and transcribe");
    println!("   Press Ctrl+C to exit\n");
}

fn setup_audio() -> Result<AudioConfig> {
    let host = cpal::default_host();

    let input_devices: Vec<_> = host.input_devices()?.collect();
    if input_devices.is_empty() {
        return Err("❌ No input devices found. Please check microphone permissions.".into());
    }

    let device = host
        .default_input_device()
        .ok_or("❌ No default input device available. Check microphone permissions.")?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    println!("📱 Device: {}", device_name);

    let config = device.default_input_config().map_err(|e| {
        format!(
            "❌ Failed to get input config: {}\n   Check microphone permissions.",
            e
        )
    })?;

    println!("   Sample rate: {} Hz", config.sample_rate().0);
    println!("   Channels: {}\n", config.channels());

    Ok(AudioConfig { device, config })
}

async fn connect_to_server(
    address: &str,
) -> Result<TranscriptionServiceClient<tonic::transport::Channel>> {
    println!("📡 Connecting to server...");
    let client = TranscriptionServiceClient::connect(address.to_string()).await?;
    println!("✅ Connected to server\n");
    Ok(client)
}

fn setup_shutdown_handler() -> Arc<AtomicBool> {
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = shutdown_flag.clone();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for Ctrl+C: {}", e);
            return;
        }
        flag_clone.store(true, Ordering::Relaxed);
    });

    shutdown_flag
}

fn should_exit(key_event: &crossterm::event::KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c'))
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
}

// ============================================================================
// Audio Recording
// ============================================================================

fn record_audio(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    stop_flag: Arc<AtomicBool>,
) -> SendResult<Vec<u8>> {
    let temp_file = create_temp_wav_file()?;
    let spec = create_wav_spec(config);

    let writer = WavWriter::new(BufWriter::new(File::create(&temp_file)?), spec)?;
    let writer_arc = Arc::new(Mutex::new(writer));

    let stream = create_audio_stream(device, config, writer_arc.clone())?;
    stream
        .play()
        .map_err(|e| format!("❌ Failed to start recording: {}", e))?;

    wait_for_stop_signal(&stop_flag);
    drop(stream);
    std::thread::sleep(Duration::from_millis(200));

    finalize_wav_file(writer_arc)?;

    let audio_data = std::fs::read(&temp_file)?;
    let _ = std::fs::remove_file(&temp_file);

    Ok(audio_data)
}

fn create_temp_wav_file() -> SendResult<std::path::PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    Ok(std::env::temp_dir().join(format!(
        "murmure-record-{}-{}.wav",
        std::process::id(),
        timestamp
    )))
}

fn create_wav_spec(config: &SupportedStreamConfig) -> WavSpec {
    WavSpec {
        channels: 1,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }
}

fn create_audio_stream(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
) -> SendResult<cpal::Stream> {
    match config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(device, config, writer),
        SampleFormat::I16 => build_stream::<i16>(device, config, writer),
        SampleFormat::I32 => build_stream::<i32>(device, config, writer),
        _ => Err("Unsupported sample format".into()),
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
) -> SendResult<cpal::Stream>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels() as usize;

    let stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            process_audio_data(data, channels, &writer);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn process_audio_data<T>(
    data: &[T],
    channels: usize,
    writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>,
) where
    T: cpal::Sample,
    f32: cpal::FromSample<T>,
{
    let mut writer = writer.lock().unwrap();

    for frame in data.chunks_exact(channels) {
        let sample = if channels == 1 {
            frame[0].to_sample::<f32>()
        } else {
            frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
        };

        let sample_i16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        let _ = writer.write_sample(sample_i16);
    }
}

fn wait_for_stop_signal(stop_flag: &Arc<AtomicBool>) {
    while !stop_flag.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn finalize_wav_file(writer_arc: Arc<Mutex<WavWriter<BufWriter<File>>>>) -> SendResult<()> {
    {
        let mut writer = writer_arc.lock().unwrap();
        writer.flush()?;
    }

    let writer = Arc::try_unwrap(writer_arc).map_err(|_| "Failed to unwrap Arc")?;

    writer.into_inner().unwrap().finalize()?;
    Ok(())
}

// ============================================================================
// Transcription
// ============================================================================

async fn transcribe_audio(
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    audio_data: Vec<u8>,
) -> Result<String> {
    let request_stream = create_transcription_stream(audio_data);
    let mut response_stream = client
        .transcribe_stream(Request::new(request_stream))
        .await?
        .into_inner();

    process_transcription_responses(&mut response_stream).await
}

fn create_transcription_stream(audio_data: Vec<u8>) -> ReceiverStream<TranscribeStreamRequest> {
    let (chunk_tx, chunk_rx) = mpsc::channel(128);

    tokio::spawn(async move {
        send_audio_chunks(&chunk_tx, audio_data).await;
        send_end_of_stream(&chunk_tx).await;
    });

    ReceiverStream::new(chunk_rx)
}

async fn send_audio_chunks(tx: &mpsc::Sender<TranscribeStreamRequest>, audio_data: Vec<u8>) {
    const CHUNK_SIZE: usize = 16384; // 16KB chunks

    for chunk in audio_data.chunks(CHUNK_SIZE) {
        let request = TranscribeStreamRequest {
            request_type: Some(murmure::transcribe_stream_request::RequestType::AudioChunk(
                chunk.to_vec(),
            )),
        };

        if tx.send(request).await.is_err() {
            return;
        }
    }
}

async fn send_end_of_stream(tx: &mpsc::Sender<TranscribeStreamRequest>) {
    let _ = tx
        .send(TranscribeStreamRequest {
            request_type: Some(murmure::transcribe_stream_request::RequestType::EndOfStream(true)),
        })
        .await;
}

async fn process_transcription_responses(
    stream: &mut tonic::Streaming<TranscribeStreamResponse>,
) -> Result<String> {
    let mut final_text = String::new();

    while let Some(result) = stream.message().await.transpose() {
        let response = result?;

        match response.response_type {
            Some(murmure::transcribe_stream_response::ResponseType::FinalText(text)) => {
                final_text = text;
            }
            Some(murmure::transcribe_stream_response::ResponseType::Error(err)) => {
                return Err(format!("Server error: {}", err).into());
            }
            _ => {}
        }

        if response.is_final {
            break;
        }
    }

    Ok(final_text)
}

// ============================================================================
// TTS (Text-To-Speech)
// ============================================================================

fn init_tts_service() -> Result<Arc<SynthesisService>> {
    let tts_config = TtsConfig::from_env().unwrap_or_default();
    let tts_model = Arc::new(TtsModel::new(tts_config.clone()));
    let service = SynthesisService::new(tts_model, Arc::new(tts_config))
        .map_err(|e| format!("Failed to initialize TTS: {}", e))?;
    Ok(Arc::new(service))
}

async fn synthesize_and_play(
    tts_service: &SynthesisService,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Synthesize text to audio
    let wav_bytes = tts_service
        .synthesize_text(text)
        .map_err(|e| format!("Synthesis failed: {}", e))?;

    // Play the audio
    play_wav_bytes(&wav_bytes)?;

    Ok(())
}

fn play_wav_bytes(wav_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Read WAV file from bytes
    let cursor = Cursor::new(wav_bytes);
    let mut reader = WavReader::new(cursor)?;
    let spec = reader.spec();

    // Convert samples to f32
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| {
            s.map(|sample| sample as f32 / i16::MAX as f32)
                .map_err(|e| format!("Failed to read WAV sample: {}", e))
        })
        .collect::<Result<Vec<f32>, _>>()?;

    if samples.is_empty() {
        return Err("No audio samples to play".into());
    }

    // Get default output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No default output device available")?;

    // Create output config matching WAV file
    let config = cpal::StreamConfig {
        channels: spec.channels as u16,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Use a channel to feed samples to the stream
    let (tx, rx) = std::sync::mpsc::channel();
    let samples_len = samples.len();
    
    // Send samples in chunks
    std::thread::spawn(move || {
        for chunk in samples.chunks(1024) {
            let chunk_vec = chunk.to_vec();
            if tx.send(chunk_vec).is_err() {
                break;
            }
        }
    });

    // Create output stream
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Try to get samples from channel, otherwise fill with zeros
            if let Ok(chunk) = rx.try_recv() {
                let len = data.len().min(chunk.len());
                data[..len].copy_from_slice(&chunk[..len]);
                if len < data.len() {
                    data[len..].fill(0.0);
                }
            } else {
                data.fill(0.0);
            }
        },
        |err| eprintln!("Playback error: {}", err),
        None,
    )?;

    stream.play()?;

    // Wait for playback to complete
    let duration = samples_len as f64 / spec.sample_rate as f64;
    std::thread::sleep(Duration::from_secs_f64(duration + 0.1));

    Ok(())
}
