# Phase 715: Tests Check - Placeholders

**Root Feature:** `quench-88fc`

## Overview

Implement a "placeholders" check that detects test placeholder patterns and reports them as items needing implementation. This helps teams track incomplete test coverage and ensures placeholder tests don't accumulate indefinitely.

Currently, placeholder tests (like `#[ignore]` in Rust or `test.todo()` in JavaScript) satisfy test correlation requirements. This new check does the inverse: it **flags** these placeholders as violations that need attention.

**Patterns to detect:**
- **Rust**: `#[ignore]` attribute on tests, `todo!()` macro in test bodies
- **JavaScript/TypeScript**: `test.todo()`, `it.todo()`, `test.fixme()`, `it.fixme()`

## Project Structure

```
crates/cli/src/checks/
├── placeholders/
│   ├── mod.rs              # PlaceholdersCheck implementation
│   ├── mod_tests.rs        # Unit tests
│   ├── rust.rs             # Rust placeholder detection
│   └── javascript.rs       # JavaScript placeholder detection

crates/cli/src/config/
└── checks.rs               # Add PlaceholdersConfig

tests/specs/checks/
└── placeholders.rs         # Behavioral specs

tests/fixtures/placeholders/
├── rust-ignore/            # Rust #[ignore] tests
├── rust-todo/              # Rust todo!() bodies
├── javascript-todo/        # JS test.todo() patterns
├── javascript-fixme/       # JS test.fixme() patterns
└── allowed/                # placeholders = "allow" config
```

## Dependencies

No new external dependencies. Uses existing:
- `regex` - Pattern matching
- `serde_json` - Metrics output
- `globset` - File pattern matching

## Implementation Phases

### Phase 1: Configuration Structure

**Goal**: Define the configuration schema for the placeholders check.

**Files to modify**:
- `crates/cli/src/config/checks.rs`
- `crates/cli/src/config/mod.rs`

**Add to checks.rs**:

```rust
/// Placeholders check configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlaceholdersConfig {
    /// Check level: error, warn, or off.
    #[serde(default)]
    pub check: CheckLevel,

    /// Patterns to detect (default: Rust #[ignore], todo!(); JS test.todo(), test.fixme())
    #[serde(default)]
    pub patterns: PlaceholderPatterns,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlaceholderPatterns {
    /// Rust patterns to detect.
    #[serde(default = "PlaceholderPatterns::default_rust")]
    pub rust: Vec<String>,

    /// JavaScript/TypeScript patterns to detect.
    #[serde(default = "PlaceholderPatterns::default_javascript")]
    pub javascript: Vec<String>,
}

impl Default for PlaceholderPatterns {
    fn default() -> Self {
        Self {
            rust: Self::default_rust(),
            javascript: Self::default_javascript(),
        }
    }
}

impl PlaceholderPatterns {
    fn default_rust() -> Vec<String> {
        vec!["ignore".to_string(), "todo".to_string()]
    }

    fn default_javascript() -> Vec<String> {
        vec!["todo".to_string(), "fixme".to_string()]
    }
}

impl Default for PlaceholdersConfig {
    fn default() -> Self {
        Self {
            check: CheckLevel::Off,  // Disabled by default
            patterns: PlaceholderPatterns::default(),
        }
    }
}
```

**Modify CheckConfig**:

```rust
pub struct CheckConfig {
    // ... existing fields ...
    pub placeholders: PlaceholdersConfig,
}
```

**Config example (quench.toml)**:

```toml
[check.placeholders]
check = "warn"              # error | warn | off (default: off)

# Customize which patterns to detect
[check.placeholders.patterns]
rust = ["ignore", "todo"]           # #[ignore], todo!()
javascript = ["todo", "fixme"]      # test.todo(), test.fixme()
```

**Verification**:
- Unit test: config deserializes correctly
- Unit test: defaults match expected patterns

---

### Phase 2: Rust Placeholder Detection

**Goal**: Detect `#[ignore]` tests and `todo!()` bodies in Rust test files.

**Files to create**:
- `crates/cli/src/checks/placeholders/rust.rs`

**Implementation**:

```rust
use std::path::Path;

/// Placeholder test detected in Rust code.
#[derive(Debug)]
pub struct RustPlaceholder {
    pub line: u32,
    pub test_name: String,
    pub kind: RustPlaceholderKind,
}

#[derive(Debug, Clone, Copy)]
pub enum RustPlaceholderKind {
    Ignore,  // #[ignore] or #[ignore = "..."]
    Todo,    // todo!() in test body
}

impl RustPlaceholderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ignore => "ignore",
            Self::Todo => "todo",
        }
    }
}

/// Find placeholder tests in Rust content.
pub fn find_rust_placeholders(content: &str, patterns: &[String]) -> Vec<RustPlaceholder> {
    let detect_ignore = patterns.iter().any(|p| p == "ignore");
    let detect_todo = patterns.iter().any(|p| p == "todo");

    let mut results = Vec::new();
    let mut state = ParseState::default();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Track #[test] attribute
        if trimmed == "#[test]" {
            state.saw_test_attr = true;
            state.test_line = line_num;
            continue;
        }

        // Track #[ignore] attribute
        if state.saw_test_attr
            && (trimmed.starts_with("#[ignore]") || trimmed.starts_with("#[ignore ="))
            && detect_ignore
        {
            state.saw_ignore_attr = true;
            state.ignore_line = line_num;
            continue;
        }

        // Match function name
        if state.saw_test_attr && trimmed.starts_with("fn ") {
            if let Some(name) = extract_fn_name(trimmed) {
                // Report #[ignore] placeholder
                if state.saw_ignore_attr && detect_ignore {
                    results.push(RustPlaceholder {
                        line: state.ignore_line,
                        test_name: name.to_string(),
                        kind: RustPlaceholderKind::Ignore,
                    });
                }

                // Scan function body for todo!()
                if detect_todo {
                    state.current_test_name = Some(name.to_string());
                    state.in_test_body = true;
                    state.brace_depth = 0;
                }
            }
            state.saw_test_attr = false;
            state.saw_ignore_attr = false;
            continue;
        }

        // Track brace depth in test body
        if state.in_test_body {
            for ch in line.chars() {
                match ch {
                    '{' => state.brace_depth += 1,
                    '}' => {
                        state.brace_depth -= 1;
                        if state.brace_depth <= 0 {
                            state.in_test_body = false;
                            state.current_test_name = None;
                        }
                    }
                    _ => {}
                }
            }

            // Check for todo!() in body
            if trimmed.contains("todo!()") || trimmed.contains("todo!(") {
                if let Some(ref name) = state.current_test_name {
                    results.push(RustPlaceholder {
                        line: line_num,
                        test_name: name.clone(),
                        kind: RustPlaceholderKind::Todo,
                    });
                }
            }
        }

        // Reset state on non-attribute lines
        if !trimmed.starts_with('#') && !trimmed.is_empty() && !state.in_test_body {
            state.saw_test_attr = false;
            state.saw_ignore_attr = false;
        }
    }

    results
}

#[derive(Default)]
struct ParseState {
    saw_test_attr: bool,
    saw_ignore_attr: bool,
    test_line: u32,
    ignore_line: u32,
    in_test_body: bool,
    brace_depth: i32,
    current_test_name: Option<String>,
}

fn extract_fn_name(line: &str) -> Option<&str> {
    line.strip_prefix("fn ")?
        .split(|c: char| c == '(' || c.is_whitespace())
        .next()
}
```

**Verification**:
- Unit test: `find_rust_placeholders_detects_ignore()`
- Unit test: `find_rust_placeholders_detects_todo()`
- Unit test: `find_rust_placeholders_extracts_test_name()`

---

### Phase 3: JavaScript Placeholder Detection

**Goal**: Detect `test.todo()`, `it.todo()`, `test.fixme()`, `it.fixme()` in JavaScript/TypeScript.

**Files to create**:
- `crates/cli/src/checks/placeholders/javascript.rs`

**Implementation**:

```rust
use regex::Regex;
use std::sync::LazyLock;

/// Placeholder test detected in JavaScript/TypeScript.
#[derive(Debug)]
pub struct JsPlaceholder {
    pub line: u32,
    pub description: String,
    pub kind: JsPlaceholderKind,
}

#[derive(Debug, Clone, Copy)]
pub enum JsPlaceholderKind {
    Todo,   // test.todo(), it.todo()
    Fixme,  // test.fixme(), it.fixme()
    Skip,   // test.skip(), it.skip() (optional)
}

impl JsPlaceholderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Fixme => "fixme",
            Self::Skip => "skip",
        }
    }
}

/// Regex patterns for JavaScript placeholder tests.
static TODO_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:test|it|describe)\.todo\s*\(\s*['"`]([^'"`]+)['"`]"#).unwrap()
});

static FIXME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:test|it|describe)\.fixme\s*\(\s*['"`]([^'"`]+)['"`]"#).unwrap()
});

static SKIP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:test|it|describe)\.skip\s*\(\s*['"`]([^'"`]+)['"`]"#).unwrap()
});

/// Find placeholder tests in JavaScript/TypeScript content.
pub fn find_js_placeholders(content: &str, patterns: &[String]) -> Vec<JsPlaceholder> {
    let detect_todo = patterns.iter().any(|p| p == "todo");
    let detect_fixme = patterns.iter().any(|p| p == "fixme");
    let detect_skip = patterns.iter().any(|p| p == "skip");

    let mut results = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = (line_idx + 1) as u32;

        if detect_todo {
            for cap in TODO_PATTERN.captures_iter(line) {
                if let Some(desc) = cap.get(1) {
                    results.push(JsPlaceholder {
                        line: line_num,
                        description: desc.as_str().to_string(),
                        kind: JsPlaceholderKind::Todo,
                    });
                }
            }
        }

        if detect_fixme {
            for cap in FIXME_PATTERN.captures_iter(line) {
                if let Some(desc) = cap.get(1) {
                    results.push(JsPlaceholder {
                        line: line_num,
                        description: desc.as_str().to_string(),
                        kind: JsPlaceholderKind::Fixme,
                    });
                }
            }
        }

        if detect_skip {
            for cap in SKIP_PATTERN.captures_iter(line) {
                if let Some(desc) = cap.get(1) {
                    results.push(JsPlaceholder {
                        line: line_num,
                        description: desc.as_str().to_string(),
                        kind: JsPlaceholderKind::Skip,
                    });
                }
            }
        }
    }

    results
}
```

**Verification**:
- Unit test: `find_js_placeholders_detects_test_todo()`
- Unit test: `find_js_placeholders_detects_it_todo()`
- Unit test: `find_js_placeholders_detects_fixme()`
- Unit test: `find_js_placeholders_extracts_description()`

---

### Phase 4: Check Implementation

**Goal**: Implement the `PlaceholdersCheck` struct and register it.

**Files to create**:
- `crates/cli/src/checks/placeholders/mod.rs`
- `crates/cli/src/checks/placeholders/mod_tests.rs`

**Files to modify**:
- `crates/cli/src/checks/mod.rs`

**Implementation (mod.rs)**:

```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Placeholders check: detects placeholder tests that need implementation.

mod javascript;
mod rust;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

use std::path::Path;
use std::sync::atomic::Ordering;

use serde_json::json;

use crate::adapter::{FileKind, GenericAdapter};
use crate::check::{Check, CheckContext, CheckResult, Violation};
use crate::config::CheckLevel;

pub struct PlaceholdersCheck;

impl Check for PlaceholdersCheck {
    fn name(&self) -> &'static str {
        "placeholders"
    }

    fn description(&self) -> &'static str {
        "Placeholder test detection"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let config = &ctx.config.check.placeholders;

        if config.check == CheckLevel::Off {
            return CheckResult::passed(self.name());
        }

        // Build file adapter for test file detection
        let test_patterns = if ctx.config.project.tests.is_empty() {
            default_test_patterns()
        } else {
            ctx.config.project.tests.clone()
        };
        let file_adapter = GenericAdapter::new(&[], &test_patterns);

        let mut violations = Vec::new();
        let mut metrics = Metrics::default();

        for file in ctx.files {
            // Only check test files
            let rel_path = file.path.strip_prefix(ctx.root).unwrap_or(&file.path);
            if file_adapter.classify(rel_path) != FileKind::Test {
                continue;
            }

            // Read file content
            let content = match std::fs::read_to_string(&file.path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Detect based on file extension
            let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");

            match ext {
                "rs" => {
                    let placeholders = rust::find_rust_placeholders(
                        &content,
                        &config.patterns.rust,
                    );

                    for p in placeholders {
                        metrics.increment_rust(p.kind);

                        let advice = format!(
                            "Implement test `{}` or remove placeholder.",
                            p.test_name
                        );

                        if let Some(v) = try_create_violation(
                            ctx,
                            rel_path,
                            p.line,
                            p.kind.as_str(),
                            &advice,
                        ) {
                            violations.push(v);
                        } else {
                            break; // Limit reached
                        }
                    }
                }
                "js" | "jsx" | "ts" | "tsx" | "mjs" | "mts" => {
                    let placeholders = javascript::find_js_placeholders(
                        &content,
                        &config.patterns.javascript,
                    );

                    for p in placeholders {
                        metrics.increment_js(p.kind);

                        let advice = format!(
                            "Implement test \"{}\" or remove placeholder.",
                            p.description
                        );

                        if let Some(v) = try_create_violation(
                            ctx,
                            rel_path,
                            p.line,
                            p.kind.as_str(),
                            &advice,
                        ) {
                            violations.push(v);
                        } else {
                            break; // Limit reached
                        }
                    }
                }
                _ => {}
            }

            // Check violation limit
            if ctx.limit.is_some_and(|l| violations.len() >= l) {
                break;
            }
        }

        let result = if violations.is_empty() {
            CheckResult::passed(self.name())
        } else if config.check == CheckLevel::Warn {
            CheckResult::passed_with_warnings(self.name(), violations)
        } else {
            CheckResult::failed(self.name(), violations)
        };

        result.with_metrics(metrics.to_json())
    }

    fn default_enabled(&self) -> bool {
        false  // Disabled by default
    }
}

fn try_create_violation(
    ctx: &CheckContext,
    path: &Path,
    line: u32,
    violation_type: &str,
    advice: &str,
) -> Option<Violation> {
    let current = ctx.violation_count.fetch_add(1, Ordering::SeqCst);
    if ctx.limit.is_some_and(|l| current >= l) {
        return None;
    }

    Some(Violation::file(path, line, violation_type, advice))
}

fn default_test_patterns() -> Vec<String> {
    vec![
        "**/tests/**".to_string(),
        "**/test/**".to_string(),
        "**/*_test.*".to_string(),
        "**/*_tests.*".to_string(),
        "**/*.test.*".to_string(),
        "**/*.spec.*".to_string(),
    ]
}

#[derive(Default)]
struct Metrics {
    rust_ignore: usize,
    rust_todo: usize,
    js_todo: usize,
    js_fixme: usize,
}

impl Metrics {
    fn increment_rust(&mut self, kind: rust::RustPlaceholderKind) {
        match kind {
            rust::RustPlaceholderKind::Ignore => self.rust_ignore += 1,
            rust::RustPlaceholderKind::Todo => self.rust_todo += 1,
        }
    }

    fn increment_js(&mut self, kind: javascript::JsPlaceholderKind) {
        match kind {
            javascript::JsPlaceholderKind::Todo => self.js_todo += 1,
            javascript::JsPlaceholderKind::Fixme => self.js_fixme += 1,
            javascript::JsPlaceholderKind::Skip => {} // Not counted separately
        }
    }

    fn to_json(&self) -> serde_json::Value {
        json!({
            "rust": {
                "ignore": self.rust_ignore,
                "todo": self.rust_todo,
            },
            "javascript": {
                "todo": self.js_todo,
                "fixme": self.js_fixme,
            }
        })
    }
}
```

**Register in checks/mod.rs**:

```rust
pub mod placeholders;

pub const CHECK_NAMES: &[&str] = &[
    "cloc", "escapes", "agents", "docs", "tests", "git", "build", "license", "placeholders",
];

pub fn all_checks() -> Vec<Arc<dyn Check>> {
    vec![
        // ... existing checks ...
        Arc::new(placeholders::PlaceholdersCheck),
    ]
}
```

**Verification**:
- Unit test: `check_name_is_placeholders()`
- Unit test: `check_disabled_by_default()`
- Unit test: `check_skips_source_files()`

---

### Phase 5: Behavioral Specs and Fixtures

**Goal**: Add comprehensive behavioral tests.

**Files to create**:
- `tests/specs/checks/placeholders.rs`
- `tests/fixtures/placeholders/` directories

**Behavioral specs**:

```rust
// tests/specs/checks/placeholders.rs

use crate::prelude::*;

/// Spec: Rust #[ignore] detection
#[test]
fn placeholders_detects_rust_ignore() {
    check("placeholders")
        .on("placeholders/rust-ignore")
        .fails()
        .stdout_has("ignore")
        .stdout_has("test_parser");
}

/// Spec: Rust todo!() detection
#[test]
fn placeholders_detects_rust_todo_body() {
    check("placeholders")
        .on("placeholders/rust-todo")
        .fails()
        .stdout_has("todo")
        .stdout_has("test_lexer");
}

/// Spec: JavaScript test.todo() detection
#[test]
fn placeholders_detects_js_test_todo() {
    check("placeholders")
        .on("placeholders/javascript-todo")
        .fails()
        .stdout_has("todo")
        .stdout_has("should handle edge case");
}

/// Spec: JavaScript test.fixme() detection
#[test]
fn placeholders_detects_js_test_fixme() {
    check("placeholders")
        .on("placeholders/javascript-fixme")
        .fails()
        .stdout_has("fixme")
        .stdout_has("broken on empty input");
}

/// Spec: placeholders = "allow" disables check
#[test]
fn placeholders_allow_config_passes() {
    check("placeholders")
        .on("placeholders/allowed")
        .passes();
}

/// Spec: warn mode reports but passes
#[test]
fn placeholders_warn_mode_passes_with_warnings() {
    let temp = temp_project()
        .config(r#"
[check.placeholders]
check = "warn"
"#)
        .file("tests/parser_test.rs", r#"
#[test]
#[ignore = "TODO"]
fn test_parser() { todo!() }
"#);

    check("placeholders")
        .pwd(temp.path())
        .passes()
        .stdout_has("WARN");
}
```

**Fixture: rust-ignore**:

```toml
# tests/fixtures/placeholders/rust-ignore/quench.toml
[check.placeholders]
check = "error"
```

```rust
// tests/fixtures/placeholders/rust-ignore/tests/parser_tests.rs
#[test]
#[ignore = "TODO: implement parser"]
fn test_parser() {
    // Not implemented yet
}
```

**Fixture: javascript-todo**:

```toml
# tests/fixtures/placeholders/javascript-todo/quench.toml
[check.placeholders]
check = "error"
```

```javascript
// tests/fixtures/placeholders/javascript-todo/tests/parser.test.js
test.todo('should handle edge case');
it.todo('validates input');
```

**Verification**:
- `cargo test --test specs -- placeholders`
- All behavioral specs pass

---

### Phase 6: Integration and Polish

**Goal**: Final integration, documentation, and cleanup.

**Tasks**:
1. Update `CACHE_VERSION` in `crates/cli/src/cache.rs`
2. Add check to documentation
3. Ensure `make check` passes
4. Add JSON output example to specs

**JSON output example**:

```json
{
  "name": "placeholders",
  "passed": false,
  "violations": [
    {
      "file": "tests/parser_tests.rs",
      "line": 2,
      "type": "ignore",
      "advice": "Implement test `test_parser` or remove placeholder."
    },
    {
      "file": "tests/math.test.js",
      "line": 5,
      "type": "todo",
      "advice": "Implement test \"should handle edge case\" or remove placeholder."
    }
  ],
  "metrics": {
    "rust": { "ignore": 1, "todo": 0 },
    "javascript": { "todo": 1, "fixme": 0 }
  }
}
```

**Verification**:
- `make check` passes
- `cargo test --all` passes
- JSON output matches schema

---

## Key Implementation Details

### Detection Patterns

**Rust `#[ignore]`**:
```rust
#[test]
#[ignore = "TODO: implement"]
fn test_feature() { ... }
```

Detection: Look for `#[test]` followed by `#[ignore]` or `#[ignore = "..."]`.

**Rust `todo!()`**:
```rust
#[test]
fn test_feature() {
    todo!()
}
```

Detection: Parse function body, track brace depth, detect `todo!()` or `todo!("...")`.

**JavaScript `test.todo()`**:
```javascript
test.todo('description');
it.todo('description');
describe.todo('description');
```

Detection: Regex `(?:test|it|describe)\.todo\s*\(\s*['"`]([^'"`]+)['"`]`.

**JavaScript `test.fixme()`**:

Note: `test.fixme()` is a Playwright pattern, not standard Jest/Vitest. We detect it for Playwright users.

```javascript
test.fixme('broken test');
```

### Configuration Interaction

The `placeholders` check and `tests.commit.placeholders` config serve different purposes:

| Config | Purpose |
|--------|---------|
| `[check.placeholders]` | New check: flags placeholders as violations |
| `[check.tests.commit].placeholders` | Existing: allows placeholders to satisfy correlation |

Both can be enabled simultaneously for different use cases.

### Default State

The check is **disabled by default** (`check = "off"`) because:
1. Placeholders are often valid during development
2. Teams should opt-in to tracking placeholder cleanup
3. Avoids noise in projects that use placeholders intentionally

Enable explicitly:
```toml
[check.placeholders]
check = "warn"  # or "error"
```

---

## Verification Plan

### Unit Tests

```bash
cargo test --package quench -- checks::placeholders
```

Tests:
- `find_rust_placeholders_detects_ignore()`
- `find_rust_placeholders_detects_todo()`
- `find_rust_placeholders_multiple_in_file()`
- `find_js_placeholders_detects_todo()`
- `find_js_placeholders_detects_fixme()`
- `find_js_placeholders_different_quote_styles()`
- `check_disabled_by_default()`
- `check_only_scans_test_files()`

### Behavioral Specs

```bash
cargo test --test specs -- placeholders
```

Specs:
- `placeholders_detects_rust_ignore()`
- `placeholders_detects_rust_todo_body()`
- `placeholders_detects_js_test_todo()`
- `placeholders_detects_js_test_fixme()`
- `placeholders_allow_config_passes()`
- `placeholders_warn_mode_passes_with_warnings()`
- `placeholders_json_output_format()`

### Integration

```bash
make check
```

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --all`
- `cargo audit`
- `cargo deny check`
