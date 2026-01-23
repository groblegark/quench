# Phase 030: Output Infrastructure - Implementation

**Root Feature:** `quench-f301`

## Overview

Implement the output infrastructure for quench that produces agent-friendly output. This includes:
- Text formatter with the `<check>: FAIL` format and color support
- JSON formatter validating against `output.schema.json`
- TTY and agent environment detection for automatic color control
- Violation limiting (default 15) to protect agent context windows
- Exit code propagation per spec (0/1/2/3)
- `--config` flag for config validation mode

**Current State**: File walking and config loading implemented. CLI has basic structure with `OutputFormat` enum. Exit codes defined in `error.rs`. Specs from Phase 025 are all `#[ignore]`.

**End State**: All Phase 025 specs pass. Quench produces correctly formatted text and JSON output with proper colors, violation limits, and exit codes.

## Project Structure

```
crates/cli/src/
├── cli.rs              # MODIFY: Add --color, --limit, --no-limit, --config flags
├── error.rs            # EXISTS: ExitCode enum (no changes needed)
├── lib.rs              # MODIFY: Export new modules
├── main.rs             # MODIFY: Integrate formatters, handle exit codes
├── output/             # NEW: Output formatting module
│   ├── mod.rs          # Output types and traits
│   ├── text.rs         # Text formatter with colors
│   ├── text_tests.rs   # Unit tests
│   ├── json.rs         # JSON formatter
│   └── json_tests.rs   # Unit tests
├── color.rs            # NEW: Color detection and styling
├── color_tests.rs      # Unit tests
└── check.rs            # NEW: Check result types (Violation, CheckResult)

tests/
├── specs/
│   └── output.rs       # Phase 025 specs (remove #[ignore] as features land)
└── fixtures/
    ├── output-test/    # EXISTS: Single cloc violation for testing
    ├── config-error/   # EXISTS: Invalid TOML for exit code 2
    └── violations/     # EXISTS: Multiple violations for limit testing
```

## Dependencies

Add to `crates/cli/Cargo.toml`:

```toml
[dependencies]
termcolor = "1.4"       # Cross-platform terminal colors
chrono = "0.4"          # ISO 8601 timestamps for JSON
```

The `serde_json` dependency is already available via dev-dependencies but needed for runtime JSON output:

```toml
[dependencies]
serde_json = "1"        # JSON serialization
```

## Implementation Phases

### Phase 30.1: Core Types and Check Result Structure

**Goal**: Define the data structures for check results and violations that both formatters will use.

**Tasks**:
1. Create `crates/cli/src/check.rs` with `Violation` and `CheckResult` types
2. Add serde derives for JSON serialization
3. Export from `lib.rs`

**Files**:

```rust
// crates/cli/src/check.rs
//! Check result types for output formatting.

use serde::Serialize;
use std::path::PathBuf;

/// A single violation within a check.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    /// File path (None for non-file violations like commit messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,

    /// Line number (None if not applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Violation category (check-specific).
    #[serde(rename = "type")]
    pub violation_type: String,

    /// Actionable guidance on how to fix.
    pub advice: String,

    // Optional context fields (check-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// Result of running a single check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// Check identifier (e.g., "cloc", "escapes").
    pub name: String,

    /// Whether this check passed.
    pub passed: bool,

    /// True if check was skipped due to an error.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub skipped: bool,

    /// Error message if check was skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// List of violations (omitted if empty).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<Violation>,
}

impl CheckResult {
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            skipped: false,
            error: None,
            violations: Vec::new(),
        }
    }

    pub fn failed(name: impl Into<String>, violations: Vec<Violation>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            skipped: false,
            error: None,
            violations,
        }
    }

    pub fn skipped(name: impl Into<String>, error: String) -> Self {
        Self {
            name: name.into(),
            passed: false,
            skipped: true,
            error: Some(error),
            violations: Vec::new(),
        }
    }
}

/// Aggregated results from all checks.
#[derive(Debug, Clone, Serialize)]
pub struct CheckOutput {
    /// ISO 8601 timestamp.
    pub timestamp: String,

    /// Whether all checks passed.
    pub passed: bool,

    /// Results for each check.
    pub checks: Vec<CheckResult>,
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
```

**Verification**:
```bash
cargo build --lib
cargo test check::tests
```

### Phase 30.2: Color Detection and Styling

**Goal**: Implement TTY detection, agent environment detection, and color scheme per spec.

**Tasks**:
1. Create `crates/cli/src/color.rs` with `ColorChoice` detection
2. Implement agent detection (`CLAUDE_CODE`, `CODEX`, `CI`, `CURSOR`)
3. Define color scheme: bold check names, red FAIL, cyan paths, yellow line numbers
4. Add `--color` flag to CLI

**Files**:

```rust
// crates/cli/src/color.rs
//! Color detection and terminal styling.
//!
//! Detection logic per docs/specs/03-output.md#colorization:
//! 1. --color=always → use color
//! 2. --color=never → no color
//! 3. --color=auto (default):
//!    - If not stdout.is_tty() → no color
//!    - If CLAUDE_CODE, CODEX, CI, or CURSOR env var set → no color
//!    - Else → use color

use std::io::IsTerminal;
use termcolor::{ColorChoice, ColorSpec, WriteColor};

/// Color mode from CLI flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    /// Always use color.
    Always,
    /// Never use color.
    Never,
    /// Auto-detect based on TTY and environment.
    #[default]
    Auto,
}

impl ColorMode {
    /// Resolve to termcolor's ColorChoice.
    pub fn resolve(self) -> ColorChoice {
        match self {
            ColorMode::Always => ColorChoice::Always,
            ColorMode::Never => ColorChoice::Never,
            ColorMode::Auto => {
                if !std::io::stdout().is_terminal() {
                    return ColorChoice::Never;
                }
                if is_agent_environment() {
                    return ColorChoice::Never;
                }
                ColorChoice::Auto
            }
        }
    }
}

/// Check if running in an AI agent environment.
fn is_agent_environment() -> bool {
    std::env::var_os("CLAUDE_CODE").is_some()
        || std::env::var_os("CODEX").is_some()
        || std::env::var_os("CURSOR").is_some()
        || std::env::var_os("CI").is_some()
}

/// Color scheme for output per spec.
pub mod scheme {
    use termcolor::{Color, ColorSpec};

    /// Bold check name (e.g., "cloc").
    pub fn check_name() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_bold(true);
        spec
    }

    /// Red "FAIL" indicator.
    pub fn fail() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Red)).set_bold(true);
        spec
    }

    /// Green "FIXED" indicator.
    pub fn fixed() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Green)).set_bold(true);
        spec
    }

    /// Cyan file path.
    pub fn path() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Cyan));
        spec
    }

    /// Yellow line number.
    pub fn line_number() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Yellow));
        spec
    }

    /// Default (no color) for advice text.
    pub fn advice() -> ColorSpec {
        ColorSpec::new()
    }
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
```

Update `crates/cli/src/cli.rs` to add `--color` flag:

```rust
// Add to CheckArgs struct:
/// Color output mode
#[arg(long, default_value = "auto", value_name = "WHEN")]
pub color: ColorMode,

/// Disable color output (shorthand for --color=never)
#[arg(long, conflicts_with = "color")]
pub no_color: bool,
```

**Verification**:
```bash
cargo build
cargo test color::tests
# Manual: verify --color=never and CLAUDE_CODE=1 disable colors
```

### Phase 30.3: Text Formatter

**Goal**: Implement text output formatter with the spec format and color support.

**Tasks**:
1. Create `crates/cli/src/output/mod.rs` with `Formatter` trait
2. Create `crates/cli/src/output/text.rs` with streaming text output
3. Implement violation formatting with proper indentation
4. Add summary line (`N checks passed, M failed`)
5. Add violation limit message (`Stopped after N violations. Use --no-limit to see all.`)

**Files**:

```rust
// crates/cli/src/output/mod.rs
//! Output formatting for check results.

pub mod json;
pub mod text;

use crate::check::{CheckOutput, CheckResult};

/// Output formatting options.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Maximum violations to show (None = unlimited).
    pub limit: Option<usize>,
    /// Whether the output was truncated due to limit.
    pub truncated: bool,
    /// Total violation count (for truncation message).
    pub total_violations: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            limit: Some(15), // Default per spec
            truncated: false,
            total_violations: 0,
        }
    }
}
```

```rust
// crates/cli/src/output/text.rs
//! Text output formatter.
//!
//! Format per docs/specs/03-output.md#text-format:
//! ```text
//! <check-name>: FAIL
//!   <file>:<line>: <brief violation description>
//!     <advice>
//! ```

use std::io::Write;
use termcolor::{ColorChoice, StandardStream, WriteColor};

use crate::check::{CheckOutput, CheckResult, Violation};
use crate::color::scheme;
use super::FormatOptions;

pub struct TextFormatter {
    stdout: StandardStream,
    options: FormatOptions,
    violations_shown: usize,
}

impl TextFormatter {
    pub fn new(color_choice: ColorChoice, options: FormatOptions) -> Self {
        Self {
            stdout: StandardStream::stdout(color_choice),
            options,
            violations_shown: 0,
        }
    }

    /// Write a single check result (streaming).
    pub fn write_check(&mut self, result: &CheckResult) -> std::io::Result<bool> {
        if result.passed {
            return Ok(false); // Silent on pass per spec
        }

        // Check name: bold
        self.stdout.set_color(&scheme::check_name())?;
        write!(self.stdout, "{}", result.name)?;
        self.stdout.reset()?;

        // ": FAIL" in red
        write!(self.stdout, ": ")?;
        self.stdout.set_color(&scheme::fail())?;
        write!(self.stdout, "FAIL")?;
        self.stdout.reset()?;
        writeln!(self.stdout)?;

        // Violations
        for violation in &result.violations {
            if let Some(limit) = self.options.limit {
                if self.violations_shown >= limit {
                    return Ok(true); // Truncated
                }
            }
            self.write_violation(violation)?;
            self.violations_shown += 1;
        }

        writeln!(self.stdout)?;
        Ok(false)
    }

    fn write_violation(&mut self, v: &Violation) -> std::io::Result<()> {
        write!(self.stdout, "  ")?;

        // File path in cyan
        if let Some(ref file) = v.file {
            self.stdout.set_color(&scheme::path())?;
            write!(self.stdout, "{}", file.display())?;
            self.stdout.reset()?;

            // Line number in yellow
            if let Some(line) = v.line {
                write!(self.stdout, ":")?;
                self.stdout.set_color(&scheme::line_number())?;
                write!(self.stdout, "{}", line)?;
                self.stdout.reset()?;
            }
            write!(self.stdout, ": ")?;
        }

        // Violation description (includes type-specific info)
        writeln!(self.stdout, "{}", self.format_violation_desc(v))?;

        // Advice (4-space indent)
        writeln!(self.stdout, "    {}", v.advice)?;

        Ok(())
    }

    fn format_violation_desc(&self, v: &Violation) -> String {
        // Format based on violation type and available fields
        match (v.value, v.threshold) {
            (Some(val), Some(thresh)) => {
                format!("{} ({} vs {})", v.violation_type, val, thresh)
            }
            _ => v.violation_type.clone(),
        }
    }

    /// Write the summary line.
    pub fn write_summary(&mut self, output: &CheckOutput) -> std::io::Result<()> {
        let passed = output.checks.iter().filter(|c| c.passed).count();
        let failed = output.checks.len() - passed;

        if failed == 0 {
            writeln!(self.stdout, "{} checks passed", passed)?;
        } else {
            writeln!(
                self.stdout,
                "{} check{} passed, {} failed",
                passed,
                if passed == 1 { "" } else { "s" },
                failed
            )?;
        }
        Ok(())
    }

    /// Write truncation message if applicable.
    pub fn write_truncation_message(&mut self, total: usize) -> std::io::Result<()> {
        if let Some(limit) = self.options.limit {
            if self.violations_shown >= limit && total > limit {
                writeln!(
                    self.stdout,
                    "Stopped after {} violations. Use --no-limit to see all.",
                    limit
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
```

**Verification**:
```bash
cargo test output::text::tests
# Remove #[ignore] from: text_output_format_check_name_fail, text_output_format_file_line,
#   text_output_format_advice_indented, text_output_summary_line, text_output_passing_summary_only
cargo test --test specs text_output
```

### Phase 30.4: JSON Formatter

**Goal**: Implement JSON output formatter that validates against `output.schema.json`.

**Tasks**:
1. Create `crates/cli/src/output/json.rs` with buffered JSON output
2. Add ISO 8601 timestamp using `chrono`
3. Ensure all fields match schema requirements
4. Buffer all output, write complete JSON at end

**Files**:

```rust
// crates/cli/src/output/json.rs
//! JSON output formatter.
//!
//! Produces output conforming to docs/specs/output.schema.json.
//! JSON is buffered and written at the end (not streamed).

use std::io::Write;

use chrono::Utc;

use crate::check::CheckOutput;

pub struct JsonFormatter<W: Write> {
    writer: W,
}

impl<W: Write> JsonFormatter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write the complete JSON output.
    pub fn write(&mut self, output: &CheckOutput) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(output)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(self.writer, "{}", json)
    }
}

/// Create CheckOutput with current timestamp.
pub fn create_output(checks: Vec<crate::check::CheckResult>) -> CheckOutput {
    let passed = checks.iter().all(|c| c.passed);
    CheckOutput {
        timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        passed,
        checks,
    }
}

#[cfg(test)]
#[path = "json_tests.rs"]
mod tests;
```

**Verification**:
```bash
cargo test output::json::tests
# Remove #[ignore] from: json_output_validates_against_schema, json_output_has_required_fields,
#   json_output_timestamp_iso8601, json_output_check_has_required_fields,
#   json_output_violation_has_required_fields
cargo test --test specs json_output
```

### Phase 30.5: CLI Integration and Exit Codes

**Goal**: Wire up formatters to main.rs, add remaining CLI flags, implement exit code propagation.

**Tasks**:
1. Add `--limit N`, `--no-limit` flags to CLI
2. Add `--config` flag for validate-only mode
3. Update `main.rs` to use formatters
4. Implement exit code priority (3 > 2 > 1 > 0)
5. Handle config errors with exit code 2

**Files**:

Update `crates/cli/src/cli.rs`:

```rust
// Add to CheckArgs:
/// Maximum violations to display (default: 15)
#[arg(long, default_value_t = 15, value_name = "N")]
pub limit: usize,

/// Show all violations (no limit)
#[arg(long, conflicts_with = "limit")]
pub no_limit: bool,

/// Validate config and exit without running checks
#[arg(long = "config-only")]
pub config_only: bool,
```

Update `crates/cli/src/main.rs` (key sections):

```rust
fn run_check(cli: &Cli, args: &CheckArgs) -> anyhow::Result<ExitCode> {
    // ... config loading ...

    // Config-only mode: validate and exit
    if args.config_only {
        // Config already loaded successfully
        return Ok(ExitCode::Success);
    }

    // Resolve color mode
    let color_choice = if args.no_color {
        ColorChoice::Never
    } else {
        args.color.resolve()
    };

    // Set up formatter options
    let limit = if args.no_limit { None } else { Some(args.limit) };
    let options = FormatOptions { limit, ..Default::default() };

    // ... run checks, collect results ...

    // Format output
    match args.output {
        OutputFormat::Text => {
            let mut formatter = TextFormatter::new(color_choice, options);
            for result in &check_results {
                formatter.write_check(result)?;
            }
            formatter.write_summary(&output)?;
            if options.truncated {
                formatter.write_truncation_message(total_violations)?;
            }
        }
        OutputFormat::Json => {
            let output = json::create_output(check_results);
            let mut formatter = JsonFormatter::new(std::io::stdout());
            formatter.write(&output)?;
        }
    }

    // Determine exit code
    let exit_code = if has_internal_error {
        ExitCode::InternalError
    } else if has_config_error {
        ExitCode::ConfigError
    } else if !output.passed {
        ExitCode::CheckFailed
    } else {
        ExitCode::Success
    };

    Ok(exit_code)
}
```

**Verification**:
```bash
cargo build
# Remove #[ignore] from all exit code specs
cargo test --test specs exit_code
# Remove #[ignore] from color specs
cargo test --test specs color
# Remove #[ignore] from violation limit specs
cargo test --test specs violation_limit
# Remove #[ignore] from config flag specs
cargo test --test specs config_flag
# Remove #[ignore] from debug output spec
cargo test --test specs quench_log
```

### Phase 30.6: Final Integration and Polish

**Goal**: Ensure all specs pass, clean up, and verify full compliance.

**Tasks**:
1. Run all Phase 025 specs (remove all `#[ignore]` attributes)
2. Fix any spec failures
3. Run `make check` for full validation
4. Update any snapshots if needed

**Verification**:
```bash
# All specs should pass
cargo test --test specs output

# Full check
make check

# Verify JSON output against schema manually
cargo run -- check -o json tests/fixtures/output-test | jq .
```

## Key Implementation Details

### Color Detection Logic

Per `docs/specs/03-output.md#colorization`:

```
if --color=always:
    use color
elif --color=never OR --no-color:
    no color
else (--color=auto, default):
    if not stdout.is_tty():
        no color
    elif env.CLAUDE_CODE or env.CODEX or env.CURSOR or env.CI:
        no color
    else:
        use color
```

### Exit Code Priority

When multiple error types occur during a run:

```
3 (internal error) > 2 (config error) > 1 (check failed) > 0 (passed)
```

Implementation: Track the highest severity error encountered and return that code.

### Violation Limiting

- Default: 15 violations
- `--limit N`: Show first N violations
- `--no-limit`: Show all violations
- When limit reached: Print truncation message and stop processing violations

```
Stopped after 15 violations. Use --no-limit to see all.
```

### JSON vs Text Output

| Aspect | Text | JSON |
|--------|------|------|
| Output mode | Streaming (as checks complete) | Buffered (complete at end) |
| Colors | Supported via termcolor | N/A |
| Violations | Formatted with indentation | Structured array |
| Summary | Human-readable line | `passed` boolean field |

### Text Format Structure

```
<check-name>: FAIL
  <file>:<line>: <violation description>
    <advice>

<check-name>: FAIL
  <file>: <violation> (<value> vs <threshold>)
    <advice>

N checks passed, M failed
```

## Verification Plan

### Spec Coverage

| Spec Category | Count | Phase |
|---------------|-------|-------|
| Text output format | 5 | 30.3 |
| JSON output format | 5 | 30.4 |
| Exit codes | 4 | 30.5 |
| Colorization | 4 | 30.2, 30.5 |
| Violation limits | 4 | 30.5 |
| Config validation | 2 | 30.5 |
| Debug output | 1 | 30.5 |
| **Total** | **25** | |

### Phase Completion Checklist

- [ ] **30.1**: `check.rs` compiles, unit tests pass
- [ ] **30.2**: `color.rs` compiles, `--color` flag works
- [ ] **30.3**: Text specs pass: `text_output_format_*`, `text_output_summary_*`, `text_output_passing_*`
- [ ] **30.4**: JSON specs pass: `json_output_*`
- [ ] **30.5**: Exit code specs pass, color specs pass, limit specs pass, config specs pass
- [ ] **30.6**: All 25 specs pass, `make check` passes

### Running Verification

```bash
# After each phase:
cargo build
cargo test

# After Phase 30.3:
cargo test --test specs text_output

# After Phase 30.4:
cargo test --test specs json_output

# After Phase 30.5:
cargo test --test specs exit_code
cargo test --test specs color
cargo test --test specs violation_limit
cargo test --test specs config_flag

# Final verification:
cargo test --test specs output
make check
```

## Summary

Phase 030 implements the complete output infrastructure:

1. **Core types** (`check.rs`): `Violation`, `CheckResult`, `CheckOutput`
2. **Color system** (`color.rs`): TTY detection, agent detection, color scheme
3. **Text formatter** (`output/text.rs`): Streaming output with colors
4. **JSON formatter** (`output/json.rs`): Buffered schema-compliant output
5. **CLI integration**: `--color`, `--limit`, `--no-limit`, `--config-only` flags
6. **Exit codes**: Proper propagation with priority handling

All 25 Phase 025 specs will pass upon completion.
