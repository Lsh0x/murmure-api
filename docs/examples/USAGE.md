# Quick Usage Guide

## Rust Streaming Client ⭐

Toggle recording mode: press SPACE to start, press again to stop and transcribe.

```bash
cd examples
cargo run --example rust_streaming_client
```

**Controls:**
- Press SPACE to start/stop recording (toggle)
- Ctrl+C to exit and see full transcript

See [README_STREAMING_CLIENT.md](README_STREAMING_CLIENT.md) for details.

## Rust File Client

The simplest way to test transcription with audio files.

### Quick Start

```bash
cd examples
cargo run --example rust_file_client -- audio.wav
```

### Streaming Mode

```bash
cargo run --example rust_file_client -- audio.wav --stream
```

See [README_RUST_FILE_CLIENT.md](README_RUST_FILE_CLIENT.md) for complete documentation.

## Rust Recording Client

The easiest way to test the server with real-time audio recording.

### ⚠️ macOS Microphone Permission

On macOS, you must grant microphone permission to your terminal app:
1. System Settings → Privacy & Security → Microphone
2. Enable your terminal app (Terminal, iTerm, Cursor, etc.)
3. Restart terminal and try again

Or run: `./examples/fix-mic-permission.sh`

### Prerequisites

1. Server running (see main README)
2. Microphone connected
3. Rust installed

### Run

```bash
# Basic usage (5 seconds)
cd examples
cargo run --example rust_record_client

# Custom duration
cargo run --example rust_record_client -- --duration 10

# Custom server
cargo run --example rust_record_client -- \
  --server http://localhost:50051 \
  --duration 8
```

### What Happens

1. Client detects your microphone
2. Records audio for specified duration
3. Sends WAV file to server
4. Displays transcription

## Python Client

For testing with pre-recorded audio files.

```bash
cd examples
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/murmure.proto
python python_client.py audio.wav
```

## See Also

- [Rust Client Documentation](README_RUST_CLIENT.md) - Detailed Rust client guide
- [Server Documentation](../SERVER.md) - Server setup and API docs

