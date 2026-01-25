# Phase 955: Tests Check - CI Mode Thresholds

## Overview

Implement threshold checking for test coverage and timing in CI mode. When thresholds are exceeded, generate violations that can be configured to either error or warn. This enables teams to enforce minimum coverage requirements and catch slow tests.

**Violation types:**
- `coverage_below_min` - coverage below global or per-package minimum
- `time_total_exceeded` - suite total time exceeds `max_total`
- `time_avg_exceeded` - average test time exceeds `max_avg`
- `time_test_exceeded` - slowest test exceeds `max_test`

## Project Structure

```
crates/cli/src/
├── config/
│   └── tests_check.rs          # UPDATE: add TestsCoverageConfig
├── checks/tests/
│   └── mod.rs                  # UPDATE: add threshold checking logic
└── cache.rs                    # UPDATE: bump CACHE_VERSION

tests/specs/checks/tests/
└── ci_metrics.rs               # UPDATE: remove #[ignore] from specs, add time_avg spec
```

## Dependencies

No new external dependencies. Uses existing:
- `serde` for configuration deserialization
- `std::collections::HashMap` for per-package thresholds
- Existing `Violation` and `CheckResult` types

## Implementation Phases

### Phase 1: Coverage Configuration

Add `TestsCoverageConfig` struct to support coverage thresholds.

**File:** `crates/cli/src/config/tests_check.rs`

```rust
/// Coverage threshold configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestsCoverageConfig {
    /// Check level: "error" | "warn" | "off"
    #[serde(default = "TestsCoverageConfig::default_check")]
    pub check: String,

    /// Minimum overall coverage percentage (0-100).
    #[serde(default)]
    pub min: Option<f64>,

    /// Per-package coverage thresholds.
    #[serde(default)]
    pub package: std::collections::HashMap<String, TestsPackageCoverageConfig>,
}

/// Per-package coverage threshold.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestsPackageCoverageConfig {
    /// Minimum coverage percentage for this package.
    pub min: f64,
}

impl Default for TestsCoverageConfig {
    fn default() -> Self {
        Self {
            check: Self::default_check(),
            min: None,
            package: std::collections::HashMap::new(),
        }
    }
}

impl TestsCoverageConfig {
    fn default_check() -> String {
        "off".to_string()
    }
}
```

Add to `TestsConfig`:

```rust
pub struct TestsConfig {
    // ... existing fields ...

    /// Coverage threshold checking.
    #[serde(default)]
    pub coverage: TestsCoverageConfig,
}
```

**Verification:** `cargo test --lib config`

### Phase 2: Coverage Threshold Checking

Implement coverage threshold validation in `run_test_suites()`.

**File:** `crates/cli/src/checks/tests/mod.rs`

Add after coverage aggregation (around line 203):

```rust
// Check coverage thresholds if configured
let coverage_violations = self.check_coverage_thresholds(
    ctx,
    &aggregated_coverage,
    &packages_coverage,
);
```

Add helper method to `TestsCheck`:

```rust
impl TestsCheck {
    /// Check coverage against configured thresholds.
    fn check_coverage_thresholds(
        &self,
        ctx: &CheckContext,
        coverage: &std::collections::HashMap<String, f64>,
        packages: &std::collections::HashMap<String, f64>,
    ) -> Vec<(Violation, bool)> {
        let config = &ctx.config.check.tests.coverage;
        if config.check == "off" {
            return Vec::new();
        }

        let is_error = config.check == "error";
        let mut violations = Vec::new();

        // Check global minimum (use first coverage value, typically "line" or "rust")
        if let Some(min) = config.min {
            for (lang, &actual) in coverage {
                if actual < min {
                    let advice = format!(
                        "Coverage {:.1}% below minimum {:.1}%",
                        actual, min
                    );
                    let v = Violation::file_only(
                        format!("<coverage:{}>", lang),
                        "coverage_below_min",
                        advice,
                    )
                    .with_threshold(actual as i64, min as i64);
                    violations.push((v, is_error));
                }
            }
        }

        // Check per-package thresholds
        for (pkg, pkg_config) in &config.package {
            if let Some(&actual) = packages.get(pkg) {
                if actual < pkg_config.min {
                    let advice = format!(
                        "Package '{}' coverage {:.1}% below minimum {:.1}%",
                        pkg, actual, pkg_config.min
                    );
                    let v = Violation::file_only(
                        format!("<coverage:{}>", pkg),
                        "coverage_below_min",
                        advice,
                    )
                    .with_threshold(actual as i64, pkg_config.min as i64);
                    violations.push((v, is_error));
                }
            }
        }

        violations
    }
}
```

**Verification:** `cargo test --test specs coverage_below_min`

### Phase 3: Time Threshold Checking

Implement time threshold validation for `max_total`, `max_avg`, and `max_test`.

**File:** `crates/cli/src/checks/tests/mod.rs`

Add after suite execution (in `run_suites()` or in threshold checking):

```rust
impl TestsCheck {
    /// Check time thresholds for a suite.
    fn check_time_thresholds(
        &self,
        ctx: &CheckContext,
        suite: &TestSuiteConfig,
        result: &SuiteResult,
    ) -> Vec<(Violation, bool)> {
        let config = &ctx.config.check.tests.time;
        if config.check == "off" {
            return Vec::new();
        }

        let is_error = config.check == "error";
        let mut violations = Vec::new();
        let suite_name = &result.name;

        // Check max_total
        if let Some(max_total) = suite.max_total {
            let max_ms = max_total.as_millis() as u64;
            if result.total_ms > max_ms {
                let advice = format!(
                    "Suite '{}' took {}ms, exceeds max_total {}ms",
                    suite_name, result.total_ms, max_ms
                );
                let v = Violation::file_only(
                    format!("<suite:{}>", suite_name),
                    "time_total_exceeded",
                    advice,
                )
                .with_threshold(result.total_ms as i64, max_ms as i64);
                violations.push((v, is_error));
            }
        }

        // Check max_avg
        if let Some(max_avg) = suite.max_avg {
            if let Some(avg_ms) = result.avg_ms {
                let max_ms = max_avg.as_millis() as u64;
                if avg_ms > max_ms {
                    let advice = format!(
                        "Suite '{}' average {}ms/test, exceeds max_avg {}ms",
                        suite_name, avg_ms, max_ms
                    );
                    let v = Violation::file_only(
                        format!("<suite:{}>", suite_name),
                        "time_avg_exceeded",
                        advice,
                    )
                    .with_threshold(avg_ms as i64, max_ms as i64);
                    violations.push((v, is_error));
                }
            }
        }

        // Check max_test
        if let Some(max_test) = suite.max_test {
            if let Some(max_ms) = result.max_ms {
                let threshold_ms = max_test.as_millis() as u64;
                if max_ms > threshold_ms {
                    let test_name = result.max_test.as_deref().unwrap_or("unknown");
                    let advice = format!(
                        "Test '{}' took {}ms, exceeds max_test {}ms",
                        test_name, max_ms, threshold_ms
                    );
                    let v = Violation::file_only(
                        format!("<test:{}>", test_name),
                        "time_test_exceeded",
                        advice,
                    )
                    .with_threshold(max_ms as i64, threshold_ms as i64);
                    violations.push((v, is_error));
                }
            }
        }

        violations
    }
}
```

**Verification:** `cargo test --test specs time_total_exceeded time_test_exceeded`

### Phase 4: Integrate Threshold Checking

Wire threshold checking into `run_test_suites()` and update result building.

**File:** `crates/cli/src/checks/tests/mod.rs`

Modify `run_test_suites()` to:
1. Collect threshold violations alongside suite failures
2. Respect check levels (error vs warn)
3. Return appropriate `CheckResult`

```rust
fn run_test_suites(&self, ctx: &CheckContext) -> CheckResult {
    let suite_results = match self.run_suites(ctx) {
        Some(r) => r,
        None => return CheckResult::passed(self.name()),
    };

    // ... existing metrics aggregation ...

    // Collect coverage threshold violations
    let coverage_violations = self.check_coverage_thresholds(
        ctx,
        &aggregated_coverage,
        &packages_coverage,
    );

    // Collect time threshold violations from each suite
    let mut time_violations = Vec::new();
    let active_suites = filter_suites_for_mode(&ctx.config.check.tests.suite, ctx.ci_mode);
    for (suite, result) in active_suites.iter().zip(suite_results.suites.iter()) {
        time_violations.extend(self.check_time_thresholds(ctx, suite, result));
    }

    // Combine all violations
    let all_threshold_violations: Vec<(Violation, bool)> = coverage_violations
        .into_iter()
        .chain(time_violations)
        .collect();

    // Build final result
    let has_threshold_errors = all_threshold_violations.iter().any(|(_, is_err)| *is_err);
    let threshold_violations: Vec<Violation> = all_threshold_violations
        .into_iter()
        .map(|(v, _)| v)
        .collect();

    // ... build metrics as before ...

    if suite_results.passed && threshold_violations.is_empty() {
        CheckResult::passed(self.name()).with_metrics(metrics)
    } else if !suite_results.passed {
        // Suite failures take precedence
        let mut violations = /* existing suite failure violations */;
        violations.extend(threshold_violations);
        CheckResult::failed(self.name(), violations).with_metrics(metrics)
    } else if has_threshold_errors {
        CheckResult::failed(self.name(), threshold_violations).with_metrics(metrics)
    } else {
        CheckResult::passed_with_warnings(self.name(), threshold_violations).with_metrics(metrics)
    }
}
```

**Verification:** `cargo test --test specs ci_metrics`

### Phase 5: Specs and Cache Version

1. Remove `#[ignore]` from existing specs in `ci_metrics.rs`
2. Add spec for `time_avg_exceeded`
3. Bump `CACHE_VERSION` in `cache.rs`

**File:** `tests/specs/checks/tests/ci_metrics.rs`

Add spec for average time threshold:

```rust
/// Spec: docs/specs/11-test-runners.md#thresholds
///
/// > max_avg = "100ms"
#[test]
fn time_avg_exceeded_generates_violation() {
    let temp = Project::empty();
    temp.config(
        r#"
[[check.tests.suite]]
runner = "cargo"
max_avg = "1ns"

[check.tests.time]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    temp.file(
        "tests/basic.rs",
        r#"
#[test]
fn test_add() { assert_eq!(test_project::add(1, 2), 3); }
"#,
    );

    let result = check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .json()
        .fails();

    assert!(result.has_violation("time_avg_exceeded"));
}
```

Update `tests_ci_violation_types_are_documented()` to include `time_avg_exceeded`.

**File:** `crates/cli/src/cache.rs`

```rust
// Bump from 25 to 26 for threshold violation changes
const CACHE_VERSION: u32 = 26;
```

**Verification:** `cargo test --test specs ci_metrics && make check`

## Key Implementation Details

### Violation Structure

All threshold violations use consistent structure:

| Field | Description |
|-------|-------------|
| `file` | Synthetic path like `<suite:cargo>` or `<coverage:rust>` |
| `type` | `coverage_below_min`, `time_total_exceeded`, etc. |
| `advice` | Human-readable description of the violation |
| `value` | Actual measured value |
| `threshold` | Configured limit |

### Check Level Behavior

| Level | Violation Behavior |
|-------|-------------------|
| `error` | Violations cause check to fail |
| `warn` | Violations reported but check passes |
| `off` | No threshold checking performed |

### Configuration Example

```toml
[check.tests.coverage]
check = "error"
min = 75

[check.tests.coverage.package.core]
min = 90

[check.tests.time]
check = "warn"

[[check.tests.suite]]
runner = "cargo"
max_total = "30s"
max_avg = "100ms"
max_test = "1s"
```

### Edge Cases

1. **No coverage data**: Skip coverage checks if coverage wasn't collected (non-CI mode or missing tools)
2. **No tests run**: Skip time checks if test_count is 0
3. **Mixed check levels**: Coverage and time have independent check levels
4. **Multiple suites**: Time thresholds are per-suite; coverage is global

## Verification Plan

### Unit Tests

```bash
# Config parsing
cargo test --lib config::tests_check

# Threshold logic
cargo test --lib checks::tests
```

### Behavioral Specs

```bash
# All CI metrics specs
cargo test --test specs ci_metrics

# Individual specs
cargo test --test specs coverage_below_min
cargo test --test specs per_package_coverage
cargo test --test specs time_total_exceeded
cargo test --test specs time_avg_exceeded
cargo test --test specs time_test_exceeded
```

### Integration

```bash
# Full check suite
make check

# Manual verification
cd /path/to/project
quench check --ci -o json 2>&1 | jq '.checks[] | select(.name == "tests")'
```

### Expected Spec Results

| Spec | Status |
|------|--------|
| `coverage_below_min_generates_violation` | Pass (with llvm-cov) |
| `per_package_coverage_thresholds_work` | Pass (with llvm-cov) |
| `time_total_exceeded_generates_violation` | Pass |
| `time_avg_exceeded_generates_violation` | Pass |
| `time_test_exceeded_generates_violation` | Pass |
| `tests_ci_violation_types_are_documented` | Pass |

## Commit Strategy

Single commit after all phases complete:

```
feat(tests): add CI mode threshold checking

Threshold violations for tests check:
- coverage.min: global minimum coverage
- coverage.package.<name>.min: per-package minimum
- suite.max_total: total suite time limit
- suite.max_avg: average test time limit
- suite.max_test: slowest test time limit

Check levels via check.tests.coverage.check and check.tests.time.check
control whether violations are errors or warnings.

Specs:
- coverage_below_min_generates_violation
- per_package_coverage_thresholds_work
- time_total_exceeded_generates_violation
- time_avg_exceeded_generates_violation
- time_test_exceeded_generates_violation
```
