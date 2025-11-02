// Example Rust client for Murmure gRPC Server
//
// To use:
// 1. Add to your Cargo.toml:
//    [dependencies]
//    tonic = "0.12"
//    tokio = { version = "1", features = ["full"] }
//
// 2. Generate stubs from proto:
//    cargo build (protobuf files generated automatically)
//
// 3. Include the generated code (adjust path as needed):
//    include!("../target/debug/build/murmure-*/out/murmure.rs");

use std::path::Path;
use tonic::Request;

// Include generated proto code (adjust path based on your build)
// This is a placeholder - you'll need to generate and include the actual stubs
/*
mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::{TranscribeFileRequest, TranscribeFileRequest, TranscribeStreamRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "http://localhost:50051";
    
    // Create client
    let mut client = TranscriptionServiceClient::connect(server_address).await?;
    
    // Read audio file
    let audio_data = std::fs::read("audio.wav")?;
    
    // Create request
    let request = Request::new(TranscribeFileRequest {
        audio_data,
        use_dictionary: true,
    });
    
    // Call RPC
    let response = client.transcribe_file(request).await?;
    let transcription = response.into_inner();
    
    if transcription.success {
        println!("Transcription: {}", transcription.text);
    } else {
        eprintln!("Error: {}", transcription.error);
    }
    
    Ok(())
}
*/

fn main() {
    println!("This is a template. Generate proto stubs first, then uncomment the code above.");
}

