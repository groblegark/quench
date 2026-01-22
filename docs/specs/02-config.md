# Configuration Specification

Quench uses convention over configuration with a single optional `quench.toml` at project root.

## File Location

```
project-root/
├── quench.toml              # Single config file (optional)
├── .quench/
│   └── baseline.json        # Metrics storage
├── crates/
│   ├── cli/                 # No config here
│   └── core/                # No config here
```

## Discovery

1. CLI flags (highest priority)
2. `quench.toml` in current directory or nearest parent (up to git root)
3. Built-in defaults (lowest priority)

## Config Sections

```toml
[project]      # Project identity and file patterns
[git]          # Git integration settings
[checks.*]     # Check-specific configuration
[ratchet]      # Regression prevention
```

## Minimal Config

Most projects need no config. Missing `quench.toml` uses all defaults.

## Full Schema

### [project]

Project identity and file patterns.

```toml
[project]
name = "my-project"                    # Optional, inferred from directory

# File patterns (language-specific defaults apply)
source = ["**/*.rs", "**/*.sh"]
tests = ["**/tests/**", "**/*_test.*", "**/*.spec.*"]
ignore = ["target/", "node_modules/", "dist/", ".git/"]

# Packages (auto-detected for cargo workspaces)
[[project.packages]]
name = "cli"
path = "crates/cli"

[[project.packages]]
name = "core"
path = "crates/core"
```

### [git]

Git integration settings.

```toml
[git]
branch = "main"                        # Default for --branch (auto: main > master > develop)
baseline = ".quench/baseline.json"     # Metrics storage path
```

### [checks.*]

Each check has its own section. Common fields:

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | Enable/disable check (default: true, except license) |
| `exclude` | [string] | Patterns to skip |

#### [checks.cloc]

Lines of code and file size limits.

```toml
[checks.cloc]
enabled = true
max_lines = 750                        # Source file limit
max_lines_test = 1100                  # Test file limit
exclude = ["**/generated/**"]

# Per-package overrides
[checks.cloc.package.cli]
max_lines = 500                        # Stricter for CLI

[checks.cloc.package.generated]
enabled = false                        # Skip entirely
```

#### [checks.escapes]

Escape hatch detection with configurable patterns.

```toml
[checks.escapes]
enabled = true

[[checks.escapes.patterns]]
name = "unsafe"
pattern = "unsafe\\s*\\{"
mode = "require_comment"               # count | require_comment | forbid
comment = "// SAFETY:"
advice = "Add a // SAFETY: comment explaining the invariants."

[[checks.escapes.patterns]]
name = "unwrap"
pattern = "\\.unwrap\\(\\)"
mode = "forbid"

[[checks.escapes.patterns]]
name = "todo"
pattern = "TODO|FIXME|XXX"
mode = "count"
threshold = 10
```

#### [checks.agents]

Agent file validation (CLAUDE.md, .cursorrules). Supports scope hierarchy.

```toml
[checks.agents]
enabled = true
files = ["CLAUDE.md", ".cursorrules"]
sync = true
sync_source = "CLAUDE.md"

# Root scope (project root)
[checks.agents.root]
required = ["CLAUDE.md"]
optional = [".cursorrules"]
sections.required = ["Project Structure", "Development"]
max_lines = 500
max_tokens = 2000
allow_tables = false

# Package scope (each package directory)
[checks.agents.package]
required = []
optional = ["CLAUDE.md"]
max_lines = 200
max_tokens = 800

# Module scope (subdirectories)
[checks.agents.module]
required = []
max_lines = 100
max_tokens = 400
```

#### [checks.docs]

Spec file validation and doc correlation.

```toml
[checks.docs]
enabled = true
path = "docs/specs"
require_index = true

# Section validation
sections.required = ["Purpose"]
sections.forbid = ["TODO"]

# Correlation (CI mode only)
correlation = false                    # Enable doc correlation
correlation_mode = "require"           # require | advisory
triggers = ["feat:", "feat("]

[checks.docs.areas]
api = "docs/api/**"
default = ["README.md", "docs/**"]
```

#### [checks.tests]

Test correlation.

```toml
[checks.tests]
enabled = true
mode = "require"                       # require | advisory
scope = "branch"                       # branch | commit
allow_placeholders = true
exclude = ["**/mod.rs", "**/main.rs"]
```

#### [checks.license]

License header validation (CI only, disabled by default).

```toml
[checks.license]
enabled = false
license = "MIT"
copyright = "Your Organization"
exclude = ["**/generated/**"]
```

#### [checks.rust]

Rust language adapter settings.

```toml
[checks.rust]
enabled = true
parse_cfg_test = true                  # Count #[cfg(test)] as test LOC

# CI mode metrics
binary_size = true
compile_time = true
test_time = true

# Optional thresholds (enables enforcement outside CI)
binary_size_max = "5 MB"
compile_time_cold_max = "60s"
test_time_max = "1s"

# Test suites
[[checks.rust.test_suites]]
runner = "cargo"

[[checks.rust.test_suites]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
```

#### [checks.shell]

Shell language adapter settings.

```toml
[checks.shell]
enabled = true
forbid_inline_disables = true          # Forbid # shellcheck disable=
```

### [ratchet]

Prevent quality regressions.

```toml
[ratchet]
enabled = true

# Metrics to ratchet (defaults shown)
coverage = true                        # Coverage can't drop
escapes = true                         # Escape counts can't increase
binary_size = false                    # Binary size can't grow
compile_time_cold = false
compile_time_hot = false
test_time_total = false
test_time_avg = false
test_time_max = false

# Tolerances
coverage_tolerance = 0.5               # Allow 0.5% drop

# Per-package
[ratchet.package.core]
coverage = true

[ratchet.package.experimental]
coverage = false                       # Don't ratchet experimental
```

## Language Detection

Quench auto-detects project languages:

| Detection | Language | Default Patterns |
|-----------|----------|------------------|
| `Cargo.toml` | Rust | `**/*.rs`, ignore `target/` |
| `*.sh` in root/bin/ | Shell | `**/*.sh`, `**/*.bash` |
| `package.json` | JS/TS | Future |

## Environment Variables

```bash
QUENCH_NO_COLOR=1              # Disable color
QUENCH_CONFIG=/path/to.toml    # Config file location
```

## Validation

Invalid config produces clear errors:

```
quench: error in quench.toml
  checks.escapes.patterns[0].mode: invalid value "warn"
    expected one of: count, require_comment, forbid
```

Unknown keys are warnings (forward compatibility):

```
quench: warning in quench.toml
  checks.unknown: unrecognized field (ignored)
```
