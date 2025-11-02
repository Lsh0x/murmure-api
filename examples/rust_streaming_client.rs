//! Push-to-Talk Streaming Client for Murmure gRPC Server
//!
//! This client uses push-to-talk mode: press and hold SPACE to record,
//! release to stop and transcribe. Perfect for precise control over recording.
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
//! - Hold SPACE to record audio
//! - Release SPACE to stop recording and transcribe
//! - Press Ctrl+C to exit
//!
//! Options:
//! - `--server <address>` - Server address (default: http://localhost:50051)
//!
//! See README_STREAMING_CLIENT.md for detailed documentation.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::io::BufWriter;
use std::fs::File;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

// Include generated proto code from build script
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::TranscribeStreamRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let server_address = args
        .iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&"http://localhost:50051".to_string())
        .clone();

    println!("🎙️  Murmure Push-to-Talk Streaming Client");
    println!("Server: {}\n", server_address);

    // Set up audio recording
    let host = cpal::default_host();
    let input_devices: Vec<_> = host.input_devices()?.collect();
    if input_devices.is_empty() {
        eprintln!("❌ No input devices found. Please check microphone permissions.");
        std::process::exit(1);
    }

    let device = host
        .default_input_device()
        .ok_or("❌ No default input device available. Check microphone permissions.")?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    println!("📱 Device: {}", device_name);

    let config = match device.default_input_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Failed to get input config: {}\n   Check microphone permissions.", e);
            std::process::exit(1);
        }
    };

    println!("   Sample rate: {} Hz", config.sample_rate().0);
    println!("   Channels: {}\n", config.channels());

    // Connect to server
    println!("📡 Connecting to server...");
    let mut client = TranscriptionServiceClient::connect(server_address.clone()).await?;
    println!("✅ Connected to server\n");

    println!("🎤 Push-to-Talk Mode");
    println!("   Hold SPACE to record, release to transcribe");
    println!("   Press Ctrl+C to exit\n");

    // Enable raw mode for key detection
    enable_raw_mode()?;

    // Track conversation text
    let mut conversation_text = String::new();
    let mut recording_count = 0;

    // Handle Ctrl+C gracefully
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    loop {
        // Check if Ctrl+C was pressed
        if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            disable_raw_mode()?;
            println!("\n📝 Conversation transcript:\n{}", conversation_text);
            break;
        }

        // Wait for key press (non-blocking)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(' ') if key_event.kind == KeyEventKind::Press => {
                        // Space pressed - start recording
                        recording_count += 1;
                        println!("🎙️  Recording #{} (hold SPACE)...", recording_count);
                        
                        // Record until space is released
                        let audio_data = record_until_key_release(&device, &config)?;
                        
                        if audio_data.is_empty() {
                            println!("⚠️  No audio recorded (too short or silent)");
                            continue;
                        }

                        println!("   📤 Sending to server for transcription...");

                        // Transcribe the audio
                        match transcribe_audio(&mut client, audio_data).await {
                            Ok(text) => {
                                if !text.trim().is_empty() {
                                    println!("✅ Transcription: {}\n", text);
                                    conversation_text.push_str(&text);
                                    conversation_text.push(' ');
                                } else {
                                    println!("⚠️  Empty transcription\n");
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Transcription error: {}\n", e);
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Escape key - exit
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}

async fn transcribe_audio(
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    audio_data: Vec<u8>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create request stream
    let (chunk_tx, chunk_rx) = mpsc::channel(128);
    
    // Send audio in chunks
    tokio::spawn(async move {
        let chunk_size = 16384; // 16KB chunks
        for audio_chunk in audio_data.chunks(chunk_size) {
            if chunk_tx.send(TranscribeStreamRequest {
                request_type: Some(murmure::transcribe_stream_request::RequestType::AudioChunk(audio_chunk.to_vec())),
            }).await.is_err() {
                return;
            }
        }
        
        // Send end of stream
        let _ = chunk_tx.send(TranscribeStreamRequest {
            request_type: Some(murmure::transcribe_stream_request::RequestType::EndOfStream(true)),
        }).await;
    });

    // Send to server
    let request = Request::new(ReceiverStream::new(chunk_rx));
    let mut response_stream = client.transcribe_stream(request).await?.into_inner();

    // Process responses
    let mut final_text = String::new();
    while let Some(result) = response_stream.message().await.transpose() {
        match result {
            Ok(response) => {
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
            Err(e) => {
                return Err(format!("Stream error: {}", e).into());
            }
        }
    }

    Ok(final_text)
}

fn record_until_key_release(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create temporary WAV file
    let temp_file = std::env::temp_dir().join(format!(
        "murmure-ptt-{}-{}.wav",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
    ));

    let spec = WavSpec {
        channels: 1,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = File::create(&temp_file)?;
    let writer = WavWriter::new(BufWriter::new(file), spec)?;
    let writer_arc = Arc::new(std::sync::Mutex::new(writer));

    let result = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(device, config, writer_arc.clone()),
        cpal::SampleFormat::I16 => build_stream::<i16>(device, config, writer_arc.clone()),
        cpal::SampleFormat::I32 => build_stream::<i32>(device, config, writer_arc.clone()),
        _ => return Err("Unsupported sample format".into()),
    };

    let (stream, _audio_stats) = match result {
        Ok((s, stats)) => (s, stats),
        Err(e) => {
            return Err(format!(
                "❌ Failed to create audio stream: {}",
                e
            ).into());
        }
    };

    if let Err(e) = stream.play() {
        return Err(format!("❌ Failed to start recording: {}", e).into());
    }

    // Record until space key is released
    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                if let KeyCode::Char(' ') = key_event.code {
                    if key_event.kind == KeyEventKind::Release {
                        break; // Space released, stop recording
                    }
                }
            }
        }
    }

    drop(stream);

    // Finalize WAV file
    {
        let mut writer = writer_arc.lock().unwrap();
        writer.flush()?;
        drop(writer);
    }

    let writer = Arc::try_unwrap(writer_arc).map_err(|_| "Failed to unwrap Arc")?;
    writer.into_inner().unwrap().finalize()?;

    // Read audio data
    let audio_data = std::fs::read(&temp_file)?;
    let _ = std::fs::remove_file(&temp_file);

    Ok(audio_data)
}


type WavWriterType = WavWriter<BufWriter<File>>;

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    writer: Arc<std::sync::Mutex<WavWriterType>>,
) -> Result<(cpal::Stream, Arc<std::sync::Mutex<(usize, i16)>>), Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels() as usize;
    let audio_stats = Arc::new(std::sync::Mutex::new((0usize, 0i16)));
    let stats_clone = audio_stats.clone();

    let stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut writer = writer.lock().unwrap();
            let mut stats = stats_clone.lock().unwrap();
            stats.0 += data.len() / channels;

            for frame in data.chunks_exact(channels) {
                let sample = if channels == 1 {
                    frame[0].to_sample::<f32>()
                } else {
                    frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                };

                let sample_i16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let amplitude = sample_i16.abs();
                if amplitude > stats.1 {
                    stats.1 = amplitude;
                }
                let _ = writer.write_sample(sample_i16);
            }
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    Ok((stream, audio_stats))
}

