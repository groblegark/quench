# Phase 005: Project Foundation - Implementation

## Overview

Implement the foundational CLI skeleton and configuration system for quench. This phase brings the CLI contract to life by implementing the command structure, global flags, config file discovery and parsing, and logging infrastructure. All Phase 003 behavioral specs marked `#[ignore = "TODO: Phase 005 - CLI skeleton"]` will be enabled upon completion.

**Current State**: Basic clap setup exists with `--help` and `--version` working. No subcommands, no config parsing, no logging infrastructure.

**End State**: Fully functional CLI skeleton with `check`, `report`, and `init` subcommands; config file discovery and parsing with version validation; unknown key warnings; and QUENCH_LOG-based tracing.

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── main.rs              # (update) Entry point with tracing setup
│   ├── lib.rs               # (update) Module exports
│   ├── cli.rs               # (new) Clap CLI definition
│   ├── cli_tests.rs         # (new) CLI unit tests
│   ├── config.rs            # (new) Config struct and parsing
│   ├── config_tests.rs      # (new) Config unit tests
│   ├── discovery.rs         # (new) Config file discovery
│   ├── discovery_tests.rs   # (new) Discovery unit tests
│   └── error.rs             # (exists) Error types
└── tests/
    └── specs.rs             # (update) Remove #[ignore] from passing specs
```

## Dependencies

All dependencies already exist in `Cargo.toml`:

- `clap` (4.x with derive, env features) - CLI parsing
- `serde` (1.x with derive) - Config deserialization
- `toml` (0.8.x) - TOML parsing
- `tracing` (0.1.x) - Structured logging
- `tracing-subscriber` (0.3.x with env-filter) - Log filtering
- `thiserror` (2.x) - Error derive macro
- `anyhow` (1.x) - Error handling in main

## Implementation Phases

### Phase 5.1: CLI Structure with Clap

**Goal**: Implement full command structure matching `docs/specs/01-cli.md#commands`.

**Tasks**:
1. Create `cli.rs` with clap derive structs
2. Implement subcommands: `check`, `report`, `init`
3. Add global flags: `-C/--config`
4. Wire up to `main.rs`
5. Ensure bare invocation shows help

**Code**:

```rust
// crates/cli/src/cli.rs
use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// A fast linting tool for AI agents that measures quality signals
#[derive(Parser)]
#[command(name = "quench")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Use specific config file
    #[arg(short = 'C', long = "config", global = true, env = "QUENCH_CONFIG")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run quality checks
    Check(CheckArgs),
    /// Generate reports from stored metrics
    Report(ReportArgs),
    /// Initialize quench configuration
    Init(InitArgs),
}

#[derive(clap::Args)]
pub struct CheckArgs {
    /// Files or directories to check
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,
}

#[derive(clap::Args)]
pub struct ReportArgs {
    /// Output format
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,
}

#[derive(clap::Args)]
pub struct InitArgs {
    /// Overwrite existing config
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
```

**Verification**:
```bash
cargo run -- --help              # Shows help with subcommands
cargo run                        # Shows help (bare invocation)
cargo run -- help                # Shows help
cargo run -- check --help        # Shows check help
cargo run -- unknown             # Error with exit code 2
```

### Phase 5.2: Config File Discovery

**Goal**: Implement config discovery per `docs/specs/02-config.md#discovery`.

**Tasks**:
1. Create `discovery.rs` with discovery logic
2. Walk from current directory up to git root
3. Look for `quench.toml` at each level
4. Support explicit path via `-C`/`QUENCH_CONFIG`

**Code**:

```rust
// crates/cli/src/discovery.rs
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Find quench.toml starting from `start_dir` and walking up to git root
pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join("quench.toml");
        if config_path.exists() {
            return Some(config_path);
        }

        // Stop at git root
        if current.join(".git").exists() {
            return None;
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}

/// Resolve config path from CLI arg, env var, or discovery
pub fn resolve_config(explicit: Option<&Path>, cwd: &Path) -> Result<Option<PathBuf>> {
    match explicit {
        Some(path) => {
            if path.exists() {
                Ok(Some(path.to_path_buf()))
            } else {
                Err(Error::Config {
                    message: format!("config file not found: {}", path.display()),
                    path: Some(path.to_path_buf()),
                })
            }
        }
        None => Ok(find_config(cwd)),
    }
}
```

**Verification**:
```bash
# In a git repo with quench.toml
cargo run -- check              # Finds quench.toml

# With explicit path
cargo run -- -C /path/to/config.toml check

# Missing explicit config
cargo run -- -C /nonexistent.toml check  # Error exit 2
```

### Phase 5.3: Config Parsing with Version Validation

**Goal**: Parse config with required `version = 1` and struct validation.

**Tasks**:
1. Create `config.rs` with serde structs
2. Implement version validation (must be 1)
3. Parse known sections: `[project]`, `[check.*]`
4. Return clear errors for invalid version

**Code**:

```rust
// crates/cli/src/config.rs
use std::path::Path;
use serde::Deserialize;
use crate::error::{Error, Result};

/// Minimum config structure for version checking
#[derive(Deserialize)]
struct VersionOnly {
    version: Option<i64>,
}

/// Full configuration
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: i64,

    #[serde(default)]
    pub project: ProjectConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

const SUPPORTED_VERSION: i64 = 1;

/// Load and validate config from path
pub fn load(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse(&content, path)
}

/// Parse config from string content
pub fn parse(content: &str, path: &Path) -> Result<Config> {
    // First check version
    let version_check: VersionOnly = toml::from_str(content).map_err(|e| Error::Config {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
    })?;

    let version = version_check.version.ok_or_else(|| Error::Config {
        message: "missing required field: version".to_string(),
        path: Some(path.to_path_buf()),
    })?;

    if version != SUPPORTED_VERSION {
        return Err(Error::Config {
            message: format!(
                "unsupported config version {} (supported: {})\n  Upgrade quench to use this config.",
                version, SUPPORTED_VERSION
            ),
            path: Some(path.to_path_buf()),
        });
    }

    // Now parse full config - but we need custom handling for unknown keys
    // See Phase 5.4 for unknown key warning logic
    toml::from_str(content).map_err(|e| Error::Config {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
    })
}
```

**Verification**:
```bash
# Valid config
echo 'version = 1' > quench.toml
cargo run -- check

# Missing version
echo '' > quench.toml
cargo run -- check  # Error: missing required field: version

# Wrong version
echo 'version = 2' > quench.toml
cargo run -- check  # Error: unsupported config version 2
```

### Phase 5.4: Unknown Key Warnings

**Goal**: Warn on unknown config keys without failing.

**Tasks**:
1. Implement custom deserialization that captures unknown keys
2. Emit warnings to stderr for unknown fields
3. Continue execution (don't fail on unknown keys)

**Code**:

```rust
// crates/cli/src/config.rs (additions)
use std::collections::BTreeMap;
use tracing::warn;

/// Config with flexible parsing that captures unknown keys
#[derive(Debug, Deserialize)]
pub struct FlexibleConfig {
    pub version: i64,

    #[serde(default)]
    pub project: Option<toml::Value>,

    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}

/// Parse config, warning on unknown keys
pub fn parse_with_warnings(content: &str, path: &Path) -> Result<Config> {
    // First validate version
    let flexible: FlexibleConfig = toml::from_str(content).map_err(|e| Error::Config {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
    })?;

    if flexible.version != SUPPORTED_VERSION {
        return Err(Error::Config {
            message: format!(
                "unsupported config version {} (supported: {})",
                flexible.version, SUPPORTED_VERSION
            ),
            path: Some(path.to_path_buf()),
        });
    }

    // Warn about unknown top-level keys
    for key in flexible.unknown.keys() {
        warn_unknown_key(path, key);
    }

    // Parse strictly now
    toml::from_str(content).map_err(|e| {
        // If strict parse fails, it's likely an unknown nested key
        // Emit as warning and return defaults
        warn!("{}: {}", path.display(), e);
        e
    }).or_else(|_| {
        // Fallback: return minimal valid config
        Ok(Config {
            version: flexible.version,
            project: ProjectConfig::default(),
        })
    })
}

fn warn_unknown_key(path: &Path, key: &str) {
    eprintln!(
        "quench: warning in {}\n  {}: unrecognized field (ignored)",
        path.display(),
        key
    );
}
```

**Verification**:
```bash
# Unknown top-level key
echo -e 'version = 1\nunknown = true' > quench.toml
cargo run -- check  # Warns but succeeds

# Unknown nested key
cat > quench.toml << 'EOF'
version = 1
[check.unknown]
field = "value"
EOF
cargo run -- check  # Warns but succeeds
```

### Phase 5.5: Tracing Setup with QUENCH_LOG

**Goal**: Implement logging per `docs/specs/02-config.md#environment-variables`.

**Tasks**:
1. Initialize tracing-subscriber in main
2. Read log level from `QUENCH_LOG` env var
3. Support levels: off, error, warn, info, debug, trace
4. Output logs to stderr (preserve stdout for output)

**Code**:

```rust
// crates/cli/src/main.rs
use tracing_subscriber::{fmt, EnvFilter};

fn init_logging() {
    let filter = EnvFilter::try_from_env("QUENCH_LOG")
        .unwrap_or_else(|_| EnvFilter::new("off"));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}

fn main() -> anyhow::Result<()> {
    init_logging();

    let cli = Cli::parse();

    // Handle commands...
    match &cli.command {
        None => {
            // Show help for bare invocation
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
        Some(Command::Check(args)) => run_check(&cli, args)?,
        Some(Command::Report(args)) => run_report(&cli, args)?,
        Some(Command::Init(args)) => run_init(&cli, args)?,
    }

    Ok(())
}
```

**Verification**:
```bash
# Default: no logging
cargo run -- check 2>&1 | grep -i debug  # Nothing

# Debug logging
QUENCH_LOG=debug cargo run -- check 2>&1 | grep -i debug  # Shows debug output

# Trace logging
QUENCH_LOG=trace cargo run -- check 2>&1 | grep -i trace  # Shows trace output
```

### Phase 5.6: Wire Commands and Enable Specs

**Goal**: Connect all pieces and enable Phase 003 specs.

**Tasks**:
1. Implement minimal `run_check`, `run_report`, `run_init` functions
2. Add JSON output stub for check command
3. Remove `#[ignore]` from Phase 005 specs
4. Ensure all specs pass

**Code**:

```rust
// crates/cli/src/main.rs (command handlers)
use crate::cli::{Cli, Command, CheckArgs, ReportArgs, InitArgs, OutputFormat};
use crate::config;
use crate::discovery;
use crate::error::ExitCode;
use tracing::debug;

fn run_check(cli: &Cli, args: &CheckArgs) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Resolve config
    let config_path = discovery::resolve_config(cli.config.as_deref(), &cwd)?;

    let config = match &config_path {
        Some(path) => {
            debug!("loading config from {}", path.display());
            config::parse_with_warnings(&std::fs::read_to_string(path)?, path)?
        }
        None => {
            debug!("no config found, using defaults");
            config::Config::default()
        }
    };

    debug!(?config, "loaded config");

    // For now, just output success
    match args.output {
        OutputFormat::Text => {
            // Silent on success per spec
        }
        OutputFormat::Json => {
            println!(r#"{{"passed": true, "violations": []}}"#);
        }
    }

    Ok(())
}

fn run_report(_cli: &Cli, args: &ReportArgs) -> anyhow::Result<()> {
    match args.output {
        OutputFormat::Text => println!("No metrics collected yet."),
        OutputFormat::Json => println!(r#"{{"metrics": {{}}}}"#),
    }
    Ok(())
}

fn run_init(_cli: &Cli, args: &InitArgs) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("quench.toml");

    if config_path.exists() && !args.force {
        eprintln!("quench.toml already exists. Use --force to overwrite.");
        std::process::exit(ExitCode::ConfigError as i32);
    }

    std::fs::write(&config_path, "version = 1\n")?;
    println!("Created quench.toml");
    Ok(())
}
```

**Verification**:
```bash
# Remove #[ignore] attributes from specs
cargo test --test specs  # All 24 tests should pass

# Full CI check
make check
```

## Key Implementation Details

### Exit Code Mapping

Per `docs/specs/01-cli.md#exit-codes`, map errors to exit codes:

```rust
// In main.rs error handling
fn main() {
    if let Err(e) = run() {
        eprintln!("quench: {}", e);
        let code = match e.downcast_ref::<Error>() {
            Some(err) => ExitCode::from(err) as i32,
            None => ExitCode::InternalError as i32,
        };
        std::process::exit(code);
    }
}
```

### Config Discovery Priority

Per `docs/specs/02-config.md#discovery`:

1. CLI flag `-C`/`--config` (highest)
2. Environment `QUENCH_CONFIG`
3. Discovery from current directory up to git root
4. Built-in defaults (lowest)

Clap handles 1 and 2 via `#[arg(env = "QUENCH_CONFIG")]`.

### Unknown Key Detection Strategy

Since `serde(deny_unknown_fields)` causes errors, use a two-pass approach:

1. First pass: Parse with `#[serde(flatten)]` to capture unknowns
2. Emit warnings for any keys in the flattened map
3. Second pass: Parse strict struct (or use captured values)

### QUENCH_LOG Levels

Per `docs/specs/02-config.md#environment-variables`:

| Level | Output |
|-------|--------|
| off | No logging (default) |
| error | Errors only |
| warn | Warnings and errors |
| info | Info, warnings, errors |
| debug | Debug and above |
| trace | Everything |

### File Organization

Follow project convention from `CLAUDE.md`:

```rust
// config.rs
pub struct Config { ... }
pub fn load(path: &Path) -> Result<Config> { ... }

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
```

```rust
// config_tests.rs
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;

#[test]
fn parses_minimal_config() { ... }
```

## Verification Plan

### Phase Completion Checklist

- [ ] `quench` (bare) shows help
- [ ] `quench help` shows help
- [ ] `quench check` works (in directory with quench.toml)
- [ ] `quench report` works
- [ ] `quench init` creates quench.toml
- [ ] `-C/--config` overrides config path
- [ ] `QUENCH_CONFIG` env var works
- [ ] Config discovery walks up to git root
- [ ] `version = 1` required in config
- [ ] Unsupported version produces clear error
- [ ] Unknown config keys produce warnings
- [ ] `QUENCH_LOG` enables tracing
- [ ] Exit code 0 on success
- [ ] Exit code 2 on config/argument errors
- [ ] All Phase 003 specs pass (19 tests un-ignored)

### Test Counts

After Phase 005, `tests/specs.rs` should have:

| Category | Tests | Status |
|----------|-------|--------|
| Commands | 6 | All passing |
| Global flags | 5 | All passing |
| Check flags | 4 | All passing |
| Config warnings | 3 | All passing |
| Env vars | 5 | All passing |
| Output snapshot | 1 | Ignored (Phase 030) |
| **Total** | **24** | **23 passing, 1 ignored** |

### Running Tests

```bash
# Unit tests
cargo test --lib

# Behavioral specs
cargo test --test specs

# All tests
cargo test --all

# Full CI check
make check
```

### Expected `make check` Output

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
   ...
   running 23 tests
   test bare_invocation_shows_help ... ok
   test help_command_shows_help ... ok
   ...
   test result: ok. 23 passed; 0 failed; 1 ignored

cargo build --all
./scripts/bootstrap
cargo audit
cargo deny check
```

## Summary

Phase 005 implements the foundational CLI and configuration system:

1. **CLI skeleton**: Full command structure with clap derive
2. **Global flags**: `-C/--config` with env var support
3. **Config discovery**: Walk up to git root
4. **Config parsing**: Version validation, serde structs
5. **Unknown key warnings**: Forward compatibility
6. **Logging**: QUENCH_LOG-based tracing to stderr

This enables 19 behavioral specs from Phase 003 and provides the foundation for all subsequent feature implementations.
