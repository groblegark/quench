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
