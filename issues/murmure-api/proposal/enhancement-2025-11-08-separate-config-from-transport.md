# Separate transport-layer config from library config

**Type:** enhancement  
**Status:** proposal  
**Created:** 2025-11-08  
**Priority:** low

---

## 🧠 Context

After cleaning up the library separation, `ServerConfig` in `murmure-lib` still contains transport-layer specific fields:
- `grpc_port: u16` - gRPC-specific configuration
- `log_level: String` - Logging configuration (transport-layer concern)

These fields are only used by `murmure-server`, not by the library itself. The library code doesn't reference `grpc_port` or `log_level` - they're only used in `murmure-server/src/main.rs`.

This creates an architectural leak where transport-layer concerns are mixed into the library's configuration structure.

## 🎯 Goal

Separate transport-layer configuration from library configuration to achieve true separation of concerns. The library should only contain configuration relevant to transcription (model paths, dictionary, etc.), while transport-layer config (ports, logging) should be in the server crate.

## 📏 Success Metrics
- [ ] `ServerConfig` in library only contains transcription-related fields
- [ ] Transport-layer config moved to server crate
- [ ] Library remains pure (no transport dependencies)
- [ ] Server continues to work correctly
- [ ] Backward compatibility maintained or migration path provided

## 🧩 Acceptance Criteria
- [ ] Create `TranscriptionConfig` in library (model_path, cc_rules_path, dictionary)
- [ ] Create `ServerConfig` in server crate (grpc_port, log_level, + TranscriptionConfig)
- [ ] Update library code to use `TranscriptionConfig`
- [ ] Update server code to use new `ServerConfig`
- [ ] All tests pass
- [ ] Documentation updated

## 🛠️ Implementation Outline
1. Create/switch to branch `refactor/separate-config-types`
2. Create `TranscriptionConfig` in `murmure-lib/src/config.rs`:
   - `model_path: Option<PathBuf>`
   - `cc_rules_path: Option<PathBuf>`
   - `dictionary: Vec<String>`
3. Move `ServerConfig` to `murmure-server/src/config.rs`:
   - `transcription: TranscriptionConfig`
   - `grpc_port: u16`
   - `log_level: String`
4. Update library code to use `TranscriptionConfig`
5. Update server code to use new `ServerConfig`
6. Update environment variable handling
7. Run tests and verify everything works
8. Move this file to `in_progress/` then `done/`
9. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Keep as-is** → Acceptable but creates architectural debt
- **Use composition** → ServerConfig contains TranscriptionConfig (recommended)
- **Separate crates** → Overkill for this use case

## ⚠️ Risks / Mitigations
- **Risk**: Breaking change for existing users → **Mitigation**: Provide migration guide, consider version bump
- **Risk**: More complex configuration → **Mitigation**: Keep API simple, use composition pattern
- **Risk**: Environment variable handling complexity → **Mitigation**: Server can handle all env vars and pass TranscriptionConfig to library

## 🔗 Discussion Notes

Current state:
- `ServerConfig` in library has `grpc_port` and `log_level`
- Library code doesn't use these fields
- Server code uses them for gRPC setup and logging

Proposed structure:
- `TranscriptionConfig` (library) - Pure transcription config
- `ServerConfig` (server) - Contains `TranscriptionConfig` + transport config

This would make the library truly transport-agnostic.

