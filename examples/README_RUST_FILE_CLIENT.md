# Rust File Client

A complete Rust client example for transcribing audio files using the Murmure gRPC server. Supports both file-based and streaming transcription modes.

## Features

- 📁 Transcribes audio files from disk
- 📡 Supports both `TranscribeFile` and `TranscribeStream` RPCs
- ⚙️ Configurable server address
- 📝 Dictionary correction toggle
- 🔄 Streaming mode with partial transcript display
- ❌ Comprehensive error handling

## Prerequisites

- Rust 1.75+ installed
- Murmure gRPC server running (see main README)
- An audio file in WAV format

## Setup

### Build

```bash
cd examples
cargo build --example rust_file_client
```

The binary will be at `target/debug/examples/rust_file_client` (or `target/release/` for release builds).

### Start the Server

In a separate terminal:

```bash
cd ../src-tauri
export MURMURE_MODEL_PATH=../resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=../resources/cc-rules
cargo run --bin murmure-server
```

## Usage

### File-based Transcription (Default)

Simple file transcription:

```bash
cargo run --example rust_file_client -- audio.wav
```

With custom server:

```bash
cargo run --example rust_file_client -- audio.wav --server http://localhost:50052
```

Disable dictionary corrections:

```bash
cargo run --example rust_file_client -- audio.wav --no-dictionary
```

### Streaming Transcription

Use streaming mode for larger files or to see partial results:

```bash
cargo run --example rust_file_client -- audio.wav --stream
```

Streaming mode will:
- Send audio in chunks
- Display partial transcriptions as they arrive
- Show final transcription when complete

### All Options

```bash
cargo run --example rust_file_client -- <audio_file> \
  --server <address> \
  --no-dictionary \
  --stream
```

## Example Output

### File-based Mode

```
📁 Murmure File Transcription Client
Audio file: audio.wav
Server: http://localhost:50051
Use dictionary: true
Mode: File-based

📖 Reading audio file...
✅ File read (176400 bytes)
📡 Connecting to server...
✅ Connected to server
🔊 Sending audio for transcription (file-based)...

📝 Transcription:
Hello, this is a test of the Murmure transcription service.
```

### Streaming Mode

```
📁 Murmure File Transcription Client
Audio file: audio.wav
Server: http://localhost:50051
Use dictionary: true
Mode: Streaming

📖 Reading audio file...
✅ File read (176400 bytes)
📡 Connecting to server...
✅ Connected to server
🔊 Sending audio for transcription (streaming)...
📡 Streaming audio chunks...
📝 Partial: Hello, this is
📝 Partial: Hello, this is a test
📝 Partial: Hello, this is a test of the

📝 Final Transcription:
Hello, this is a test of the Murmure transcription service.
```

## Code Structure

- `main()` - Entry point, argument parsing, orchestration
- `transcribe_file()` - File-based transcription using `TranscribeFile` RPC
- `transcribe_stream()` - Streaming transcription using `TranscribeStream` RPC

## Audio Requirements

- **Format**: WAV (PCM)
- **Sample Rate**: 16 kHz (server will resample if needed)
- **Channels**: Mono (server will convert if needed)
- **Bit Depth**: 16-bit

## Troubleshooting

### File Not Found

```
Error: Audio file not found: audio.wav
```

**Solution**: Ensure the file path is correct and the file exists.

### Connection Refused

```
Error: Connection refused
```

**Solution**: 
- Ensure the Murmure server is running
- Check the server address and port
- Verify firewall settings

### Invalid Audio Format

If you get transcription errors, verify your audio file:
- Is it a valid WAV file?
- Is it 16-bit PCM format?
- Try converting with `ffmpeg` if needed:

```bash
ffmpeg -i input.wav -ar 16000 -ac 1 -sample_fmt s16 output.wav
```

## Comparison with Python Client

The Rust file client is equivalent to the Python client but with:
- Better type safety
- No external dependencies needed (self-contained)
- Faster execution (compiled binary)
- Same functionality (file-based and streaming)

## See Also

- [Rust Recording Client](README_RUST_CLIENT.md) - Microphone recording client
- [Main Server README](../README_SERVER.md) - Server setup and API docs
- [Python Client](README.md#python-client) - Python implementation

