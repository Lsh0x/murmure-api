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
use std::io::{self, Write};

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

    println!("🎙️  Murmure Toggle Recording Client");
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

    println!("🎤 Toggle Recording Mode");
    println!("   Press SPACE to start recording");
    println!("   Press SPACE again to stop and transcribe");
    println!("   Press Ctrl+C to exit\n");

    // Enable raw mode for key detection
    enable_raw_mode()?;

    // Track conversation text
    let mut conversation_text = String::new();
    let mut recording_count = 0;
    let mut is_recording = false;
    let mut recording_stop_flag: Option<Arc<AtomicBool>> = None;
    let mut recording_handle: Option<tokio::task::JoinHandle<Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>>> = None;

    // Handle Ctrl+C gracefully - spawn signal handler before raw mode
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for Ctrl+C: {}", e);
            return;
        }
        shutdown_flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    loop {
        // Check if Ctrl+C was pressed
        if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            disable_raw_mode()?;
            
            // Stop any ongoing recording
            if is_recording {
                println!("\n🛑 Stopping recording...");
                if let Some(flag) = recording_stop_flag.take() {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if let Some(handle) = recording_handle.take() {
                    let _ = handle.await;
                }
            }
            
            println!("\n📝 Conversation transcript:\n{}", conversation_text);
            break;
        }

        // Wait for key press (non-blocking)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                // Check for Ctrl+C in key events (backup method)
                if let KeyCode::Char('c') = key_event.code {
                    if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        disable_raw_mode()?;
                        println!("\n\n🛑 Ctrl+C detected - Shutting down...");
                        
                        // Stop any ongoing recording
                        if is_recording {
                            if let Some(flag) = recording_stop_flag.take() {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            if let Some(handle) = recording_handle.take() {
                                let _ = handle.await;
                            }
                        }
                        
                        println!("\n📝 Conversation transcript:\n{}", conversation_text);
                        break;
                    }
                }
                
                match key_event.code {
                    KeyCode::Char(' ') if key_event.kind == KeyEventKind::Press => {
                        // Temporarily disable raw mode for cleaner output
                        disable_raw_mode()?;
                        
                        if !is_recording {
                            // Start recording
                            recording_count += 1;
                            is_recording = true;
                            println!("\n🎙️  Recording #{} started (press SPACE again to stop)...", recording_count);
                            io::stdout().flush()?;
                            
                            // Re-enable raw mode
                            enable_raw_mode()?;
                            
                            // Create stop flag
                            let stop_flag = Arc::new(AtomicBool::new(false));
                            recording_stop_flag = Some(stop_flag.clone());
                            
                            // Start recording in background
                            let device_clone = device.clone();
                            let config_clone = config.clone();
                            recording_handle = Some(tokio::spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    record_audio_continuous(&device_clone, &config_clone, stop_flag)
                                }).await.map_err(|e| Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Task error: {}", e)
                                )) as Box<dyn std::error::Error + Send + Sync>)?
                            }));
                        } else {
                            // Stop recording
                            is_recording = false;
                            println!("\n   ⏹️  Stopping recording...");
                            io::stdout().flush()?;
                            
                            // Signal stop
                            if let Some(flag) = recording_stop_flag.take() {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            
                            // Wait for recording to finish
                            if let Some(handle) = recording_handle.take() {
                                let audio_data = match handle.await {
                                    Ok(Ok(data)) => data,
                                    Ok(Err(e)) => {
                                        eprintln!("\n❌ Recording error: {}", e);
                                        enable_raw_mode()?;
                                        continue;
                                    }
                                    Err(e) => {
                                        eprintln!("\n❌ Task error: {}", e);
                                        enable_raw_mode()?;
                                        continue;
                                    }
                                };
                                
                                if audio_data.is_empty() {
                                    println!("⚠️  No audio recorded (too short or silent)\n");
                                    enable_raw_mode()?;
                                    continue;
                                }

                                print!("   📤 Sending to server for transcription...");
                                io::stdout().flush()?;

                                // Transcribe the audio
                                match transcribe_audio(&mut client, audio_data).await {
                                    Ok(text) => {
                                        if !text.trim().is_empty() {
                                            println!("\r   ✅ Transcription #{}: {}", recording_count, text);
                                            println!();
                                            conversation_text.push_str(&text);
                                            conversation_text.push(' ');
                                        } else {
                                            println!("\r   ⚠️  Empty transcription\n");
                                        }
                                        io::stdout().flush()?;
                                    }
                                    Err(e) => {
                                        println!("\r   ❌ Transcription error: {}\n", e);
                                        io::stdout().flush()?;
                                    }
                                }
                            }
                            
                            // Re-enable raw mode
                            enable_raw_mode()?;
                        }
                    }
                    KeyCode::Esc => {
                        // Escape key - exit
                        if is_recording {
                            println!("\n🛑 Stopping recording...");
                            if let Some(flag) = recording_stop_flag.take() {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            if let Some(handle) = recording_handle.take() {
                                let _ = handle.await;
                            }
                        }
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

fn record_audio_continuous(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    stop_flag: Arc<AtomicBool>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Create temporary WAV file
    let temp_file = std::env::temp_dir().join(format!(
        "murmure-record-{}-{}.wav",
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

    // Record continuously until stop_flag is set
    while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // Stop recording
    drop(stream);

    // Small delay to ensure all audio data is written
    std::thread::sleep(Duration::from_millis(200));

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

