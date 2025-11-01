# Murmure gRPC Server

A standalone gRPC server implementation of Murmure for speech-to-text transcription. This server can run locally or in cloud environments (Docker, Kubernetes, AWS, etc.) and provides real-time streaming transcription capabilities via gRPC.

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.75+) - [Install Rust](https://www.rust-lang.org/tools/install)
- **Protocol Buffers Compiler** - `protoc` (for building from source)
  - macOS: `brew install protobuf`
  - Ubuntu/Debian: `sudo apt-get install protobuf-compiler`
  - Windows: Download from [protobuf releases](https://github.com/protocolbuffers/protobuf/releases)

### 1. Download the Model

Download the Parakeet model (required):

```bash
# Create resources directory
mkdir -p resources

# Download and extract model
cd resources
curl -L -o /tmp/parakeet-model.zip \
  "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
unzip /tmp/parakeet-model.zip
rm /tmp/parakeet-model.zip
cd ..
```

You should now have `resources/parakeet-tdt-0.6b-v3-int8/` directory.

### 2. Configure Environment

Copy the example environment file and modify as needed:

```bash
cp .env.example .env
```

Edit `.env` to set your paths:

```bash
MURMURE_MODEL_PATH=./resources/parakeet-tdt-0.6b-v3-int8
MURMURE_CC_RULES_PATH=./resources/cc-rules
MURMURE_GRPC_PORT=50051
MURMURE_LOG_LEVEL=info
```

### 3. Build the Server

```bash
cd src-tauri
cargo build --release --bin murmure-server
```

The binary will be at `target/release/murmure-server`.

### 4. Run the Server

```bash
# Load environment variables and run
export $(cat ../.env | xargs)
./target/release/murmure-server
```

Or use environment variables directly:

```bash
export MURMURE_MODEL_PATH=./resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=./resources/cc-rules
./target/release/murmure-server
```

The server will start and listen on port 50051 (or your configured port).

## 📋 Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `MURMURE_MODEL_PATH` | Path to Parakeet model directory | Tries `./resources/` | Yes |
| `MURMURE_CC_RULES_PATH` | Path to cc-rules directory | Tries `./resources/cc-rules` | Yes* |
| `MURMURE_DICTIONARY` | JSON array of custom words | `[]` | No |
| `MURMURE_GRPC_PORT` | gRPC server port | `50051` | No |
| `MURMURE_LOG_LEVEL` | Logging level (trace/debug/info/warn/error) | `info` | No |

\* Required for dictionary corrections. If not provided, dictionary features are disabled.

### Example Configuration

```bash
# Development
export MURMURE_MODEL_PATH=./resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=./resources/cc-rules
export MURMURE_DICTIONARY='["John Doe", "Company Inc"]'
export MURMURE_GRPC_PORT=50051
export MURMURE_LOG_LEVEL=debug

# Production
export MURMURE_MODEL_PATH=/opt/murmure/models/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=/opt/murmure/resources/cc-rules
export MURMURE_GRPC_PORT=50051
export MURMURE_LOG_LEVEL=info
```

### Config File (Optional)

You can also use a `config.json` file in the working directory:

```json
{
  "model_path": "./resources/parakeet-tdt-0.6b-v3-int8",
  "cc_rules_path": "./resources/cc-rules",
  "dictionary": ["word1", "word2"],
  "grpc_port": 50051,
  "log_level": "info"
}
```

Environment variables take precedence over config file values.

## 🐳 Docker Deployment

### Build Docker Image

```bash
docker build -t murmure-server .
```

### Run with Docker Compose

```bash
docker-compose up
```

### Run with Docker

```bash
docker run -d \
  --name murmure-server \
  -p 50051:50051 \
  -v $(pwd)/resources:/app/resources:ro \
  -e MURMURE_MODEL_PATH=/app/resources/parakeet-tdt-0.6b-v3-int8 \
  -e MURMURE_CC_RULES_PATH=/app/resources/cc-rules \
  -e MURMURE_GRPC_PORT=50051 \
  murmure-server
```

## 🔌 gRPC API

### Service: TranscriptionService

#### TranscribeFile

Transcribe a complete audio file (non-streaming).

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
        bool end_of_stream = 2;  // Signal stream end
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

### Audio Requirements

- **Format**: WAV (PCM)
- **Sample Rate**: 16 kHz (automatically resampled if different)
- **Channels**: Mono (automatically converted from stereo)
- **Bit Depth**: 16-bit

## 📝 Example Clients

See the `examples/` directory for example clients in Python and Rust.

### Python Example

```bash
cd examples
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/murmure.proto
python python_client.py audio.wav
```

### Quick Test with grpcurl

```bash
# Install grpcurl: https://github.com/fullstorydev/grpcurl
# Test file transcription (requires audio file as base64)
echo '{"audio_data":"BASE64_ENCODED_WAV","use_dictionary":true}' | \
  grpcurl -plaintext -d @ localhost:50051 murmure.TranscriptionService/TranscribeFile
```

## 🏗️ Architecture

The server maintains the same core engine as the desktop app:

- **`engine/`** - Parakeet transcription engine (unchanged for easy upstream updates)
- **`audio.rs`** - Audio utilities (extracted, no UI dependencies)
- **`model.rs`** - Model path management (config-based)
- **`dictionary.rs`** - Dictionary logic with phonetic corrections
- **`transcription.rs`** - Service wrapper for transcription operations
- **`server/grpc.rs`** - gRPC service implementation

This structure allows easy integration of upstream updates from the desktop app.

## 🔧 Development

### Build from Source

```bash
cd src-tauri
cargo build --release --bin murmure-server
```

### Run in Development Mode

```bash
cd src-tauri
cargo run --bin murmure-server
```

### Test Build

```bash
cd src-tauri
cargo test
cargo clippy
```

## 📊 Performance

- **First request**: Slower (~2-5 seconds) due to model loading
- **Subsequent requests**: Faster (~0.5-2 seconds depending on audio length)
- **Streaming**: Supports real-time transcription as audio chunks arrive
- **Memory**: ~500MB-1GB for the model in memory
- **CPU**: Optimized for CPU inference (GPU support not currently implemented)

## 🐛 Troubleshooting

### Model Not Found

```
Error: Model 'parakeet-tdt-0.6b-v3-int8' not found
```

**Solution**: Set `MURMURE_MODEL_PATH` to the correct path or place model in `./resources/`

### CC Rules Not Found

```
Warning: CC rules not found, skipping dictionary correction
```

**Solution**: Set `MURMURE_CC_RULES_PATH` to the correct path. Dictionary features will be disabled if not found.

### Port Already in Use

```
Error: Address already in use
```

**Solution**: Change `MURMURE_GRPC_PORT` to an available port or stop the conflicting service.

### Compilation Errors

If you see protobuf-related errors:

1. Ensure `protoc` is installed and in PATH
2. Check that `proto/murmure.proto` exists
3. Try `cargo clean && cargo build`

### Server Won't Start

1. Check that model files exist at the specified path
2. Verify environment variables are set correctly
3. Check logs for detailed error messages
4. Ensure you have sufficient disk space and memory

## 📚 Additional Resources

- [gRPC Documentation](https://grpc.io/docs/)
- [Protocol Buffers Guide](https://developers.google.com/protocol-buffers)
- [Murmure Desktop App](https://github.com/Kieirra/murmure)
- [Parakeet Model](https://github.com/Kieirra/murmure-model)

## 📄 License

Same as the main Murmure project - GNU GPL v3

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

---

**Note**: This is a server implementation extracted from the Murmure desktop app. The core transcription engine remains unchanged to facilitate easy integration of upstream updates.

