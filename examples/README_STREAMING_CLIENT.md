# Rust Push-to-Talk Streaming Client

A push-to-talk client: press and hold **SPACE** to record, release to stop and transcribe. Perfect for precise control over recording.

## Features

- 🎤 **Push-to-Talk**: Press and hold SPACE to record
- 🛑 **Release to transcribe**: Release SPACE to stop and transcribe
- 📡 Real-time transcription using streaming gRPC
- 📝 Accumulates full conversation transcript
- ⌨️ Simple keyboard controls (SPACE for record, Ctrl+C to exit)

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
# Custom server address
cargo run --example rust_streaming_client -- --server http://localhost:50052
```

### Options

- `--server <address>` - Server address (default: http://localhost:50051)

### Controls

- **Hold SPACE** - Start recording audio
- **Release SPACE** - Stop recording and transcribe
- **Ctrl+C** - Exit and show full conversation transcript
- **ESC** - Exit immediately

## How It Works

1. **Press SPACE** - Starts recording audio from your microphone
2. **Hold SPACE** - Continues recording while you speak
3. **Release SPACE** - Stops recording and sends audio to server
4. **Transcription** - Server processes and returns transcription
5. **Repeat** - Press SPACE again for the next recording
6. **Exit** - Press Ctrl+C to see full conversation transcript

## Example Output

```
🎙️  Murmure Push-to-Talk Streaming Client
Server: http://localhost:50051

📱 Device: MacBook Pro Microphone
   Sample rate: 48000 Hz
   Channels: 1

📡 Connecting to server...
✅ Connected to server

🎤 Push-to-Talk Mode
   Hold SPACE to record, release to transcribe
   Press Ctrl+C to exit

🎙️  Recording #1 (hold SPACE)...
   📤 Sending to server for transcription...
✅ Transcription: Hello, this is my first message.

🎙️  Recording #2 (hold SPACE)...
   📤 Sending to server for transcription...
✅ Transcription: And this is my second message.

^C
📝 Conversation transcript:
Hello, this is my first message. And this is my second message.
```

## Push-to-Talk Benefits

- **Precise control** - Record exactly what you want to say
- **No partial sentences** - Complete thoughts before transcription
- **Natural pauses** - Take breaks between recordings
- **Better accuracy** - Full sentences improve transcription quality

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

