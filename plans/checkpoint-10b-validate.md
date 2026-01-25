# Checkpoint 10B: Dogfooding Milestone 2 - Validation

## Overview

Validate and formally document the completion of Dogfooding Milestone 2. The milestone implementation was completed in checkpoint-10a-precheck; this checkpoint focuses on verification of the criteria and creation of the formal documentation report.

## Project Structure

```
quench/
├── reports/
│   ├── dogfood-m2.md                # Existing baseline report
│   └── dogfooding-milestone-2.md    # New: Formal validation report
├── scripts/
│   └── install-hooks                # Pre-commit hook installer
└── .git/hooks/
    └── pre-commit                   # Installed hook (not tracked)
```

## Dependencies

None. All required functionality is already implemented:
- `quench check --staged` is functional
- `scripts/install-hooks` is complete
- Pre-commit hook is installed

## Implementation Phases

### Phase 1: Verify Pre-Commit Hook Installation

**Goal:** Confirm the pre-commit hook is correctly installed and functional.

**Steps:**
1. Check `.git/hooks/pre-commit` exists and is executable
2. Verify hook script content matches `scripts/install-hooks` output
3. Confirm worktree handling works (if applicable)

**Verification:**
```bash
test -x .git/hooks/pre-commit && echo "Hook executable: OK"
grep -q "quench check --staged" .git/hooks/pre-commit && echo "Hook content: OK"
```

### Phase 2: Validate `quench check --staged` Performance

**Goal:** Run staged checks and verify performance meets the <200ms target.

**Steps:**
1. Stage a minor change (e.g., whitespace in a file)
2. Run `quench check --staged --timing`
3. Verify all checks pass
4. Confirm timing is under 200ms

**Verification:**
```bash
quench check --staged --timing
# Expected: total < 200ms
```

### Phase 3: Run Full Check Suite

**Goal:** Confirm all fast checks pass on the complete codebase.

**Steps:**
1. Run `quench check` without filters
2. Verify PASS output for: cloc, escapes, agents, docs, tests
3. Document any new findings or regressions

**Verification:**
```bash
quench check
# Expected: PASS for all enabled checks
```

### Phase 4: Create Formal Documentation

**Goal:** Create `reports/dogfooding-milestone-2.md` with complete documentation.

**Documentation must include:**
1. Pre-commit hook setup instructions
2. `quench check --staged` performance on quench codebase
3. Any issues found during dogfooding (or confirmation of none)
4. Reproduction steps for future maintainers

**Template:**
```markdown
# Dogfooding Milestone 2: Validation Report

**Date:** 2026-01-24
**Checkpoint:** 10b-validate

## Executive Summary

Dogfooding Milestone 2 has been validated. All criteria met.

## Pre-Commit Hook Setup

### Installation
```bash
./scripts/install-hooks
```

### Hook Behavior
- Runs `quench check --staged` on every commit
- Uses local build if available (target/release or target/debug)
- Falls back to installed `quench` binary

### Worktree Support
The installation script correctly handles git worktrees by resolving
the actual hooks directory from the gitdir reference.

## Performance Results

### Staged Check Performance
[Include actual timing output]

### Full Check Performance
[Include actual timing output]

## Issues Found

[Document any issues or "None"]

## Verification Checklist

- [ ] Pre-commit hook installed
- [ ] `quench check --staged` runs on commit
- [ ] All fast checks pass
- [ ] Performance under 200ms target
```

### Phase 5: Test Hook with Real Commit

**Goal:** Verify the pre-commit hook runs during an actual commit.

**Steps:**
1. Make a trivial change (e.g., add blank line to a file)
2. Stage the change
3. Attempt to commit
4. Verify quench output appears and commit succeeds

**Verification:**
```bash
git add <file>
git commit -m "test: verify pre-commit hook"
# Observe quench output before commit message prompt
```

### Phase 6: Finalize and Commit

**Goal:** Commit the validation report.

**Steps:**
1. Add `reports/dogfooding-milestone-2.md` to staging
2. Commit with message: `docs(dogfood): validate Milestone 2 completion`
3. Run `./done` to complete the checkpoint

## Key Implementation Details

### Pre-Commit Hook Resolution

The `scripts/install-hooks` handles both regular repos and worktrees:

```bash
if [ -f "${REPO_DIR}/.git" ]; then
    # Worktree: .git is a file containing "gitdir: /path/to/hooks"
    GIT_DIR="$(sed 's/^gitdir: //' "${REPO_DIR}/.git")"
    HOOKS_DIR="${GIT_DIR}/../../hooks"
else
    HOOKS_DIR="${REPO_DIR}/.git/hooks"
fi
```

### Staged File Detection

The `--staged` flag uses git2 to diff HEAD against the index:

```rust
// crates/cli/src/git.rs
fn get_staged_files(repo: &Repository) -> Result<Vec<PathBuf>> {
    let head_tree = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head_tree), None, None)?;
    // Extract paths from diff deltas
}
```

### Performance Baseline

From `reports/dogfood-m2.md`:
- Discovery: 18ms
- Checking: 4ms
- Output: 0ms
- **Total: 24ms** (well under 200ms target)
- Cache effectiveness: 834/834 files (100%)

## Verification Plan

| Step | Command | Expected Result |
|------|---------|-----------------|
| Hook exists | `test -x .git/hooks/pre-commit` | Exit 0 |
| Hook content | `grep "quench check --staged" .git/hooks/pre-commit` | Match found |
| Staged check | `quench check --staged` | PASS, <200ms |
| Full check | `quench check` | All checks PASS |
| Commit test | `git commit` with staged changes | Hook runs, commit succeeds |

## Completion Criteria

- [ ] Pre-commit hook verified as installed and executable
- [ ] `quench check --staged` performance validated (<200ms)
- [ ] All fast checks pass on quench codebase
- [ ] `reports/dogfooding-milestone-2.md` created with full documentation
- [ ] Validation report committed
- [ ] `./done` executed successfully
