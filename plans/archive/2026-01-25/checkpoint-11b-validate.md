# Checkpoint 11B: Tests CI Mode Complete - Validation

## Overview

Validation checkpoint to confirm that CI mode for the tests check is complete and meets all checkpoint criteria. This checkpoint builds on the pre-check verification (11a) by adding exact output tests (snapshots) and creating the formal documentation report.

**Checkpoint criteria:**
- [x] `quench check --ci --tests` runs tests and collects coverage
- [x] Coverage and timing metrics in JSON output
- [ ] Snapshot tests for CI tests output
- [ ] Documentation: `reports/checkpoint-11-tests-ci-mode.md`

## Project Structure

```
quench/
├── crates/cli/src/
│   └── checks/tests/
│       └── mod.rs               # CI threshold checking (verified in 11a)
├── tests/specs/
│   └── checks/tests/
│       ├── ci_metrics.rs        # Existing: 9 CI threshold specs (all passing)
│       └── ci_output.rs         # NEW: Exact output specs for CI mode
├── tests/fixtures/
│   └── tests-ci/                # NEW: Fixture for CI output tests
│       ├── quench.toml
│       ├── Cargo.toml
│       ├── src/lib.rs
│       └── tests/basic.rs
└── reports/
    └── checkpoint-11-tests-ci-mode.md  # NEW: Formal validation report
```

## Dependencies

No new dependencies. Uses existing test infrastructure:
- `cargo test --test specs` for behavioral specs
- `make check` for full validation suite

## Implementation Phases

### Phase 1: Create CI Output Test Fixture

**Goal:** Create a minimal, deterministic fixture for exact output testing.

**Location:** `tests/fixtures/tests-ci/`

The fixture needs:
- Simple Cargo project with predictable test output
- Configured test suite with timing thresholds
- Coverage threshold configuration

**Files to create:**

```toml
# tests/fixtures/tests-ci/quench.toml
[[check.tests.suite]]
runner = "cargo"

[check.tests.coverage]
check = "warn"
min = 50

[check.tests.time]
check = "warn"
```

```toml
# tests/fixtures/tests-ci/Cargo.toml
[package]
name = "ci_test_project"
version = "0.1.0"
edition = "2021"
```

```rust
// tests/fixtures/tests-ci/src/lib.rs
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

```rust
// tests/fixtures/tests-ci/tests/basic.rs
#[test]
fn test_add() {
    assert_eq!(ci_test_project::add(1, 2), 3);
}
```

**Verification:**
```bash
quench check tests --ci -o json -C tests/fixtures/tests-ci
# Should show metrics with test_count, total_ms, suites array
```

### Phase 2: Add Exact Output Specs for CI Mode

**Goal:** Create snapshot tests validating CI output format.

**File:** `tests/specs/checks/tests/ci_output.rs`

**Specs to add:**

1. **Text output format for CI mode pass:**
```rust
/// Spec: CI mode text output shows test results summary
#[test]
fn tests_ci_text_output_passes() {
    check("tests")
        .on("tests-ci")
        .args(&["--ci"])
        .passes()
        .stdout_has("tests: PASS");
}
```

2. **JSON output includes timing metrics:**
```rust
/// Spec: CI mode JSON output includes timing metrics structure
#[test]
fn tests_ci_json_output_timing_structure() {
    let result = check("tests")
        .on("tests-ci")
        .args(&["--ci"])
        .json()
        .passes();
    let metrics = result.require("metrics");

    // Verify required fields
    assert!(metrics.get("test_count").is_some());
    assert!(metrics.get("total_ms").is_some());
    let suites = metrics.get("suites").unwrap().as_array().unwrap();
    assert!(!suites.is_empty());

    // Suite should have required fields
    let suite = &suites[0];
    assert!(suite.get("name").is_some());
    assert!(suite.get("runner").is_some());
    assert!(suite.get("passed").is_some());
    assert!(suite.get("test_count").is_some());
    assert!(suite.get("total_ms").is_some());
}
```

3. **Text output for threshold violations:**
```rust
/// Spec: CI mode text output shows threshold violations
#[test]
fn tests_ci_text_output_timing_violation() {
    let temp = Project::empty();
    temp.config(r#"
[[check.tests.suite]]
runner = "cargo"
max_total = "1ms"

[check.tests.time]
check = "error"
"#);
    temp.cargo_project("test_project");

    check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("time_total_exceeded")
        .stdout_has("threshold");
}
```

4. **JSON violation structure:**
```rust
/// Spec: CI violation has threshold and value fields
#[test]
fn tests_ci_json_violation_has_threshold_and_value() {
    let temp = Project::empty();
    temp.config(r#"
[[check.tests.suite]]
runner = "cargo"
max_total = "1ms"

[check.tests.time]
check = "error"
"#);
    temp.cargo_project("test_project");

    let result = check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .json()
        .fails();

    let v = result.require_violation("time_total_exceeded");
    assert!(v.get("threshold").is_some());
    assert!(v.get("value").is_some());
}
```

**Register the module:**
Add to `tests/specs/checks/tests/mod.rs`:
```rust
mod ci_output;
```

### Phase 3: Verify All Specs Pass

**Goal:** Confirm all specs pass including the new ones.

**Verification:**
```bash
# Run new CI output specs
cargo test --test specs ci_output

# Run all CI-related specs
cargo test --test specs ci_metrics
cargo test --test specs ci_output

# Run full spec suite
cargo test --test specs
# Expected: 565+ passed (561 existing + 4 new), 11 ignored
```

### Phase 4: Run Full Validation Suite

**Goal:** Verify `make check` passes completely.

**Verification:**
```bash
make check
```

**Expected checks:**
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all`
- [x] `cargo build --all`
- [x] `cargo audit`
- [x] `cargo deny check`

### Phase 5: Create Documentation Report

**Goal:** Create `reports/checkpoint-11-tests-ci-mode.md` with validation results.

**Template:**
```markdown
# Checkpoint 11: Tests CI Mode Complete

**Date:** 2026-01-25
**Checkpoint:** 11b-validate

## Executive Summary

Tests CI mode is complete. All criteria met:
- `quench check --ci --tests` runs tests and collects coverage
- Coverage and timing metrics included in JSON output
- Exact output tests validate CI output format

## Implemented Features

### CI Mode Test Execution

```bash
quench check tests --ci
```

Behavior:
- Runs all configured test suites
- Collects timing metrics (total_ms, avg_ms, max_ms, max_test)
- Collects coverage when available (coverage, coverage_by_package)
- Checks against configured thresholds

### Metrics JSON Structure

```json
{
  "test_count": 42,
  "total_ms": 1234,
  "avg_ms": 29,
  "max_ms": 156,
  "max_test": "tests::slow_test",
  "suites": [
    {
      "name": "default",
      "runner": "cargo",
      "passed": true,
      "test_count": 42,
      "total_ms": 1234
    }
  ],
  "coverage": {"rust": 85.5},
  "coverage_by_package": {"core": 90.2, "utils": 78.1}
}
```

### Threshold Configuration

```toml
[[check.tests.suite]]
runner = "cargo"
max_total = "30s"      # Suite time limit
max_test = "1s"        # Slowest test limit
max_avg = "100ms"      # Average test time limit

[check.tests.coverage]
check = "error"        # error | warn | off
min = 75               # Global coverage minimum

[check.tests.coverage.package.core]
min = 90               # Per-package minimum

[check.tests.time]
check = "warn"         # Check level for timing violations
```

### Violation Types

| Type | Trigger | Fields |
|------|---------|--------|
| `coverage_below_min` | Coverage below threshold | `threshold`, `value`, `package` (optional) |
| `time_total_exceeded` | Suite time exceeds limit | `threshold`, `value`, `suite` |
| `time_avg_exceeded` | Average test time exceeds limit | `threshold`, `value`, `suite` |
| `time_test_exceeded` | Slowest test exceeds limit | `threshold`, `value`, `suite`, `test` |

## Test Coverage

### Behavioral Specs

| Spec | Status |
|------|--------|
| `ci_mode_reports_aggregated_timing_metrics` | Pass |
| `ci_mode_reports_per_suite_timing` | Pass |
| `ci_mode_reports_per_package_coverage` | Pass |
| `coverage_below_min_generates_violation` | Pass |
| `per_package_coverage_thresholds_work` | Pass |
| `time_total_exceeded_generates_violation` | Pass |
| `time_avg_exceeded_generates_violation` | Pass |
| `time_test_exceeded_generates_violation` | Pass |
| `tests_ci_violation_types_are_documented` | Pass |
| `tests_ci_text_output_passes` | Pass |
| `tests_ci_json_output_timing_structure` | Pass |
| `tests_ci_text_output_timing_violation` | Pass |
| `tests_ci_json_violation_has_threshold_and_value` | Pass |

### Full Suite Results

```
test result: ok. 565 passed; 0 failed; 11 ignored
```

## Verification Checklist

- [x] `quench check --ci --tests` runs tests and collects coverage
- [x] Coverage metrics in JSON output (coverage, coverage_by_package)
- [x] Timing metrics in JSON output (test_count, total_ms, avg_ms, max_ms, suites)
- [x] Threshold violations generated correctly
- [x] Exact output specs validate format stability
- [x] All specs pass
- [x] `make check` passes

## Remaining Work

The following tests check features are deferred to future phases:
- `checks_tests::timing::*` (5 tests) - Per-runner timing extraction (Phase 9XX)
- `checks_tests::coverage::*` (4 tests) - Per-runner coverage collection (Phase 940)
```

### Phase 6: Commit and Complete

**Goal:** Commit the validation changes.

**Files to commit:**
- `tests/fixtures/tests-ci/` (new)
- `tests/specs/checks/tests/ci_output.rs` (new)
- `tests/specs/checks/tests/mod.rs` (modified)
- `reports/checkpoint-11-tests-ci-mode.md` (new)

**Commit message:**
```
docs(tests): validate CI mode tests complete (checkpoint 11b)

Validation checkpoint for CI mode tests:
- Add exact output specs for CI mode (4 tests)
- Create tests-ci fixture for deterministic output testing
- Document implementation in checkpoint report

Specs verified:
- tests_ci_text_output_passes
- tests_ci_json_output_timing_structure
- tests_ci_text_output_timing_violation
- tests_ci_json_violation_has_threshold_and_value

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

**Completion:**
```bash
./done
```

## Key Implementation Details

### CI Output Format Stability

The exact output specs ensure format stability for:
- Text output structure ("tests: PASS/FAIL" header)
- JSON metrics field names (test_count, total_ms, etc.)
- Violation field presence (threshold, value for CI violations)

### Test Fixture Design

The `tests-ci` fixture is intentionally minimal:
- Single crate with one test
- Predictable test count (1)
- No external dependencies that could affect timing
- Configured thresholds at warn level (allows pass while testing structure)

### Check Level Semantics

- `check = "error"`: Threshold violations cause check failure
- `check = "warn"`: Threshold violations are reported but don't fail
- `check = "off"`: Thresholds are not checked (default)

## Verification Plan

| Step | Command | Expected Result |
|------|---------|-----------------|
| New specs compile | `cargo build --test specs` | Success |
| CI output specs pass | `cargo test --test specs ci_output` | 4 passed |
| All CI specs pass | `cargo test --test specs -- ci_` | 13 passed |
| Full suite | `cargo test --test specs` | 565+ passed, 11 ignored |
| Full check | `make check` | All checks pass |

## Completion Criteria

- [ ] `tests/fixtures/tests-ci/` created with minimal Cargo project
- [ ] `tests/specs/checks/tests/ci_output.rs` created with 4 specs
- [ ] All new specs pass
- [ ] No regressions in existing specs
- [ ] `make check` passes
- [ ] `reports/checkpoint-11-tests-ci-mode.md` created
- [ ] Changes committed
- [ ] `./done` executed
