<!-- Repo-Specific Agent Rules Template -->
# Repository Agent Guide

## Overview

- **Repository Name**: murmure-api
- **Purpose**: A standalone gRPC server implementation of the Murmure speech-to-text application. Provides real-time transcription capabilities via gRPC API with bidirectional streaming support, custom dictionary support, and Docker-ready deployment. All processing happens locally on-device with no data leaving the server.
- **Primary Owners**: Lsh0x (based on git remote)
- **Tech Stack**: Rust (1.75+), gRPC, Protocol Buffers, ONNX Runtime (Parakeet model), Docker

## Alignment With Global Rules

- This repository inherits the global rules from `~/.cursor/AGENT_GLOBAL.md`.
- Deviations or extensions MUST be documented below with rationale.
- Artifact storage is discovered via `~/.flowmates/config.json` → `{flowmates_repo}/projects/{repo-identifier}/` (centralized in flowmates repository).

## Repository Layout

| Path | Description | Notes |
| --- | --- | --- |
| `murmure-stt/` | Core library with transcription engine, audio processing, dictionary, and model management | Shared library used by server and examples |
| `murmure-server/` | gRPC server implementation | Main server crate with gRPC handlers |
| `examples/` | Runnable usage examples and clients | Rust clients demonstrating API usage |
| `proto/` | Protocol Buffer definitions | gRPC service and message definitions |
| `resources/` | Model files and configuration | Parakeet ONNX model, cc-rules for phonetic correction |
| `docs/` | Extended documentation, guides | Server docs, build status, client examples |
| `tests/` | Integration/system tests | API tests, voice test files |
| `scripts/` | Git hooks and workflow validation scripts | Pre-commit hooks, workflow state validation |

## Domain Conventions

- **Privacy First**: Never store user data except for processing. All transcription happens locally.
- **Security**: No compromises, no open CORS, no unsafe shortcuts.
- **Clean Code**: Follow SRP and SOLID principles, avoid duplication.
- **Simplicity**: Prefer minimal, understandable solutions over over-engineered features.
- **Error Handling**: Use Rust's Result types, provide clear error messages in gRPC responses.
- **Audio Format**: WAV format, 16kHz, mono, 16-bit PCM (standardized for Parakeet model).
- **Model Management**: Parakeet ONNX model must be available at runtime (configured via `MURMURE_MODEL_PATH`).

## Build & Test Commands

- `cargo build --release` - Build all workspace crates in release mode
- `cargo test --workspace` - Run all tests across workspace
- `cargo fmt --all` - Format all Rust code
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` - Lint with clippy
- `docker-compose up` - Start server via Docker (requires model in resources/)
- `cd murmure-server && cargo run` - Run server locally (requires environment variables)

## Documentation Touchpoints

- `README.md`: Quick start, architecture synopsis, deployment instructions
- `README_SERVER.md`: Server-specific documentation
- `docs/SERVER.md`: Detailed server configuration and API documentation
- `docs/GRPC_CLIENTS.md`: Client implementation guides
- `docs/QUICKSTART.md`: Quick start guide
- `CONTRIBUTING.md`: Development principles and contribution guidelines
- `proto/murmure.proto`: gRPC service and message definitions (source of truth for API)

## Repo-Specific Agents

Define any additional agents or overrides. Extend the global agent list using the same JSON structure. Reference global agents when behaviour changes.

```json
{
  "agents": [
    {
      "id": "murmure-transcription-agent",
      "name": "Murmure Transcription Agent",
      "model": "auto",
      "context": [
        "Deep expertise in speech-to-text transcription, audio processing, and gRPC streaming.",
        "Understands Parakeet model integration, audio format requirements (16kHz mono 16-bit WAV), and dictionary-based phonetic correction.",
        "Familiar with ONNX runtime, gRPC bidirectional streaming patterns, and Rust async/await patterns."
      ],
      "files": [
        "./proto/murmure.proto",
        "./docs/SERVER.md",
        "./murmure-stt/src/engine/",
        "./murmure-server/src/server/grpc.rs"
      ],
      "capabilities": [
        "review-transcription-logic",
        "validate-audio-format-handling",
        "assess-grpc-streaming-patterns",
        "review-dictionary-integration"
      ],
      "triggers": [
        "user-request: transcription-feature",
        "pre-merge: audio-processing-change",
        "pre-merge: grpc-api-change"
      ]
    }
  ]
}
```

### Agent Overrides

- **Example**: Override `tester-agent` to enforce repository-specific test requirements.

```json
{
  "overrides": [
    {
      "id": "tester-agent",
      "additionalContext": [
        "Always test with actual audio files in tests/voices/ before code merges.",
        "Verify gRPC streaming behavior with examples/rust_streaming_client.rs.",
        "Ensure model path configuration is validated at startup."
      ],
      "additionalTriggers": [
        "pre-release",
        "audio-format-change"
      ]
    }
  ]
}
```

## Exceptions to Global Rules

- **Model Files**: Large ONNX model files in `resources/parakeet-tdt-0.6b-v3-int8/` are gitignored but required for runtime. Document download instructions clearly.
- **Privacy Constraint**: No telemetry, no external API calls, no data persistence beyond in-memory processing. This is a hard requirement.

## Onboarding Checklist

- [ ] Clone repository and run bootstrap script / `init-repo-agent`.
- [ ] Download Parakeet model to `resources/parakeet-tdt-0.6b-v3-int8/` (see README.md).
- [ ] Ensure `cc-rules` are available in `resources/cc-rules/`.
- [ ] Configure environment variables (`MURMURE_MODEL_PATH`, `MURMURE_CC_RULES_PATH`, `MURMURE_GRPC_PORT`).
- [ ] Run full test suite: `cargo test --workspace`.
- [ ] Test server locally: `cd murmure-server && cargo run`.
- [ ] Test with example clients: `cd examples && cargo run --example rust_client`.
- [ ] Read `AGENT.md`, `README.md`, `docs/SERVER.md`, and `CONTRIBUTING.md`.

## Appendix

- **Issue Tracker**: Uses flowmates workflow system (`issues/murmure-api/`)
- **CI/CD**: Check `docs/BUILD_STATUS.md` for build and deployment status
- **Model Source**: [Murmure Model Releases](https://github.com/Kieirra/murmure-model/releases)
- **Original Desktop App**: [Murmure Desktop Repository](https://github.com/Kieirra/murmure)
- **Protocol Buffers**: gRPC service defined in `proto/murmure.proto`
- **Docker Deployment**: `docker-compose.yml` for containerized deployment

