# Configuration Specification

Quench uses convention over configuration with a single optional `quench.toml` at project root.

## Single Config File

All configuration lives in one place: `quench.toml` at project root.

**No scattered config files.** Per-package and per-module behavior is configured
inline in the root config, not via separate files in subdirectories.

```
project-root/
├── quench.toml              # Single config file (optional)
├── crates/
│   ├── cli/                 # No quench.toml here
│   └── core/                # No quench.toml here
```

## Discovery

1. CLI flags (highest priority)
2. `quench.toml` in current directory or nearest parent (up to git root)
3. Built-in defaults (lowest priority)

## Config File Schema

### Minimal Config

Most projects need no config. Missing `quench.toml` uses all defaults.

### Full Schema

```toml
# Project identification (optional, inferred from directory name)
[project]
name = "my-project"

# Subprojects for per-package metrics
# Auto-detected for cargo workspaces, but can be explicit
[[project.subprojects]]
name = "cli"
path = "crates/cli/src"
tests = "crates/cli/tests"

[[project.subprojects]]
name = "core"
path = "crates/core/src"

# General settings
[general]
# Patterns for source files (language-specific defaults apply)
source_patterns = ["**/*.rs", "**/*.sh"]
# Patterns for test files
test_patterns = ["**/*_test.rs", "**/*_tests.rs", "**/*.spec.rs", "**/tests/**", "**/test/**"]
# Patterns to always ignore
ignore = ["target/", "node_modules/", "dist/", ".git/"]

# Check-specific configuration
[checks.loc]
enabled = true
test_ratio_min = 0.5
test_ratio_max = 4.0

[checks.file-size]
enabled = true
source_avg = 300
source_max = 700
test_avg = 500
test_max = 1100
skip = ["**/generated/**", "**/bindgen.rs"]

[checks.escapes]
enabled = true

[[checks.escapes.patterns]]
name = "unsafe"
pattern = "unsafe\\s*\\{"
mode = "require_comment"
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
threshold = 10          # default: 0

[checks.agent]
enabled = true
# See below for root/package/module configuration

[checks.test-correlation]
enabled = true
mode = "smart"

# Adapter settings
[adapters.rust]
enabled = true
parse_cfg_test = true

[adapters.shell]
enabled = true
severity = "warning"
forbid_inline_disables = true

# Comparison settings
[compare]
ratchet_escapes = true
ratchet_coverage = true
coverage_variance = 1.0
```

## Per-Package Overrides (Inline)

Override settings for specific packages without separate config files:

```toml
# Default file-size limits
[checks.file-size]
source_max = 700

# Override for specific package
[checks.file-size.overrides.cli]
source_max = 500        # Stricter for CLI

[checks.file-size.overrides.generated]
enabled = false         # Skip entirely for generated code
```

The override key matches subproject name or a glob pattern:

```toml
# By exact subproject name
[checks.escapes.overrides.core]
# ... core-specific settings

# By glob pattern
[checks.escapes.overrides."crates/*/generated"]
enabled = false
```

## Root/Package/Module Configuration

For checks that behave differently at different scopes (like `agent`),
configure all scopes in the root config:

```toml
[checks.agent]
enabled = true

# Root (project root) settings
[checks.agent.root]
required = true
require_sections = ["Project Structure", "Development", "Landing the Plane"]
max_lines = 500
max_tokens = 2000
forbid_tables = true
allow_box_diagrams = true
sync_cursorrules = true

# Package (subproject/crate) settings - applies to each subproject root
[checks.agent.package]
required = false                    # Optional at package level
require_sections = []               # No required sections
max_lines = 200
max_tokens = 800

# Module (subdirectory) settings
[checks.agent.module]
required = false                    # Optional at module level
max_lines = 100
max_tokens = 400
```

## Language-Specific Package Detection

Adapters can auto-detect packages:

### Rust (Cargo Workspaces)

If `Cargo.toml` has `[workspace]`, quench reads `members` and discovers packages:

```toml
# Auto-generated equivalent:
[[project.subprojects]]
name = "cli"              # from [package] name in crates/cli/Cargo.toml
path = "crates/cli/src"
tests = "crates/cli/tests"
```

### Shell (Convention)

Shell projects typically use:
- `bin/` or `scripts/` for source
- `tests/` for BATS tests

```toml
# Default for shell projects:
[[project.subprojects]]
name = "scripts"
path = "bin"
tests = "tests"
```

## Smart Defaults

### Language Detection

Quench auto-detects project language(s):
- `Cargo.toml` → Rust project
- `*.sh` in root or `bin/` → Shell scripts
- `package.json` → JavaScript/TypeScript (future)

### Source/Test Patterns by Language

**Rust** (when detected):
```toml
source_patterns = ["**/*.rs"]
test_patterns = ["**/*_test.rs", "**/*_tests.rs", "**/tests/**/*.rs"]
ignore = ["target/"]
```

**Shell** (when detected):
```toml
source_patterns = ["**/*.sh", "**/*.bash"]
test_patterns = ["**/tests/**/*.bats", "**/test/**/*.bats"]
```

**Generic fallback**:
```toml
source_patterns = ["src/**/*", "lib/**/*"]
test_patterns = ["test/**/*", "tests/**/*", "**/*_test.*", "**/*.test.*", "**/*.spec.*"]
```

## CLI Overrides

All config can be overridden via CLI:

```bash
# Select specific checks
quench --check loc --check escapes

# Skip checks
quench --skip test-correlation

# Override thresholds
quench --file-size-max 1000

# Target specific subproject
quench --package cli
```

## Environment Variables

```bash
# Disable color
QUENCH_NO_COLOR=1

# Config file location
QUENCH_CONFIG=/path/to/quench.toml

# Verbose mode
QUENCH_VERBOSE=1
```

## Config Validation

Invalid config produces clear errors:

```
quench: error in quench.toml
  checks.escapes.patterns[0].mode: invalid value "warn"
    expected one of: count, require_comment, forbid
```

Unknown keys are warnings (forward compatibility):

```
quench: warning in quench.toml
  checks.escapes.unknown_key: unrecognized field (ignored)
```
