# Quench (Quality Bench) - Master Design Plan

A fast, configurable quality linting CLI tool in Rust.

**Location**: `/Users/kestred/Developer/quench/`
**Scope**: Full implementation (all adapters, comparison engine, weekly reports)

## Overview

Quench consolidates quality checking patterns from otters, v0, wok, and claudeless into a unified, high-performance tool with:
- **Performance**: Near-cloc speed using ripgrep-inspired parallel file scanning
- **Extensibility**: Pluggable language adapters (Rust, Shell, future TypeScript)
- **Configurability**: TOML config with per-project/subproject settings
- **Comprehensive Reporting**: JSON, Markdown, terminal output with trending

## Project Structure

```
... TODO ...
```

## Configuration (quench.toml)

See [docs/specs](docs/specs/00-overview.md)

## CLI Interface

```
quench [OPTIONS] <COMMAND>

Commands:
  check       Run quality checks (default)
  compare     Compare against baseline/branch
  report      Generate formatted reports
  init        Initialize quench.toml
  lint        Run individual lint checks

quench check [OPTIONS] [SUBPROJECT]
  -f, --fast           Skip slow checks (coverage, timing)
  -o, --output <FMT>   terminal|json|markdown
  --include <CHECK>    Include only these checks
  --exclude <CHECK>    Exclude these checks
  --baseline <FILE>    Compare against baseline
  --branch <BRANCH>    Compare against git branch
  --save <FILE>        Save metrics to file

quench compare <TARGET>
  --json               Output as JSON
  --fail-on-regression Exit non-zero on regression

quench report
  -o, --output <FMT>   json|markdown
  --weekly             Weekly trending report
  --days <N>           Days to include (default: 7)

quench lint <LINT>
  loc, file-size, escapes, forbidden, justified,
  fmt, clippy, deny, audit, coverage, shellcheck
```

## Key Types

### MetricsReport
```rust
pub struct MetricsReport {
    pub timestamp: DateTime<Utc>,
    pub git_sha: Option<String>,
    pub subprojects: HashMap<String, SubprojectMetrics>,
    pub totals: TotalMetrics,
    pub checks: Vec<CheckResult>,
}

pub struct SubprojectMetrics {
    pub loc: LocMetrics,           // source/test LOC & file counts
    pub file_size: FileSizeMetrics, // avg/max, over_limit violations
    pub escapes: EscapeMetrics,    // unsafe/unwrap/expect counts
    pub coverage: Option<CoverageMetrics>,
}
```

### LanguageAdapter Trait
```rust
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn file_patterns(&self) -> &[&str];
    fn is_available(&self) -> bool;
    async fn run_checks(&self, root: &Path, config: &Config) -> Result<Vec<CheckResult>>;
    async fn run_fast_checks(&self, root: &Path, config: &Config) -> Result<Vec<CheckResult>>;
}
```

## Performance Strategy

1. **ignore crate**: Parallel, gitignore-aware file walking (same as ripgrep)
2. **rayon**: Parallel metric collection across files
3. **memmap2**: Memory-mapped files for fast reading
4. **grep-regex/grep-searcher**: ripgrep's regex engine for pattern matching
5. **dashmap**: Concurrent file content cache

```rust
// Scanner using ignore crate
let walker = WalkBuilder::new(root)
    .threads(num_cpus)
    .git_ignore(true)
    .build_parallel();

// Parallel pattern matching
files.par_iter()
    .flat_map(|f| check_patterns(f))
    .collect()
```

## Comparison Engine

1. **Baseline comparison**: Load baseline.json, compare metrics, detect regressions
2. **Branch comparison**: Stash, checkout branch, collect metrics, compare, restore
3. **Ratcheting**: Fail if escapes increase, coverage drops beyond variance, files over limit increase

## Built-in Checks

| Check | Description | Fast Mode |
|-------|-------------|-----------|
| `loc` | Lines of code (source vs test) | Yes |
| `file_size` | File size limits (avg/max) | Yes |
| `escapes` | unsafe/unwrap/expect detection | Yes |
| `forbidden` | Forbidden patterns in prod code | Yes |
| `justified` | Required justification comments | Yes |
| `git` | Commit analysis by type | Yes |
| `coverage` | LLVM-cov line coverage | No |
| `compile_time` | Build timing | No |
| `test_time` | Test suite timing | No |
| `memory` | Peak RSS profiling | No |

## Output Formats

1. **Terminal**: Colored tables, pass/fail icons, progress bars
2. **JSON**: Machine-readable MetricsReport struct
3. **Markdown**: Summary tables, per-subproject breakdown, violations list

## Implementation Phases

### Phase 1: Core Infrastructure
- Project scaffolding with Cargo workspace
- Configuration system (TOML parsing, validation)
- Fast file scanner (ignore crate)
- Basic CLI with clap

### Phase 2: Built-in Checks
- LOC counting (parallel, source/test separation)
- File size limits
- Escape hatch detection
- Forbidden patterns
- Justification comments
- Git commit analysis

### Phase 3: Adapters
- Adapter trait and registry
- Rust adapter (fmt, clippy, deny, audit, coverage)
- Shell adapter (shellcheck)
- Generic adapter

### Phase 4: Reporting
- Terminal output (crossterm colors)
- JSON output
- Markdown output
- Weekly trending

### Phase 5: Comparison Engine
- Baseline comparison
- Branch comparison (git2)
- Ratcheting logic

### Phase 6: Polish
- Performance benchmarks vs cloc
- Documentation
- Integration tests

## Key Dependencies

```toml
# Core
rayon = "1.8"              # Parallel iteration
ignore = "0.4"             # Fast gitignore-aware walking
grep-regex = "0.1"         # ripgrep's regex engine
memmap2 = "0.9"            # Memory-mapped files
dashmap = "5"              # Concurrent HashMap
git2 = "0.18"              # Git operations
serde = "1"                # Serialization
toml = "0.8"               # Config parsing
chrono = "0.4"             # Timestamps

# CLI
clap = "4"                 # Argument parsing
crossterm = "0.27"         # Terminal colors
indicatif = "0.17"         # Progress bars
tokio = "1"                # Async runtime
```

## Verification Plan

1. **Unit tests**: Per-module tests for config, checks, comparison
2. **Integration tests**: Full CLI invocation with fixture projects
3. **Benchmarks**: criterion benchmarks comparing scan speed to cloc
4. **Manual testing**: Run against otters, wok, claudeless to verify patterns detected

## Critical Files to Create

1. `crates/core/src/config/schema.rs` - Config types
2. `crates/core/src/scanner/walker.rs` - Parallel file scanning
3. `crates/core/src/metrics/types.rs` - Metric data structures
4. `crates/adapters/src/traits.rs` - LanguageAdapter trait
5. `crates/cli/src/commands/check.rs` - Main check command
