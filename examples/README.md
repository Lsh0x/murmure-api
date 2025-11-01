# Example Clients for Murmure gRPC Server

Example implementations showing how to interact with the Murmure gRPC server.

## Available Examples

1. **Rust Streaming Client** (`rust_streaming_client.rs`) - **NEW!** Continuous conversation transcription
2. **Rust Recording Client** (`rust_record_client.rs`) - Records audio from microphone and transcribes
3. **Rust File Client** (`rust_file_client.rs`) - Transcribes audio files (supports streaming and file-based modes)
4. **Python Client** (`python_client.py`) - Simple file-based transcription client
5. **Rust Client Template** (`rust_client.rs`) - Template for file-based transcription

## Rust Recording Client (Recommended)

A complete Rust client that records audio from your microphone and transcribes it.

### Quick Start

```bash
cd examples
cargo run --example rust_record_client -- --duration 5
```

### Features

- Records from your default microphone
- Configurable recording duration
- Sends to Murmure server for transcription
- Displays results

### Documentation

See [README_RUST_CLIENT.md](README_RUST_CLIENT.md) for complete documentation.

**Note**: On macOS, microphone permission is required. See troubleshooting section in the docs.

## Rust Streaming Client ⭐ NEW!

A continuous streaming client for real-time conversation transcription.

### Quick Start

```bash
cd examples
cargo run --example rust_streaming_client
```

### Features

- Records audio continuously in chunks
- Real-time transcription updates
- Accumulates full conversation transcript
- Press Ctrl+C to stop and view full transcript

### Usage

```bash
# Default (2 second chunks)
cargo run --example rust_streaming_client

# Custom chunk duration
cargo run --example rust_streaming_client -- --chunk-duration 3

# Custom server
cargo run --example rust_streaming_client -- \
  --server http://localhost:50052 \
  --chunk-duration 2
```

### Documentation

See [README_STREAMING_CLIENT.md](README_STREAMING_CLIENT.md) for complete documentation.

## Rust File Client

A Rust client for transcribing audio files from disk.

### Quick Start

```bash
cd examples
# File-based transcription (default)
cargo run --example rust_file_client -- audio.wav

# Streaming mode
cargo run --example rust_file_client -- audio.wav --stream

# Custom server and options
cargo run --example rust_file_client -- audio.wav \
  --server http://localhost:50052 \
  --no-dictionary
```

### Features

- Transcribes audio files from disk
- Supports both file-based and streaming RPCs
- Configurable server address
- Dictionary correction toggle
- Error handling and user feedback

### Options

- `<audio_file>` - Path to WAV file (required)
- `--server <address>` - Server address (default: http://localhost:50051)
- `--no-dictionary` - Disable dictionary corrections
- `--stream` - Use streaming RPC instead of file-based

## Python Client

### Setup

1. Install dependencies:
```bash
pip install grpcio grpcio-tools
```

2. Generate Python stubs from the proto file:
```bash
cd examples
python -m grpc_tools.protoc \
    -I../proto \
    --python_out=. \
    --grpc_python_out=. \
    ../proto/murmure.proto
```

This will create `murmure_pb2.py` and `murmure_pb2_grpc.py`.

3. Update `python_client.py` to import from the generated files:
```python
from murmure_pb2 import TranscribeFileRequest, TranscribeFileResponse, TranscribeStreamRequest, TranscribeStreamResponse
from murmure_pb2_grpc import TranscriptionServiceStub
```

4. Run:
```bash
python python_client.py audio.wav [server_address]
```

## Rust Client

The Rust client example is a template. To use it:

1. The proto files are automatically generated during build
2. Include the generated code in your project
3. Adjust import paths based on your project structure

For a complete Rust client, you would:

1. Add dependencies:
```toml
[dependencies]
tonic = "0.12"
tokio = { version = "1", features = ["full"] }
```

2. Generate and include proto stubs (see rust_client.rs for template)

3. Build and run your client

## Streaming Example

The Python client includes a streaming example that:
- Reads audio in chunks
- Sends chunks via bidirectional stream
- Receives partial transcripts as they become available
- Gets final transcription when stream ends

## Notes

- Audio files must be WAV format (16kHz, mono, 16-bit)
- The server automatically resamples if needed
- Dictionary corrections are applied if `use_dictionary=true`

