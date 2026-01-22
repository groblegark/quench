# Phase 040: Check Framework - Implementation

**Root Feature:** `quench-c29c`

## Overview

Implement the check framework that enables parallel execution of multiple checks with proper error isolation, skip handling, and CLI toggle flags. This builds on the output infrastructure from Phase 030 and makes the Phase 035 behavioral specs pass.

**Current State**: Output infrastructure complete (Phase 030). Only `cloc` check implemented (hardcoded in main.rs). `CheckResult` and `Violation` types exist in `check.rs`. Phase 035 behavioral specs exist in `tests/specs/checks.rs` with `#[ignore]`.

**End State**: All 8 checks registered (cloc, escapes, agents, docs, tests, git, build, license). Check toggle flags (`--[no-]cloc`, etc.) work. Checks run in parallel via rayon. Error isolation ensures one check failure doesn't block others. All Phase 035 specs pass.

## Project Structure

```
crates/cli/src/
├── cli.rs              # MODIFY: Add check toggle flags (--cloc, --no-cloc, etc.)
├── check.rs            # MODIFY: Add Check trait, CheckConfig, metrics/by_package fields
├── check_tests.rs      # MODIFY: Add Check trait tests
├── checks/             # NEW: Individual check implementations
│   ├── mod.rs          # Check registry and discovery
│   ├── cloc.rs         # Cloc check (extract from main.rs)
│   ├── cloc_tests.rs   # Unit tests
│   └── stub.rs         # Stub implementation for unimplemented checks
├── runner.rs           # NEW: Parallel check runner with error recovery
├── runner_tests.rs     # Unit tests for runner
├── lib.rs              # MODIFY: Export new modules
└── main.rs             # MODIFY: Use check runner instead of direct cloc call

tests/
├── specs/
│   └── checks.rs       # Phase 035 specs (remove #[ignore] as features land)
└── fixtures/
    └── check-framework/# EXISTS: Fixture for check framework tests
```

## Dependencies

No new dependencies. Existing dependencies provide all needed functionality:

- `rayon` - Already in use for parallel file walking
- `serde` - Already in use for JSON output
- `clap` - Already in use for CLI

## Implementation Phases

### Phase 40.1: Check Trait and Extended Types

**Goal**: Define the `Check` trait and extend result types with metrics and by_package fields.

**Tasks**:
1. Define object-safe `Check` trait in `check.rs`
2. Add `metrics` and `by_package` fields to `CheckResult`
3. Create `CheckConfig` struct for check-specific configuration
4. Add `CheckContext` for passing shared state to checks

**Files**:

```rust
// crates/cli/src/check.rs (additions)

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde_json::Value as JsonValue;

use crate::config::Config;
use crate::walker::WalkedFile;

/// Context passed to all checks during execution.
pub struct CheckContext<'a> {
    /// Project root directory.
    pub root: &'a Path,
    /// Discovered files from the walker.
    pub files: &'a [WalkedFile],
    /// Parsed configuration.
    pub config: &'a Config,
    /// Violation limit (None = unlimited).
    pub limit: Option<usize>,
    /// Running violation count across all checks.
    pub violation_count: &'a std::sync::atomic::AtomicUsize,
}

/// The Check trait defines a single quality check.
///
/// Object-safe to allow dynamic dispatch via `Box<dyn Check>`.
pub trait Check: Send + Sync {
    /// Unique identifier for this check (e.g., "cloc", "escapes").
    fn name(&self) -> &'static str;

    /// Human-readable description for help output.
    fn description(&self) -> &'static str;

    /// Run the check and return results.
    ///
    /// Implementations should:
    /// - Return `CheckResult::skipped()` if prerequisites are missing
    /// - Respect `ctx.limit` for early termination
    /// - Handle errors gracefully without panicking
    fn run(&self, ctx: &CheckContext) -> CheckResult;

    /// Whether this check can auto-fix violations.
    fn fixable(&self) -> bool {
        false
    }

    /// Whether this check is enabled by default in fast mode.
    fn default_enabled(&self) -> bool {
        true
    }
}

// Extend CheckResult to support metrics and by_package
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,

    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub skipped: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<Violation>,

    /// Aggregated metrics for this check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<JsonValue>,

    /// Per-package breakdown of metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_package: Option<HashMap<String, JsonValue>>,
}

impl CheckResult {
    /// Create a result with metrics.
    pub fn with_metrics(mut self, metrics: JsonValue) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Add per-package metrics breakdown.
    pub fn with_by_package(mut self, by_package: HashMap<String, JsonValue>) -> Self {
        self.by_package = Some(by_package);
        self
    }
}
```

**Verification**:
```bash
cargo build --lib
cargo test check::tests
```

### Phase 40.2: Check Registry and Stub Implementations

**Goal**: Create the check registry with all 8 checks registered. Unimplemented checks use stubs.

**Tasks**:
1. Create `crates/cli/src/checks/mod.rs` with registry
2. Create `crates/cli/src/checks/stub.rs` for placeholder checks
3. Register all 8 checks with correct names and default states
4. Implement check discovery by name

**Files**:

```rust
// crates/cli/src/checks/mod.rs
//! Check registry and discovery.
//!
//! All 8 built-in checks are registered here:
//! - cloc: Lines of code, file size limits (enabled by default)
//! - escapes: Escape hatch detection (enabled by default)
//! - agents: CLAUDE.md, .cursorrules validation (enabled by default)
//! - docs: File refs, specs validation (enabled by default)
//! - tests: Test correlation (enabled by default)
//! - git: Commit message format (disabled by default)
//! - build: Binary/bundle size + build time (disabled by default)
//! - license: License header validation (disabled by default)

pub mod cloc;
pub mod stub;

use std::sync::Arc;

use crate::check::Check;

/// All registered check names in canonical order.
pub const CHECK_NAMES: &[&str] = &[
    "cloc", "escapes", "agents", "docs", "tests", "git", "build", "license",
];

/// Checks enabled by default in fast mode.
pub const DEFAULT_ENABLED: &[&str] = &["cloc", "escapes", "agents", "docs", "tests"];

/// Create all registered checks.
pub fn all_checks() -> Vec<Arc<dyn Check>> {
    vec![
        Arc::new(cloc::ClocCheck),
        Arc::new(stub::StubCheck::new("escapes", "Escape hatch detection", true)),
        Arc::new(stub::StubCheck::new("agents", "Agent file validation", true)),
        Arc::new(stub::StubCheck::new("docs", "Documentation validation", true)),
        Arc::new(stub::StubCheck::new("tests", "Test correlation", true)),
        Arc::new(stub::StubCheck::new("git", "Commit message format", false)),
        Arc::new(stub::StubCheck::new("build", "Build metrics", false)),
        Arc::new(stub::StubCheck::new("license", "License headers", false)),
    ]
}

/// Get a check by name.
pub fn get_check(name: &str) -> Option<Arc<dyn Check>> {
    all_checks().into_iter().find(|c| c.name() == name)
}

/// Filter checks based on enabled/disabled flags.
pub fn filter_checks(
    enabled: &[String],
    disabled: &[String],
) -> Vec<Arc<dyn Check>> {
    let all = all_checks();

    if !enabled.is_empty() {
        // Explicit enable: only run specified checks
        all.into_iter()
            .filter(|c| enabled.iter().any(|e| e == c.name()))
            .collect()
    } else {
        // Default mode: run default checks minus disabled
        all.into_iter()
            .filter(|c| c.default_enabled())
            .filter(|c| !disabled.iter().any(|d| d == c.name()))
            .collect()
    }
}
```

```rust
// crates/cli/src/checks/stub.rs
//! Stub check implementation for unimplemented checks.

use crate::check::{Check, CheckContext, CheckResult};

/// A stub check that always passes.
/// Used for checks not yet implemented.
pub struct StubCheck {
    name: &'static str,
    description: &'static str,
    default_enabled: bool,
}

impl StubCheck {
    pub fn new(name: &'static str, description: &'static str, default_enabled: bool) -> Self {
        Self { name, description, default_enabled }
    }
}

impl Check for StubCheck {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn run(&self, _ctx: &CheckContext) -> CheckResult {
        // Stub checks always pass (no implementation yet)
        CheckResult::passed(self.name)
    }

    fn default_enabled(&self) -> bool {
        self.default_enabled
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn stub_check_always_passes() {
        let check = StubCheck::new("test", "Test check", true);
        // Would need mock context - test via integration tests
        assert_eq!(check.name(), "test");
        assert!(check.default_enabled());
    }
}
```

**Verification**:
```bash
cargo build --lib
cargo test checks::
```

### Phase 40.3: Cloc Check Extraction

**Goal**: Extract the cloc check from `main.rs` into a proper `Check` implementation.

**Tasks**:
1. Create `crates/cli/src/checks/cloc.rs`
2. Move cloc logic from `main.rs` to the new module
3. Implement `Check` trait for `ClocCheck`
4. Add metrics collection (source_lines, test_lines, ratio)

**Files**:

```rust
// crates/cli/src/checks/cloc.rs
//! Cloc (count lines of code) check.
//!
//! Validates file size limits per docs/specs/checks/cloc.md.

use std::io::BufRead;
use std::path::Path;

use serde_json::json;

use crate::check::{Check, CheckContext, CheckResult, Violation};
use crate::config::CheckLevel;

/// The cloc check validates file size limits.
pub struct ClocCheck;

impl Check for ClocCheck {
    fn name(&self) -> &'static str {
        "cloc"
    }

    fn description(&self) -> &'static str {
        "Lines of code and file size limits"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let cloc_config = &ctx.config.check.cloc;

        // Skip if disabled
        if cloc_config.check == CheckLevel::Off {
            return CheckResult::passed(self.name());
        }

        let mut violations = Vec::new();
        let mut source_lines: usize = 0;
        let mut test_lines: usize = 0;

        for file in ctx.files {
            // Skip non-text files
            if !is_text_file(&file.path) {
                continue;
            }

            match count_lines(&file.path) {
                Ok(line_count) => {
                    let is_test = is_test_file(&file.path);

                    // Accumulate metrics
                    if is_test {
                        test_lines += line_count;
                    } else {
                        source_lines += line_count;
                    }

                    let max_lines = if is_test {
                        cloc_config.max_lines_test
                    } else {
                        cloc_config.max_lines
                    };

                    if line_count > max_lines {
                        // Check violation limit
                        let current = ctx.violation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if let Some(limit) = ctx.limit {
                            if current >= limit {
                                break;
                            }
                        }

                        let display_path = file.path.strip_prefix(ctx.root).unwrap_or(&file.path);
                        violations.push(
                            Violation::file_only(
                                display_path,
                                "file_too_large",
                                format!(
                                    "Split into smaller modules. {} lines exceeds {} line limit.",
                                    line_count, max_lines
                                ),
                            )
                            .with_threshold(line_count as i64, max_lines as i64),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to count lines in {}: {}", file.path.display(), e);
                }
            }
        }

        let result = if violations.is_empty() {
            CheckResult::passed(self.name())
        } else {
            CheckResult::failed(self.name(), violations)
        };

        // Add metrics
        let ratio = if source_lines > 0 {
            test_lines as f64 / source_lines as f64
        } else {
            0.0
        };

        result.with_metrics(json!({
            "source_lines": source_lines,
            "test_lines": test_lines,
            "ratio": (ratio * 100.0).round() / 100.0,
        }))
    }

    fn default_enabled(&self) -> bool {
        true
    }
}

/// Check if a file appears to be a text file (not binary).
fn is_text_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        ext.as_str(),
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "c" | "cpp" | "h" | "hpp"
            | "java" | "kt" | "scala" | "rb" | "php" | "cs" | "swift" | "m" | "mm"
            | "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" | "lua" | "pl"
            | "pm" | "r" | "sql" | "md" | "txt" | "toml" | "yaml" | "yml" | "json"
            | "xml" | "html" | "css" | "scss" | "sass" | "less" | "vue" | "svelte"
    )
}

/// Check if a file is a test file based on its filename.
fn is_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    file_name.contains("_test.")
        || file_name.contains("_tests.")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_tests.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.ends_with(".test.js")
        || file_name.ends_with(".test.ts")
        || file_name.ends_with(".test.tsx")
        || file_name.ends_with(".spec.js")
        || file_name.ends_with(".spec.ts")
        || file_name.ends_with(".spec.tsx")
}

/// Count the number of lines in a file.
fn count_lines(path: &Path) -> std::io::Result<usize> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    Ok(reader.lines().count())
}

#[cfg(test)]
#[path = "cloc_tests.rs"]
mod tests;
```

**Verification**:
```bash
cargo build
cargo test checks::cloc::tests
```

### Phase 40.4: CLI Check Toggle Flags

**Goal**: Add `--[no-]<check>` flags to the CLI.

**Tasks**:
1. Update `CheckArgs` in `cli.rs` with check toggle flags
2. Parse enabled/disabled checks from flags
3. Wire flags to check filtering

**Files**:

```rust
// crates/cli/src/cli.rs (additions to CheckArgs)

#[derive(clap::Args)]
pub struct CheckArgs {
    // ... existing fields ...

    // Check enable flags (run only these checks)
    /// Run only the cloc check
    #[arg(long)]
    pub cloc: bool,

    /// Run only the escapes check
    #[arg(long)]
    pub escapes: bool,

    /// Run only the agents check
    #[arg(long)]
    pub agents: bool,

    /// Run only the docs check
    #[arg(long)]
    pub docs: bool,

    /// Run only the tests check
    #[arg(long = "tests")]
    pub tests_check: bool,  // Renamed to avoid conflict with #[test]

    /// Run only the git check
    #[arg(long)]
    pub git: bool,

    /// Run only the build check
    #[arg(long)]
    pub build: bool,

    /// Run only the license check
    #[arg(long)]
    pub license: bool,

    // Check disable flags (skip these checks)
    /// Skip the cloc check
    #[arg(long)]
    pub no_cloc: bool,

    /// Skip the escapes check
    #[arg(long)]
    pub no_escapes: bool,

    /// Skip the agents check
    #[arg(long)]
    pub no_agents: bool,

    /// Skip the docs check
    #[arg(long)]
    pub no_docs: bool,

    /// Skip the tests check
    #[arg(long)]
    pub no_tests: bool,

    /// Skip the git check
    #[arg(long)]
    pub no_git: bool,

    /// Skip the build check
    #[arg(long)]
    pub no_build: bool,

    /// Skip the license check
    #[arg(long)]
    pub no_license: bool,
}

impl CheckArgs {
    /// Get list of explicitly enabled checks.
    pub fn enabled_checks(&self) -> Vec<String> {
        let mut enabled = Vec::new();
        if self.cloc { enabled.push("cloc".to_string()); }
        if self.escapes { enabled.push("escapes".to_string()); }
        if self.agents { enabled.push("agents".to_string()); }
        if self.docs { enabled.push("docs".to_string()); }
        if self.tests_check { enabled.push("tests".to_string()); }
        if self.git { enabled.push("git".to_string()); }
        if self.build { enabled.push("build".to_string()); }
        if self.license { enabled.push("license".to_string()); }
        enabled
    }

    /// Get list of explicitly disabled checks.
    pub fn disabled_checks(&self) -> Vec<String> {
        let mut disabled = Vec::new();
        if self.no_cloc { disabled.push("cloc".to_string()); }
        if self.no_escapes { disabled.push("escapes".to_string()); }
        if self.no_agents { disabled.push("agents".to_string()); }
        if self.no_docs { disabled.push("docs".to_string()); }
        if self.no_tests { disabled.push("tests".to_string()); }
        if self.no_git { disabled.push("git".to_string()); }
        if self.no_build { disabled.push("build".to_string()); }
        if self.no_license { disabled.push("license".to_string()); }
        disabled
    }
}
```

**Verification**:
```bash
cargo build
./target/debug/quench check --help  # Verify all flags shown
# Remove #[ignore] from check_toggles_shown_in_help
cargo test --test specs check_toggles_shown_in_help
```

### Phase 40.5: Check Runner with Parallel Execution

**Goal**: Implement the check runner that executes checks in parallel with error recovery.

**Tasks**:
1. Create `crates/cli/src/runner.rs` with `CheckRunner`
2. Use rayon for parallel check execution
3. Implement error isolation (continue on check failure/error)
4. Implement early termination on violation limit
5. Collect and merge results

**Files**:

```rust
// crates/cli/src/runner.rs
//! Parallel check runner with error recovery.
//!
//! Runs checks in parallel using rayon, isolating errors so one
//! check failure doesn't prevent other checks from running.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::check::{Check, CheckContext, CheckResult};
use crate::config::Config;
use crate::walker::WalkedFile;

/// Configuration for the check runner.
pub struct RunnerConfig {
    /// Maximum violations before early termination (None = unlimited).
    pub limit: Option<usize>,
}

/// The check runner executes multiple checks in parallel.
pub struct CheckRunner {
    config: RunnerConfig,
}

impl CheckRunner {
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Run all provided checks and return results.
    ///
    /// Checks run in parallel. Errors are isolated - one check failing
    /// doesn't prevent other checks from running.
    pub fn run(
        &self,
        checks: Vec<Arc<dyn Check>>,
        files: &[WalkedFile],
        config: &Config,
        root: &std::path::Path,
    ) -> Vec<CheckResult> {
        let violation_count = AtomicUsize::new(0);

        // Run checks in parallel
        let results: Vec<CheckResult> = checks
            .into_par_iter()
            .map(|check| {
                let ctx = CheckContext {
                    root,
                    files,
                    config,
                    limit: self.config.limit,
                    violation_count: &violation_count,
                };

                // Catch panics to ensure error isolation
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    check.run(&ctx)
                })) {
                    Ok(result) => result,
                    Err(_) => {
                        // Check panicked - return skipped result
                        CheckResult::skipped(
                            check.name(),
                            "Internal error: check panicked".to_string(),
                        )
                    }
                }
            })
            .collect();

        // Sort results by canonical check order for consistent output
        let mut sorted = results;
        sorted.sort_by_key(|r| {
            crate::checks::CHECK_NAMES
                .iter()
                .position(|&n| n == r.name)
                .unwrap_or(usize::MAX)
        });

        sorted
    }

    /// Check if early termination is needed based on violation count.
    pub fn should_terminate(&self, violation_count: usize) -> bool {
        if let Some(limit) = self.config.limit {
            violation_count >= limit
        } else {
            false
        }
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
```

```rust
// crates/cli/src/runner_tests.rs
//! Unit tests for the check runner.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::check::{Check, CheckContext, CheckResult, Violation};

/// Mock check that can be configured to pass, fail, or panic.
struct MockCheck {
    name: &'static str,
    behavior: MockBehavior,
    ran: AtomicBool,
}

enum MockBehavior {
    Pass,
    Fail(usize),  // Number of violations
    Panic,
    Skip(String),
}

impl MockCheck {
    fn new(name: &'static str, behavior: MockBehavior) -> Self {
        Self {
            name,
            behavior,
            ran: AtomicBool::new(false),
        }
    }

    fn did_run(&self) -> bool {
        self.ran.load(Ordering::SeqCst)
    }
}

impl Check for MockCheck {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        "Mock check"
    }

    fn run(&self, _ctx: &CheckContext) -> CheckResult {
        self.ran.store(true, Ordering::SeqCst);

        match &self.behavior {
            MockBehavior::Pass => CheckResult::passed(self.name),
            MockBehavior::Fail(count) => {
                let violations: Vec<_> = (0..*count)
                    .map(|i| Violation::file_only(
                        format!("file{}.rs", i),
                        "test_violation",
                        "Fix this",
                    ))
                    .collect();
                CheckResult::failed(self.name, violations)
            }
            MockBehavior::Panic => panic!("Mock check panicked"),
            MockBehavior::Skip(msg) => CheckResult::skipped(self.name, msg.clone()),
        }
    }

    fn default_enabled(&self) -> bool {
        true
    }
}

#[test]
fn runner_executes_all_checks() {
    let runner = CheckRunner::new(RunnerConfig { limit: None });
    let config = Config::default();
    let files = vec![];
    let root = std::path::Path::new(".");

    let checks: Vec<Arc<dyn Check>> = vec![
        Arc::new(MockCheck::new("check1", MockBehavior::Pass)),
        Arc::new(MockCheck::new("check2", MockBehavior::Fail(1))),
        Arc::new(MockCheck::new("check3", MockBehavior::Pass)),
    ];

    let results = runner.run(checks.clone(), &files, &config, root);

    assert_eq!(results.len(), 3, "all checks should have results");
}

#[test]
fn runner_isolates_panicking_check() {
    let runner = CheckRunner::new(RunnerConfig { limit: None });
    let config = Config::default();
    let files = vec![];
    let root = std::path::Path::new(".");

    let passing = Arc::new(MockCheck::new("passing", MockBehavior::Pass));
    let panicking = Arc::new(MockCheck::new("panicking", MockBehavior::Panic));

    let checks: Vec<Arc<dyn Check>> = vec![
        passing.clone(),
        panicking.clone(),
    ];

    let results = runner.run(checks, &files, &config, root);

    // Both checks should have results
    assert_eq!(results.len(), 2);

    // Passing check should have run and passed
    let pass_result = results.iter().find(|r| r.name == "passing").unwrap();
    assert!(pass_result.passed);

    // Panicking check should be skipped with error
    let panic_result = results.iter().find(|r| r.name == "panicking").unwrap();
    assert!(panic_result.skipped);
    assert!(panic_result.error.is_some());
}

#[test]
fn runner_continues_after_check_failure() {
    let runner = CheckRunner::new(RunnerConfig { limit: None });
    let config = Config::default();
    let files = vec![];
    let root = std::path::Path::new(".");

    let check1 = Arc::new(MockCheck::new("check1", MockBehavior::Fail(5)));
    let check2 = Arc::new(MockCheck::new("check2", MockBehavior::Pass));

    let checks: Vec<Arc<dyn Check>> = vec![check1.clone(), check2.clone()];

    let results = runner.run(checks, &files, &config, root);

    // Both checks should run
    assert!(check1.did_run());
    assert!(check2.did_run());

    // First failed, second passed
    assert!(!results[0].passed || !results[1].passed);
    assert!(results.iter().any(|r| r.passed));
}
```

**Verification**:
```bash
cargo test runner::tests
```

### Phase 40.6: Main Integration

**Goal**: Wire everything together in `main.rs` and remove the hardcoded cloc check.

**Tasks**:
1. Update `main.rs` to use `CheckRunner`
2. Use `filter_checks` with CLI flags
3. Remove old cloc implementation
4. Update `lib.rs` exports

**Files**:

```rust
// crates/cli/src/main.rs (updated run_check function)

fn run_check(cli: &Cli, args: &CheckArgs) -> anyhow::Result<ExitCode> {
    let cwd = std::env::current_dir()?;

    // Determine root directory
    let root = if args.paths.is_empty() {
        cwd.clone()
    } else {
        let path = &args.paths[0];
        if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        }
    };

    // Load config
    let config_path = if cli.config.is_some() {
        discovery::resolve_config(cli.config.as_deref(), &cwd)?
    } else {
        discovery::find_config(&root)
    };

    let config = match &config_path {
        Some(path) => {
            tracing::debug!("loading config from {}", path.display());
            config::load_with_warnings(path)?
        }
        None => {
            tracing::debug!("no config found, using defaults");
            config::Config::default()
        }
    };

    // Config-only mode: validate and exit
    if args.config_only {
        return Ok(ExitCode::Success);
    }

    // Configure walker
    let walker_config = WalkerConfig {
        max_depth: Some(args.max_depth),
        ignore_patterns: config.project.ignore.patterns.clone(),
        ..Default::default()
    };

    let walker = FileWalker::new(walker_config);
    let (rx, handle) = walker.walk(&root);

    // Collect files
    let files: Vec<_> = rx.iter().collect();
    let stats = handle.join();

    if args.verbose {
        eprintln!("Scanned {} files", files.len());
        if stats.errors > 0 {
            eprintln!("Warning: {} walk error(s)", stats.errors);
        }
    }

    // Filter checks based on CLI flags
    let checks = checks::filter_checks(
        &args.enabled_checks(),
        &args.disabled_checks(),
    );

    // Create runner
    let limit = if args.no_limit { None } else { Some(args.limit) };
    let runner = CheckRunner::new(RunnerConfig { limit });

    // Run checks
    let check_results = runner.run(checks, &files, &config, &root);

    // Create output
    let output = json::create_output(check_results);
    let total_violations = output.total_violations();

    // ... rest of formatting and output (unchanged) ...
}
```

**Verification**:
```bash
cargo build
# Remove #[ignore] from Phase 035 specs one by one and verify
cargo test --test specs checks::
```

### Phase 40.7: Remove #[ignore] and Final Verification

**Goal**: Remove all `#[ignore]` attributes from Phase 035 specs and ensure they pass.

**Tasks**:
1. Remove `#[ignore]` from all check name specs
2. Remove `#[ignore]` from all toggle flag specs
3. Remove `#[ignore]` from all combination specs
4. Remove `#[ignore]` from all error isolation specs
5. Run `make check` for full validation

**Verification Order**:

```bash
# Check names (Phase 035.2)
cargo test --test specs check_names_are_exactly_8_known_checks
cargo test --test specs check_toggles_shown_in_help

# Enable flags (Phase 035.3)
cargo test --test specs cloc_flag_enables_only_cloc_check
cargo test --test specs escapes_flag_enables_only_escapes_check

# Disable flags (Phase 035.3)
cargo test --test specs no_cloc_flag_disables_cloc_check
cargo test --test specs no_escapes_flag_disables_escapes_check
cargo test --test specs no_docs_flag_disables_docs_check
cargo test --test specs no_tests_flag_disables_tests_check

# Flag combinations (Phase 035.4)
cargo test --test specs multiple_enable_flags_run_multiple_checks
cargo test --test specs multiple_disable_flags_skip_multiple_checks
cargo test --test specs no_cloc_no_escapes_skips_both
cargo test --test specs all_checks_disabled_except_one

# Error isolation (Phase 035.5)
cargo test --test specs check_failure_doesnt_block_other_checks
cargo test --test specs skipped_check_shows_error_but_continues
cargo test --test specs skipped_check_text_output_shows_reason
cargo test --test specs skipped_check_json_has_required_fields

# Full validation
make check
```

## Key Implementation Details

### Check Trait Design

The `Check` trait is object-safe for dynamic dispatch:

```rust
pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, ctx: &CheckContext) -> CheckResult;
    fn fixable(&self) -> bool { false }
    fn default_enabled(&self) -> bool { true }
}
```

Key design decisions:
- `Send + Sync` required for parallel execution with rayon
- Static string returns (`&'static str`) avoid lifetime complexity
- Default implementations for optional methods
- `CheckContext` provides all shared state

### Check Names and Defaults

| Name | Default | Description |
|------|---------|-------------|
| `cloc` | Enabled | Lines of code, file size limits |
| `escapes` | Enabled | Escape hatch detection |
| `agents` | Enabled | CLAUDE.md, .cursorrules validation |
| `docs` | Enabled | File refs, specs validation |
| `tests` | Enabled | Test correlation |
| `git` | Disabled | Commit message format |
| `build` | Disabled | Binary/bundle size + build time |
| `license` | Disabled | License header validation |

### Flag Semantics

1. **No flags**: Run all default-enabled checks
2. **`--<check>`**: Run ONLY that check (exclusive mode)
3. **Multiple `--<check>`**: Run only the specified checks
4. **`--no-<check>`**: Skip that check, run all other defaults
5. **Multiple `--no-<check>`**: Skip all specified, run remaining defaults

### Error Isolation

Checks are isolated via:
1. **Panic catching**: `std::panic::catch_unwind` around each check
2. **Independent execution**: rayon's parallel iterator runs checks independently
3. **Result collection**: All results collected regardless of individual failures

When a check fails:
- `passed: false` in result
- `violations` array populated
- Other checks still run

When a check is skipped (error):
- `skipped: true` in result
- `error` field explains why
- Other checks still run

### Early Termination

In non-CI mode with violation limit:
1. Shared `AtomicUsize` tracks violation count
2. Checks increment counter when adding violations
3. When limit reached, checks stop adding violations
4. Full check still completes (for accurate metrics)

### Parallel Execution

Using rayon for parallelism:

```rust
checks.into_par_iter()
    .map(|check| check.run(&ctx))
    .collect()
```

Key points:
- Work-stealing balances load across checks
- Each check runs independently
- Results sorted to canonical order after collection

## Verification Plan

### Spec Coverage

| Spec Category | Count | Phase |
|---------------|-------|-------|
| Check names | 2 | 40.7 |
| Enable flags | 2 | 40.7 |
| Disable flags | 4 | 40.7 |
| Flag combinations | 4 | 40.7 |
| Error isolation | 1 | 40.7 |
| Skipped checks | 3 | 40.7 |
| **Total** | **16** | |

### Phase Completion Checklist

- [ ] **40.1**: Check trait and extended types compile
- [ ] **40.2**: Check registry with all 8 checks registered
- [ ] **40.3**: Cloc check extracted and working
- [ ] **40.4**: CLI toggle flags added and parsing correctly
- [ ] **40.5**: Check runner with parallel execution and error isolation
- [ ] **40.6**: Main.rs integrated with runner
- [ ] **40.7**: All 16 Phase 035 specs pass, `make check` passes

### Running Verification

```bash
# After each phase:
cargo build
cargo test

# After Phase 40.4:
./target/debug/quench check --help | grep -E '\-\-(no-)?cloc'

# After Phase 40.5:
cargo test runner::tests

# After Phase 40.6:
./target/debug/quench check --cloc -o json | jq '.checks | length'  # Should be 1

# Final verification:
cargo test --test specs checks::
make check
```

## Summary

Phase 040 implements the check framework:

1. **Check trait** (`check.rs`): Object-safe trait for check implementations
2. **Check registry** (`checks/mod.rs`): All 8 checks registered with discovery
3. **Cloc check** (`checks/cloc.rs`): Extracted and enhanced with metrics
4. **CLI flags** (`cli.rs`): `--[no-]<check>` toggle flags for all checks
5. **Check runner** (`runner.rs`): Parallel execution with error isolation
6. **Integration** (`main.rs`): Wired up with filtering and output

All 16 Phase 035 specs will pass upon completion.
