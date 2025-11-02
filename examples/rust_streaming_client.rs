//! Streaming Conversation Client for Murmure gRPC Server
//!
//! This client records audio continuously and sends it to the server in real-time,
//! receiving partial transcriptions as the conversation progresses.
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
//! Press Ctrl+C to stop recording and exit.
//!
//! Options:
//! - `--server <address>` - Server address (default: http://localhost:50051)
//! - `--chunk-duration <seconds>` - Duration of each audio chunk sent to server (default: 2)
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

    let chunk_duration = args
        .iter()
        .position(|a| a == "--chunk-duration")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2);

    println!("🎙️  Murmure Streaming Conversation Client");
    println!("Server: {}", server_address);
    println!("Chunk duration: {} seconds\n", chunk_duration);

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

    println!("🎤 Starting streaming conversation...");
    println!("   Recording in {} second chunks", chunk_duration);
    println!("   Press Ctrl+C to stop\n");


    // Track conversation text
    let mut conversation_text = String::new();

    // Stream audio chunks to server continuously
    let mut chunk_counter = 0;
    
    // Handle Ctrl+C gracefully with a shared flag
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        println!("\n\n🛑 Stopping streaming conversation...");
    });

    loop {
        // Check if Ctrl+C was pressed
        if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
            println!("\n📝 Conversation transcript:\n{}", conversation_text);
            break;
        }

        chunk_counter += 1;
        
        // Record one chunk (blocking call)
        let audio_data = record_audio_chunk(&device, &config, chunk_duration)?;
        
        if audio_data.is_empty() {
            eprintln!("⚠️  Warning: Empty audio chunk, skipping...");
            continue;
        }

        // Create request stream for this chunk
        let (chunk_tx, chunk_rx) = mpsc::channel(128);
        let chunk_data = audio_data;
        
        // Split into smaller chunks for streaming
        tokio::spawn(async move {
            let chunk_size = 16384; // 16KB chunks
            for audio_chunk in chunk_data.chunks(chunk_size) {
                if chunk_tx.send(TranscribeStreamRequest {
                    request_type: Some(murmure::transcribe_stream_request::RequestType::AudioChunk(audio_chunk.to_vec())),
                }).await.is_err() {
                    return;
                }
            }
            
            // Send end of stream for this chunk
            let _ = chunk_tx.send(TranscribeStreamRequest {
                request_type: Some(murmure::transcribe_stream_request::RequestType::EndOfStream(true)),
            }).await;
        });

        // Send chunk to server and process responses
        let request = Request::new(ReceiverStream::new(chunk_rx));
        let mut response_stream = match client.transcribe_stream(request).await {
            Ok(stream) => stream.into_inner(),
            Err(e) => {
                eprintln!("❌ Failed to start stream: {}", e);
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        };

        // Process responses for this chunk
        while let Some(result) = response_stream.message().await.transpose() {
            match result {
                Ok(response) => {
                    match response.response_type {
                        Some(murmure::transcribe_stream_response::ResponseType::PartialText(text)) => {
                            if !text.is_empty() {
                                print!("\r   📝 Partial: {}", text);
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            }
                        }
                        Some(murmure::transcribe_stream_response::ResponseType::FinalText(text)) => {
                            if !text.is_empty() {
                                // Clear the partial text line
                                print!("\r");
                                for _ in 0..100 {
                                    print!(" ");
                                }
                                print!("\r");
                                
                                println!("✅ Chunk {}: {}", chunk_counter, text);
                                conversation_text.push_str(&text);
                                conversation_text.push(' ');
                            }
                        }
                        Some(murmure::transcribe_stream_response::ResponseType::Error(err)) => {
                            eprintln!("\n❌ Error: {}", err);
                        }
                        None => {}
                    }

                    if response.is_final {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("❌ Response error: {}", e);
                    break;
                }
            }
        }

        // Small delay before next chunk
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

fn record_audio_chunk(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    duration_secs: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create temporary WAV file
    let temp_file = std::env::temp_dir().join(format!(
        "murmure-stream-{}-{}.wav",
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

    // Record for specified duration
    std::thread::sleep(Duration::from_secs(duration_secs));
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

