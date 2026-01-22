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
base = "main"                          # Default for --base (auto: main > master > develop)
baseline = ".quench/baseline.json"     # Metrics storage path
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

Test correlation and metrics.

```toml
[checks.tests]
check = "error"                        # error | warn | off

# Commit checking (source changes need test changes)
[checks.tests.commit]
check = "error"                        # error | warn | off
scope = "branch"                       # branch | commit
placeholders = "allow"
exclude = ["**/mod.rs", "**/main.rs"]
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

#### [checks.rust]

Rust language adapter settings.

```toml
[checks.rust]
check = "error"                        # error | warn | off
split_cfg_test = true                  # Count #[cfg(test)] as test LOC

# Lint suppression (#[allow(...)])
[checks.rust.suppress]
check = "comment"                      # forbid | comment | allow
# comment = "// JUSTIFIED:"            # optional: require specific pattern (default: any)
# allow = ["dead_code"]                # lints that don't need comment
# forbid = ["unsafe_code"]             # lints never allowed

[checks.rust.suppress.test]
check = "allow"                        # tests can suppress freely

# Policy
[checks.rust.policy]
lint_changes = "standalone"            # lint config changes must be standalone
lint_config = ["rustfmt.toml", "clippy.toml"]

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
check = "error"                        # error | warn | off

# Lint suppression (# shellcheck disable=)
[checks.shell.suppress]
check = "forbid"                       # forbid | comment | allow
# allow = ["SC2034"]                   # codes that don't need comment
# forbid = ["SC1090"]                  # codes never allowed

[checks.shell.suppress.test]
check = "allow"                        # tests can suppress freely

# Policy
[checks.shell.policy]
lint_changes = "standalone"
lint_config = [".shellcheckrc"]
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
