# Clean up duplicate issue file in in_progress

**Type:** chore  
**Status:** done  
**Created:** 2025-11-08  
**Priority:** low

---

## 🧠 Context

After merging multiple PRs, there are several leftover duplicate issue files across different directories. These duplicates create confusion and clutter in the issue tracking system:

**Current duplicates identified:**
1. `enhancement-2025-11-08-rename-lib-to-tts.md` - exists in `todo/`, `in_progress/`, and `done/`
2. `enhancement-2025-11-08-refactor-murmure-lib-to-tts.md` - exists in `todo/` and `done/` (in_progress version removed in post-merge cleanup)
3. `enhancement-2025-11-08-fix-clippy-warnings.md` - exists in `todo/`, `proposal/`, and `done/`
4. `enhancement-2025-11-08-cleanup-lib-separation.md` - exists in `todo/` and `done/`

These files should have been removed when issues were moved to `done/`, but duplicates were accidentally committed during merges. This creates confusion about which version is authoritative.

## 🎯 Goal

Remove all duplicate issue files to maintain clean issue tracking state. For each completed issue, only the `done/` version should remain. All other duplicates (in `todo/`, `in_progress/`, `proposal/`) should be removed.

## 📏 Success Metrics
- [ ] All duplicate issue files identified and catalogued
- [ ] Duplicate files removed (keep only `done/` versions for completed issues)
- [ ] Only authoritative versions of issue files remain
- [ ] Repository state is clean and unambiguous

## 🧩 Acceptance Criteria
- [x] All duplicate issue files identified:
  - `enhancement-2025-11-08-rename-lib-to-tts.md` - removed from `todo/` and `in_progress/` ✅
  - `enhancement-2025-11-08-refactor-murmure-lib-to-tts.md` - removed from `todo/` and `in_progress/` ✅
  - `enhancement-2025-11-08-fix-clippy-warnings.md` - removed from `todo/` and `proposal/` ✅
  - `enhancement-2025-11-08-cleanup-lib-separation.md` - removed from `todo/` ✅
- [x] Only `done/` versions remain for completed issues ✅
- [x] Git history preserved (file deletions committed) ✅
- [x] No broken references or confusion about issue state ✅

## 🛠️ Implementation Steps
1. Create/switch to branch `chore/cleanup-duplicate-issue-files`
2. Identify all duplicate issue files by comparing across directories
3. For each completed issue, remove duplicates from `todo/`, `in_progress/`, and `proposal/`
4. Keep only the `done/` version for completed issues
5. Commit deletions: `git commit -m "chore: remove duplicate issue files"`
6. Verify no broken references remain
7. Move this file to `in_progress/` then `done/`
8. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Keep both files** → Creates confusion and clutter
- **Move to archive** → Overkill for a simple duplicate

## ⚠️ Risks / Mitigations
- **Risk**: Accidentally deleting the wrong file → **Mitigation**: Verify file paths carefully, ensure `done/` version remains
- **Risk**: Breaking issue tracking → **Mitigation**: This is just cleanup, the actual issue is already in `done/`

## 🔗 Discussion Notes

**Post-merge analysis findings:**
- PR #8 merge: duplicate `enhancement-2025-11-08-rename-lib-to-tts.md` in `in_progress/`
- PR #9 merge: duplicate `enhancement-2025-11-08-refactor-murmure-lib-to-tts.md` in `in_progress/` (removed)
- Multiple issues have duplicates across directories
- This is a systematic cleanup task to maintain clean issue tracking

**Related:**
- PR #8: refactor: rename murmure-lib to murmure-stt
- PR #9: refactor: remove leftover murmure-tts directory
- Issues with duplicates: rename-lib-to-tts, refactor-murmure-lib-to-tts, fix-clippy-warnings, cleanup-lib-separation

