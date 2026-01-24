# Checkpoint 15E: Performance - Ratcheting

**Plan:** `checkpoint-15e-perf`
**Root Feature:** `quench-ratchet-perf`
**Depends On:** checkpoint-15d-benchmark (escapes ratcheting complete)

## Overview

Extend the ratcheting system to support performance metrics: build time, test time, and binary size. This enables CI pipelines to detect performance regressions while allowing tolerances for inherently noisy metrics.

**Current State**: Escapes ratcheting is complete. Data structures exist for `build_time`, `test_time`, `binary_size` in `BaselineMetrics` but comparison logic is not implemented. The `build` check is a stub. Tolerance fields exist in config but aren't parsed.

**End State**:
- Binary size ratcheting with size tolerance (e.g., "100KB")
- Build time ratcheting with duration tolerance (e.g., "5s")
- Test time ratcheting with duration tolerance
- Performance metrics collected in `--ci` mode via build/tests checks
- `make check` passes

## Project Structure

Files to create/modify:

```
crates/cli/src/
├── checks/
│   ├── build/                    # NEW: Build check module
│   │   ├── mod.rs               # Build check implementation
│   │   └── mod_tests.rs         # Unit tests
│   └── tests/                   # MODIFY: Test check module
│       ├── mod.rs               # Add timing collection
│       └── timing.rs            # NEW: Test timing extraction
├── config/
│   └── ratchet.rs               # Add tolerance parsing
├── ratchet.rs                   # Extend with performance comparisons
├── ratchet_tests.rs             # Add performance metric tests
├── tolerance.rs                 # NEW: Duration/size parsing
└── tolerance_tests.rs           # NEW: Tolerance parsing tests

tests/specs/
├── ratchet_perf.rs              # NEW: Performance ratcheting tests
└── fixtures/
    └── ratchet-perf/            # NEW: Test fixtures
```

## Dependencies

No new external dependencies required. Uses existing:
- `std::time::Duration` for timing
- `std::process::Command` for build execution
- Existing `serde` for baseline serialization

## Implementation Phases

### Phase 15E.1: Tolerance Parsing

**Goal**: Parse human-readable duration and size strings for tolerance values.

**Create `crates/cli/src/tolerance.rs`:**

```rust
//! Tolerance value parsing for ratcheting.

use std::time::Duration;

/// Parse a duration string like "5s", "1m30s", "500ms".
pub fn parse_duration(s: &str) -> Result<Duration, ParseError> {
    let s = s.trim();

    // Handle combined format: "1m30s"
    if let Some((min_part, sec_part)) = s.split_once('m') {
        let mins: u64 = min_part.parse()?;
        let secs: f64 = if sec_part.is_empty() {
            0.0
        } else {
            let sec_str = sec_part.trim_end_matches('s');
            sec_str.parse()?
        };
        return Ok(Duration::from_secs(mins * 60) + Duration::from_secs_f64(secs));
    }

    // Handle simple formats: "5s", "500ms"
    if let Some(ms_str) = s.strip_suffix("ms") {
        let ms: u64 = ms_str.parse()?;
        return Ok(Duration::from_millis(ms));
    }

    if let Some(s_str) = s.strip_suffix('s') {
        let secs: f64 = s_str.parse()?;
        return Ok(Duration::from_secs_f64(secs));
    }

    // Plain number = seconds
    let secs: f64 = s.parse()?;
    Ok(Duration::from_secs_f64(secs))
}

/// Parse a size string like "100KB", "5MB", "1GB".
pub fn parse_size(s: &str) -> Result<u64, ParseError> {
    let s = s.trim().to_uppercase();

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix("GB") {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("MB") {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("KB") {
        (n, 1024)
    } else if let Some(n) = s.strip_suffix("B") {
        (n, 1)
    } else {
        // Plain number = bytes
        (s.as_str(), 1)
    };

    let num: f64 = num_str.trim().parse()?;
    Ok((num * multiplier as f64) as u64)
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid number: {0}")]
    InvalidNumber(#[from] std::num::ParseFloatError),
    #[error("invalid integer: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
}
```

**Milestone**: Duration and size strings can be parsed into usable values.

---

### Phase 15E.2: Extend RatchetConfig with Parsed Tolerances

**Goal**: Add methods to `RatchetConfig` for accessing parsed tolerance values.

**Modify `crates/cli/src/config/ratchet.rs`:**

```rust
use crate::tolerance::{parse_duration, parse_size};
use std::time::Duration;

impl RatchetConfig {
    /// Get binary size tolerance in bytes.
    pub fn binary_size_tolerance_bytes(&self) -> Option<u64> {
        self.binary_size_tolerance
            .as_ref()
            .and_then(|s| parse_size(s).ok())
    }

    /// Get build time tolerance as Duration.
    pub fn build_time_tolerance_duration(&self) -> Option<Duration> {
        self.build_time_tolerance
            .as_ref()
            .and_then(|s| parse_duration(s).ok())
    }

    /// Get test time tolerance as Duration (uses build_time_tolerance by default).
    pub fn test_time_tolerance_duration(&self) -> Option<Duration> {
        // Test time uses same tolerance as build time if not separately configured
        self.build_time_tolerance_duration()
    }
}
```

**Add tolerance field for test time (optional):**

```rust
/// Test time tolerance (e.g., "2s"). Defaults to build_time_tolerance.
#[serde(default)]
pub test_time_tolerance: Option<String>,
```

**Milestone**: Config provides parsed tolerances for use in comparison.

---

### Phase 15E.3: Extend CurrentMetrics and Comparison

**Goal**: Add performance metrics to `CurrentMetrics` and extend `compare()`.

**Modify `crates/cli/src/ratchet.rs`:**

```rust
use std::collections::HashMap;
use std::time::Duration;

/// Current metrics extracted from check results.
#[derive(Debug, Clone, Default)]
pub struct CurrentMetrics {
    pub escapes: Option<EscapesCurrent>,
    pub binary_size: Option<HashMap<String, u64>>,
    pub build_time: Option<BuildTimeCurrent>,
    pub test_time: Option<TestTimeCurrent>,
}

/// Current build time metrics.
#[derive(Debug, Clone)]
pub struct BuildTimeCurrent {
    pub cold: Option<Duration>,
    pub hot: Option<Duration>,
}

/// Current test time metrics.
#[derive(Debug, Clone)]
pub struct TestTimeCurrent {
    pub total: Duration,
    pub avg: Duration,
    pub max: Duration,
}

impl CurrentMetrics {
    /// Extract metrics from check output.
    pub fn from_output(output: &CheckOutput) -> Self {
        let mut metrics = Self::default();

        // Escapes (existing)
        if let Some(escapes_result) = output.checks.iter().find(|c| c.name == "escapes") {
            if let Some(ref m) = escapes_result.metrics {
                metrics.escapes = extract_escapes_metrics(m);
            }
        }

        // Build metrics
        if let Some(build_result) = output.checks.iter().find(|c| c.name == "build") {
            if let Some(ref m) = build_result.metrics {
                metrics.binary_size = extract_binary_size(m);
                metrics.build_time = extract_build_time(m);
            }
        }

        // Test metrics
        if let Some(tests_result) = output.checks.iter().find(|c| c.name == "tests") {
            if let Some(ref m) = tests_result.metrics {
                metrics.test_time = extract_test_time(m);
            }
        }

        metrics
    }
}
```

**Extend `compare()` for performance metrics:**

```rust
pub fn compare(
    current: &CurrentMetrics,
    baseline: &BaselineMetrics,
    config: &RatchetConfig,
) -> RatchetResult {
    let mut comparisons = Vec::new();
    let mut improvements = Vec::new();
    let mut passed = true;

    // Escapes (existing code)
    // ...

    // Binary size: ratchets down (smaller is better)
    if config.binary_size {
        if let (Some(curr), Some(base)) = (&current.binary_size, &baseline.binary_size) {
            let tolerance = config.binary_size_tolerance_bytes().unwrap_or(0);
            for (target, &curr_size) in curr {
                let base_size = base.get(target).copied().unwrap_or(0);
                let max_allowed = base_size.saturating_add(tolerance);

                let comparison = MetricComparison {
                    name: format!("binary_size.{}", target),
                    current: curr_size as f64,
                    baseline: base_size as f64,
                    tolerance: tolerance as f64,
                    min_allowed: max_allowed as f64,
                    passed: curr_size <= max_allowed,
                    improved: curr_size < base_size,
                };

                if !comparison.passed {
                    passed = false;
                }
                if comparison.improved {
                    improvements.push(MetricImprovement {
                        name: comparison.name.clone(),
                        old_value: base_size as f64,
                        new_value: curr_size as f64,
                    });
                }
                comparisons.push(comparison);
            }
        }
    }

    // Build time cold: ratchets down (faster is better)
    if config.build_time_cold {
        compare_timing(
            "build_time.cold",
            current.build_time.as_ref().and_then(|t| t.cold),
            baseline.build_time.as_ref().map(|t| t.cold),
            config.build_time_tolerance_duration(),
            &mut comparisons,
            &mut improvements,
            &mut passed,
        );
    }

    // Build time hot: ratchets down
    if config.build_time_hot {
        compare_timing(
            "build_time.hot",
            current.build_time.as_ref().and_then(|t| t.hot),
            baseline.build_time.as_ref().map(|t| t.hot),
            config.build_time_tolerance_duration(),
            &mut comparisons,
            &mut improvements,
            &mut passed,
        );
    }

    // Test time total/avg/max: ratchet down
    if config.test_time_total {
        compare_timing(
            "test_time.total",
            current.test_time.as_ref().map(|t| t.total),
            baseline.test_time.as_ref().map(|t| t.total),
            config.test_time_tolerance_duration(),
            &mut comparisons,
            &mut improvements,
            &mut passed,
        );
    }

    // ... similar for test_time_avg, test_time_max

    RatchetResult { passed, comparisons, improvements }
}

fn compare_timing(
    name: &str,
    current: Option<Duration>,
    baseline: Option<f64>,
    tolerance: Option<Duration>,
    comparisons: &mut Vec<MetricComparison>,
    improvements: &mut Vec<MetricImprovement>,
    passed: &mut bool,
) {
    if let (Some(curr), Some(base)) = (current, baseline) {
        let curr_secs = curr.as_secs_f64();
        let tolerance_secs = tolerance.map(|d| d.as_secs_f64()).unwrap_or(0.0);
        let max_allowed = base + tolerance_secs;

        let comparison = MetricComparison {
            name: name.to_string(),
            current: curr_secs,
            baseline: base,
            tolerance: tolerance_secs,
            min_allowed: max_allowed,
            passed: curr_secs <= max_allowed,
            improved: curr_secs < base,
        };

        if !comparison.passed {
            *passed = false;
        }
        if comparison.improved {
            improvements.push(MetricImprovement {
                name: name.to_string(),
                old_value: base,
                new_value: curr_secs,
            });
        }
        comparisons.push(comparison);
    }
}
```

**Milestone**: Ratchet comparison supports binary_size, build_time, and test_time.

---

### Phase 15E.4: Build Check Implementation

**Goal**: Implement the build check to collect binary size and build time metrics.

**Create `crates/cli/src/checks/build/mod.rs`:**

```rust
//! Build check: binary size and build time metrics.
//!
//! CI-only check that measures:
//! - Binary sizes for configured targets
//! - Cold build time (clean build)
//! - Hot build time (incremental build)

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::adapter::Adapter;
use crate::check::{Check, CheckContext, CheckResult};

pub struct BuildCheck;

impl Check for BuildCheck {
    fn name(&self) -> &'static str {
        "build"
    }

    fn description(&self) -> &'static str {
        "Build metrics (size, time)"
    }

    fn default_enabled(&self) -> bool {
        false // CI-only
    }

    fn ci_only(&self) -> bool {
        true
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        // Skip if not in CI mode
        if !ctx.ci_mode {
            return CheckResult::stub(self.name());
        }

        let mut metrics = BuildMetrics::default();

        // Get adapter for build commands
        let adapter = ctx.adapter.as_ref();

        // Measure binary sizes
        if let Some(targets) = adapter.and_then(|a| a.build_targets()) {
            for target in targets {
                if let Some(size) = measure_binary_size(&ctx.root, &target) {
                    metrics.sizes.insert(target, size);
                }
            }
        }

        // Measure build times (only if configured)
        if ctx.config.ratchet.build_time_cold || ctx.config.ratchet.build_time_hot {
            if let Some(adapter) = adapter {
                if ctx.config.ratchet.build_time_cold {
                    metrics.time_cold = measure_cold_build(&ctx.root, adapter);
                }
                if ctx.config.ratchet.build_time_hot {
                    metrics.time_hot = measure_hot_build(&ctx.root, adapter);
                }
            }
        }

        // Build result with metrics
        CheckResult::passed(self.name()).with_metrics(metrics.to_json())
    }
}

#[derive(Default)]
struct BuildMetrics {
    sizes: std::collections::HashMap<String, u64>,
    time_cold: Option<Duration>,
    time_hot: Option<Duration>,
}

impl BuildMetrics {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "size": self.sizes,
            "time": {
                "cold": self.time_cold.map(|d| d.as_secs_f64()),
                "hot": self.time_hot.map(|d| d.as_secs_f64()),
            }
        })
    }
}

fn measure_binary_size(root: &Path, target: &str) -> Option<u64> {
    // Look in target/release for Rust binaries
    let binary_path = root.join("target/release").join(target);
    std::fs::metadata(&binary_path).ok().map(|m| m.len())
}

fn measure_cold_build(root: &Path, adapter: &dyn Adapter) -> Option<Duration> {
    // Clean first
    let clean_cmd = adapter.clean_command()?;
    Command::new(&clean_cmd[0])
        .args(&clean_cmd[1..])
        .current_dir(root)
        .output()
        .ok()?;

    // Time the build
    let build_cmd = adapter.build_command()?;
    let start = Instant::now();
    let output = Command::new(&build_cmd[0])
        .args(&build_cmd[1..])
        .current_dir(root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(start.elapsed())
    } else {
        None
    }
}

fn measure_hot_build(root: &Path, adapter: &dyn Adapter) -> Option<Duration> {
    // Touch a source file to trigger incremental rebuild
    if let Some(touch_cmd) = adapter.touch_command() {
        Command::new(&touch_cmd[0])
            .args(&touch_cmd[1..])
            .current_dir(root)
            .output()
            .ok()?;
    }

    // Time the build
    let build_cmd = adapter.build_command()?;
    let start = Instant::now();
    let output = Command::new(&build_cmd[0])
        .args(&build_cmd[1..])
        .current_dir(root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(start.elapsed())
    } else {
        None
    }
}
```

**Add to adapter trait** (`crates/cli/src/adapter/mod.rs`):

```rust
/// Commands for build metrics collection.
fn build_targets(&self) -> Option<Vec<String>> { None }
fn clean_command(&self) -> Option<Vec<String>> { None }
fn build_command(&self) -> Option<Vec<String>> { None }
fn touch_command(&self) -> Option<Vec<String>> { None }
```

**Implement for Rust adapter:**

```rust
fn build_targets(&self) -> Option<Vec<String>> {
    // Parse Cargo.toml [[bin]] sections
    // ...
}

fn clean_command(&self) -> Option<Vec<String>> {
    Some(vec!["cargo".into(), "clean".into()])
}

fn build_command(&self) -> Option<Vec<String>> {
    Some(vec!["cargo".into(), "build".into(), "--release".into()])
}

fn touch_command(&self) -> Option<Vec<String>> {
    Some(vec!["touch".into(), "src/lib.rs".into()])
}
```

**Milestone**: Build check collects binary size and build time metrics.

---

### Phase 15E.5: Test Time Metrics

**Goal**: Extend the tests check to collect timing metrics.

**Add timing extraction to tests check** (`crates/cli/src/checks/tests/mod.rs`):

```rust
use std::process::Command;
use std::time::{Duration, Instant};

/// Collected test timing metrics.
#[derive(Debug, Clone, Default)]
pub struct TestTimingMetrics {
    pub total: Duration,
    pub avg: Duration,
    pub max: Duration,
    pub slowest_test: Option<String>,
}

impl TestTimingMetrics {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total": self.total.as_secs_f64(),
            "avg": self.avg.as_secs_f64(),
            "max": self.max.as_secs_f64(),
            "slowest_test": self.slowest_test,
        })
    }
}

/// Run tests and collect timing metrics.
fn run_tests_with_timing(root: &Path, adapter: &dyn Adapter) -> Option<TestTimingMetrics> {
    let test_cmd = adapter.test_command()?;

    let start = Instant::now();
    let output = Command::new(&test_cmd[0])
        .args(&test_cmd[1..])
        .current_dir(root)
        .output()
        .ok()?;
    let total = start.elapsed();

    if !output.status.success() {
        return None;
    }

    // Parse test output for per-test timing
    let (avg, max, slowest) = parse_test_timing(&output.stdout, &output.stderr);

    Some(TestTimingMetrics {
        total,
        avg,
        max,
        slowest_test: slowest,
    })
}

/// Parse test output to extract per-test timing.
fn parse_test_timing(stdout: &[u8], _stderr: &[u8]) -> (Duration, Duration, Option<String>) {
    // For cargo test --format=json, parse test durations
    // For now, use total/test_count for avg, no max detection
    let output = String::from_utf8_lossy(stdout);

    // Count "test ... ok" lines for rough avg
    let test_count = output.matches("test ").count().max(1);

    // TODO: Parse JSON test output for accurate per-test times
    let avg = Duration::ZERO;
    let max = Duration::ZERO;

    (avg, max, None)
}
```

**Milestone**: Tests check collects timing metrics in CI mode.

---

### Phase 15E.6: Baseline Update for Performance Metrics

**Goal**: Update baseline with performance metrics when improved.

**Extend `update_baseline()` in `crates/cli/src/ratchet.rs`:**

```rust
pub fn update_baseline(
    baseline: &mut Baseline,
    current: &CurrentMetrics,
    _improvements: &[MetricImprovement],
) {
    // Update escapes (existing)
    // ...

    // Update binary size
    if let Some(curr_sizes) = &current.binary_size {
        let base_sizes = baseline.metrics.binary_size.get_or_insert_with(HashMap::new);
        for (target, &size) in curr_sizes {
            base_sizes.insert(target.clone(), size);
        }
    }

    // Update build time
    if let Some(curr_time) = &current.build_time {
        let base_time = baseline.metrics.build_time.get_or_insert_with(|| {
            crate::baseline::BuildTimeMetrics { cold: 0.0, hot: 0.0 }
        });
        if let Some(cold) = curr_time.cold {
            base_time.cold = cold.as_secs_f64();
        }
        if let Some(hot) = curr_time.hot {
            base_time.hot = hot.as_secs_f64();
        }
    }

    // Update test time
    if let Some(curr_time) = &current.test_time {
        baseline.metrics.test_time = Some(crate::baseline::TestTimeMetrics {
            total: curr_time.total.as_secs_f64(),
            avg: curr_time.avg.as_secs_f64(),
            max: curr_time.max.as_secs_f64(),
        });
    }

    baseline.touch();
}
```

**Milestone**: Performance metrics are persisted to baseline file.

---

### Phase 15E.7: Testing and Quality Gates

**Goal**: Add behavioral tests and verify all quality gates pass.

**Create `tests/specs/ratchet_perf.rs`:**

```rust
//! Behavioral tests for performance ratcheting.

use crate::prelude::*;

#[test]
fn binary_size_regression_fails() {
    let project = TempProject::new()
        .file("quench.toml", r#"
version = 1
[ratchet]
check = "error"
binary_size = true
"#)
        .file(".quench/baseline.json", r#"{
  "version": 1,
  "updated": "2026-01-20T00:00:00Z",
  "metrics": {
    "binary_size": { "myapp": 1000000 }
  }
}"#);

    // Create a mock larger binary
    project.create_file("target/release/myapp", &vec![0u8; 1_500_000]);

    cli()
        .on(&project)
        .arg("check")
        .arg("--ci")
        .fails()
        .stdout_has("binary_size.myapp: 1500000 (max: 1000000 from baseline)");
}

#[test]
fn binary_size_within_tolerance_passes() {
    let project = TempProject::new()
        .file("quench.toml", r#"
version = 1
[ratchet]
check = "error"
binary_size = true
binary_size_tolerance = "100KB"
"#)
        .file(".quench/baseline.json", r#"{
  "version": 1,
  "updated": "2026-01-20T00:00:00Z",
  "metrics": {
    "binary_size": { "myapp": 1000000 }
  }
}"#);

    // Create binary slightly larger but within tolerance
    project.create_file("target/release/myapp", &vec![0u8; 1_050_000]);

    cli()
        .on(&project)
        .arg("check")
        .arg("--ci")
        .succeeds();
}

#[test]
fn build_time_disabled_by_default() {
    let project = TempProject::new()
        .file("quench.toml", r#"
version = 1
[ratchet]
check = "error"
"#);

    // Build time metrics should not be collected or checked by default
    cli()
        .on(&project)
        .arg("check")
        .succeeds();
}
```

**Add unit tests to `crates/cli/src/tolerance_tests.rs`:**

```rust
use super::*;

#[test]
fn parse_duration_seconds() {
    assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
    assert_eq!(parse_duration("1.5s").unwrap(), Duration::from_secs_f64(1.5));
}

#[test]
fn parse_duration_milliseconds() {
    assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
}

#[test]
fn parse_duration_combined() {
    assert_eq!(parse_duration("1m30s").unwrap(), Duration::from_secs(90));
}

#[test]
fn parse_size_bytes() {
    assert_eq!(parse_size("1024").unwrap(), 1024);
    assert_eq!(parse_size("1024B").unwrap(), 1024);
}

#[test]
fn parse_size_kilobytes() {
    assert_eq!(parse_size("100KB").unwrap(), 100 * 1024);
}

#[test]
fn parse_size_megabytes() {
    assert_eq!(parse_size("5MB").unwrap(), 5 * 1024 * 1024);
}
```

**Run quality gates:**

```bash
cargo test --all
make check
```

**Milestone**: All tests pass, `make check` passes.

---

## Key Implementation Details

### Metric Collection Flow

1. In `--ci` mode, build and tests checks execute actual commands
2. Build check measures binary sizes and optionally build times
3. Tests check measures total/avg/max test duration
4. Metrics returned via `CheckResult::with_metrics()`
5. `CurrentMetrics::from_output()` extracts all metric types
6. `ratchet::compare()` compares with tolerances

### Tolerance Application

Tolerance is added to the baseline value for "ratchet down" metrics:

```
max_allowed = baseline + tolerance
passed = current <= max_allowed
```

For example, with `binary_size_tolerance = "100KB"`:
- Baseline: 1,000,000 bytes
- Tolerance: 102,400 bytes
- Max allowed: 1,102,400 bytes
- Current 1,050,000 = PASS
- Current 1,200,000 = FAIL

### CI-Only Metrics

Build time and test time metrics are inherently CI-only:
- Require actual build/test execution
- Too slow for interactive use
- Environment-dependent (different on CI vs local)

The `ci_only()` trait method on checks gates execution.

### Ratchet Direction by Metric

| Metric | Good Direction | Tolerance Type |
|--------|----------------|----------------|
| binary_size | Smaller | Size (bytes) |
| build_time_cold | Faster | Duration |
| build_time_hot | Faster | Duration |
| test_time_total | Faster | Duration |
| test_time_avg | Faster | Duration |
| test_time_max | Faster | Duration |

### Adapter Integration

Language adapters provide build/test commands:

| Adapter | Clean | Build | Touch | Test |
|---------|-------|-------|-------|------|
| Rust | `cargo clean` | `cargo build --release` | `touch src/lib.rs` | `cargo test` |
| Go | `go clean -cache` | `go build ./...` | `touch main.go` | `go test ./...` |
| JS/TS | `rm -rf dist` | `npm run build` | - | `npm test` |

## Verification Plan

### Automated Verification

```bash
# Unit tests
cargo test tolerance
cargo test ratchet

# Behavioral tests
cargo test --test specs ratchet_perf

# Full suite
cargo test --all

# Quality gates
make check
```

### Manual Verification

```bash
# Test tolerance parsing
cargo test tolerance -- --nocapture

# Test binary size ratcheting
TEMP=$(mktemp -d) && cd "$TEMP"
cargo init --lib
mkdir -p .quench target/release
cat > quench.toml << 'EOF'
version = 1
[ratchet]
check = "error"
binary_size = true
binary_size_tolerance = "100KB"
EOF
echo '{"version":1,"updated":"2026-01-20T00:00:00Z","metrics":{"binary_size":{"myapp":1000000}}}' > .quench/baseline.json
dd if=/dev/zero of=target/release/myapp bs=1 count=1050000 2>/dev/null
quench check --ci && echo "PASS: within tolerance"
dd if=/dev/zero of=target/release/myapp bs=1 count=1200000 2>/dev/null
quench check --ci || echo "PASS: regression detected"
cd - && rm -rf "$TEMP"
```

### Success Criteria

- [ ] Duration strings parsed ("5s", "1m30s", "500ms")
- [ ] Size strings parsed ("100KB", "5MB")
- [ ] Binary size ratcheting with tolerance works
- [ ] Build time cold/hot ratcheting works
- [ ] Test time total/avg/max ratcheting works
- [ ] Performance metrics collected only in `--ci` mode
- [ ] Baseline updated with performance metrics on improvement
- [ ] All existing tests pass
- [ ] `make check` passes
