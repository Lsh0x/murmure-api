# Murmure gRPC Server

A standalone gRPC server version of Murmure for speech-to-text transcription. This server can run locally or in the cloud (Docker) and provides real-time streaming transcription capabilities.

## Features

- **gRPC API** with bidirectional streaming support
- **Real-time transcription** via streaming audio
- **Custom dictionary** support for phonetic correction
- **Docker-ready** for easy deployment
- **Environment-based configuration**

## Quick Start

### Using Docker (Recommended)

1. Ensure you have the Parakeet model in `resources/parakeet-tdt-0.6b-v3-int8/`
2. Ensure you have cc-rules in `resources/cc-rules/`

```bash
docker-compose up
```

The server will start on port 50051.

### Running Locally

1. Set environment variables:
```bash
export MURMURE_MODEL_PATH=/path/to/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=/path/to/cc-rules
export MURMURE_GRPC_PORT=50051
```

2. Build and run:
```bash
cd src-tauri
cargo build --release --bin murmure-server
./target/release/murmure-server
```

## Configuration

### Environment Variables

- `MURMURE_MODEL_PATH` - Path to Parakeet model directory (required)
- `MURMURE_CC_RULES_PATH` - Path to cc-rules directory (required)
- `MURMURE_DICTIONARY` - JSON array of custom dictionary words (optional)
  - Example: `MURMURE_DICTIONARY='["John Doe", "Jane Smith"]'`
- `MURMURE_GRPC_PORT` - gRPC server port (default: 50051)
- `MURMURE_LOG_LEVEL` - Logging level (default: info)

### Config File (Optional)

Create `config.json`:

```json
{
  "model_path": "/path/to/model",
  "cc_rules_path": "/path/to/cc-rules",
  "dictionary": ["word1", "word2"],
  "grpc_port": 50051,
  "log_level": "info"
}
```

## gRPC API

### Service: TranscriptionService

#### TranscribeFile

Transcribe a complete audio file.

**Request:**
```protobuf
message TranscribeFileRequest {
    bytes audio_data = 1;        // WAV format, 16kHz, mono, 16-bit
    bool use_dictionary = 2;     // Apply dictionary corrections
}
```

**Response:**
```protobuf
message TranscribeFileResponse {
    string text = 1;             // Transcribed text
    bool success = 2;            // Success indicator
    string error = 3;            // Error message if failed
}
```

#### TranscribeStream

Bidirectional streaming for real-time audio transcription.

**Request Stream:**
```protobuf
message TranscribeStreamRequest {
    oneof request_type {
        bytes audio_chunk = 1;   // Audio chunk data
        bool end_of_stream = 2;   // Signal stream end
    }
}
```

**Response Stream:**
```protobuf
message TranscribeStreamResponse {
    oneof response_type {
        string partial_text = 1;  // Partial transcription
        string final_text = 2;    // Final transcription
        string error = 3;         // Error message
    }
    bool is_final = 4;            // Is this final result?
}
```

## Audio Requirements

- **Format**: WAV
- **Sample Rate**: 16 kHz (automatically resampled if different)
- **Channels**: Mono (automatically converted if stereo)
- **Bit Depth**: 16-bit PCM

## Example Clients

See `examples/` directory for:
- Python client example
- Rust client example
- Streaming example

## Docker Deployment

### Build

```bash
docker build -t murmure-server .
```

### Run

```bash
docker run -d \
  -p 50051:50051 \
  -v /path/to/resources:/app/resources:ro \
  -e MURMURE_MODEL_PATH=/app/resources/parakeet-tdt-0.6b-v3-int8 \
  -e MURMURE_CC_RULES_PATH=/app/resources/cc-rules \
  murmure-server
```

## Troubleshooting

### Model Not Found

Ensure `MURMURE_MODEL_PATH` points to the correct directory containing the Parakeet model files.

### CC Rules Not Found

Ensure `MURMURE_CC_RULES_PATH` points to the cc-rules directory.

### Port Already in Use

Change `MURMURE_GRPC_PORT` to an available port.

### Compilation Errors

Make sure protobuf compiler is installed:
```bash
apt-get install protobuf-compiler  # Debian/Ubuntu
brew install protobuf              # macOS
```

## Architecture

The server maintains the same core engine as the Tauri desktop app:
- `engine/` - Parakeet transcription engine (unchanged)
- `audio.rs` - Audio utilities (extracted, no Tauri)
- `model.rs` - Model path management (config-based)
- `dictionary.rs` - Dictionary logic (config-based)
- `transcription.rs` - Service wrapper
- `server/grpc.rs` - gRPC service implementation

This structure allows easy integration of upstream updates from the desktop app.

