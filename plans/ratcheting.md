# Metric Ratcheting Implementation Plan

## Overview

Implement metric ratcheting: baseline tracking for coverage, escapes, build size, and timing metrics with regression detection and `--fix` updates. Ratcheting prevents quality regressions while allowing gradual improvement—metrics can improve but never regress past the baseline.

**Spec Reference:** `docs/specs/04-ratcheting.md`

## Project Structure

```
crates/cli/src/
├── baseline.rs          # ✅ Baseline file I/O (exists)
├── baseline_tests.rs    # ✅ Unit tests (exists)
├── ratchet.rs           # 🔄 Comparison logic (needs coverage)
├── ratchet_tests.rs     # 🔄 Unit tests (needs expansion)
├── tolerance.rs         # ✅ Duration/size parsing (exists)
├── tolerance_tests.rs   # ✅ Unit tests (exists)
├── cmd_check.rs         # ✅ --fix integration (exists)
├── output/
│   └── text.rs          # ✅ Ratchet output formatting (exists)
└── config/
    └── ratchet.rs       # ✅ RatchetConfig schema (exists)

tests/
├── specs/modes/
│   └── ratchet.rs       # 🔄 Behavioral specs (needs expansion)
└── fixtures/
    └── report/with-baseline/  # ✅ Baseline fixture (exists)
```

## Dependencies

No new external dependencies required. Uses existing:
- `serde`, `serde_json` - Serialization
- `chrono` - Timestamps
- `thiserror` - Error types

## Implementation Phases

### Phase 1: Coverage Ratcheting (Priority: High)

**Goal:** Wire up coverage metrics to ratcheting system.

**Current State:** Coverage is collected by test runners but not extracted into `CurrentMetrics` or compared against baseline.

**Files to Modify:**
- `crates/cli/src/ratchet.rs`

**Implementation:**

1. Add `coverage` field to `CurrentMetrics`:
```rust
pub struct CurrentMetrics {
    pub escapes: Option<EscapesCurrent>,
    pub coverage: Option<CoverageCurrent>,  // ADD
    pub binary_size: Option<HashMap<String, u64>>,
    pub build_time: Option<BuildTimeCurrent>,
    pub test_time: Option<TestTimeCurrent>,
}

#[derive(Debug, Clone)]
pub struct CoverageCurrent {
    pub total: f64,
    pub by_package: HashMap<String, f64>,
}
```

2. Extract coverage from tests check output in `from_output()`:
```rust
// Extract coverage metrics from tests check
if let Some(tests_result) = output.checks.iter().find(|c| c.name == "tests")
    && let Some(ref metrics_json) = tests_result.metrics
{
    metrics.coverage = extract_coverage_metrics(metrics_json);
    metrics.test_time = extract_test_time(metrics_json);
}

fn extract_coverage_metrics(json: &serde_json::Value) -> Option<CoverageCurrent> {
    let coverage = json.get("coverage")?;

    // Extract total from first language (typically "rust" or "typescript")
    let total = coverage.as_object()?.values().next()?.as_f64()?;

    // Extract per-package if available
    let by_package = json
        .get("coverage_by_package")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
                .collect()
        })
        .unwrap_or_default();

    Some(CoverageCurrent { total, by_package })
}
```

3. Add coverage comparison to `compare()`:
```rust
// Coverage: ratchets UP (higher is better)
if config.coverage
    && let (Some(curr), Some(base)) = (&current.coverage, &baseline.coverage)
{
    let tolerance = config.coverage_tolerance_pct().unwrap_or(0.0);
    let min_allowed = base.total - tolerance;

    let comparison = MetricComparison {
        name: "coverage.total".to_string(),
        current: curr.total,
        baseline: base.total,
        tolerance,
        threshold: min_allowed,  // min allowed (floor)
        passed: curr.total >= min_allowed,
        improved: curr.total > base.total,
    };

    if !comparison.passed {
        passed = false;
    }
    if comparison.improved {
        improvements.push(MetricImprovement {
            name: "coverage.total".to_string(),
            old_value: base.total,
            new_value: curr.total,
        });
    }
    comparisons.push(comparison);
}
```

4. Add coverage update to `update_baseline()`:
```rust
if let Some(curr_cov) = &current.coverage {
    baseline.metrics.coverage = Some(CoverageMetrics {
        total: curr_cov.total,
        by_package: if curr_cov.by_package.is_empty() {
            None
        } else {
            Some(curr_cov.by_package.clone())
        },
    });
}
```

**Tests:**
- Add to `ratchet_tests.rs`:
  - `coverage_regression_fails()`
  - `coverage_within_tolerance_passes()`
  - `coverage_improvement_tracked()`
  - `extract_coverage_from_tests_output()`

**Verification:**
```bash
cargo test --all -- ratchet
cargo test --test specs -- ratchet
```

---

### Phase 2: Behavioral Specs for Non-Escapes Metrics

**Goal:** Add behavioral specs for coverage, binary size, and timing ratchets.

**Files to Create/Modify:**
- `tests/specs/modes/ratchet.rs` (expand existing)

**Specs to Add:**

```rust
/// Spec: docs/specs/04-ratcheting.md#coverage
///
/// > Coverage can't drop below baseline minus tolerance.
#[test]
#[ignore = "TODO: Phase 1202 - Coverage ratcheting"]
fn coverage_regression_fails() {
    let temp = Project::empty();
    temp.config(COVERAGE_RATCHET_CONFIG);
    // ... baseline with 80% coverage
    // ... source with 75% coverage
    cli().pwd(temp.path())
        .fails()
        .stdout_has("coverage.total: 75.0% (min: 80.0% from baseline)");
}

/// Spec: docs/specs/04-ratcheting.md#binary-size
#[test]
#[ignore = "TODO: Phase 1215 - Binary size ratcheting"]
fn binary_size_regression_fails() { ... }

/// Spec: docs/specs/04-ratcheting.md#tolerance
#[test]
fn coverage_within_tolerance_passes() { ... }

/// Spec: docs/specs/04-ratcheting.md#stale-baseline
#[test]
fn stale_baseline_warns() { ... }

/// Spec: docs/specs/04-ratcheting.md#warn-level
#[test]
fn warn_level_reports_but_passes() { ... }
```

**Verification:**
```bash
cargo test --test specs -- ratchet --ignored  # See pending specs
cargo test --test specs -- ratchet            # Run implemented
```

---

### Phase 3: Fix Tolerance Parsing Edge Cases

**Goal:** Ensure tolerance parsing handles all specified formats.

**Current State:** Basic parsing exists but may have edge cases.

**Files to Verify:**
- `crates/cli/src/tolerance.rs`
- `crates/cli/src/tolerance_tests.rs`

**Edge Cases to Test:**
- `"0.5"` → 0.5 percentage points (coverage)
- `"100KB"` → 102400 bytes
- `"5MB"` → 5242880 bytes
- `"1.5s"` → 1.5 seconds
- `"1m30s"` → 90 seconds
- `"500ms"` → 500 milliseconds

**Add Tests if Missing:**
```rust
#[test]
fn parse_fractional_seconds() {
    assert_eq!(parse_duration("1.5s"), Ok(Duration::from_secs_f64(1.5)));
}

#[test]
fn parse_coverage_tolerance_as_percentage_points() {
    // 0.5 means 0.5 percentage points, not 0.5%
    let config = RatchetConfig {
        coverage_tolerance: Some(0.5),
        ..Default::default()
    };
    assert_eq!(config.coverage_tolerance_pct(), Some(0.5));
}
```

**Verification:**
```bash
cargo test -- tolerance
```

---

### Phase 4: Per-Package Ratcheting

**Goal:** Support per-package ratcheting for workspaces.

**Current State:** Config schema exists but comparison doesn't use per-package breakdown.

**Files to Modify:**
- `crates/cli/src/config/ratchet.rs` - Add `RatchetPackageConfig`
- `crates/cli/src/ratchet.rs` - Per-package comparison logic

**Implementation:**

1. Add per-package config (if not present):
```rust
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RatchetConfig {
    // ... existing fields ...

    /// Per-package ratchet settings.
    #[serde(default)]
    pub package: HashMap<String, RatchetPackageConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RatchetPackageConfig {
    pub coverage: Option<bool>,
    pub escapes: Option<bool>,
}
```

2. Add per-package comparison:
```rust
// In compare(), for coverage:
if let Some(by_pkg) = &curr.coverage.by_package
    && let Some(base_by_pkg) = baseline.coverage.as_ref().and_then(|c| c.by_package.as_ref())
{
    for (pkg, &curr_pct) in by_pkg {
        // Check if this package has ratcheting enabled
        let pkg_config = config.package.get(pkg);
        if pkg_config.is_some_and(|c| c.coverage == Some(false)) {
            continue; // Skip disabled packages
        }

        if let Some(&base_pct) = base_by_pkg.get(pkg) {
            // Compare package coverage...
        }
    }
}
```

**Specs to Add:**
```rust
/// Spec: docs/specs/04-ratcheting.md#per-package
#[test]
#[ignore = "TODO: Phase 1225 - Per-package ratcheting"]
fn per_package_coverage_ratchet() { ... }
```

**Verification:**
```bash
cargo test -- ratchet::package
cargo test --test specs -- per_package
```

---

### Phase 5: Output Format Alignment

**Goal:** Ensure output matches spec exactly.

**Current State:** Basic output exists but may differ from spec.

**Spec Output (regression):**
```
ratchet: FAIL
  escapes.unsafe: 5 (max: 3 from baseline)
    Reduce unsafe blocks or add // SAFETY: comments.
```

**Spec Output (improvement with --fix):**
```
ratchet: updated baseline
  coverage: 78.4% → 82.1% (new floor)
  escapes.unsafe: 5 → 3 (new ceiling)
```

**Files to Verify:**
- `crates/cli/src/output/text.rs` - `write_ratchet()`
- `crates/cli/src/cmd_check.rs` - --fix output messages

**Specs to Add:**
```rust
/// Spec: docs/specs/04-ratcheting.md#output
#[test]
fn ratchet_fail_output_format() {
    // Test exact output format for failures
    cli().pwd(temp.path())
        .fails()
        .stdout_eq("ratchet: FAIL\n  escapes.unsafe: 5 (max: 3 from baseline)\n    Reduce unsafe blocks or add // SAFETY: comments.\n");
}
```

**Verification:**
```bash
cargo test --test specs -- output_format
```

---

### Phase 6: JSON Output Schema

**Goal:** Ensure JSON output includes ratchet section per spec.

**Spec JSON:**
```json
{
  "ratchet": {
    "coverage": {
      "current": 76.2,
      "baseline": 78.4,
      "tolerance": 0.5,
      "min_allowed": 77.9,
      "passed": false
    }
  }
}
```

**Files to Verify:**
- `crates/cli/src/output/json.rs`

**Verification:**
```bash
quench check --ci -o json | jq '.ratchet'
```

---

## Key Implementation Details

### Ratchet Direction by Metric Type

| Metric | Direction | Threshold Meaning |
|--------|-----------|-------------------|
| Coverage | UP (higher is better) | Floor (min allowed) |
| Escapes | DOWN (lower is better) | Ceiling (max allowed) |
| Binary size | DOWN | Ceiling |
| Build time | DOWN | Ceiling |
| Test time | DOWN | Ceiling |

### Tolerance Application

- **Coverage:** `min_allowed = baseline - tolerance`
- **Escapes:** No tolerance (discrete counts)
- **Binary size:** `max_allowed = baseline + tolerance`
- **Timing:** `max_allowed = baseline + tolerance`

### Baseline Update Rules

1. `--fix` always updates baseline to current values
2. Improvements are reported with old → new format
3. No improvements = "baseline synced" message
4. Regressions are NOT prevented by --fix (user explicitly requested update)

### Error Handling

- Missing baseline: Pass silently (nothing to compare)
- Stale baseline: Warn if > `stale_days`
- Parse error: Report and skip ratcheting
- Version mismatch: Error (future version)

## Verification Plan

### Unit Tests
```bash
cargo test -- baseline      # Baseline I/O
cargo test -- ratchet       # Comparison logic
cargo test -- tolerance     # Parsing
```

### Behavioral Specs
```bash
cargo test --test specs -- ratchet    # All ratchet specs
cargo test --test specs -- modes      # Mode-related specs
```

### Integration Tests
```bash
# Create baseline
quench check --fix -C tests/fixtures/rust-simple

# Verify regression detection
echo "unsafe {}" >> tests/fixtures/rust-simple/src/lib.rs
quench check -C tests/fixtures/rust-simple  # Should fail

# Verify --fix updates
quench check --fix -C tests/fixtures/rust-simple  # Should update baseline
```

### Manual Verification
```bash
# Full workflow
cd /tmp && cargo new ratchet-test && cd ratchet-test
quench init
quench check --fix              # Create baseline
cat .quench/baseline.json       # Verify format
quench check                    # Should pass
# Add unsafe block, verify failure
quench check --fix              # Update baseline
```

## Phase Summary

| Phase | Description | Priority | Estimate |
|-------|-------------|----------|----------|
| 1 | Coverage ratcheting | High | Core feature |
| 2 | Behavioral specs | High | Prevents regression |
| 3 | Tolerance edge cases | Medium | Robustness |
| 4 | Per-package ratcheting | Medium | Workspace support |
| 5 | Output format alignment | Low | Polish |
| 6 | JSON schema | Low | Tooling support |

## Notes

- **CACHE_VERSION:** Bump in `cache.rs` if check output format changes
- **Spec tests:** Mark unimplemented with `#[ignore = "TODO: Phase N"]`
- **Backwards compat:** Baseline version field handles future migrations
