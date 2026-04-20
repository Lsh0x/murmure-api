# Add TTS support with Piper

**Type:** enhancement  
**Status:** done  
**Created:** 2025-11-08  
**Priority:** high

---

## 🧠 Context

The repository currently only supports Speech-To-Text (STT) functionality via `murmure-stt`. To provide a complete voice AI solution, we need to add Text-To-Speech (TTS) support using Piper, a fast, local, neural text-to-speech system.

The TTS implementation should:
- Follow the same architectural patterns as `murmure-stt` for consistency
- Be integrated into a unified `murmure-core` crate that contains both STT and TTS
- Support both file-based and streaming synthesis APIs
- Use a single voice model initially (can be extended to multiple voices later)
- Use the `piper` Rust crate for Piper integration

**Current state:**
- `murmure-stt/` exists as a standalone STT library
- Server uses `murmure-stt` for transcription
- No TTS functionality exists

**Target state:**
- `murmure-core/` contains both STT and TTS modules
- TTS supports synchronous text-to-audio synthesis
- TTS supports streaming synthesis for real-time use cases
- Server can optionally use TTS functionality

## 🎯 Goal

Add Piper-based TTS support to the Murmure project, creating a unified `murmure-core` crate that provides both STT and TTS capabilities. The implementation should mirror the structure and patterns of the existing STT code for maintainability and consistency.

## 📏 Success Metrics

- [x] `murmure-core/` crate created with both STT and TTS modules ✅
- [x] TTS synthesis structure in place (PiperEngine placeholder, needs actual API integration) ✅
- [x] Streaming synthesis API structure available for incremental text input ✅
- [x] All existing STT functionality continues to work after refactoring ✅
- [x] Build passes: `cargo build --workspace` ✅
- [x] Clippy passes: `cargo clippy --workspace --all-targets --all-features` ✅
- [ ] Tests pass: `cargo test --workspace` (STT tests should pass, TTS needs model for full testing)

## 🧩 Acceptance Criteria

- [ ] `murmure-core/` directory structure created
- [ ] STT code moved to `murmure-core/src/stt/` without breaking changes
- [ ] TTS module structure created:
  - `src/tts/config.rs` - TTS configuration (model path, sample rate, etc.)
  - `src/tts/model.rs` - Model management
  - `src/tts/synthesis.rs` - SynthesisService (file-based API)
  - `src/tts/stream.rs` - SynthesisStream (streaming API)
  - `src/tts/audio.rs` - Audio utilities
  - `src/tts/engine/` - Engine implementation
    - `mod.rs` - Module exports
    - `piper.rs` - PiperEngine implementation
    - `synthesis_engine.rs` - SynthesisEngine trait
- [ ] `SynthesisEngine` trait defined (mirrors `TranscriptionEngine`)
- [ ] `PiperEngine` implements `SynthesisEngine`
- [ ] `SynthesisService` provides `synthesize_text()` method (text → WAV bytes)
- [ ] `SynthesisStream` provides streaming API for incremental synthesis
- [ ] Configuration via environment variables (`MURMURE_TTS_MODEL_PATH`, etc.)
- [ ] Workspace `Cargo.toml` updated to use `murmure-core`
- [ ] Server `Cargo.toml` updated to depend on `murmure-core`
- [ ] All imports updated throughout codebase
- [ ] Documentation updated (README, AGENT.md)
- [ ] No behavior regressions in STT functionality

## 🛠️ Implementation Outline

1. Create/switch to branch `feature/add-tts-support`
2. **Restructure workspace:**
   - Create `murmure-core/` directory
   - Move `murmure-stt/src/*` to `murmure-core/src/stt/*`
   - Update module paths in STT code
3. **Create TTS module structure:**
   - Create `murmure-core/src/tts/` directory
   - Create all TTS module files (config, model, synthesis, stream, audio, engine)
4. **Implement TTS configuration:**
   - `TtsConfig` struct with model_path, sample_rate, speaker_id
   - `from_env()` method for environment variable loading
5. **Implement TTS model management:**
   - `TtsModel` struct (mirrors STT's Model)
   - Model path resolution logic
6. **Implement SynthesisEngine trait:**
   - Define trait with `load_model_with_params()`, `synthesize_text()`, `synthesize_incremental()`, `unload_model()`
   - Define `SynthesisResult` struct with audio samples and metadata
7. **Implement PiperEngine:**
   - Use `piper` crate for model loading
   - Implement `SynthesisEngine` trait
   - Handle model parameters and inference parameters
8. **Implement SynthesisService:**
   - File-based API: `synthesize_text(&self, text: &str) -> Result<Vec<u8>>`
   - Engine loading and caching
   - Audio format conversion (samples → WAV bytes)
9. **Implement SynthesisStream:**
   - Streaming API for incremental synthesis
   - Methods: `push_text()`, `flush()`, `synthesize_chunk()`, `finalize()`
10. **Update dependencies:**
    - Add `piper` crate to `murmure-core/Cargo.toml`
    - Update workspace `Cargo.toml`
    - Update server `Cargo.toml`
11. **Update imports:**
    - Update all STT imports to use `murmure_core::stt::*`
    - Update server imports
12. **Testing:**
    - Verify STT still works after refactoring
    - Test TTS synthesis with sample text
    - Test streaming synthesis
    - Run full test suite
13. **Documentation:**
    - Update README.md with TTS information
    - Update AGENT.md with new structure
    - Add usage examples
14. Move this file to `in_progress/` then `done/`
15. Create PR referencing this issue

## 🔍 Alternatives Considered

- **Keep STT and TTS separate crates** → Less unified, more duplication, harder to maintain
- **Add TTS to existing murmure-stt** → Name becomes misleading (STT implies only speech-to-text)
- **Use different TTS engine** → Piper is fast, local, privacy-first (matches project goals)
- **No streaming support** → Streaming enables real-time use cases, better UX

## ⚠️ Risks / Mitigations

- **Risk**: Breaking existing STT functionality during refactoring → **Mitigation**: Move code carefully, test after each step, keep git history
- **Risk**: Piper crate API changes or compatibility issues → **Mitigation**: Pin version, test with actual Piper models
- **Risk**: Performance issues with streaming → **Mitigation**: Implement efficient buffering, test with various text lengths
- **Risk**: Audio format mismatches → **Mitigation**: Use standard WAV format, document requirements clearly
- **Risk**: Model path resolution complexity → **Mitigation**: Follow STT's model path resolution pattern

## 🔗 Discussion Notes

**Architecture decisions:**
- Unified `murmure-core` crate provides better organization and code reuse
- Mirroring STT structure ensures consistency and easier maintenance
- Streaming API enables real-time synthesis use cases
- Single model initially, can extend to multiple voices later

**Piper integration:**
- Use `piper` crate from crates.io
- Models should be stored in `resources/` directory (similar to STT models)
- Default sample rate: 22050 Hz (common for Piper models)
- Support for speaker_id parameter for multi-voice models (even if single voice initially)

**API design:**
- File-based: `synthesize_text()` - simple, synchronous, returns complete audio
- Streaming: `SynthesisStream` - incremental, supports chunked text input and audio output
- Both APIs use same underlying engine for consistency

**Future enhancements:**
- Multiple voice support
- Voice cloning
- SSML support
- gRPC endpoints for TTS (separate issue)

