# Phase 1301: Report Command - Specs

**Root Feature:** `quench-1301`

## Overview

Write behavioral specifications for the `quench report` command. These specs test the CLI as a black box, verifying that the report command correctly reads baseline files and outputs metrics in text, JSON, and HTML formats.

This phase focuses on spec creation only. Implementation will follow in a separate phase.

## Project Structure

```
tests/specs/
├── cli/
│   └── report.rs       # NEW: Report command behavioral specs
├── prelude.rs          # Existing test helpers (extend with report builder)
└── main.rs             # Update to include report module

tests/fixtures/
├── report/
│   ├── with-baseline/  # NEW: Fixture with .quench/baseline.json
│   │   ├── quench.toml
│   │   ├── CLAUDE.md
│   │   └── .quench/
│   │       └── baseline.json
│   └── no-baseline/    # NEW: Fixture without baseline
│       ├── quench.toml
│       └── CLAUDE.md
```

Key files to modify:
- `tests/specs/cli/mod.rs` - Add `mod report;`
- `tests/specs/prelude.rs` - Add `report()` builder helper

## Dependencies

No new external dependencies required. Uses existing:
- `assert_cmd` for CLI testing
- `serde_json` for JSON validation
- `predicates` for output assertions

## Implementation Phases

### Phase 1: Create Test Fixtures

Create fixtures with known baseline data for deterministic testing.

**Fixture: `tests/fixtures/report/with-baseline/`**

```toml
# quench.toml
version = 1

[check.agents]
required = []
```

```markdown
# CLAUDE.md
# Project

## Directory Structure
Minimal.

## Landing the Plane
- Done
```

```json
// .quench/baseline.json
{
  "version": 1,
  "updated": "2026-01-20T12:00:00Z",
  "commit": "abc1234",
  "metrics": {
    "coverage": {
      "total": 85.5
    },
    "escapes": {
      "source": {
        "unsafe": 3,
        "unwrap": 0
      }
    }
  }
}
```

**Fixture: `tests/fixtures/report/no-baseline/`**
- Standard quench.toml and CLAUDE.md
- No .quench directory

**Milestone:** Fixtures created and checked into repo.

### Phase 2: Extend Test Prelude

Add a `report()` builder function to `tests/specs/prelude.rs` following the existing `check()` pattern.

```rust
// In tests/specs/prelude.rs

/// Create a report command builder
pub fn report() -> ReportBuilder<Text> {
    ReportBuilder::new()
}

/// Report command builder for fluent test assertions
pub struct ReportBuilder<Mode = Text> {
    dir: Option<std::path::PathBuf>,
    args: Vec<String>,
    _mode: PhantomData<Mode>,
}

impl ReportBuilder<Text> {
    fn new() -> Self {
        Self {
            dir: None,
            args: Vec::new(),
            _mode: PhantomData,
        }
    }

    pub fn json(self) -> ReportBuilder<Json> {
        ReportBuilder {
            dir: self.dir,
            args: self.args,
            _mode: PhantomData,
        }
    }

    pub fn html(self) -> ReportBuilder<Html> {
        ReportBuilder {
            dir: self.dir,
            args: self.args,
            _mode: PhantomData,
        }
    }

    pub fn runs(self) -> RunAssert {
        run_passes(self.command())
    }
}

impl<Mode: 'static> ReportBuilder<Mode> {
    /// Set fixture directory by name
    pub fn on(mut self, fixture_name: &str) -> Self {
        self.dir = Some(fixture(fixture_name));
        self
    }

    /// Add CLI arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    /// Build the command
    fn command(self) -> Command {
        let is_json = std::any::TypeId::of::<Mode>() == std::any::TypeId::of::<Json>();
        let is_html = std::any::TypeId::of::<Mode>() == std::any::TypeId::of::<Html>();

        let mut cmd = quench_cmd();
        cmd.arg("report");

        if is_json {
            cmd.args(["-o", "json"]);
        } else if is_html {
            cmd.args(["-o", "html"]);
        }

        cmd.args(&self.args);

        if let Some(dir) = self.dir {
            cmd.current_dir(dir);
        }

        cmd
    }
}

pub struct Html;
```

**Milestone:** `report()` builder compiles and can run basic commands.

### Phase 3: Write Baseline Reading Specs

Create `tests/specs/cli/report.rs` with specs for baseline file reading.

```rust
//! Behavioral specs for quench report command.
//!
//! Tests that quench report correctly:
//! - Reads baseline files
//! - Outputs metrics in various formats
//!
//! Reference: docs/specs/01-cli.md#quench-report

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// BASELINE READING
// =============================================================================

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > Reports read from .quench/baseline.json
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_reads_baseline_file() {
    report()
        .on("report/with-baseline")
        .runs()
        .stdout_has("coverage");
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > Report without baseline shows appropriate message
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_without_baseline_shows_message() {
    report()
        .on("report/no-baseline")
        .runs()
        .stdout_has("No baseline");
}
```

**Milestone:** Baseline reading specs compile with `#[ignore]`.

### Phase 4: Write Text Format Specs

Add specs for text output format.

```rust
// =============================================================================
// TEXT FORMAT
// =============================================================================

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > Default output format is text
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_default_format_is_text() {
    // Should not be JSON or HTML
    report()
        .on("report/with-baseline")
        .runs()
        .stdout_lacks("{")
        .stdout_lacks("<html");
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > Text format shows summary with metrics
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_text_shows_summary() {
    report()
        .on("report/with-baseline")
        .runs()
        .stdout_has("coverage: 85.5%")
        .stdout_has("escapes.unsafe: 3");
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > Text format shows baseline commit and timestamp
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_text_shows_baseline_info() {
    report()
        .on("report/with-baseline")
        .runs()
        .stdout_has("abc1234")
        .stdout_has("2026-01-20");
}
```

**Milestone:** Text format specs compile with `#[ignore]`.

### Phase 5: Write JSON Format Specs

Add specs for JSON output format.

```rust
// =============================================================================
// JSON FORMAT
// =============================================================================

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > JSON format outputs machine-readable metrics
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_json_outputs_metrics() {
    let output = report()
        .on("report/with-baseline")
        .json()
        .runs();

    let json: serde_json::Value = serde_json::from_str(&output.stdout()).unwrap();

    assert!(json.get("metrics").is_some(), "should have metrics field");
    assert!(
        json["metrics"]["coverage"]["total"].as_f64() == Some(85.5),
        "should have coverage metric"
    );
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > JSON format includes baseline metadata
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_json_includes_metadata() {
    let output = report()
        .on("report/with-baseline")
        .json()
        .runs();

    let json: serde_json::Value = serde_json::from_str(&output.stdout()).unwrap();

    assert!(json.get("updated").is_some(), "should have updated field");
    assert!(json.get("commit").is_some(), "should have commit field");
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > JSON format with no baseline outputs empty metrics
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_json_no_baseline_empty_metrics() {
    let output = report()
        .on("report/no-baseline")
        .json()
        .runs();

    let json: serde_json::Value = serde_json::from_str(&output.stdout()).unwrap();

    assert!(json.get("metrics").is_some(), "should have metrics field");
    // Metrics should be empty object or have null/empty values
}
```

**Milestone:** JSON format specs compile with `#[ignore]`.

### Phase 6: Write HTML Format and File Output Specs

Add specs for HTML output and file writing.

```rust
// =============================================================================
// HTML FORMAT
// =============================================================================

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > HTML format produces valid HTML document
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_html_produces_valid_html() {
    report()
        .on("report/with-baseline")
        .html()
        .runs()
        .stdout_has("<!DOCTYPE html>")
        .stdout_has("<html")
        .stdout_has("</html>");
}

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > HTML format includes metrics data
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_html_includes_metrics() {
    report()
        .on("report/with-baseline")
        .html()
        .runs()
        .stdout_has("85.5")  // coverage value
        .stdout_has("coverage");  // metric name
}

// =============================================================================
// FILE OUTPUT
// =============================================================================

/// Spec: docs/specs/01-cli.md#quench-report
///
/// > -o report.html writes to file instead of stdout
#[test]
#[ignore = "TODO: Phase 1302 - Report Implementation"]
fn report_writes_to_file() {
    let temp = Project::with_defaults();

    // Create baseline
    temp.file(".quench/baseline.json", r#"{
        "version": 1,
        "updated": "2026-01-20T12:00:00Z",
        "metrics": {"coverage": {"total": 75.0}}
    }"#);

    quench_cmd()
        .args(["report", "-o", "report.html"])
        .current_dir(temp.path())
        .assert()
        .success();

    let output_path = temp.path().join("report.html");
    assert!(output_path.exists(), "report.html should be created");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("<!DOCTYPE html>"), "should be HTML");
    assert!(content.contains("75.0"), "should include metrics");
}
```

**Milestone:** All report specs compile with `#[ignore]`.

## Key Implementation Details

### Output Format Detection

The `-o` flag has dual purpose per spec:
- `-o json` / `-o html` / `-o text` - format selection
- `-o report.html` / `-o report.json` - file output (format inferred from extension)

Detection logic for implementation:
```rust
fn parse_output_arg(arg: &str) -> (OutputFormat, Option<PathBuf>) {
    match arg {
        "text" => (OutputFormat::Text, None),
        "json" => (OutputFormat::Json, None),
        "html" => (OutputFormat::Html, None),
        path if path.ends_with(".html") => (OutputFormat::Html, Some(path.into())),
        path if path.ends_with(".json") => (OutputFormat::Json, Some(path.into())),
        path => (OutputFormat::Text, Some(path.into())),
    }
}
```

### Text Output Format

Based on the baseline structure, text output should show:
```
Quench Report
=============
Baseline: abc1234 (2026-01-20)

Coverage:     85.5%

Escapes:
  unsafe:     3
  unwrap:     0
```

### JSON Output Format

Mirror the baseline structure with top-level metadata:
```json
{
  "updated": "2026-01-20T12:00:00Z",
  "commit": "abc1234",
  "metrics": {
    "coverage": { "total": 85.5 },
    "escapes": { "source": { "unsafe": 3, "unwrap": 0 } }
  }
}
```

### HTML Output Format

Static dashboard with embedded metrics:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Quench Report</title>
    <style>/* inline CSS */</style>
</head>
<body>
    <h1>Quench Report</h1>
    <p>Baseline: abc1234 (2026-01-20)</p>
    <section class="metrics">
        <h2>Coverage</h2>
        <p>85.5%</p>
    </section>
    <!-- ... -->
</body>
</html>
```

## Verification Plan

### Spec Compilation

All specs should compile with `#[ignore]`:

```bash
# Should compile without errors
cargo test --test specs report -- --list

# Should show ignored tests
cargo test --test specs report -- --ignored --list
```

### Fixture Validation

Fixtures should be valid:

```bash
# Baseline JSON should be valid
cat tests/fixtures/report/with-baseline/.quench/baseline.json | jq .

# Config should be valid
quench check --config-only -C tests/fixtures/report/with-baseline/quench.toml
```

### Integration

After Phase 1302 implementation:

```bash
# All specs should pass
cargo test --test specs report

# Full check suite
make check
```

## Checklist

- [ ] Create `tests/fixtures/report/with-baseline/` directory
- [ ] Create `tests/fixtures/report/with-baseline/quench.toml`
- [ ] Create `tests/fixtures/report/with-baseline/CLAUDE.md`
- [ ] Create `tests/fixtures/report/with-baseline/.quench/baseline.json`
- [ ] Create `tests/fixtures/report/no-baseline/` directory
- [ ] Create `tests/fixtures/report/no-baseline/quench.toml`
- [ ] Create `tests/fixtures/report/no-baseline/CLAUDE.md`
- [ ] Extend `tests/specs/prelude.rs` with `report()` builder
- [ ] Add `Html` marker type to prelude
- [ ] Create `tests/specs/cli/report.rs` with baseline reading specs
- [ ] Add text format specs to report.rs
- [ ] Add JSON format specs to report.rs
- [ ] Add HTML format specs to report.rs
- [ ] Add file output specs to report.rs
- [ ] Update `tests/specs/cli/mod.rs` to include `report` module
- [ ] Run `cargo test --test specs -- --list` to verify compilation
- [ ] Run `make check` to verify no regressions
