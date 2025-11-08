# Refactor murmure-lib to murmure-tts

**Type:** enhancement  
**Status:** in_progress  
**Created:** 2025-11-08  
**Priority:** medium

---

## 🧠 Context

The codebase has been refactored and `murmure-lib/` has already been renamed to `murmure-stt/` (Speech-To-Text), which is the correct and active library crate. However, there's a leftover `murmure-tts/` directory that exists but is not part of the workspace, creating confusion.

**Current state (verified):**
- `murmure-lib/` does NOT exist (already removed/renamed)
- `murmure-stt/` is the active, working library crate (in workspace, used by server)
- `murmure-tts/` exists as a leftover directory (NOT in workspace, duplicate of murmure-stt)

**Note on naming:** While "TTS" typically means Text-To-Speech (synthesis), this library performs Speech-To-Text (transcription/STT). The naming should be clarified, but for this refactoring, we'll consolidate under `murmure-tts` as requested.

## 🎯 Goal

Clean up the leftover `murmure-tts/` directory to eliminate confusion. The active library is correctly named `murmure-stt/` (Speech-To-Text), and `murmure-tts/` is a duplicate leftover that should be removed.

Tasks:
1. Remove the leftover `murmure-tts/` directory
2. Verify no references to `murmure-tts` exist in the codebase
3. Update documentation if needed
4. Ensure workspace remains clean and functional

## 📏 Success Metrics
- [ ] `murmure-tts/` is a complete, working crate with all necessary source code
- [ ] `murmure-lib/` directory is removed or properly integrated
- [ ] All references to `murmure-lib` updated to `murmure-tts`
- [ ] Workspace `Cargo.toml` includes `murmure-tts` (or appropriate name)
- [ ] All builds pass
- [ ] All tests pass

## 🧩 Acceptance Criteria
- [ ] `murmure-tts/` has complete source code structure (src/lib.rs, all modules)
- [ ] `murmure-tts/Cargo.toml` is properly configured with all dependencies
- [ ] `murmure-lib/` directory removed or integrated
- [ ] Workspace `Cargo.toml` updated to include `murmure-tts`
- [ ] `murmure-server/Cargo.toml` updated to depend on `murmure-tts`
- [ ] All import statements updated (`use murmure_lib::` → `use murmure_tts::` or appropriate)
- [ ] All documentation references updated
- [ ] Examples updated if they reference the library
- [ ] All builds pass: `cargo build --workspace`
- [ ] All tests pass: `cargo test --workspace`
- [ ] No behavior regressions

## 🛠️ Implementation Outline
1. Create/switch to branch `refactor/murmure-lib-to-tts`
2. **Analyze current state:**
   - Review `murmure-lib/` source files and determine what needs to be moved
   - Review `murmure-tts/Cargo.toml` and ensure it's properly configured
   - Compare with `murmure-stt/` to understand what's already implemented
3. **Consolidate code:**
   - Move/merge source files from `murmure-lib/` to `murmure-tts/src/`
   - Ensure `murmure-tts/src/lib.rs` properly exports all public APIs
   - Update `murmure-tts/Cargo.toml` with all necessary dependencies
4. **Update workspace:**
   - Update root `Cargo.toml` to include `murmure-tts` in members
   - Update `murmure-server/Cargo.toml` to depend on `murmure-tts`
   - Update examples if they reference the library
5. **Update imports:**
   - Search and replace all `use murmure_lib::` → `use murmure_tts::`
   - Update any other references to `murmure-lib`
6. **Update documentation:**
   - README.md
   - README_SERVER.md
   - docs/SERVER.md
   - AGENT.md
   - Any other relevant docs
7. **Clean up:**
   - Remove `murmure-lib/` directory if fully integrated
   - Verify no orphaned files remain
8. **Verify:**
   - Run `cargo build --workspace` to ensure everything compiles
   - Run `cargo test --workspace` to ensure all tests pass
   - Check for any remaining references to old names
9. Move this file to `in_progress/` then `done/`
10. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Keep `murmure-lib` and fix it** → But it's incomplete and confusing
- **Merge into `murmure-stt`** → But user specifically requested `murmure-tts`
- **Delete `murmure-lib` entirely** → But it may contain unique code that needs to be preserved
- **Keep both directories** → Creates confusion and maintenance burden

## ⚠️ Risks / Mitigations
- **Risk**: Missing code during merge → **Mitigation**: Carefully compare all files, check git history
- **Risk**: Breaking changes for existing code → **Mitigation**: Run full test suite, verify all imports
- **Risk**: Dependency conflicts → **Mitigation**: Review Cargo.toml files, ensure compatibility
- **Risk**: Name confusion (TTS vs STT) → **Mitigation**: Document the naming decision, consider future rename to `murmure-stt` if needed
- **Risk**: Git history loss → **Mitigation**: Use `git mv` where possible, preserve history

## 🔗 Discussion Notes

**Current directory structure:**
- `murmure-lib/` - Partial source code (audio.rs, engine/, model.rs), no Cargo.toml
- `murmure-tts/` - Only Cargo.toml, no source code
- `murmure-stt/` - Complete, working library crate

**Files to review/merge:**
- `murmure-lib/src/audio.rs`
- `murmure-lib/src/engine/mod.rs`
- `murmure-lib/src/engine/timestamp.rs`
- `murmure-lib/src/model.rs`

**Naming consideration:**
- The library performs Speech-To-Text (STT), not Text-To-Speech (TTS)
- Consider whether `murmure-tts` is the right name, or if `murmure-stt` would be more accurate
- For now, proceeding with `murmure-tts` as requested, but this should be discussed

**Integration strategy:**
- Compare `murmure-lib/` files with `murmure-stt/` to identify unique code
- Merge unique code into `murmure-tts/`
- Ensure no functionality is lost during the merge

