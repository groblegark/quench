# Tests Configuration Guide

Configuration reference for the `tests` check.

## Commit Checking

```toml
[check.tests.commit]
check = "error"
# Scope of checking:
# "branch" - all changes on branch together (default)
# "commit" - per-commit with asymmetric rules (tests-first OK)
scope = "branch"
# Whether placeholder tests (#[ignore], test.todo()) count
placeholders = "allow"  # "allow" | "forbid" (default: allow)
```

## Commit Types

```toml
[check.tests.commit]
check = "error"
# Only these commit types require test changes (default shown)
types = ["feat", "feature", "story", "breaking"]
```

## Exclude from Commit Checking

```toml
[check.tests.commit]
check = "error"
scope = "branch"
# Never require tests for these files
exclude = ["**/mod.rs", "**/main.rs", "**/generated/**"]
```

## Custom Test Patterns

```toml
[check.tests]
check = "error"
# Patterns to identify test files
test_patterns = [
  "tests/**/*",
  "test/**/*",
  "**/*_test.rs",
  "**/*_tests.rs",
  "**/*.spec.ts",
]
source_patterns = ["src/**/*.rs"]
```

## Single Test Suite

```toml
[[check.tests.suite]]
runner = "cargo"
max_total = "30s"  # Maximum total suite time
max_test = "1s"    # Maximum per-test time
```

## Multiple Test Suites

```toml
# Unit tests
[[check.tests.suite]]
runner = "cargo"
max_total = "30s"
max_test = "1s"

# CLI integration tests
[[check.tests.suite]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
targets = ["myapp"]  # Instrument Rust binary
max_total = "10s"
max_test = "500ms"
```

## CI-Only Suites

```toml
# Fast unit tests
[[check.tests.suite]]
runner = "cargo"
max_total = "30s"

# Slow integration tests (CI only)
[[check.tests.suite]]
runner = "pytest"
path = "tests/integration/"
ci = true          # Only run in --ci mode
targets = ["myserver"]
max_total = "60s"
```

## Shell Script Coverage

```toml
[[check.tests.suite]]
runner = "bats"
path = "tests/"
# Instrument shell scripts via kcov
targets = ["scripts/*.sh", "bin/*"]
```

## Coverage Thresholds

```toml
[check.tests.coverage]
check = "error"
min = 75  # Minimum coverage percentage
```

## Per-Package Coverage

```toml
[check.tests.coverage]
check = "error"
min = 75  # Default for all packages

[check.tests.coverage.package.core]
min = 90  # Stricter for core

[check.tests.coverage.package.cli]
min = 60                   # More lenient for CLI
exclude = ["src/main.rs"]  # Skip entry points
```

## Test Time Check

```toml
[check.tests.time]
# How to handle test time violations:
# "error" - fail if thresholds exceeded
# "warn" - report but don't fail (default)
# "off" - don't check
check = "warn"
```

## Complete Example

```toml
[check.tests]
check = "error"

[check.tests.commit]
check = "error"
types = ["feat", "feature", "story", "breaking"]
scope = "branch"
placeholders = "allow"
exclude = ["**/mod.rs", "**/main.rs"]

test_patterns = ["tests/**/*", "**/*_test.rs"]
source_patterns = ["src/**/*.rs"]

[[check.tests.suite]]
runner = "cargo"
max_total = "30s"
max_test = "1s"

[[check.tests.suite]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
targets = ["myapp"]
max_total = "10s"
max_test = "500ms"

[[check.tests.suite]]
runner = "pytest"
path = "tests/integration/"
ci = true
targets = ["myserver"]
max_total = "60s"

[check.tests.coverage]
check = "error"
min = 75

[check.tests.coverage.package.core]
min = 90

[check.tests.time]
check = "warn"
```
