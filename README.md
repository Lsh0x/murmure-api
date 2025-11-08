# Murmure gRPC Server

A standalone gRPC server implementation of the [Murmure](https://github.com/Kieirra/murmure) speech-to-text application. This server allows you to run Murmure as a service that can be deployed locally or in the cloud (Docker, Kubernetes, AWS, etc.).

> **Note**: This repository contains a gRPC server extracted from the original Murmure desktop application. For the desktop application installation and usage, see the [official Murmure repository](https://github.com/Kieirra/murmure).

## Features

- **gRPC API** with bidirectional streaming support
- **Real-time transcription** via streaming audio (STT)
- **Text-to-speech synthesis** with Piper TTS (TTS)
- **Custom dictionary** support for phonetic correction
- **Docker-ready** for easy deployment
- **Privacy First**: All processing happens locally on your device. No data ever leaves your server.
- **Powered by Parakeet**: NVIDIA's state-of-the-art speech recognition model runs entirely on-device
- **Powered by Piper**: Fast, local neural text-to-speech system

## Quick Start

### Prerequisites

- **Rust** (1.75+) - [Install Rust](https://www.rust-lang.org/tools/install)
- **Protocol Buffers Compiler** - `protoc` (for building from source)
  - macOS: `brew install protobuf`
  - Ubuntu/Debian: `sudo apt-get install protobuf-compiler`
  - Windows: Download from [protobuf releases](https://github.com/protocolbuffers/protobuf/releases)

### 1. Download the Models

#### Speech-To-Text (STT) Model

Download the Parakeet model (required for transcription):

```bash
# Create resources directory structure
mkdir -p resources/stt

# Download and extract STT model
cd resources/stt
curl -L -o /tmp/parakeet-model.zip \
  "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
unzip /tmp/parakeet-model.zip
rm /tmp/parakeet-model.zip
cd ../..
```

You should now have `resources/stt/parakeet-tdt-0.6b-v3-int8/` directory.

#### Text-To-Speech (TTS) Model (Optional)

Download a Piper TTS model for text-to-speech synthesis:

```bash
# Create TTS model directory
mkdir -p resources/tts

# Download a Piper model (example: English US voice)
cd resources/tts

# Download model files (example: en_US-lessac-medium)
curl -L -o en_US-lessac-medium.onnx \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/en_US-lessac-medium/en_US-lessac-medium.onnx"

curl -L -o en_US-lessac-medium.onnx.json \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/en_US-lessac-medium/en_US-lessac-medium.onnx.json"

cd ../..
```

**Available Piper Models:**

Piper models are available from the [Piper Voices Repository](https://huggingface.co/rhasspy/piper-voices) on Hugging Face. You can download models for various languages and voices:

- **English (US)**: `en_US-lessac-medium`, `en_US-amy-medium`, `en_US-libritts-high`, etc.
- **English (UK)**: `en_GB-alba-medium`, `en_GB-northern_english_male-medium`, etc.
- **French**: `fr_FR-siwis-medium`, `fr_FR-upmc-medium`, etc.
- **German**: `de_DE-thorsten-medium`, `de_DE-eva-medium`, etc.
- And many more languages...

**Example: Download different voice models:**

```bash
# Download French voice
curl -L -o resources/tts/fr_FR-siwis-medium.onnx \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/fr/fr_FR/fr_FR-siwis-medium/fr_FR-siwis-medium.onnx"

curl -L -o resources/tts/fr_FR-siwis-medium.onnx.json \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/fr/fr_FR/fr_FR-siwis-medium/fr_FR-siwis-medium.onnx.json"

# Download German voice
curl -L -o resources/tts/de_DE-thorsten-medium.onnx \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/de/de_DE/de_DE-thorsten-medium/de_DE-thorsten-medium.onnx"

curl -L -o resources/tts/de_DE-thorsten-medium.onnx.json \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/de/de_DE/de_DE-thorsten-medium/de_DE-thorsten-medium.onnx.json"
```

**Note:** Each Piper model consists of two files:
- `.onnx` - The model weights file
- `.onnx.json` - The model configuration file

Both files are required for the model to work.

### 2. Configure Environment

Copy the example environment file and modify as needed:

```bash
cp .env.example .env
```

Edit `.env` to set your paths:

```bash
# STT (Speech-To-Text) Configuration
MURMURE_MODEL_PATH=./resources/stt/parakeet-tdt-0.6b-v3-int8
MURMURE_CC_RULES_PATH=./resources/cc-rules
MURMURE_DICTIONARY='["John Doe", "Jane Smith"]'  # Optional

# TTS (Text-To-Speech) Configuration (Optional)
MURMURE_TTS_MODEL_PATH=./resources/tts
MURMURE_TTS_SAMPLE_RATE=22050  # Default: 22050
MURMURE_TTS_SPEAKER_ID=0  # Optional, for multi-voice models

# Server Configuration
MURMURE_GRPC_PORT=50051
MURMURE_LOG_LEVEL=info
```

### 3. Build and Run the Server

```bash
cd murmure-server
cargo build --release
```

Run the server:

```bash
# Load environment variables and run
export $(cat ../.env | xargs)
./target/release/murmure-server
```

Or use environment variables directly:

```bash
export MURMURE_MODEL_PATH=./resources/stt/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=./resources/cc-rules
./target/release/murmure-server
```

The server will start and listen on port 50051 (or your configured port).

### 4. Run Example Clients

#### Rust Streaming Client (Recommended)

Toggle recording mode: press SPACE to start, press again to stop and transcribe.

```bash
cd examples
cargo run --example rust_streaming_client
```

#### Rust Recording Client

Record audio from microphone for a fixed duration:

```bash
cd examples
cargo run --example rust_record_client -- --duration 5
```

#### Rust File Client

Transcribe audio files from disk:

```bash
cd examples
cargo run --example rust_file_client -- audio.wav
```

#### Python Client

```bash
cd examples
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/murmure.proto
python python_client.py audio.wav
```

## Docker Deployment

### Using Docker Compose (Recommended)

1. Ensure you have the Parakeet model in `resources/stt/parakeet-tdt-0.6b-v3-int8/`
2. Ensure you have cc-rules in `resources/cc-rules/`

```bash
docker-compose up
```

The server will start on port 50051.

### Using Dockerfile

```bash
docker build -t murmure-server .
docker run -p 50051:50051 \
  -e MURMURE_MODEL_PATH=/app/resources/stt/parakeet-tdt-0.6b-v3-int8 \
  -e MURMURE_CC_RULES_PATH=/app/resources/cc-rules \
  -e MURMURE_GRPC_PORT=50051 \
  murmure-server
```

## Configuration

### Environment Variables

#### Speech-To-Text (STT) Configuration

- `MURMURE_MODEL_PATH` - Path to Parakeet model directory (required for STT)
- `MURMURE_CC_RULES_PATH` - Path to cc-rules directory (required for STT)
- `MURMURE_DICTIONARY` - JSON array of custom dictionary words (optional)
  - Example: `MURMURE_DICTIONARY='["John Doe", "Jane Smith"]'`

#### Text-To-Speech (TTS) Configuration

- `MURMURE_TTS_MODEL_PATH` - Path to Piper model directory (optional, for TTS)
  - Should point to directory containing `.onnx` and `.onnx.json` files
  - Example: `MURMURE_TTS_MODEL_PATH=./resources/tts`
- `MURMURE_TTS_SAMPLE_RATE` - Audio sample rate in Hz (default: 22050)
- `MURMURE_TTS_SPEAKER_ID` - Speaker ID for multi-voice models (optional)
  - Use `0` for single-voice models or to select first voice

#### Server Configuration

- `MURMURE_GRPC_PORT` - gRPC server port (default: 50051)
- `MURMURE_LOG_LEVEL` - Logging level (default: info)

### Config File (Optional)

Create `config.json`:

```json
{
  "model_path": "/path/to/resources/stt/parakeet-tdt-0.6b-v3-int8",
  "cc_rules_path": "/path/to/resources/cc-rules",
  "dictionary": ["word1", "word2"],
  "tts": {
    "model_path": "/path/to/resources/tts",
    "sample_rate": 22050,
    "speaker_id": 0
  },
  "grpc_port": 50051,
  "log_level": "info"
}
```

## Supported Languages

Bulgarian (bg), Croatian (hr), Czech (cs), Danish (da), Dutch (nl), English (en), Estonian (et), Finnish (fi), French (fr), German (de), Greek (el), Hungarian (hu), Italian (it), Latvian (lv), Lithuanian (lt), Maltese (mt), Polish (pl), Portuguese (pt), Romanian (ro), Slovak (sk), Slovenian (sl), Spanish (es), Swedish (sv), Russian (ru), Ukrainian (uk)

## 📚 Documentation

### Server Documentation

- **[Server Guide](docs/SERVER.md)** - Complete guide to the standalone gRPC server, including setup, configuration, Docker deployment, and API reference
- **[Quick Start Guide](docs/QUICKSTART.md)** - Get the server running in 5 minutes
- **[gRPC Clients Guide](docs/GRPC_CLIENTS.md)** - How to generate gRPC clients in Node.js, Python, Rust, Go, Java, C# and more

### Example Clients

- **[Examples Overview](docs/examples/README.md)** - Overview of all available client examples
- **[Rust Recording Client](docs/examples/README_RUST_CLIENT.md)** - Record audio from microphone and transcribe
- **[Rust Streaming Client](docs/examples/README_STREAMING_CLIENT.md)** - Toggle recording for conversational transcription
- **[Rust File Client](docs/examples/README_RUST_FILE_CLIENT.md)** - Transcribe audio files from disk
- **[Quick Usage Guide](docs/examples/USAGE.md)** - Quick reference for all example clients

### Original Project

- **[Official Murmure Repository](https://github.com/Kieirra/murmure)** - The original desktop application

## Technology

### Speech-To-Text (STT)

Murmure uses NVIDIA's Parakeet TDT, a highly optimized, experimental transformer-based speech recognition model designed for low-latency, on-device inference. It combines fast transcription with strong accuracy across multiple languages, running efficiently on consumer GPUs or CPUs.

### Text-To-Speech (TTS)

Murmure uses [Piper TTS](https://github.com/rhasspy/piper), a fast, local neural text-to-speech system. Piper provides high-quality voice synthesis with low latency, running entirely on-device for complete privacy. Models are available for many languages and voices from the [Piper Voices Repository](https://huggingface.co/rhasspy/piper-voices).

### Unified Core Library

The project uses a unified `murmure-core` crate that provides both STT and TTS functionality:
- `murmure-core/src/stt/` - Speech-to-text modules
- `murmure-core/src/tts/` - Text-to-speech modules

This unified structure allows for easy integration of both capabilities in your applications.

## License

Murmure is free and open source, released under the GNU GPL v3 License.
You can inspect, modify, and redistribute it freely as long as derivative works remain open source.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Acknowledgments

- Thanks to [NVIDIA](https://www.nvidia.com/) for the Parakeet TDT model
- Thanks to [Piper TTS](https://github.com/rhasspy/piper) and the [Piper Voices Repository](https://huggingface.co/rhasspy/piper-voices) for TTS models
- Thanks to the [original Murmure project](https://github.com/Kieirra/murmure) for the excellent desktop application
