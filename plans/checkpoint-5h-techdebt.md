# Checkpoint 5H: Tech Debt - Shell Adapter

**Root Feature:** `quench-68fa`

## Overview

Tech debt cleanup checkpoint for the Shell adapter. This checkpoint addresses inconsistencies and incomplete refactoring from previous checkpoints (5C, 5E-5G).

**Issues identified:**

1. **Missing SPDX headers** - Two shell adapter files lack the standard license headers that all other adapter files have
2. **Orphaned type alias** - `ShellPolicyCheckResult` alias was marked for removal in checkpoint 5C but remains with a misleading "backward compatibility" comment
3. **Inconsistent exports** - Shell adapter exports a language-specific type alias while Rust adapter exports the common type directly

**Goals:**
1. Add SPDX license headers to `shell/suppress.rs` and `shell/suppress_tests.rs`
2. Remove unnecessary `ShellPolicyCheckResult` type alias
3. Align Shell adapter exports with Rust adapter pattern
4. Ensure all quality gates pass

**Non-Goals:**
- Functional changes
- New features
- Performance changes

## Project Structure

Files to modify:

```
quench/
└── crates/cli/src/adapter/shell/
    ├── mod.rs              # Update export from ShellPolicyCheckResult to PolicyCheckResult
    ├── policy.rs           # Remove ShellPolicyCheckResult type alias
    ├── suppress.rs         # Add SPDX header
    └── suppress_tests.rs   # Add SPDX header
```

Reference files (no changes needed):

```
quench/
└── crates/cli/src/adapter/rust/
    ├── mod.rs              # Reference: exports PolicyCheckResult directly
    └── policy.rs           # Reference: no language-specific alias
```

## Dependencies

None. Tech debt cleanup only with no new dependencies.

## Implementation Phases

### Phase 1: Add SPDX Headers to Suppress Files

**Goal:** Add missing SPDX-License-Identifier headers to shell adapter files for consistency with all other adapter files.

**File 1:** `crates/cli/src/adapter/shell/suppress.rs`

**Change:**

```diff
+// SPDX-License-Identifier: MIT
+// Copyright (c) 2026 Alfred Jean LLC
+
 //! Shellcheck suppress directive parsing.
 //!
 //! Parses `# shellcheck disable=SC2034,SC2086` comments in shell scripts.
```

**File 2:** `crates/cli/src/adapter/shell/suppress_tests.rs`

**Change:**

```diff
+// SPDX-License-Identifier: MIT
+// Copyright (c) 2026 Alfred Jean LLC
+
 //! Unit tests for shellcheck suppress directive parsing.
 #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
```

**Verification:**
```bash
# All shell adapter files should have SPDX headers
grep -l "SPDX-License-Identifier" crates/cli/src/adapter/shell/*.rs | wc -l
# Should output: 5 (mod.rs, policy.rs, policy_tests.rs, suppress.rs, suppress_tests.rs)
```

**Milestone:** All shell adapter files have consistent SPDX headers.

**Status:** [ ] Pending

---

### Phase 2: Remove ShellPolicyCheckResult Type Alias

**Goal:** Remove the unnecessary `ShellPolicyCheckResult` type alias that was marked for removal in checkpoint 5C.

**Context:**

The checkpoint-5c-refactor.md plan stated:
> ```rust
> **Update `shell/policy.rs`:**
> pub use crate::adapter::common::policy::{check_lint_policy, PolicyCheckResult};
> // Remove ShellPolicyCheckResult, use PolicyCheckResult
> ```

The alias was created during refactoring but the "backward compatibility" comment is misleading - the Shell adapter is new and has no legacy API to maintain.

**Comparison with Rust adapter:**

| Aspect | Rust Adapter | Shell Adapter (Current) | Shell Adapter (Fixed) |
|--------|--------------|-------------------------|------------------------|
| policy.rs | Re-exports `PolicyCheckResult` | Defines `ShellPolicyCheckResult = PolicyCheckResult` | Re-exports `PolicyCheckResult` |
| mod.rs export | `PolicyCheckResult` | `ShellPolicyCheckResult` | `PolicyCheckResult` |

**File:** `crates/cli/src/adapter/shell/policy.rs`

**Change:**

```diff
 // Re-export from common
 pub use crate::adapter::common::policy::PolicyCheckResult;

-/// Alias for backward compatibility.
-pub type ShellPolicyCheckResult = PolicyCheckResult;
-
 /// Check shell lint policy against changed files.
```

**Verification:**
```bash
# ShellPolicyCheckResult should no longer exist in source
grep -r "ShellPolicyCheckResult" crates/cli/src/
# Should return no results
```

**Milestone:** Unnecessary type alias removed.

**Status:** [ ] Pending

---

### Phase 3: Update Shell Adapter Exports

**Goal:** Update `shell/mod.rs` to export `PolicyCheckResult` directly, matching the Rust adapter pattern.

**File:** `crates/cli/src/adapter/shell/mod.rs`

**Change:**

```diff
-pub use policy::{ShellPolicyCheckResult, check_lint_policy};
+pub use policy::{PolicyCheckResult, check_lint_policy};
```

Also update the return type of `check_lint_policy` method:

```diff
     /// Check lint policy against changed files.
     ///
     /// Returns policy check result with violation details.
     pub fn check_lint_policy(
         &self,
         changed_files: &[&Path],
         policy: &ShellPolicyConfig,
-    ) -> ShellPolicyCheckResult {
+    ) -> PolicyCheckResult {
         policy::check_lint_policy(changed_files, policy, |p| self.classify(p))
     }
```

**Verification:**
```bash
# Shell adapter should now export PolicyCheckResult like Rust adapter
grep "pub use policy::" crates/cli/src/adapter/shell/mod.rs
grep "pub use policy::" crates/cli/src/adapter/rust/mod.rs
# Both should show PolicyCheckResult
```

**Milestone:** Shell adapter exports aligned with Rust adapter pattern.

**Status:** [ ] Pending

---

### Phase 4: Quality Gates

**Goal:** Verify all quality checks pass.

```bash
make check
```

This runs:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --all`
- `./scripts/bootstrap`
- `cargo audit`
- `cargo deny check`

**Milestone:** All quality gates pass.

**Status:** [ ] Pending

## Key Implementation Details

### Why These Changes Matter

1. **SPDX Headers:** Required for license compliance and consistency. Every source file in the crate should clearly indicate its license.

2. **Type Alias Removal:** The `ShellPolicyCheckResult` alias:
   - Was marked for removal in checkpoint 5C refactoring plan
   - Has a misleading "backward compatibility" comment
   - Creates inconsistency with Rust adapter's cleaner approach
   - Adds unnecessary indirection for no benefit

3. **Export Consistency:** Both adapters should follow the same export pattern:
   - Common types (`PolicyCheckResult`) exported directly
   - Language-specific functions (`check_lint_policy`) exported normally
   - No unnecessary language-prefixed aliases

### Impact Assessment

| Change | Risk | Impact |
|--------|------|--------|
| Add SPDX headers | None | Cosmetic only, no functional change |
| Remove type alias | Low | Internal API only, no external consumers |
| Update exports | Low | Internal API only, no external consumers |

### Files Changed Summary

| File | Change Type | Lines Modified |
|------|-------------|----------------|
| `shell/suppress.rs` | Add header | +3 lines |
| `shell/suppress_tests.rs` | Add header | +3 lines |
| `shell/policy.rs` | Remove alias | -3 lines |
| `shell/mod.rs` | Update export + return type | 2 lines changed |

**Net change:** ~3 lines added (headers dominate)

## Verification Plan

1. **SPDX header verification:**
   ```bash
   # Count files with SPDX headers in shell adapter
   grep -l "SPDX-License-Identifier" crates/cli/src/adapter/shell/*.rs | wc -l
   # Expected: 5
   ```

2. **Type alias removal verification:**
   ```bash
   # Ensure ShellPolicyCheckResult no longer exists
   grep -r "ShellPolicyCheckResult" crates/cli/src/
   # Expected: no output
   ```

3. **Export consistency verification:**
   ```bash
   # Compare exports between adapters
   grep "pub use policy::" crates/cli/src/adapter/*/mod.rs
   # Expected: both show PolicyCheckResult
   ```

4. **Unit tests:**
   ```bash
   cargo test -p quench --lib shell
   cargo test -p quench --lib policy
   ```

5. **Full quality gates:**
   ```bash
   make check
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Add SPDX headers to suppress files | [ ] Pending |
| 2 | Remove ShellPolicyCheckResult alias | [ ] Pending |
| 3 | Update shell adapter exports | [ ] Pending |
| 4 | Quality gates | [ ] Pending |

## Notes

- This is a tech debt cleanup with no functional changes
- All changes are internal API only
- Risk is minimal as changes are cosmetic/organizational
- Completes cleanup started in checkpoint 5C refactoring
