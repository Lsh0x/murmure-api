# Fix clippy warnings

**Type:** enhancement  
**Status:** in_progress  
**Created:** 2025-11-08  
**Priority:** medium

---

## 🧠 Context

Post-merge code analysis identified several clippy warnings that should be addressed to maintain code quality and enable stricter linting:

1. **Redundant import** - Single component path import in examples
2. **Unused constant** - `MODEL_FILENAME` constant in `murmure-lib/src/model.rs` is never used
3. **Module inception** - Module `engine/mod.rs` has the same name as its containing module

These warnings prevent enabling `-D warnings` in CI/CD pipelines and reduce code quality standards.

## 🎯 Goal

Fix all clippy warnings to enable strict linting (`-D warnings`) in the build process, improving code quality and catching potential issues early.

## 📏 Success Metrics
- [ ] All clippy warnings resolved
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes without errors
- [ ] CI/CD can enforce strict clippy checks

## 🧩 Acceptance Criteria
- [ ] Redundant import removed or fixed
- [ ] Unused `MODEL_FILENAME` constant either used or removed
- [ ] Module inception issue resolved (rename module or restructure)
- [ ] All tests pass after changes
- [ ] No behavior regressions

## 🛠️ Implementation Outline
1. Create/switch to branch `refactor/fix-clippy-warnings`
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` to identify all issues
3. Fix redundant import in examples
4. Remove or use `MODEL_FILENAME` constant in `murmure-lib/src/model.rs`
5. Resolve module inception in `murmure-lib/src/engine/mod.rs` (rename or restructure)
6. Verify all clippy checks pass
7. Run full test suite to ensure no regressions
8. Move this file to `in_progress/` then `done/`
9. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Suppress warnings with attributes** → Not recommended, better to fix root causes
- **Lower clippy strictness** → Reduces code quality benefits

## ⚠️ Risks / Mitigations
- **Risk**: Module restructuring might break imports → **Mitigation**: Run full test suite, check all imports
- **Risk**: Removing unused constant might be needed later → **Mitigation**: Check git history and comments to understand original intent

## 🔗 Discussion Notes

Clippy errors found:
```
error: this import is redundant
error: constant `MODEL_FILENAME` is never used
error: module has the same name as its containing module
```

These are pre-existing issues but should be fixed to enable strict linting.

