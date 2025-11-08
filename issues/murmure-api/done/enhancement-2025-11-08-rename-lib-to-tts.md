# Rename murmure-lib to murmure-stt

**Type:** enhancement  
**Status:** done  
**Created:** 2025-11-08  
**Priority:** medium

---

## 🧠 Context

The library crate is currently named `murmure-lib`, which is generic and doesn't clearly indicate its purpose. Renaming it to `murmure-tts` (Text-To-Speech, though it's actually Speech-To-Text) would better reflect its core functionality as a transcription library.

However, "TTS" typically means Text-To-Speech (synthesis), while this library does Speech-To-Text (transcription/STT). We should consider:
- `murmure-tts` - Common but potentially confusing (TTS = synthesis)
- `murmure-stt` - More accurate (STT = Speech-To-Text)
- `murmure-transcribe` - Clear and descriptive
- `murmure-core` - Generic but indicates core functionality

**Note**: Need to decide on the final name. For now, using `murmure-tts` as requested, but this should be discussed.

## 🎯 Goal

Rename the `murmure-lib` crate to a more descriptive name that better reflects its purpose as a transcription library. This will improve clarity and make the codebase more maintainable.

## 📏 Success Metrics
- [ ] Crate renamed successfully
- [ ] All references updated (Cargo.toml, imports, documentation)
- [ ] All builds pass
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Examples updated

## 🧩 Acceptance Criteria
- [ ] `murmure-lib/` directory renamed to new name
- [ ] `Cargo.toml` package name updated
- [ ] All `Cargo.toml` dependencies updated (murmure-server, examples)
- [ ] All import statements updated (`use murmure_lib::` → `use murmure_tts::`)
- [ ] All documentation references updated
- [ ] README files updated
- [ ] AGENT.md updated
- [ ] All builds pass
- [ ] All tests pass

## 🛠️ Implementation Steps
1. Create/switch to branch `refactor/rename-lib-to-tts`
2. **Decide on final name** (murmure-tts, murmure-stt, murmure-transcribe, or murmure-core)
3. Rename directory: `mv murmure-lib/ murmure-{new-name}/`
4. Update `murmure-{new-name}/Cargo.toml`:
   - Change `name = "murmure-lib"` to `name = "murmure-{new-name}"`
   - Update `lib.name` if needed
5. Update workspace `Cargo.toml`:
   - Change `members` entry
6. Update `murmure-server/Cargo.toml`:
   - Change dependency `murmure-lib = { path = "../murmure-lib" }` to new name
7. Update all import statements:
   - `use murmure_lib::` → `use murmure_{new_name}::`
   - Search and replace across codebase
8. Update documentation:
   - README.md
   - README_SERVER.md
   - docs/SERVER.md
   - AGENT.md
   - Any other docs
9. Update examples if they reference the library name
10. Verify builds: `cargo build --workspace`
11. Verify tests: `cargo test --workspace`
12. Move this file to `in_progress/` then `done/`
13. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Keep `murmure-lib`** → Generic, doesn't indicate purpose
- **`murmure-tts`** → Common but TTS = synthesis (confusing)
- **`murmure-stt`** → Accurate (STT = Speech-To-Text) but less common
- **`murmure-transcribe`** → Clear and descriptive
- **`murmure-core`** → Generic but indicates core functionality

## ⚠️ Risks / Mitigations
- **Risk**: Breaking change for external users → **Mitigation**: This is likely internal-only, but document the change
- **Risk**: Missed references → **Mitigation**: Use search/replace, verify with builds
- **Risk**: Git history loss → **Mitigation**: Use `git mv` to preserve history
- **Risk**: Name confusion (TTS vs STT) → **Mitigation**: Discuss and decide on best name before implementation

## 🔗 Discussion Notes

**Name Decision Needed:**
- `murmure-tts` - Requested but potentially confusing (TTS = Text-To-Speech synthesis)
- `murmure-stt` - More accurate (STT = Speech-To-Text transcription)
- `murmure-transcribe` - Clear and descriptive
- `murmure-core` - Generic but safe

**Files to update:**
- `Cargo.toml` (workspace, library, server, examples)
- All Rust source files with imports
- All documentation files
- AGENT.md

**Git considerations:**
- Use `git mv` to preserve file history
- Consider creating a migration guide if external users exist

