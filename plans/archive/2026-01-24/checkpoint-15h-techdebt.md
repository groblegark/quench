# Checkpoint 15H: Tech Debt - Ratcheting

**Root Feature:** `quench-0765`

## Overview

This checkpoint addresses code quality issues and tech debt in the ratcheting system identified after the 15G bug fixes. The focus is on reducing code duplication, improving naming clarity, and adding missing utility methods.

**Tech debt items:**

1. **Duplicate `format_value` code** - Same method duplicated in `MetricComparison` and `MetricImprovement`
2. **Misleading `min_allowed` field name** - Field represents threshold/max allowed, not minimum
3. **Missing `coverage_tolerance()` accessor** - Config has field but no accessor method like other tolerances
4. **Silent fallback in test time extraction** - Uses `.unwrap_or(0.0)` without clarity about partial data
5. **Double baseline load in `--fix` flow** - Baseline loaded twice unnecessarily

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── ratchet.rs                 # REFACTOR: Extract format_value, rename field
│   ├── config/
│   │   └── ratchet.rs             # ADD: coverage_tolerance() accessor
│   └── main.rs                    # REFACTOR: Avoid double baseline load
└── plans/
    └── checkpoint-15h-techdebt.md # This plan
```

## Dependencies

No new dependencies. Uses existing infrastructure.

## Implementation Phases

### Phase 1: Extract Shared `format_value` Function

**Goal:** Eliminate code duplication between `MetricComparison::format_value` and `MetricImprovement::format_value`.

**Problem:** Both structs implement identical `format_value` methods (lines 166-175 and 211-220):

```rust
// In MetricComparison (line 166)
pub fn format_value(&self, value: f64) -> String {
    if self.name.starts_with("build_time.") || self.name.starts_with("test_time.") {
        format!("{:.1}s", value)
    } else if self.name.starts_with("coverage.") {
        format!("{:.1}%", value * 100.0)
    } else {
        format!("{}", value as i64)
    }
}

// In MetricImprovement (line 211) - identical logic
```

**Fix:** Extract a standalone function and have both structs delegate to it.

**File:** `crates/cli/src/ratchet.rs`

```rust
/// Format a metric value based on its type (determined by name prefix).
fn format_metric_value(name: &str, value: f64) -> String {
    if name.starts_with("build_time.") || name.starts_with("test_time.") {
        format!("{:.1}s", value)
    } else if name.starts_with("coverage.") {
        format!("{:.1}%", value * 100.0)
    } else {
        format!("{}", value as i64)
    }
}

impl MetricComparison {
    /// Format the value based on metric type.
    pub fn format_value(&self, value: f64) -> String {
        format_metric_value(&self.name, value)
    }
}

impl MetricImprovement {
    /// Format the value based on metric type.
    pub fn format_value(&self, value: f64) -> String {
        format_metric_value(&self.name, value)
    }
}
```

**Verification:**
```bash
cargo test ratchet
cargo clippy -- -D warnings
```

---

### Phase 2: Rename `min_allowed` to `threshold`

**Goal:** Clarify the semantics of the threshold field in `MetricComparison`.

**Problem:** The field `min_allowed` (line 160) is misleadingly named. For "lower is better" metrics (escapes, binary size, timing), it actually represents the *maximum* allowed value. For "higher is better" metrics (coverage), it would represent the minimum. The current name only makes sense for one direction.

**Fix:** Rename to `threshold` which is direction-agnostic, and update the doc comment to clarify semantics.

**File:** `crates/cli/src/ratchet.rs`

```rust
/// Comparison of a single metric.
#[derive(Debug, Clone)]
pub struct MetricComparison {
    pub name: String,
    pub current: f64,
    pub baseline: f64,
    pub tolerance: f64,
    /// The allowed threshold (baseline ± tolerance).
    /// For "lower is better" metrics: max allowed = baseline + tolerance.
    /// For "higher is better" metrics: min allowed = baseline - tolerance.
    pub threshold: f64,
    pub passed: bool,
    pub improved: bool,
}
```

Update all usages (search for `min_allowed`):
- `ratchet.rs:246`: `threshold: base_count as f64`
- `ratchet.rs:281`: `threshold: max_allowed as f64`
- `ratchet.rs:392`: `threshold: max_allowed`
- `output/text.rs`: Update format strings if they reference the field

**Verification:**
```bash
cargo build --all-targets
cargo test --all
```

---

### Phase 3: Add `coverage_tolerance()` Accessor

**Goal:** Add missing accessor method for coverage tolerance to match other tolerance accessors.

**Problem:** `RatchetConfig` has `coverage_tolerance: Option<f64>` but unlike `binary_size_tolerance` and `build_time_tolerance`, there's no accessor method. This breaks the pattern and makes the API inconsistent.

**File:** `crates/cli/src/config/ratchet.rs`

```rust
impl RatchetConfig {
    /// Get coverage tolerance in percentage points.
    pub fn coverage_tolerance_pct(&self) -> Option<f64> {
        self.coverage_tolerance
    }

    // existing methods...
}
```

Note: Coverage tolerance is already in percentage points (0.5 = 0.5% drop allowed), so no conversion needed. The accessor just provides API consistency.

**Verification:**
```bash
cargo build
cargo doc --no-deps  # Verify docs generate
```

---

### Phase 4: Document Test Time Extraction Behavior

**Goal:** Make the partial data handling in `extract_test_time` explicit rather than silent.

**Problem:** Lines 130-131 silently default `avg` and `max` to 0.0 if missing:

```rust
let avg = json.get("avg").and_then(|v| v.as_f64()).unwrap_or(0.0);
let max = json.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
```

This can lead to confusing comparisons (e.g., baseline has `avg: 1.5s`, current has `avg: 0.0s` due to missing data, which looks like a huge "improvement").

**Fix:** Add documentation explaining the behavior. A full fix would require schema changes to distinguish "0" from "missing", but for now, clarity via documentation is sufficient.

**File:** `crates/cli/src/ratchet.rs`

```rust
/// Extract test time metrics from JSON.
///
/// Note: `avg` and `max` default to 0.0 if not present in the metrics.
/// This allows ratcheting on just `total` without requiring all fields.
/// However, be aware that missing fields will appear as "improved to 0".
fn extract_test_time(json: &serde_json::Value) -> Option<TestTimeCurrent> {
    let total = json.get("total").and_then(|v| v.as_f64())?;
    // Default to 0.0 for optional timing fields
    let avg = json.get("avg").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let max = json.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
    // ...
}
```

**Verification:**
```bash
cargo doc --no-deps
```

---

### Phase 5: Optimize `--fix` Flow to Avoid Double Load

**Goal:** Eliminate redundant baseline load in the `--fix` code path.

**Problem:** In `main.rs`, when `--fix` is used:
1. Line 406: Baseline loaded for comparison
2. Line 448: Baseline loaded again for update

The baseline is immutable during a single run, so the second load is unnecessary.

**Fix:** Restructure to reuse the already-loaded baseline.

**File:** `crates/cli/src/main.rs`

```rust
// Ratchet checking
let baseline_path = root.join(&config.git.baseline);
let (ratchet_result, baseline) = if config.ratchet.check != CheckLevel::Off {
    match Baseline::load(&baseline_path) {
        Ok(Some(baseline)) => {
            // Warn if baseline is stale
            if baseline.is_stale(config.ratchet.stale_days) {
                eprintln!(
                    "warning: baseline is {} days old. Consider refreshing with --fix.",
                    baseline.age_days()
                );
            }

            let current = CurrentMetrics::from_output(&output);
            let result = ratchet::compare(&current, &baseline.metrics, &config.ratchet);
            (Some(result), Some(baseline))
        }
        Ok(None) => {
            // No baseline yet - pass but suggest creating one
            if args.verbose {
                eprintln!(
                    "No baseline found at {}. Run with --fix to create.",
                    baseline_path.display()
                );
            }
            (None, None)
        }
        Err(e) => {
            eprintln!("quench: warning: failed to load baseline: {}", e);
            (None, None)
        }
    }
} else {
    (None, None)
};

// Handle --fix: update/sync baseline
if args.fix {
    let current = CurrentMetrics::from_output(&output);

    // Use existing baseline or create new
    let mut baseline = baseline
        .map(|b| b.with_commit(&root))
        .unwrap_or_else(|| Baseline::new().with_commit(&root));

    let baseline_existed = baseline_path.exists();
    ratchet::update_baseline(&mut baseline, &current);

    // ... rest of save and message logic
}
```

**Note:** This is a refactor that changes control flow. Ensure all branches still behave correctly:
- Ratchet enabled, baseline exists: Compare, then update on --fix
- Ratchet enabled, no baseline: No compare, create on --fix
- Ratchet disabled: Skip all ratchet logic

**Verification:**
```bash
cargo test --test specs ratchet
cargo run -- check --fix  # Test on quench itself
```

---

### Phase 6: Final Verification

**Goal:** Ensure all changes work together and CI passes.

**Verification checklist:**

```bash
# All unit tests pass
cargo test --all

# All spec tests pass
cargo test --test specs

# Clippy clean
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check

# Full CI check
make check

# Dogfooding
cargo run -- check
```

## Key Implementation Details

### Value Formatting Strategy

The shared `format_metric_value` function determines format by metric name prefix:

| Metric prefix | Format | Example |
|---------------|--------|---------|
| `escapes.*` | Integer | `5` |
| `binary_size.*` | Integer | `1024` |
| `build_time.*` | 1 decimal + "s" | `12.5s` |
| `test_time.*` | 1 decimal + "s" | `3.2s` |
| `coverage.*` | 1 decimal + "%" | `85.3%` |

### Threshold Semantics

The renamed `threshold` field represents the boundary value for comparison:

| Metric Direction | Threshold Meaning | Pass Condition |
|------------------|-------------------|----------------|
| Lower is better | Max allowed | `current <= threshold` |
| Higher is better | Min allowed | `current >= threshold` |

### Baseline Caching

The `--fix` optimization caches the baseline from the comparison phase:
- If comparison loaded baseline → reuse for update
- If comparison found no baseline → create new
- If comparison failed to load → create new

This avoids redundant I/O without changing behavior.

## Verification Plan

### Phase 1: Format Value Extraction
```bash
cargo test ratchet::format
# Verify both MetricComparison and MetricImprovement still work
```

### Phase 2: Threshold Rename
```bash
cargo build --all-targets  # Compiler catches all usages
cargo test --test specs ratchet  # Behavior unchanged
```

### Phase 3: Coverage Tolerance
```bash
cargo doc --no-deps  # Method appears in docs
```

### Phase 4: Documentation
```bash
cargo doc --no-deps  # Doc comments render correctly
```

### Phase 5: Baseline Caching
```bash
# Test all --fix scenarios
cargo test --test specs fix_creates_baseline
cargo test --test specs fix_updates_baseline
cargo test --test specs fix_baseline_synced
```

### Phase 6: Full CI
```bash
make check
cargo run -- check
```

## Exit Criteria

- [ ] `format_metric_value` extracted as shared function
- [ ] Both `MetricComparison` and `MetricImprovement` delegate to shared function
- [ ] `min_allowed` renamed to `threshold` with clear documentation
- [ ] `coverage_tolerance_pct()` accessor added to `RatchetConfig`
- [ ] `extract_test_time` has documentation about default behavior
- [ ] `--fix` flow reuses baseline from comparison phase
- [ ] All tests pass: `make check`
- [ ] Dogfooding passes: `quench check` on quench
