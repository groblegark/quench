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
version = 1    # Config format version (required)
[project]      # Project identity and file patterns
[git]          # Git integration settings
[rust]         # Rust language config (optional, has defaults)
[shell]        # Shell language config (optional, has defaults)
[checks.*]     # Check-specific configuration
[ratchet]      # Regression prevention
```

## Minimal Config

Most projects need no config. Missing `quench.toml` uses all defaults.

When a config file exists, `version` is required:

```toml
version = 1
```

## Version

The `version` field is a required integer at the top of the config file.

- Bumped only on breaking changes (renamed/removed fields, changed semantics)
- Additions don't bump the version
- Quench validates version before parsing:
  ```
  quench: unsupported config version 2 (supported: 1)
    Upgrade quench to use this config.
  ```

Current version: **1**

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
base = "main"                          # Default for --base (auto: main > master > develop)
baseline = ".quench/baseline.json"     # Metrics storage path
```

### [rust]

Rust language configuration. Auto-detected when `Cargo.toml` exists.

```toml
[rust]
# Source/test patterns (defaults shown, override if needed)
# source = ["**/*.rs"]
# tests = ["tests/**", "test/**/*.rs", "*_test.rs", "*_tests.rs"]
# ignore = ["target/"]

split_cfg_test = true                  # Count #[cfg(test)] as test LOC

# Build targets for coverage + binary size (default: all [[bin]] entries)
# targets = ["myapp", "myserver"]

# Build metrics (CI mode)
binary_size = true
compile_time = true

# Thresholds (optional)
binary_size_max = "5 MB"
compile_time_cold_max = "60s"
compile_time_hot_max = "2s"

# Lint suppression (#[allow(...)])
[rust.suppress]
check = "comment"                      # forbid | comment | allow
# comment = "// JUSTIFIED:"            # optional: require specific pattern

[rust.suppress.test]
check = "allow"                        # tests can suppress freely

# Policy
[rust.policy]
lint_changes = "standalone"
lint_config = ["rustfmt.toml", "clippy.toml"]
```

### [shell]

Shell language configuration. Auto-detected when `*.sh` files exist in root, `bin/`, or `scripts/`.

```toml
[shell]
# Source/test patterns (defaults shown)
# source = ["**/*.sh", "**/*.bash"]
# tests = ["tests/**/*.bats", "test/**/*.bats", "*_test.sh"]

# Suppress (# shellcheck disable=)
[shell.suppress]
check = "forbid"                       # forbid | comment | allow

[shell.suppress.test]
check = "allow"

[shell.policy]
lint_changes = "standalone"
lint_config = [".shellcheckrc"]
```

### [checks.*]

Each check has its own section. Common fields:

| Field | Type | Description |
|-------|------|-------------|
| `check` | string | `"error"` \| `"warn"` \| `"off"` (default: `"error"`, except license) |
| `exclude` | [string] | Patterns to skip |

#### [checks.cloc]

Lines of code and file size limits.

```toml
[checks.cloc]
check = "error"                        # error | warn | off
max_lines = 750                        # Source file limit
max_lines_test = 1100                  # Test file limit
max_tokens = 20000                     # Use false to disable
exclude = ["**/generated/**"]

# Per-package overrides
[checks.cloc.package.cli]
max_lines = 500                        # Stricter for CLI

[checks.cloc.package.generated]
check = "off"                          # Skip entirely
```

#### [checks.escapes]

Escape hatch detection with configurable patterns.

```toml
[checks.escapes]
check = "error"                        # error | warn | off

[[checks.escapes.patterns]]
name = "unsafe"
pattern = "unsafe\\s*\\{"
action = "comment"             # count | comment | forbid
comment = "// SAFETY:"
advice = "Add a // SAFETY: comment explaining the invariants."

[[checks.escapes.patterns]]
name = "unwrap"
pattern = "\\.unwrap\\(\\)"
action = "forbid"

[[checks.escapes.patterns]]
name = "todo"
pattern = "TODO|FIXME|XXX"
action = "count"
threshold = 10
```

#### [checks.agents]

Agent file validation (CLAUDE.md, .cursorrules). Supports scope hierarchy.

```toml
[checks.agents]
check = "error"                        # error | warn | off
files = ["CLAUDE.md", ".cursorrules"]
sync = true
sync_source = "CLAUDE.md"

# Root scope (project root)
[checks.agents.root]
required = ["CLAUDE.md"]
optional = [".cursorrules"]
sections.required = ["Project Structure", "Development"]
max_lines = 500
max_tokens = 20000                     # Use false to disable
tables = "forbid"

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

TOC validation, link validation, spec files, and commit checking.

```toml
[checks.docs]
check = "error"                            # error | warn | off

# TOC validation (directory trees in markdown)
[checks.docs.toc]
check = "error"                            # error | warn | off
# include = ["**/*.md", "**/*.mdc"]        # optional, defaults shown
exclude = ["plans/**", "plan.md", "*_plan.md", "plan_*"]

# Link validation (markdown links)
[checks.docs.links]
check = "error"                            # error | warn | off
# include = ["**/*.md", "**/*.mdc"]        # optional
exclude = ["plans/**"]

# Specs validation
[checks.docs.specs]
check = "error"                            # error | warn | off
path = "docs/specs"
# extension = ".md"                        # optional
# index_file = "docs/specs/CLAUDE.md"      # optional, auto-detected
index = "auto"                             # auto | toc | linked | exists

# Commit checking (CI mode only)
[checks.docs.commit]
check = "off"                              # error | warn | off (default: off)
on_commit = ["feat:", "feat(", "story:", "story("]

# Area mappings (reusable across features)
[checks.docs.areas.api]
docs = "docs/api/**"
source = "src/api/**"
```

#### [checks.tests]

Test correlation, execution, and metrics.

```toml
[checks.tests]
check = "error"                        # error | warn | off

# Commit checking (source changes need test changes)
[checks.tests.commit]
check = "error"                        # error | warn | off
scope = "branch"                       # branch | commit
placeholders = "allow"
exclude = ["**/mod.rs", "**/main.rs"]

# Test suites (time limits per-suite)
[[checks.tests.suites]]
runner = "cargo"
# covers Rust automatically via llvm-cov
max_total = "30s"
max_test = "1s"

[[checks.tests.suites]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
targets = ["myapp"]                     # instrument Rust binary
max_total = "10s"
max_test = "500ms"

[[checks.tests.suites]]
runner = "pytest"
path = "tests/integration/"
ci = true                              # only run in CI mode (slow)
targets = ["myserver"]                  # also instrument Rust binary
max_total = "60s"

[[checks.tests.suites]]
runner = "bats"
path = "tests/scripts/"
targets = ["scripts/*.sh"]              # shell scripts via kcov

# Coverage settings
[checks.tests.coverage]
check = "error"                        # error | warn | off
min = 75                               # minimum coverage %

# Per-package coverage thresholds
[checks.tests.coverage.package.core]
min = 90

[checks.tests.coverage.package.cli]
min = 60
exclude = ["src/main.rs"]

# Test time check level (thresholds are per-suite)
[checks.tests.time]
check = "warn"                         # error | warn | off
```

#### [checks.license]

License header validation (CI only, disabled by default).

```toml
[checks.license]
check = "off"                          # error | warn | off (default: off)
license = "MIT"
copyright = "Your Organization"
exclude = ["**/generated/**"]
```

### [ratchet]

Prevent quality regressions.

```toml
[ratchet]
check = "error"                        # error | warn | off

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
    expected one of: count, comment, forbid
```

Unknown keys are warnings (forward compatibility):

```
quench: warning in quench.toml
  checks.unknown: unrecognized field (ignored)
```
