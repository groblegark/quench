# Phase 1020: Build Check - Output

## Overview

Complete the build check output formatting. The build check implementation already measures binary sizes and build times, stores them in `BuildMetrics`, and outputs them as JSON. This phase focuses on:

- Text output formatting with human-readable size/time values
- Per-target breakdown in both text and JSON output
- Exact output specification tests to ensure consistent formatting
- Verification that JSON metrics structure matches the spec

The existing implementation in `crates/cli/src/checks/build/mod.rs` produces metrics; this phase ensures the output is well-formatted and tested.

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── checks/build/
│   │   ├── mod.rs            # Already implemented
│   │   └── mod_tests.rs      # Add unit tests for JSON structure
│   ├── file_size.rs          # Extend: add format_size() for human-readable output
│   ├── file_size_tests.rs    # Extend: add formatting tests
│   └── output/
│       ├── text.rs           # Extend: build-specific violation formatting
│       └── text_tests.rs     # Extend: text output tests
└── tests/specs/checks/
    └── build.rs              # Add exact output tests
```

## Dependencies

**Existing:**
- `serde_json` - Already used for metrics output
- `termcolor` - Already used for text formatting

**No new external dependencies required.**

## Implementation Phases

### Phase 1: Human-Readable Size Formatting

**Goal:** Add a `format_size()` function to display binary sizes in human-readable form.

**Files:**
- `crates/cli/src/file_size.rs` - Add formatting function
- `crates/cli/src/file_size_tests.rs` - Add formatting tests

**Implementation:**

```rust
// crates/cli/src/file_size.rs

/// Format a byte count as a human-readable string.
///
/// Examples:
/// - 1024 → "1.0 KB"
/// - 1048576 → "1.0 MB"
/// - 1073741824 → "1.0 GB"
/// - 512 → "512 B"
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f = bytes as f64;

    if bytes_f >= GB {
        format!("{:.1} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.1} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.1} KB", bytes_f / KB)
    } else {
        format!("{} B", bytes)
    }
}
```

**Verification:**
```bash
cargo test --lib file_size
```

### Phase 2: Text Output for Size Violations

**Goal:** Enhance text formatter to show human-readable size comparisons for build violations.

**Files:**
- `crates/cli/src/output/text.rs` - Add build-specific formatting

**Expected text output for size violations:**
```
build: FAIL
  oversized: 5.1 MB (max: 5.0 MB)
    Reduce binary size. Check for unnecessary dependencies.
```

**Implementation:**

```rust
// crates/cli/src/output/text.rs - In format_violation_desc()

fn format_violation_desc(&self, v: &Violation) -> String {
    match v.violation_type.as_str() {
        // ... existing cases ...

        // Build check - size violations with human-readable formatting
        "size_exceeded" => {
            let target = v.target.as_deref().unwrap_or("binary");
            match (v.value, v.threshold) {
                (Some(val), Some(thresh)) => {
                    format!(
                        "{}: {} (max: {})",
                        target,
                        crate::file_size::format_size(val as u64),
                        crate::file_size::format_size(thresh as u64)
                    )
                }
                _ => format!("{}: size exceeded", target),
            }
        }

        // Build check - time violations
        "time_cold_exceeded" | "time_hot_exceeded" => {
            let kind = if v.violation_type == "time_cold_exceeded" {
                "cold build"
            } else {
                "hot build"
            };
            match (v.value, v.threshold) {
                (Some(val), Some(thresh)) => {
                    format!(
                        "{}: {:.1}s (max: {:.1}s)",
                        kind,
                        val as f64 / 1000.0,  // millis to seconds
                        thresh as f64 / 1000.0
                    )
                }
                _ => format!("{} time exceeded", kind),
            }
        }

        // Build check - missing target
        "missing_target" => {
            let target = v.target.as_deref().unwrap_or("unknown");
            format!("target not found: {}", target)
        }

        // ... rest of existing cases ...
    }
}
```

**Verification:**
```bash
cargo test --lib output
```

### Phase 3: JSON Output Structure Verification

**Goal:** Add unit tests to verify the JSON metrics structure matches the spec.

**Files:**
- `crates/cli/src/checks/build/mod_tests.rs` - Add JSON structure tests

**Expected JSON structure:**
```json
{
  "metrics": {
    "size": {
      "myapp": 5242880,
      "myserver": 2097152
    },
    "time": {
      "cold": 45.234,
      "hot": 2.456
    }
  }
}
```

**Implementation:**

```rust
// crates/cli/src/checks/build/mod_tests.rs

use super::*;
use serde_json::json;

#[test]
fn build_metrics_json_structure() {
    let mut metrics = BuildMetrics::default();
    metrics.sizes.insert("myapp".to_string(), 5_242_880);
    metrics.sizes.insert("myserver".to_string(), 2_097_152);
    metrics.time_cold = Some(Duration::from_secs_f64(45.234));
    metrics.time_hot = Some(Duration::from_secs_f64(2.456));

    let json = metrics.to_json();

    // Verify structure
    assert!(json.get("size").is_some());
    assert!(json.get("time").is_some());

    // Verify size values
    let size = json.get("size").unwrap();
    assert_eq!(size.get("myapp").and_then(|v| v.as_u64()), Some(5_242_880));
    assert_eq!(size.get("myserver").and_then(|v| v.as_u64()), Some(2_097_152));

    // Verify time values (as floats)
    let time = json.get("time").unwrap();
    let cold = time.get("cold").and_then(|v| v.as_f64()).unwrap();
    assert!((cold - 45.234).abs() < 0.001);
    let hot = time.get("hot").and_then(|v| v.as_f64()).unwrap();
    assert!((hot - 2.456).abs() < 0.001);
}

#[test]
fn build_metrics_json_empty_time() {
    let mut metrics = BuildMetrics::default();
    metrics.sizes.insert("myapp".to_string(), 1024);

    let json = metrics.to_json();

    let time = json.get("time").unwrap();
    assert!(time.get("cold").unwrap().is_null());
    assert!(time.get("hot").unwrap().is_null());
}
```

**Verification:**
```bash
cargo test --lib checks::build
```

### Phase 4: Exact Output Spec Tests

**Goal:** Add behavioral specs that verify exact text and JSON output format.

**Files:**
- `tests/specs/checks/build.rs` - Add exact output tests

**Implementation:**

```rust
// tests/specs/checks/build.rs

/// Spec: docs/specs/checks/build.md#text-output
///
/// > Size violations show human-readable sizes
#[test]
fn build_size_exceeded_text_output() {
    let temp = Project::empty();
    temp.config(
        r#"
[check.build]
check = "error"
size_max = "100 bytes"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "texttest"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/main.rs", "fn main() { println!(\"Hello\"); }");

    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(temp.path())
        .output()
        .expect("build should succeed");

    // Verify text output contains human-readable size
    check("build")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("texttest:")
        .stdout_has("(max: 100 B)");
}

/// Spec: docs/specs/checks/build.md#json-output
///
/// > Per-target breakdown in metrics.size
#[test]
fn build_json_per_target_breakdown() {
    let temp = Project::empty();
    temp.config(
        r#"
[check.build]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "multibin"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "app1"
path = "src/bin/app1.rs"

[[bin]]
name = "app2"
path = "src/bin/app2.rs"
"#,
    );
    temp.file("src/bin/app1.rs", "fn main() {}");
    temp.file("src/bin/app2.rs", "fn main() { println!(\"larger\"); }");

    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(temp.path())
        .output()
        .expect("build should succeed");

    let result = check("build")
        .pwd(temp.path())
        .args(&["--ci"])
        .json()
        .passes();

    let metrics = result.require("metrics");
    let size = metrics.get("size").and_then(|v| v.as_object()).unwrap();

    // Both targets should be present
    assert!(size.contains_key("app1"), "should have app1 in per-target breakdown");
    assert!(size.contains_key("app2"), "should have app2 in per-target breakdown");

    // Both should have non-zero sizes
    assert!(size.get("app1").and_then(|v| v.as_u64()).unwrap() > 0);
    assert!(size.get("app2").and_then(|v| v.as_u64()).unwrap() > 0);
}

/// Spec: docs/specs/checks/build.md#json-output
///
/// > Time metrics structure with cold and hot
#[test]
fn build_json_time_structure() {
    let temp = Project::empty();
    temp.config(
        r#"
[check.build]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "timestructure"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/main.rs", "fn main() {}");

    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(temp.path())
        .output()
        .expect("build should succeed");

    let result = check("build")
        .pwd(temp.path())
        .args(&["--ci"])
        .json()
        .passes();

    let metrics = result.require("metrics");

    // Verify time object exists with cold and hot keys
    let time = metrics.get("time").unwrap();
    assert!(time.get("cold").is_some(), "time.cold should exist");
    assert!(time.get("hot").is_some(), "time.hot should exist");
}

/// Spec: docs/specs/checks/build.md#text-output
///
/// > Time violations show seconds with one decimal
#[test]
fn build_time_violation_text_format() {
    let temp = Project::empty();
    temp.config(
        r#"
[check.build]
check = "error"
time_cold_max = "1ms"

[ratchet]
build_time_cold = true
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "timeformat"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/main.rs", "fn main() {}");

    check("build")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("cold build:")
        .stdout_has("(max:");
}
```

**Verification:**
```bash
cargo test --test specs -- build_size_exceeded_text
cargo test --test specs -- build_json_per_target
cargo test --test specs -- build_json_time
cargo test --test specs -- build_time_violation
```

### Phase 5: Missing Target Text Output

**Goal:** Ensure missing_target violations have proper text formatting.

**Files:**
- `crates/cli/src/output/text.rs` - Verify missing_target formatting

**Expected output:**
```
build: FAIL
  target not found: myapp
    Configured build target not found. Verify target exists and builds successfully.
```

**Implementation:**

The `missing_target` case is added in Phase 2. This phase verifies it with a spec test:

```rust
// tests/specs/checks/build.rs

/// Spec: docs/specs/checks/build.md#text-output
///
/// > Missing target shows target name
#[test]
fn build_missing_target_text_output() {
    let temp = Project::empty();
    temp.config(
        r#"
[check.build]
check = "error"
targets = ["nonexistent"]
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "missingtext"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/lib.rs", "pub fn foo() {}");

    check("build")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("target not found: nonexistent");
}
```

**Verification:**
```bash
cargo test --test specs -- build_missing_target_text
```

## Key Implementation Details

### Size Format Units

| Bytes | Output |
|-------|--------|
| 512 | "512 B" |
| 1,024 | "1.0 KB" |
| 1,536 | "1.5 KB" |
| 1,048,576 | "1.0 MB" |
| 5,242,880 | "5.0 MB" |
| 1,073,741,824 | "1.0 GB" |

Single decimal place for KB/MB/GB, no decimal for bytes.

### Time Format

Time values stored as `Duration` are formatted as seconds with one decimal place:
- 45.234s → "45.2s"
- 2.456s → "2.5s"
- 0.001s → "0.0s"

### JSON Metrics Structure

```json
{
  "metrics": {
    "size": {
      "<target-name>": <bytes-as-integer>,
      ...
    },
    "time": {
      "cold": <seconds-as-float-or-null>,
      "hot": <seconds-as-float-or-null>
    }
  }
}
```

- `size` is an object mapping target names to byte counts
- `time.cold` and `time.hot` are floats (seconds) or null if not measured

### Text Output Format

**Size violation:**
```
build: FAIL
  <target>: <actual-size> (max: <threshold>)
    Reduce binary size. Check for unnecessary dependencies.
```

**Time violation:**
```
build: FAIL
  cold build: <actual>s (max: <threshold>s)
    Cold build time exceeded threshold. Consider optimizing dependencies or build configuration.
```

**Missing target:**
```
build: FAIL
  target not found: <target>
    Configured build target not found. Verify target exists and builds successfully.
```

## Verification Plan

### Unit Tests

```bash
# Size formatting
cargo test --lib file_size

# Text output formatting
cargo test --lib output

# Build metrics JSON structure
cargo test --lib checks::build
```

### Behavioral Specs

```bash
# All build output specs
cargo test --test specs -- build

# Specific output tests
cargo test --test specs -- build_size_exceeded_text
cargo test --test specs -- build_json_per_target
cargo test --test specs -- build_time_violation
cargo test --test specs -- build_missing_target
```

### Integration Test

```bash
# Test on quench itself
quench check --build --ci

# Verify JSON output
quench check --build --ci -o json | jq '.checks[] | select(.name == "build") | .metrics'

# Verify text output format
quench check --build --ci -o text
```

### Checklist

- [ ] `format_size()` function added to `file_size.rs`
- [ ] Size formatting unit tests pass
- [ ] Text formatter handles `size_exceeded` with human-readable sizes
- [ ] Text formatter handles `time_cold_exceeded` and `time_hot_exceeded`
- [ ] Text formatter handles `missing_target`
- [ ] JSON metrics structure unit tests pass
- [ ] Exact output spec tests added
- [ ] All build specs pass
- [ ] `make check` passes

### Exit Criteria

All the following conditions are met:

1. `quench check --build --ci` on a Rust project outputs human-readable sizes
2. JSON output contains per-target size breakdown: `metrics.size.<target-name>`
3. JSON output contains time structure: `metrics.time.{cold, hot}`
4. Text output format matches spec for all violation types
5. All existing and new build specs pass
