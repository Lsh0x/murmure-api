//! Example Rust client for Murmure gRPC Server
//!
//! This client transcribes audio files using the Murmure gRPC server.
//! Supports both file-based and streaming transcription methods.
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
//! # File-based transcription (default)
//! cargo run --example rust_file_client -- audio.wav
//!
//! # With custom server
//! cargo run --example rust_file_client -- audio.wav --server http://localhost:50052
//!
//! # Try streaming mode
//! cargo run --example rust_file_client -- audio.wav --stream
//! ```
//!
//! Options:
//! - Audio file path (required)
//! - `--server <address>` - Server address (default: http://localhost:50051)
//! - `--no-dictionary` - Disable dictionary corrections
//! - `--stream` - Use streaming RPC instead of file-based

use std::path::PathBuf;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

// Include generated proto code from build script
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::{TranscribeFileRequest, TranscribeStreamRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <audio_file> [--server <address>] [--no-dictionary] [--stream]",
            args[0]
        );
        eprintln!(
            "Example: {} audio.wav --server http://localhost:50051",
            args[0]
        );
        std::process::exit(1);
    }

    let audio_file = PathBuf::from(&args[1]);
    if !audio_file.exists() {
        eprintln!("Error: Audio file not found: {}", audio_file.display());
        std::process::exit(1);
    }

    let server_address = args
        .iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&"http://localhost:50051".to_string())
        .clone();

    let use_dictionary = !args.contains(&"--no-dictionary".to_string());
    let use_streaming = args.contains(&"--stream".to_string());

    println!("📁 Murmure File Transcription Client");
    println!("Audio file: {}", audio_file.display());
    println!("Server: {}", server_address);
    println!("Use dictionary: {}", use_dictionary);
    println!(
        "Mode: {}\n",
        if use_streaming {
            "Streaming"
        } else {
            "File-based"
        }
    );

    // Read audio file
    println!("📖 Reading audio file...");
    let audio_data = std::fs::read(&audio_file)?;
    println!("✅ File read ({} bytes)", audio_data.len());

    // Connect to server
    println!("📡 Connecting to server...");
    let mut client = TranscriptionServiceClient::connect(server_address.clone()).await?;
    println!("✅ Connected to server");

    // Transcribe
    if use_streaming {
        transcribe_stream(&mut client, &audio_data).await?;
    } else {
        transcribe_file(&mut client, &audio_data, use_dictionary).await?;
    }

    Ok(())
}

async fn transcribe_file(
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    audio_data: &[u8],
    use_dictionary: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔊 Sending audio for transcription (file-based)...");

    let request = Request::new(TranscribeFileRequest {
        audio_data: audio_data.to_vec(),
        use_dictionary,
    });

    let response = client.transcribe_file(request).await?;
    let transcription = response.into_inner();

    if transcription.success {
        println!("\n📝 Transcription:");
        println!("{}", transcription.text);
    } else {
        eprintln!("\n❌ Transcription failed: {}", transcription.error);
        std::process::exit(1);
    }

    Ok(())
}

async fn transcribe_stream(
    client: &mut TranscriptionServiceClient<tonic::transport::Channel>,
    audio_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔊 Sending audio for transcription (streaming)...");

    use tokio::sync::mpsc;

    // Split audio into chunks
    let chunk_size = 8192;
    let chunks: Vec<Vec<u8>> = audio_data
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    // Create channel for request stream
    let (mut tx, rx) = mpsc::channel(128);

    // Spawn task to send chunks
    tokio::spawn(async move {
        for chunk in chunks {
            let request = TranscribeStreamRequest {
                request_type: Some(murmure::transcribe_stream_request::RequestType::AudioChunk(
                    chunk,
                )),
            };
            if tx.send(request).await.is_err() {
                break;
            }
        }
        // Send end of stream
        let _ = tx
            .send(TranscribeStreamRequest {
                request_type: Some(
                    murmure::transcribe_stream_request::RequestType::EndOfStream(true),
                ),
            })
            .await;
    });

    let request = Request::new(ReceiverStream::new(rx));
    let mut response_stream = client.transcribe_stream(request).await?.into_inner();

    println!("📡 Streaming audio chunks...");

    let mut final_text: Option<String> = None;

    while let Some(response) = response_stream.message().await? {
        match response.response_type {
            Some(murmure::transcribe_stream_response::ResponseType::PartialText(text)) => {
                if !text.is_empty() {
                    println!("📝 Partial: {}", text);
                }
            }
            Some(murmure::transcribe_stream_response::ResponseType::FinalText(text)) => {
                final_text = Some(text);
            }
            Some(murmure::transcribe_stream_response::ResponseType::Error(err)) => {
                eprintln!("❌ Error: {}", err);
                std::process::exit(1);
            }
            None => {}
        }

        if response.is_final {
            break;
        }
    }

    if let Some(text) = final_text {
        println!("\n📝 Final Transcription:");
        println!("{}", text);
    } else {
        eprintln!("\n⚠️  No final transcription received");
    }

    Ok(())
}
