# Clean up library separation from server

**Type:** enhancement  
**Status:** done  
**Created:** 2025-11-08  
**Priority:** medium

---

## 🧠 Context

The `murmure-lib` should be a pure library without any transport layer code (HTTP/gRPC). Currently, there are leftover directories and files from the desktop app extraction that pollute the library:

1. **`murmure-lib/src/server/murmure.rs`** - Contains generated protobuf code that shouldn't be in source (not used, not declared in lib.rs)
2. **`murmure-lib/src/http_api/`** - Empty directory, leftover from desktop app
3. **`murmure-lib/src/shortcuts/`** - Likely desktop app leftover (needs verification)

The library should only contain:
- Core transcription logic (audio, engine, model)
- Configuration management
- Dictionary/phonetic correction
- No transport layer dependencies

## 🎯 Goal

Remove all dead code and transport layer remnants from `murmure-lib` to make it a pure, reusable library that can be used by any transport layer (gRPC, HTTP, CLI, etc.).

## 📏 Success Metrics
- [ ] All dead code directories removed from `murmure-lib/src/`
- [ ] Library has no gRPC/protobuf dependencies (already verified)
- [ ] Library compiles successfully after cleanup
- [ ] Server and examples still work (they generate their own protobuf code)
- [ ] No broken imports or references

## 🧩 Acceptance Criteria
- [ ] `murmure-lib/src/server/` directory removed
- [ ] `murmure-lib/src/http_api/` directory removed
- [ ] `murmure-lib/src/shortcuts/` verified and removed if unused
- [ ] All tests pass
- [ ] Server builds and runs correctly
- [ ] Examples build and run correctly
- [ ] Documentation updated if needed

## 🛠️ Implementation Steps
1. Create/switch to branch `refactor/cleanup-lib-separation`
2. Verify `shortcuts/` is not used anywhere
3. Remove `murmure-lib/src/server/` directory
4. Remove `murmure-lib/src/http_api/` directory
5. Remove `murmure-lib/src/shortcuts/` if unused
6. Verify library compiles: `cargo build -p murmure-lib`
7. Verify server still works: `cargo build -p murmure-server`
8. Verify examples still work: `cargo build --examples`
9. Run tests: `cargo test --workspace`
10. Move this file to `in_progress/` then `done/`
11. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Keep directories for future use** → Not needed, server generates its own protobuf code
- **Move to separate crate** → Overkill, just remove dead code

## ⚠️ Risks / Mitigations
- **Risk**: Removing code might break something → **Mitigation**: Verify with builds and tests before removing
- **Risk**: Examples might depend on removed code → **Mitigation**: Examples generate their own protobuf code, should be fine

## 🔗 Discussion Notes

Current state:
- `murmure-lib/src/server/murmure.rs` - Generated protobuf code, not used (not in lib.rs)
- `murmure-lib/src/http_api/` - Empty directory
- `murmure-lib/src/shortcuts/` - Need to verify usage

The server correctly generates protobuf code in `murmure-server/build.rs` and uses it via `include!(concat!(env!("OUT_DIR"), "/murmure.rs"))`.

