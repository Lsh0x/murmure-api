# Rust Recording Client

A complete Rust client example that records audio from your microphone and transcribes it using the Murmure gRPC server.

## Features

- 🎤 Records audio from your default microphone
- 📡 Sends audio to Murmure gRPC server
- 🔊 Real-time transcription
- 📝 Displays transcription results
- ⚙️ Configurable recording duration and server address

## Prerequisites

- Rust 1.75+ installed
- Murmure gRPC server running (see main README)
- A microphone connected to your computer

## Setup

### 1. Install Dependencies

The example uses the following crates (already in `Cargo.toml`):
- `tonic` - gRPC client
- `tokio` - Async runtime
- `cpal` - Cross-platform audio I/O
- `hound` - WAV file handling

### 2. Build the Example

```bash
cd examples
cargo build --example rust_record_client
```

### 3. Start the Server

In a separate terminal:

```bash
cd ../src-tauri
export MURMURE_MODEL_PATH=../resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=../resources/cc-rules
cargo run --bin murmure-server
```

## Usage

### Basic Usage

Record for 5 seconds (default):

```bash
cargo run --example rust_record_client
```

### Custom Duration

Record for 10 seconds:

```bash
cargo run --example rust_record_client -- --duration 10
```

### Custom Server Address

Connect to a different server:

```bash
cargo run --example rust_record_client -- \
  --server http://localhost:50052 \
  --duration 5
```

### All Options

```bash
cargo run --example rust_record_client -- \
  --server http://localhost:50051 \
  --duration 8
```

## How It Works

1. **Audio Recording**
   - Detects your default microphone
   - Configures audio stream (sample rate, channels)
   - Records audio to a temporary WAV file
   - Converts to mono if needed
   - Saves as 16-bit PCM WAV

2. **gRPC Communication**
   - Connects to Murmure server
   - Sends WAV data via `TranscribeFile` RPC
   - Receives transcription result

3. **Result Display**
   - Shows transcription text
   - Displays errors if transcription fails

## Example Output

```
🎤 Murmure Audio Recording Client
Server: http://localhost:50051
Recording duration: 5 seconds
Press Ctrl+C to stop early

🎙️  Recording audio...
   Device: Built-in Microphone
   Sample rate: 48000 Hz
   Channels: 2
   Recording...
✅ Recording complete (176400 bytes)
📡 Connecting to server...
✅ Connected to server
🔊 Sending audio for transcription...

📝 Transcription:
Hello, this is a test of the Murmure transcription service.
```

## Troubleshooting

### No Input Device Found

```
Error: No input device available
```

**Solution**: Ensure your microphone is connected and recognized by your OS.

### Connection Refused

```
Error: Connection refused
```

**Solution**: 
- Ensure the Murmure server is running
- Check the server address and port
- Verify firewall settings

### Audio Recording Issues

If you encounter audio-related errors:
- Check microphone permissions (macOS requires permission)
- Try a different audio device
- Verify `cpal` supports your audio backend

### Build Errors

If you see proto-related errors:
- Ensure `protoc` is installed
- Run `cargo clean` and rebuild
- Check that `build.rs` generates proto files correctly

## Code Structure

- `main()` - Entry point, argument parsing, orchestration
- `record_audio()` - Audio capture and WAV file creation
- `build_stream()` - Creates CPAL audio input stream

## Extending the Client

### Push-to-Talk Mode

Add a keyboard listener to start/stop recording:

```rust
use crossterm::event::{read, Event, KeyCode};

// Wait for spacebar
loop {
    if let Ok(Event::Key(key)) = read() {
        if key.code == KeyCode::Char(' ') {
            break; // Start recording
        }
    }
}
```

### Streaming Mode

Use the `TranscribeStream` RPC for real-time transcription:

```rust
let mut stream = client
    .transcribe_stream(Request::new(tokio_stream::StreamExt::boxed(audio_chunks)))
    .await?;

while let Some(response) = stream.message().await? {
    println!("Partial: {}", response.partial_text);
}
```

### Save Recordings

To keep recordings, modify the temp file path:

```rust
let output_file = PathBuf::from("recordings").join(format!("recording-{}.wav", timestamp));
// ... recording code ...
// Don't delete the file
```

## See Also

- [Main Server README](../README_SERVER.md)
- [Python Client Example](README.md)
- [gRPC API Documentation](../docs/SERVER.md)

