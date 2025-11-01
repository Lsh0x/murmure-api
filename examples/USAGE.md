# Quick Usage Guide

## Rust Recording Client

The easiest way to test the server with real-time audio recording.

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
- [Main Server README](../README_SERVER.md) - Server setup and API docs

