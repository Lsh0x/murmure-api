# Add shell script linting

**Type:** enhancement  
**Status:** proposal  
**Created:** 2025-11-08  
**Priority:** low

---

## 🧠 Context

The repository now includes shell scripts (`scripts/pre-commit-hook`, `scripts/pre-work-hook`) for git hooks and workflow validation. These scripts should be linted to ensure quality and catch potential issues early.

Currently, `shellcheck` is not installed or integrated into the development workflow, which means shell script issues might go undetected.

## 🎯 Goal

Add shell script linting to the development workflow to improve code quality and catch potential issues in shell scripts before they cause problems.

## 📏 Success Metrics
- [ ] Shellcheck installed or available in CI
- [ ] Shell scripts pass shellcheck validation
- [ ] Linting integrated into pre-commit hook or CI pipeline
- [ ] Documentation updated with linting requirements

## 🧩 Acceptance Criteria
- [ ] Shellcheck available (via installation instructions or CI)
- [ ] All shell scripts pass shellcheck validation
- [ ] Linting added to development workflow (pre-commit or CI)
- [ ] Documentation updated (CONTRIBUTING.md or similar)

## 🛠️ Implementation Outline
1. Create/switch to branch `chore/add-shell-linting`
2. Add shellcheck to development dependencies or CI setup
3. Run shellcheck on existing scripts and fix any issues
4. Add shellcheck to pre-commit hook or CI pipeline
5. Update documentation with shellcheck installation and usage
6. Move this file to `in_progress/` then `done/`
7. Create PR referencing this issue

## 🔍 Alternatives Considered
- **Manual review only** → Less reliable, doesn't catch all issues
- **Different linter** → shellcheck is the standard for shell scripts

## ⚠️ Risks / Mitigations
- **Risk**: Shellcheck might flag false positives → **Mitigation**: Review and suppress only when appropriate
- **Risk**: Additional tool dependency → **Mitigation**: Make it optional for local dev, required in CI

## 🔗 Discussion Notes

Shell scripts added:
- `scripts/pre-commit-hook` - Git pre-commit hook
- `scripts/pre-work-hook` - Pre-work validation hook

These scripts are critical for workflow enforcement and should be linted for quality.

