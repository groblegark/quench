# Phase 035: Check Framework - Specs

**Root Feature:** `quench-52de`

## Overview

Write behavioral specs for the check framework that will be implemented in Phase 040. This includes:
- Check toggle flags (`--cloc`, `--no-cloc`, `--escapes`, etc.)
- Check name validation (exactly 8 checks)
- Flag combination behavior
- Error isolation (one check failing doesn't block others)
- Skip reporting (when a check can't run, report error and continue)

**Current State**: Output infrastructure complete (Phase 030). CLI has basic structure with `--color`, `--limit`, `--config-only` flags. Only `cloc` check implemented (hardcoded in main.rs).

**End State**: `tests/specs/checks.rs` contains complete behavioral specs for the check framework, all marked with `#[ignore = "TODO: Phase 040"]`. Phase 040 will implement the framework to make these specs pass.

## Project Structure

```
tests/
├── specs/
│   ├── mod.rs              # ADD: mod checks;
│   ├── checks.rs           # NEW: Check framework specs
│   ├── output.rs           # EXISTS: Output specs (Phase 025)
│   ├── file_walking.rs     # EXISTS: File walking specs
│   └── prelude.rs          # EXISTS: Test helpers
└── fixtures/
    ├── check-framework/    # NEW: Multi-violation fixture for check tests
    │   ├── quench.toml
    │   ├── oversized.rs    # Triggers cloc
    │   └── has_unsafe.rs   # For future escapes check
    └── skipped-check/      # NEW: Fixture that causes check to skip
        ├── quench.toml     # Config that enables a check that will error
        └── src/
            └── main.rs

docs/specs/checks/
├── cloc.md                 # Reference for cloc check
├── escape-hatches.md       # Reference for escapes check
├── agents.md               # Reference for agents check
├── docs.md                 # Reference for docs check
├── tests.md                # Reference for tests check
├── git.md                  # Reference for git check
├── build.md                # Reference for build check
└── license-headers.md      # Reference for license check
```

## Dependencies

No new dependencies required. Uses existing test infrastructure:
- `assert_cmd` - CLI testing
- `predicates` - Output assertions
- `tempfile` - Temporary directories
- `serde_json` - JSON parsing

## Implementation Phases

### Phase 35.1: Create Spec File Structure

**Goal**: Set up the spec file and fixtures for check framework testing.

**Tasks**:
1. Create `tests/specs/checks.rs` with module header
2. Add `mod checks;` to `tests/specs/mod.rs` (if exists) or lib
3. Create fixture `tests/fixtures/check-framework/`
4. Create fixture `tests/fixtures/skipped-check/`

**Files**:

```rust
// tests/specs/checks.rs
//! Behavioral specs for the check framework.
//!
//! Tests that quench correctly handles:
//! - Check toggle flags (--cloc, --no-cloc, etc.)
//! - Check name validation
//! - Multiple check flag combinations
//! - Error isolation between checks
//! - Skipped check reporting
//!
//! Reference: docs/specs/01-cli.md#check-toggles

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;
```

Fixture `tests/fixtures/check-framework/quench.toml`:
```toml
version = 1
```

Fixture `tests/fixtures/check-framework/oversized.rs`:
```rust
// A file with many lines to trigger cloc check
// (content will be >750 lines for testing)
```

**Verification**:
```bash
cargo test --test specs checks::
```

### Phase 35.2: Check Name Specs

**Goal**: Specify that check names are exactly: cloc, escapes, agents, docs, tests, git, build, license.

**Tasks**:
1. Add spec verifying check names in JSON output
2. Add spec verifying check names in text output (help)
3. Add spec verifying unknown check names are rejected

**Files**:

```rust
// tests/specs/checks.rs - Check Names section

// =============================================================================
// Check Names
// =============================================================================

/// Spec: docs/specs/00-overview.md#built-in-checks
///
/// > Built-in checks: cloc, escapes, agents, docs, tests, git, build, license
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn check_names_are_exactly_8_known_checks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    // All 8 checks should be present
    assert!(names.contains(&"cloc"), "should have cloc check");
    assert!(names.contains(&"escapes"), "should have escapes check");
    assert!(names.contains(&"agents"), "should have agents check");
    assert!(names.contains(&"docs"), "should have docs check");
    assert!(names.contains(&"tests"), "should have tests check");
    assert!(names.contains(&"git"), "should have git check");
    assert!(names.contains(&"build"), "should have build check");
    assert!(names.contains(&"license"), "should have license check");

    // No other checks should be present
    assert_eq!(names.len(), 8, "should have exactly 8 checks");
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > Check toggles appear in help: --[no-]cloc, --[no-]escapes, etc.
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn check_toggles_shown_in_help() {
    quench_cmd()
        .args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("--cloc"))
        .stdout(predicates::str::contains("--escapes"))
        .stdout(predicates::str::contains("--agents"))
        .stdout(predicates::str::contains("--docs"))
        .stdout(predicates::str::contains("--tests"))
        .stdout(predicates::str::contains("--git"))
        .stdout(predicates::str::contains("--build"))
        .stdout(predicates::str::contains("--license"));
}
```

**Verification**:
```bash
cargo test --test specs check_names -- --ignored
```

### Phase 35.3: Enable/Disable Flag Specs

**Goal**: Specify behavior of `--<check>` and `--no-<check>` flags.

**Tasks**:
1. Add spec for `--cloc` enabling only cloc check
2. Add spec for `--no-cloc` disabling cloc check
3. Add specs for other checks (escapes, agents, docs, tests, git, build, license)

**Files**:

```rust
// tests/specs/checks.rs - Enable/Disable Flags section

// =============================================================================
// Enable Flags (--<check>)
// =============================================================================

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --cloc: Only run cloc check (implies --no-* for others)
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn cloc_flag_enables_only_cloc_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--cloc", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    assert_eq!(checks.len(), 1, "only one check should run");
    assert_eq!(
        checks[0].get("name").and_then(|n| n.as_str()),
        Some("cloc"),
        "check should be cloc"
    );
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --escapes: Only run escapes check
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn escapes_flag_enables_only_escapes_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--escapes", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    assert_eq!(checks.len(), 1, "only one check should run");
    assert_eq!(
        checks[0].get("name").and_then(|n| n.as_str()),
        Some("escapes"),
        "check should be escapes"
    );
}

// =============================================================================
// Disable Flags (--no-<check>)
// =============================================================================

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --no-cloc: Skip cloc check, run all others
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn no_cloc_flag_disables_cloc_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-cloc", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"cloc"), "cloc should not be present");
    assert_eq!(names.len(), 7, "7 checks should run (all except cloc)");
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --no-escapes: Skip escapes check
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn no_escapes_flag_disables_escapes_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-escapes", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"escapes"), "escapes should not be present");
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --no-docs: Skip docs check
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn no_docs_flag_disables_docs_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-docs", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"docs"), "docs should not be present");
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > --no-tests: Skip tests check
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn no_tests_flag_disables_tests_check() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-tests", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"tests"), "tests should not be present");
}
```

**Verification**:
```bash
cargo test --test specs enable_flag -- --ignored
cargo test --test specs disable_flag -- --ignored
```

### Phase 35.4: Flag Combination Specs

**Goal**: Specify behavior when multiple check flags are used together.

**Tasks**:
1. Add spec for `--cloc --escapes` (runs both)
2. Add spec for `--no-docs --no-tests` (skips both)
3. Add spec for mixing enable and disable flags

**Files**:

```rust
// tests/specs/checks.rs - Flag Combinations section

// =============================================================================
// Flag Combinations
// =============================================================================

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > Multiple enable flags combine: --cloc --escapes runs both checks
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn multiple_enable_flags_run_multiple_checks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--cloc", "--escapes", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert_eq!(names.len(), 2, "two checks should run");
    assert!(names.contains(&"cloc"), "cloc should be present");
    assert!(names.contains(&"escapes"), "escapes should be present");
}

/// Spec: docs/specs/01-cli.md#check-toggles
///
/// > Multiple disable flags combine: --no-docs --no-tests skips both
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn multiple_disable_flags_skip_multiple_checks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-docs", "--no-tests", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"docs"), "docs should not be present");
    assert!(!names.contains(&"tests"), "tests should not be present");
    assert_eq!(names.len(), 6, "6 checks should run");
}

/// Spec: docs/specs/01-cli.md#examples
///
/// > quench check --no-cloc --no-escapes: Skip multiple checks
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn no_cloc_no_escapes_skips_both() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "--no-cloc", "--no-escapes", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    let names: Vec<&str> = checks
        .iter()
        .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(!names.contains(&"cloc"), "cloc should not be present");
    assert!(!names.contains(&"escapes"), "escapes should not be present");
}

/// Spec: docs/specs/01-cli.md#check-toggles (edge case)
///
/// > All checks can be disabled except one
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn all_checks_disabled_except_one() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args([
            "check",
            "--no-cloc",
            "--no-escapes",
            "--no-agents",
            "--no-docs",
            "--no-tests",
            "--no-git",
            "--no-build",
            // license is the only one NOT disabled
            "-o", "json"
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    assert_eq!(checks.len(), 1, "only one check should run");
    assert_eq!(
        checks[0].get("name").and_then(|n| n.as_str()),
        Some("license"),
        "only license check should run"
    );
}
```

**Verification**:
```bash
cargo test --test specs flag_combination -- --ignored
```

### Phase 35.5: Error Isolation and Skipped Check Specs

**Goal**: Specify that check failures don't prevent other checks from running, and skipped checks are reported.

**Tasks**:
1. Add spec that failing check doesn't block other checks
2. Add spec that skipped check shows error but continues
3. Add spec for JSON output of skipped checks

**Files**:

```rust
// tests/specs/checks.rs - Error Isolation section

// =============================================================================
// Error Isolation
// =============================================================================

/// Spec: docs/specs/00-overview.md (implied)
///
/// > Check failure doesn't prevent other checks from running
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn check_failure_doesnt_block_other_checks() {
    // Use fixture that triggers cloc failure (oversized file)
    let output = quench_cmd()
        .args(["check", "-o", "json"])
        .current_dir(fixture("check-framework"))
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    // All 8 checks should have run, even though cloc failed
    assert_eq!(checks.len(), 8, "all checks should have run");

    // Find cloc check - it should have failed
    let cloc = checks.iter().find(|c| {
        c.get("name").and_then(|n| n.as_str()) == Some("cloc")
    }).unwrap();
    assert_eq!(
        cloc.get("passed").and_then(|p| p.as_bool()),
        Some(false),
        "cloc should have failed"
    );

    // Other checks should have completed (may pass or fail, but not skipped)
    let other_checks: Vec<_> = checks.iter().filter(|c| {
        c.get("name").and_then(|n| n.as_str()) != Some("cloc")
    }).collect();

    for check in other_checks {
        assert!(
            check.get("skipped").and_then(|s| s.as_bool()) != Some(true),
            "check {} should not be skipped due to cloc failure",
            check.get("name").and_then(|n| n.as_str()).unwrap_or("unknown")
        );
    }
}

// =============================================================================
// Skipped Checks
// =============================================================================

/// Spec: docs/specs/03-output.md (implied)
///
/// > Skipped check shows error message but continues with other checks
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn skipped_check_shows_error_but_continues() {
    // This test uses a fixture that causes a specific check to skip
    // (e.g., git check in a non-git directory)
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();
    // Don't initialize git - git check should skip

    let output = quench_cmd()
        .args(["check", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    // Find git check
    let git = checks.iter().find(|c| {
        c.get("name").and_then(|n| n.as_str()) == Some("git")
    });

    if let Some(git_check) = git {
        // Git check should be skipped with error message
        assert_eq!(
            git_check.get("skipped").and_then(|s| s.as_bool()),
            Some(true),
            "git check should be skipped"
        );
        assert!(
            git_check.get("error").is_some(),
            "skipped check should have error message"
        );
    }

    // Other checks should still have run
    let non_git_checks = checks.iter().filter(|c| {
        c.get("name").and_then(|n| n.as_str()) != Some("git")
    });
    assert!(non_git_checks.count() >= 7, "other checks should have run");
}

/// Spec: docs/specs/03-output.md#text-format (implied)
///
/// > Skipped check shows in text output with reason
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn skipped_check_text_output_shows_reason() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();
    // Don't initialize git - git check should skip

    quench_cmd()
        .args(["check"])
        .current_dir(dir.path())
        .assert()
        // Look for skip indicator in output
        .stdout(
            predicates::str::contains("SKIP")
                .or(predicates::str::contains("skip"))
                .or(predicates::str::contains("git"))
        );
}

/// Spec: docs/specs/output.schema.json
///
/// > Skipped check has `skipped: true` and `error` field in JSON
#[test]
#[ignore = "TODO: Phase 040 - Check Framework"]
fn skipped_check_json_has_required_fields() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("quench.toml"), "version = 1\n").unwrap();

    let output = quench_cmd()
        .args(["check", "-o", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json.get("checks").and_then(|v| v.as_array()).unwrap();

    // Find any skipped check
    let skipped: Vec<_> = checks.iter().filter(|c| {
        c.get("skipped").and_then(|s| s.as_bool()) == Some(true)
    }).collect();

    for check in skipped {
        assert!(
            check.get("error").is_some(),
            "skipped check should have 'error' field"
        );
        assert_eq!(
            check.get("passed").and_then(|p| p.as_bool()),
            Some(false),
            "skipped check should have passed=false"
        );
    }
}
```

**Verification**:
```bash
cargo test --test specs error_isolation -- --ignored
cargo test --test specs skipped_check -- --ignored
```

### Phase 35.6: Fixtures and Final Integration

**Goal**: Create test fixtures and ensure all specs compile.

**Tasks**:
1. Create `check-framework` fixture with oversized file
2. Verify all specs compile (even if ignored)
3. Run `make check` to ensure no regressions

**Files**:

`tests/fixtures/check-framework/quench.toml`:
```toml
version = 1

[check.cloc]
max_lines = 10  # Low limit to trigger failure easily
```

`tests/fixtures/check-framework/oversized.rs`:
```rust
// This file is intentionally >10 lines to trigger cloc failure
// Line 1
// Line 2
// Line 3
// Line 4
// Line 5
// Line 6
// Line 7
// Line 8
// Line 9
// Line 10
// Line 11 - this exceeds the 10 line limit
fn main() {}
```

`tests/fixtures/check-framework/has_unsafe.rs`:
```rust
// For future escapes check testing
fn main() {
    unsafe {
        // Missing SAFETY comment
        std::ptr::null::<i32>().read();
    }
}
```

**Verification**:
```bash
# All specs should compile
cargo test --test specs checks::

# Specs should be ignored (not run)
cargo test --test specs checks:: 2>&1 | grep -c "ignored"

# Full validation
make check
```

## Key Implementation Details

### Check Names

The 8 check names are:
| Name | Flag | Description |
|------|------|-------------|
| `cloc` | `--[no-]cloc` | Lines of code, file size limits |
| `escapes` | `--[no-]escapes` | Escape hatch detection |
| `agents` | `--[no-]agents` | CLAUDE.md, .cursorrules validation |
| `docs` | `--[no-]docs` | File refs, specs, doc correlation |
| `tests` | `--[no-]tests` | Test correlation + coverage/time |
| `git` | `--[no-]git` | Commit message format |
| `build` | `--[no-]build` | Binary/bundle size + build time |
| `license` | `--[no-]license` | License header validation |

### Flag Semantics

1. **No flags**: Run all checks enabled by default (fast mode)
2. **`--<check>` flag**: Run ONLY that check (exclusive mode)
3. **Multiple `--<check>` flags**: Run only the specified checks
4. **`--no-<check>` flag**: Skip that check, run all others
5. **Multiple `--no-<check>` flags**: Skip all specified checks

### Default Check Enablement

Per `docs/specs/01-cli.md`:
- **Fast mode (default)**: cloc, escapes, agents, docs, tests
- **CI mode (`--ci`)**: All checks including build, license
- **Disabled by default**: git, build, license

### Error Isolation

Checks run independently. A failure in one check should not:
- Prevent other checks from running
- Affect the results of other checks
- Change the exit code beyond adding to "failed count"

### Skipped Checks

A check is "skipped" when it cannot run due to missing prerequisites:
- `git` check in non-git directory
- `build` check when no build tool detected
- `license` check when no license config

Skipped checks:
- Have `skipped: true` in JSON output
- Have `error` field explaining why
- Have `passed: false`
- Show in text output with reason

## Verification Plan

### Spec Coverage

| Spec Category | Count | Phase |
|---------------|-------|-------|
| Check names | 2 | 35.2 |
| Enable flags | 2 | 35.3 |
| Disable flags | 4 | 35.3 |
| Flag combinations | 4 | 35.4 |
| Error isolation | 1 | 35.5 |
| Skipped checks | 3 | 35.5 |
| **Total** | **16** | |

### Phase Completion Checklist

- [ ] **35.1**: Spec file and fixtures created
- [ ] **35.2**: Check name specs added with `#[ignore]`
- [ ] **35.3**: Enable/disable flag specs added with `#[ignore]`
- [ ] **35.4**: Flag combination specs added with `#[ignore]`
- [ ] **35.5**: Error isolation and skipped check specs added with `#[ignore]`
- [ ] **35.6**: All specs compile, fixtures work, `make check` passes

### Running Verification

```bash
# After each phase:
cargo test --test specs checks::

# Verify specs are ignored:
cargo test --test specs checks:: -- --ignored 2>&1 | grep "ignored"

# Count ignored specs (should be 16):
cargo test --test specs checks:: -- --ignored 2>&1 | grep -c "TODO: Phase 040"

# Full validation:
make check
```

## Summary

Phase 035 creates behavioral specs for the check framework:

1. **Check names**: Verify exactly 8 checks exist with correct names
2. **Enable flags**: `--cloc`, `--escapes`, etc. run only specified checks
3. **Disable flags**: `--no-cloc`, `--no-escapes`, etc. skip checks
4. **Combinations**: Multiple flags combine correctly
5. **Error isolation**: Failures don't block other checks
6. **Skipped checks**: Missing prerequisites reported gracefully

All specs are marked `#[ignore = "TODO: Phase 040"]` and will be implemented in Phase 040.
