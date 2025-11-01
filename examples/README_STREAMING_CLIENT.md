# Rust Streaming Conversation Client

A continuous streaming client that records audio in chunks and transcribes them in real-time, creating a conversation-like experience.

## Features

- 🎤 Continuous audio recording in configurable chunks
- 📡 Real-time transcription using streaming gRPC
- 💬 Displays partial and final transcriptions as they arrive
- 📝 Accumulates conversation transcript
- 🛑 Graceful shutdown with Ctrl+C

## Prerequisites

- Rust 1.75+ installed
- Murmure gRPC server running
- Microphone connected and accessible
- **macOS**: Microphone permission granted to terminal app

## Setup

### Build

```bash
cd examples
cargo build --example rust_streaming_client
```

### Start the Server

In a separate terminal:

```bash
cd ../src-tauri
export MURMURE_MODEL_PATH=../resources/parakeet-tdt-0.6b-v3-int8
export MURMURE_CC_RULES_PATH=../resources/cc-rules
cargo run --bin murmure-server
```

## Usage

### Basic Usage

```bash
cd examples
cargo run --example rust_streaming_client
```

### Custom Options

```bash
# Custom chunk duration (default: 2 seconds)
cargo run --example rust_streaming_client -- --chunk-duration 3

# Custom server address
cargo run --example rust_streaming_client -- \
  --server http://localhost:50052 \
  --chunk-duration 2
```

### Options

- `--server <address>` - Server address (default: http://localhost:50051)
- `--chunk-duration <seconds>` - Duration of each audio chunk (default: 2)

Press **Ctrl+C** to stop recording and see the full conversation transcript.

## How It Works

1. **Records audio in chunks** - Each chunk is a few seconds of audio (default: 2 seconds)
2. **Sends chunk to server** - Uses `TranscribeStream` RPC for each chunk
3. **Receives transcriptions** - Shows partial text (if available) and final transcription
4. **Accumulates conversation** - Builds up a complete transcript of the conversation
5. **Displays on exit** - Shows full conversation transcript when you stop with Ctrl+C

## Example Output

```
🎙️  Murmure Streaming Conversation Client
Server: http://localhost:50051
Chunk duration: 2 seconds

📱 Device: MacBook Pro Microphone
   Sample rate: 48000 Hz
   Channels: 1

📡 Connecting to server...
✅ Connected to server

🎤 Starting streaming conversation...
   Recording in 2 second chunks
   Press Ctrl+C to stop

✅ Chunk 1: Hello, this is the first part of the conversation.
✅ Chunk 2: And this is the second part.
✅ Chunk 3: The conversation continues in real-time.

^C
🛑 Stopping streaming conversation...

📝 Conversation transcript:
Hello, this is the first part of the conversation. And this is the second part. The conversation continues in real-time.
```

## Chunk Duration

The `--chunk-duration` option controls how long each audio segment is:

- **Shorter chunks (1-2 seconds)**: More frequent updates, lower latency
- **Longer chunks (3-5 seconds)**: Better transcription accuracy, less network overhead

**Recommended**: Start with 2 seconds and adjust based on your needs.

## Microphone Permission (macOS)

See [README_RUST_CLIENT.md](README_RUST_CLIENT.md#microphone-permission-macos) for detailed instructions.

Quick fix:
```bash
./examples/fix-mic-permission.sh
```

## Differences from Other Clients

| Client | Use Case | Mode |
|--------|----------|------|
| `rust_record_client` | Single recording session | File-based transcription |
| `rust_file_client` | Transcribe existing audio files | File or streaming RPC |
| `rust_streaming_client` | Continuous conversation | **Streaming chunks** |

## Troubleshooting

### Silent Audio

If you see empty transcriptions:
- Check microphone permission (macOS)
- Verify microphone is not muted
- Check audio input levels in system settings

See [README_RUST_CLIENT.md](README_RUST_CLIENT.md#troubleshooting) for more details.

### Connection Issues

```
❌ Failed to start stream: ...
```

**Solution**: Ensure the server is running and accessible at the specified address.

### High Latency

If transcription feels slow:
- Try shorter chunk durations (e.g., `--chunk-duration 1`)
- Check network latency to server
- Consider running server locally

## Advanced Usage

### Integration Ideas

- **Voice assistant**: Combine with LLM for Q&A
- **Meeting transcription**: Record and transcribe meetings
- **Live captioning**: Display transcriptions in real-time
- **Voice notes**: Continuous note-taking from speech

### Customization

The client can be extended to:
- Save transcripts to file automatically
- Add timestamps to each chunk
- Filter or process transcriptions before display
- Integrate with other services (databases, APIs, etc.)

## See Also

- [Rust Recording Client](README_RUST_CLIENT.md) - Single recording session
- [Rust File Client](README_RUST_FILE_CLIENT.md) - File-based transcription
- [Main Server README](../README_SERVER.md) - Server setup and API docs

