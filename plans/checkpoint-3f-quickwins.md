# Checkpoint 3F: Quick Wins - Escapes Works

**Root Feature:** `quench-fda7`

## Overview

Code cleanup checkpoint following the completion of the escapes check implementation. Now that escapes is fully functional (checkpoint-3e completed), this is the ideal time to remove dead code and consolidate scaffolding.

**Cleanup targets identified:**

| Category | Location | Impact |
|----------|----------|--------|
| Dead code | `check.rs` `fixable()` method | ~4 lines removed |
| Scaffolding | `git.rs` duplicate stub logic | ~15 lines consolidated |
| Dead code | `checks/mod.rs` unused imports | Minor cleanup |

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── check.rs              # Remove unused fixable() method
│   └── checks/
│       ├── mod.rs            # Simplify GitCheck to use StubCheck
│       ├── git.rs            # DELETE - merge into stub usage
│       └── git_tests.rs      # DELETE - covered by stub_tests.rs
└── plans/
    └── checkpoint-3f-quickwins.md
```

## Dependencies

No new dependencies. This is a pure cleanup phase.

## Implementation Phases

### Phase 1: Remove Dead `fixable()` Method

The `fixable()` method on the `Check` trait (lines 45-48 in `check.rs`) is defined but never called anywhere in the codebase.

**Evidence:** `grep -r "fixable" crates/` shows only the definition, no usage.

**Changes:**

Remove from `crates/cli/src/check.rs`:
```rust
// DELETE these lines (45-48):
/// Whether this check can auto-fix violations.
fn fixable(&self) -> bool {
    false
}
```

**Milestone:** Dead code removed from Check trait.

**Verification:**
```bash
cargo build -p quench
cargo test -p quench
```

---

### Phase 2: Consolidate GitCheck into StubCheck

The `GitCheck` struct in `git.rs` is essentially a `StubCheck` with extra logic to detect git repos. Since it always returns `CheckResult::stub()` when in a git repo, it can be simplified.

**Current behavior:**
- In git repo → returns `CheckResult::stub("git")`
- Not in git repo → returns `CheckResult::skipped("git", "Not a git repository")`

**Problem:** The git repo detection is useful, but the implementation duplicates stub behavior. However, keeping the skip-when-not-in-repo logic is valuable.

**Decision:** Keep `git.rs` as-is for now. The git repo detection logic is meaningful behavior that `StubCheck` doesn't provide. The ~15 lines are justified for the skip-on-non-git behavior.

**Alternative approach - simplify default_enabled:**
The `GitCheck::default_enabled()` returns `false`, which is already the correct behavior. No changes needed here.

**Milestone:** Reviewed and confirmed git.rs structure is appropriate.

---

### Phase 3: Clean Up Unused Test Annotations

Review test files for any unused `#[allow(...)]` attributes that are no longer needed.

**Files to review:**
- `crates/cli/src/check_tests.rs`
- `crates/cli/src/checks/stub_tests.rs`
- `crates/cli/src/checks/git_tests.rs`
- `crates/cli/src/checks/escapes_tests.rs`

**Milestone:** Test files follow consistent annotation patterns.

---

### Phase 4: Final Verification

Run the full verification suite to ensure all changes are correct.

**Verification checklist:**
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all`
- [ ] `cargo build --all`
- [ ] `./scripts/bootstrap`
- [ ] `cargo audit`
- [ ] `cargo deny check`

**Run `make check` to verify all gates pass.**

**Milestone:** All quality gates pass, codebase is cleaner.

## Key Implementation Details

### Why Remove fixable()?

The `fixable()` method was scaffolding for a planned auto-fix feature that hasn't been implemented:
- No check overrides it (all inherit `false`)
- No code calls it
- The feature may never be implemented, or may use a different design

Removing it now keeps the trait minimal and avoids confusion about whether auto-fix is supported.

### Why Keep GitCheck Separate?

Unlike other stub checks, `GitCheck` has conditional behavior:
- **StubCheck:** Always returns `CheckResult::stub()`
- **GitCheck:** Skips with error if not in git repo, otherwise stubs

This distinction is meaningful for output formatting (skipped vs stub) and allows future implementation of git validation without changing the check's registration.

### No Behavior Changes

This cleanup phase should produce **no observable behavior changes**:
- Same checks registered
- Same output format
- Same pass/fail logic

All changes are internal cleanup for maintainability.

## Verification Plan

1. **Before any changes** - snapshot current behavior:
   ```bash
   cargo build --release
   ./target/release/quench check tests/fixtures/bench-medium --json > /tmp/before.json
   ```

2. **After all changes** - verify identical output:
   ```bash
   cargo build --release
   ./target/release/quench check tests/fixtures/bench-medium --json > /tmp/after.json
   diff /tmp/before.json /tmp/after.json  # Should be empty (except timestamp)
   ```

3. **Full quality gates:**
   ```bash
   make check
   ```

## Summary

| Phase | Task | Lines Affected | Status |
|-------|------|----------------|--------|
| 1 | Remove dead `fixable()` method | -4 lines | [ ] Pending |
| 2 | Review GitCheck (no changes) | 0 lines | [ ] Pending |
| 3 | Clean up test annotations | ~0 lines | [ ] Pending |
| 4 | Final verification | 0 lines | [ ] Pending |

## Notes

- Total expected reduction: ~4 lines of dead code
- No new dependencies
- No behavior changes
- Minimal changes - "quick wins" focus
