//! Example Rust client for Murmure gRPC Server
//!
//! This client records audio from your microphone and sends it to the Murmure server
//! for transcription.
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
//! cargo run --example rust_record_client -- --duration 5
//! ```
//!
//! Options:
//! - `--server <address>` - Server address (default: http://localhost:50051)
//! - `--duration <seconds>` - Recording duration (default: 5)
//!
//! See README_RUST_CLIENT.md for detailed documentation.

use std::sync::Arc;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::io::BufWriter;
use std::fs::File;
use tonic::Request;

// Include generated proto code from build script
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::TranscribeFileRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let server_address = args
        .iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&"http://localhost:50051".to_string())
        .clone();

    let duration_secs = args
        .iter()
        .position(|a| a == "--duration")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);

    println!("🎤 Murmure Audio Recording Client");
    println!("Server: {}", server_address);
    println!("Recording duration: {} seconds", duration_secs);
    println!("Press Ctrl+C to stop early\n");

    // Record audio
    println!("🎙️  Recording audio...");
    let audio_data = record_audio(duration_secs)?;
    println!("✅ Recording complete ({} bytes)", audio_data.len());

    // Connect to server
    println!("📡 Connecting to server...");
    let mut client = TranscriptionServiceClient::connect(server_address).await?;
    println!("✅ Connected to server");

    // Transcribe
    println!("🔊 Sending audio for transcription...");
    let request = Request::new(TranscribeFileRequest {
        audio_data,
        use_dictionary: true,
    });

    let response = client.transcribe_file(request).await?;
    let transcription = response.into_inner();

    if transcription.success {
        println!("\n📝 Transcription:");
        println!("{}", transcription.text);
    } else {
        eprintln!("❌ Transcription failed: {}", transcription.error);
    }

    Ok(())
}

fn record_audio(duration_secs: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;

    let config = device.default_input_config()?;
    println!("   Device: {}", device.name()?);
    println!("   Sample rate: {} Hz", config.sample_rate().0);
    println!("   Channels: {}", config.channels());

    // Create temporary WAV file
    let temp_file = std::env::temp_dir().join(format!("murmure-record-{}.wav", std::process::id()));
    
    let spec = WavSpec {
        channels: 1, // Force mono
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = File::create(&temp_file)?;
    let writer = WavWriter::new(BufWriter::new(file), spec)?;
    let writer_arc = Arc::new(std::sync::Mutex::new(writer));

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, writer_arc.clone())?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, writer_arc.clone())?,
        cpal::SampleFormat::I32 => build_stream::<i32>(&device, &config, writer_arc.clone())?,
        _ => return Err("Unsupported sample format".into()),
    };

    stream.play()?;
    println!("   Recording...");

    // Record for specified duration
    std::thread::sleep(Duration::from_secs(duration_secs));

    drop(stream);

    // Finalize WAV file
    {
        let mut writer = writer_arc.lock().unwrap();
        writer.flush()?;
        drop(writer); // Explicitly drop to get ownership back
    }
    
    // Get the writer back and finalize
    let writer = Arc::try_unwrap(writer_arc).map_err(|_| "Failed to unwrap Arc")?;
    writer.into_inner().unwrap().finalize()?;

    // Read WAV file into memory
    let audio_data = std::fs::read(&temp_file)?;
    
    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    Ok(audio_data)
}

type WavWriterType = WavWriter<BufWriter<File>>;

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    writer: Arc<std::sync::Mutex<WavWriterType>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels() as usize;

    let stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut writer = writer.lock().unwrap();
            for frame in data.chunks_exact(channels) {
                let sample = if channels == 1 {
                    frame[0].to_sample::<f32>()
                } else {
                    // Convert to mono by averaging
                    frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                };

                // Convert to i16 and write
                let sample_i16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let _ = writer.write_sample(sample_i16);
            }
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

