# Phase 705: Tests Check - Change Detection

**Plan:** `phase-705`
**Root Feature:** `quench-tests`
**Reference:** `docs/specs/checks/tests.md`
**Depends On:** Phase 701 (behavioral specs)

## Overview

Implement the core change detection infrastructure for the tests check. This phase adds git diff parsing, source file classification (added/modified/deleted), lines changed counting, and test file pattern matching. When complete, the tests check will detect when source file changes lack corresponding test changes.

The behavioral specs from Phase 701 (`tests/specs/checks/tests/correlation.rs`) define the acceptance criteria:
- `staged_flag_checks_only_staged_files()`
- `base_flag_compares_against_git_ref()`
- `source_change_without_test_change_generates_violation()`
- `test_change_without_source_change_passes_tdd()`
- `inline_cfg_test_change_satisfies_test_requirement()`
- `placeholder_test_satisfies_test_requirement()`
- `excluded_files_dont_require_tests()`
- `json_includes_source_files_changed_metrics()`
- `tests_violation_type_is_always_missing_tests()`

## Project Structure

Files to create/modify:

```
crates/cli/src/
├── checks/
│   ├── mod.rs                  # MODIFY: Replace stub with real TestsCheck
│   └── tests/                  # NEW: Tests check module
│       ├── mod.rs              # NEW: Check implementation
│       ├── diff.rs             # NEW: Git diff parsing
│       ├── correlation.rs      # NEW: Source/test correlation logic
│       ├── mod_tests.rs        # NEW: Unit tests for check
│       ├── diff_tests.rs       # NEW: Unit tests for diff
│       └── correlation_tests.rs # NEW: Unit tests for correlation
└── config/
    └── mod.rs                  # MODIFY: Extend TestsCommitConfig
```

## Dependencies

- No new external crates required
- Uses `std::process::Command` for git operations (existing pattern)
- Uses `globset` for pattern matching (already in Cargo.toml)
- Leverages existing `CheckContext.changed_files` and `CheckContext.base_branch`

## Implementation Phases

### Phase 1: Git Diff Parsing Module

Create the diff parsing infrastructure that handles `--staged` and `--base` flags.

**File:** `crates/cli/src/checks/tests/diff.rs`

```rust
//! Git diff parsing for change detection.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Type of change detected in git diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// A file change detected from git diff.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

impl FileChange {
    /// Total lines changed (added + deleted).
    pub fn lines_changed(&self) -> usize {
        self.lines_added + self.lines_deleted
    }
}

/// Get changed files from staged area (--staged flag).
pub fn get_staged_changes(root: &Path) -> Result<Vec<FileChange>, String> {
    // git diff --cached --numstat
    // git diff --cached --name-status
    todo!()
}

/// Get changed files comparing to base ref (--base flag).
pub fn get_base_changes(root: &Path, base: &str) -> Result<Vec<FileChange>, String> {
    // git diff --numstat {base}..HEAD
    // git diff --name-status {base}..HEAD
    todo!()
}

/// Parse git diff --numstat output into line counts.
fn parse_numstat(output: &str) -> Vec<(PathBuf, usize, usize)> {
    todo!()
}

/// Parse git diff --name-status output into change types.
fn parse_name_status(output: &str) -> Vec<(PathBuf, ChangeType)> {
    todo!()
}
```

**Key Implementation Details:**

1. Use `git diff --numstat` for line counts (format: `added\tdeleted\tpath`)
2. Use `git diff --name-status` for change types (format: `A/M/D\tpath`)
3. Combine both outputs to build `FileChange` structs
4. Handle binary files (numstat shows `-` for counts)
5. Handle renamed files (`R100\told\tnew` in name-status)

**Git Commands:**
```bash
# Staged changes
git diff --cached --numstat
git diff --cached --name-status

# Base comparison
git diff --numstat {base}..HEAD
git diff --name-status {base}..HEAD
```

**Verification:**
- [ ] Unit tests for `parse_numstat()`
- [ ] Unit tests for `parse_name_status()`
- [ ] Unit tests for renamed file handling
- [ ] Unit tests for binary file handling

### Phase 2: Source/Test Correlation Module

Create the logic that matches source files to their corresponding test files.

**File:** `crates/cli/src/checks/tests/correlation.rs`

```rust
//! Source/test file correlation logic.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};

use super::diff::FileChange;

/// Configuration for correlation detection.
pub struct CorrelationConfig {
    /// Patterns that identify test files.
    pub test_patterns: Vec<String>,
    /// Patterns that identify source files.
    pub source_patterns: Vec<String>,
    /// Files excluded from requiring tests.
    pub exclude_patterns: Vec<String>,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            test_patterns: vec![
                "tests/**/*".to_string(),
                "test/**/*".to_string(),
                "**/*_test.*".to_string(),
                "**/*_tests.*".to_string(),
                "**/*.spec.*".to_string(),
            ],
            source_patterns: vec!["src/**/*".to_string()],
            exclude_patterns: vec![
                "**/mod.rs".to_string(),
                "**/lib.rs".to_string(),
                "**/main.rs".to_string(),
                "**/generated/**".to_string(),
            ],
        }
    }
}

/// Result of correlation analysis.
#[derive(Debug)]
pub struct CorrelationResult {
    /// Source files that have corresponding test changes.
    pub with_tests: Vec<PathBuf>,
    /// Source files missing test changes.
    pub without_tests: Vec<PathBuf>,
    /// Test-only changes (TDD workflow).
    pub test_only: Vec<PathBuf>,
}

/// Analyze changes for source/test correlation.
pub fn analyze_correlation(
    changes: &[FileChange],
    config: &CorrelationConfig,
) -> CorrelationResult {
    todo!()
}

/// Check if a source file has a corresponding test file in the changes.
fn has_corresponding_test(
    source: &Path,
    test_changes: &[&PathBuf],
    source_patterns: &GlobSet,
) -> bool {
    // For src/parser.rs, look for:
    // - tests/parser.rs, tests/parser_test.rs, tests/parser_tests.rs
    // - src/parser_test.rs, src/parser_tests.rs
    // - test/parser.rs
    todo!()
}

/// Extract the base name for correlation (e.g., "parser" from "src/parser.rs").
fn correlation_base_name(path: &Path) -> Option<&str> {
    path.file_stem()?.to_str()
}

/// Build a GlobSet from pattern strings.
fn build_glob_set(patterns: &[String]) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|e| e.to_string())?;
        builder.add(glob);
    }
    builder.build().map_err(|e| e.to_string())
}
```

**Key Implementation Details:**

1. Classify changes as source/test using glob patterns
2. Match source files to potential test files by base name
3. Handle multiple naming conventions (parser.rs -> parser_test.rs, parser_tests.rs)
4. Respect excluded files (mod.rs, lib.rs, main.rs)
5. Track TDD workflow (test changes without source changes = OK)

**Matching Algorithm:**
```
For source file: src/parser.rs
Base name: "parser"
Look for test changes matching ANY of:
  - tests/parser.rs
  - tests/parser_test.rs
  - tests/parser_tests.rs
  - src/parser_test.rs
  - src/parser_tests.rs
  - test/parser.rs
  - **/parser.spec.*
  - **/test_parser.*
```

**Verification:**
- [ ] Unit tests for `has_corresponding_test()`
- [ ] Unit tests for excluded file filtering
- [ ] Unit tests for TDD detection (tests-only changes)

### Phase 3: Inline Test Detection

Add support for detecting `#[cfg(test)]` blocks in Rust source files.

**Add to:** `crates/cli/src/checks/tests/correlation.rs`

```rust
/// Check if a file has inline test changes (Rust #[cfg(test)] blocks).
pub fn has_inline_test_changes(
    file_path: &Path,
    root: &Path,
) -> Result<bool, String> {
    // Use git diff to get the actual changed lines
    // Parse the diff to see if changes touch #[cfg(test)] blocks
    todo!()
}

/// Parse diff hunks to detect if changes are within #[cfg(test)] blocks.
fn changes_in_cfg_test(diff_content: &str) -> bool {
    // Track state: are we inside a #[cfg(test)] block?
    // Look for #[cfg(test)] followed by mod { ... }
    // Check if any + lines are within that block
    todo!()
}
```

**Key Implementation Details:**

1. Run `git diff {base}..HEAD -- {file}` to get per-file diff
2. Parse diff hunks to identify changed lines
3. Track `#[cfg(test)]` block boundaries using brace counting
4. Changes within `#[cfg(test)]` blocks satisfy the test requirement

**Example Diff Analysis:**
```diff
+pub fn parse() -> bool {
+    true
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_parse() {    // <- This satisfies test requirement
+        assert!(parse());
+    }
+}
```

**Verification:**
- [ ] Unit tests for `#[cfg(test)]` detection
- [ ] Unit tests for nested module detection
- [ ] Unit tests for changes outside cfg(test) block

### Phase 4: Configuration Extension

Extend the tests check configuration to support all spec-defined options.

**Modify:** `crates/cli/src/config/mod.rs`

```rust
/// Tests commit check configuration.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestsCommitConfig {
    /// Check level: "error" | "warn" | "off"
    #[serde(default = "TestsCommitConfig::default_check")]
    pub check: String,

    /// Scope: "branch" | "commit"
    #[serde(default = "TestsCommitConfig::default_scope")]
    pub scope: String,

    /// Placeholder handling: "allow" | "forbid"
    #[serde(default = "TestsCommitConfig::default_placeholders")]
    pub placeholders: String,

    /// Test file patterns (extends defaults).
    #[serde(default = "TestsCommitConfig::default_test_patterns")]
    pub test_patterns: Vec<String>,

    /// Source file patterns.
    #[serde(default = "TestsCommitConfig::default_source_patterns")]
    pub source_patterns: Vec<String>,

    /// Excluded patterns (never require tests).
    #[serde(default = "TestsCommitConfig::default_exclude")]
    pub exclude: Vec<String>,
}

impl Default for TestsCommitConfig {
    fn default() -> Self {
        Self {
            check: Self::default_check(),
            scope: Self::default_scope(),
            placeholders: Self::default_placeholders(),
            test_patterns: Self::default_test_patterns(),
            source_patterns: Self::default_source_patterns(),
            exclude: Self::default_exclude(),
        }
    }
}

impl TestsCommitConfig {
    fn default_check() -> String { "off".to_string() }
    fn default_scope() -> String { "branch".to_string() }
    fn default_placeholders() -> String { "allow".to_string() }

    fn default_test_patterns() -> Vec<String> {
        vec![
            "tests/**/*".to_string(),
            "test/**/*".to_string(),
            "**/*_test.*".to_string(),
            "**/*_tests.*".to_string(),
            "**/*.spec.*".to_string(),
        ]
    }

    fn default_source_patterns() -> Vec<String> {
        vec!["src/**/*".to_string()]
    }

    fn default_exclude() -> Vec<String> {
        vec![
            "**/mod.rs".to_string(),
            "**/lib.rs".to_string(),
            "**/main.rs".to_string(),
            "**/generated/**".to_string(),
        ]
    }
}
```

**Verification:**
- [ ] Config parsing tests for all new fields
- [ ] Default value tests

### Phase 5: Tests Check Implementation

Create the main tests check that ties everything together.

**File:** `crates/cli/src/checks/tests/mod.rs`

```rust
//! Tests check implementation.
//!
//! Reference: docs/specs/checks/tests.md

mod correlation;
mod diff;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

use std::sync::Arc;

use serde_json::json;

use crate::check::{Check, CheckContext, CheckResult, Violation};
use crate::config::TestsCommitConfig;

use self::correlation::{analyze_correlation, CorrelationConfig, CorrelationResult};
use self::diff::{get_base_changes, get_staged_changes, ChangeType};

pub struct TestsCheck;

impl TestsCheck {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Check for TestsCheck {
    fn name(&self) -> &'static str {
        "tests"
    }

    fn description(&self) -> &'static str {
        "Test correlation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let config = &ctx.config.check.tests.commit;

        // Skip if disabled
        if config.check == "off" {
            return CheckResult::passed(self.name());
        }

        // Need either --staged or --base for change detection
        let changes = if ctx.staged {
            match get_staged_changes(ctx.root) {
                Ok(c) => c,
                Err(e) => return CheckResult::skipped(self.name(), e),
            }
        } else if let Some(base) = ctx.base_branch {
            match get_base_changes(ctx.root, base) {
                Ok(c) => c,
                Err(e) => return CheckResult::skipped(self.name(), e),
            }
        } else {
            // No change context available
            return CheckResult::passed(self.name());
        };

        // Analyze correlation
        let correlation_config = build_correlation_config(config);
        let result = analyze_correlation(&changes, &correlation_config);

        // Build violations for source files without tests
        let mut violations = Vec::new();
        for path in &result.without_tests {
            let change = changes.iter().find(|c| &c.path == path);
            let advice = format!(
                "Add tests in tests/{}_tests.rs or update inline #[cfg(test)] block",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );

            let mut v = Violation::file_only(path, "missing_tests", advice);
            if let Some(c) = change {
                v = v.with_change_type(match c.change_type {
                    ChangeType::Added => "added",
                    ChangeType::Modified => "modified",
                    ChangeType::Deleted => "deleted",
                });
                v.lines = Some(c.lines_changed() as i64);
            }
            violations.push(v);

            if ctx.limit.is_some_and(|l| violations.len() >= l) {
                break;
            }
        }

        // Build metrics
        let metrics = json!({
            "source_files_changed": result.with_tests.len() + result.without_tests.len(),
            "with_test_changes": result.with_tests.len(),
            "without_test_changes": result.without_tests.len(),
            "scope": config.scope,
        });

        if violations.is_empty() {
            CheckResult::passed(self.name()).with_metrics(metrics)
        } else if config.check == "warn" {
            CheckResult::passed_with_warnings(self.name(), violations).with_metrics(metrics)
        } else {
            CheckResult::failed(self.name(), violations).with_metrics(metrics)
        }
    }

    fn default_enabled(&self) -> bool {
        true
    }
}

fn build_correlation_config(config: &TestsCommitConfig) -> CorrelationConfig {
    CorrelationConfig {
        test_patterns: config.test_patterns.clone(),
        source_patterns: config.source_patterns.clone(),
        exclude_patterns: config.exclude.clone(),
    }
}
```

**Modify:** `crates/cli/src/checks/mod.rs`

Replace the stub registration:
```rust
// Before:
Arc::new(stub::StubCheck::new("tests", "Test correlation", true))

// After:
tests::TestsCheck::new()
```

**Verification:**
- [ ] Behavioral specs pass (remove `#[ignore]` from Phase 701 specs)
- [ ] Integration with existing runner works
- [ ] JSON output includes all required fields

### Phase 6: Placeholder Test Detection

Add support for recognizing placeholder tests (`#[ignore]` in Rust).

**Add to:** `crates/cli/src/checks/tests/correlation.rs`

```rust
/// Check if a test file contains placeholder tests for a given source file.
pub fn has_placeholder_test(
    test_path: &Path,
    source_base: &str,
    root: &Path,
) -> Result<bool, String> {
    // Read test file content
    // Look for #[test] #[ignore = "..."] patterns
    // Check if the test name relates to the source file
    todo!()
}

/// Parse Rust test file for placeholder tests.
fn find_placeholder_tests(content: &str) -> Vec<String> {
    // Find patterns like:
    // #[test]
    // #[ignore = "TODO: implement parser"]
    // fn test_parser() { ... }
    todo!()
}
```

**Key Implementation Details:**

1. Parse test files looking for `#[test]` followed by `#[ignore = "..."]`
2. Match placeholder test names to source file base names
3. Only count placeholders when `placeholders = "allow"` in config

**Rust Placeholder Pattern:**
```rust
#[test]
#[ignore = "TODO: implement parser"]
fn test_parser() { todo!() }
```

**Verification:**
- [ ] Unit tests for placeholder detection
- [ ] Behavioral spec: `placeholder_test_satisfies_test_requirement()`

## Key Implementation Details

### Git Diff Parsing Strategy

Use two git commands and merge results:

```bash
# Get line counts
git diff --numstat {base}..HEAD
# Output: 10\t5\tsrc/parser.rs (10 added, 5 deleted)

# Get change types
git diff --name-status {base}..HEAD
# Output: M\tsrc/parser.rs (Modified)
```

Merge by path to create `FileChange` structs with both line counts and change types.

### Test File Matching Algorithm

For a source file `src/foo/parser.rs`:

1. Extract base name: `"parser"`
2. Generate candidate patterns:
   - `tests/**/parser.rs`
   - `tests/**/parser_test.rs`
   - `tests/**/parser_tests.rs`
   - `src/**/parser_test.rs`
   - `src/**/parser_tests.rs`
   - `test/**/parser.rs`
3. Check if any changed test file matches any candidate
4. Also check inline `#[cfg(test)]` changes in the source file itself

### Inline Test Detection

For Rust files, parse diff to detect `#[cfg(test)]` changes:

1. Get file diff: `git diff {base}..HEAD -- path/to/file.rs`
2. Parse hunks looking for `#[cfg(test)]` boundaries
3. Track brace depth to identify cfg(test) block extent
4. If any `+` lines are within a cfg(test) block, test requirement is satisfied

### Violation Output Format

Per spec (`docs/specs/checks/tests.md#json-output`):

```json
{
  "file": "src/parser.rs",
  "line": null,
  "type": "missing_tests",
  "change_type": "modified",
  "lines_changed": 79,
  "advice": "Add tests in tests/parser_tests.rs or update inline #[cfg(test)] block"
}
```

### Metrics Output

```json
{
  "source_files_changed": 5,
  "with_test_changes": 3,
  "without_test_changes": 2,
  "scope": "branch"
}
```

## Verification Plan

### Unit Tests

Each module has sibling `_tests.rs` files:

```bash
# Run all unit tests
cargo test --all -- tests::

# Run specific module tests
cargo test --all -- diff_tests
cargo test --all -- correlation_tests
```

### Behavioral Specs

Remove `#[ignore]` from Phase 701 specs and verify:

```bash
# Run correlation specs
cargo test --test specs checks::tests::correlation

# Specific spec
cargo test --test specs staged_flag_checks_only_staged_files
```

### Integration Verification

```bash
# Full test suite
make check

# Manual verification
cd /tmp && git init test-proj && cd test-proj
echo '[check.tests.commit]\ncheck = "error"' > quench.toml
mkdir -p src tests
echo 'pub fn parse() {}' > src/parser.rs
git add . && git commit -m "initial"
git checkout -b feature
echo 'pub fn parse() { /* updated */ }' > src/parser.rs
git add . && git commit -m "feat: update parser"
quench check --base main  # Should fail: missing tests
```

### Spec Checklist

After implementation, all these should pass:

- [ ] `staged_flag_checks_only_staged_files`
- [ ] `base_flag_compares_against_git_ref`
- [ ] `source_change_without_test_change_generates_violation`
- [ ] `test_change_without_source_change_passes_tdd`
- [ ] `inline_cfg_test_change_satisfies_test_requirement`
- [ ] `placeholder_test_satisfies_test_requirement`
- [ ] `excluded_files_dont_require_tests`
- [ ] `json_includes_source_files_changed_metrics`
- [ ] `tests_violation_type_is_always_missing_tests`

## Notes

### Cache Versioning

If check logic changes how results are computed, bump `CACHE_VERSION` in `crates/cli/src/cache.rs`.

### Not In Scope

This phase focuses on change detection. The following are out of scope:
- CI mode test execution
- Coverage collection
- Test timing metrics
- Test suite configuration (`[[check.tests.suite]]`)

These will be addressed in future phases.
