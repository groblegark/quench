# Coverage Specification

Coverage is a language adapter feature that measures test coverage.

## Purpose

Track code coverage to identify untested paths:
- Line coverage percentage
- Aggregate coverage from multiple test suites
- CI reporting with optional threshold enforcement

**Slow check**: Only runs in `--ci` mode unless threshold is configured.

## Rust Adapter

Uses `cargo llvm-cov` by default.

### Default Behavior

```bash
cargo llvm-cov --json
```

Output:
```
rust: coverage
  line: 78.4%
  function: 72.1%
```

### Multiple Test Suites

Coverage aggregates from all configured test suites. Uses same `test_suites` config as test time measurement (see `04-test-runners.md`).

```toml
[checks.rust]
coverage = true
test_time = true

# Test suites (shared by coverage and test time)
[[checks.rust.test_suites]]
runner = "cargo"             # Default, can omit

[[checks.rust.test_suites]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"        # Build binary before running

[[checks.rust.test_suites]]
runner = "pytest"
path = "tests/integration/"
setup = "cargo build"
```

All suites contribute to both coverage (if instrumented) and test time metrics.

### Configuration

```toml
[checks.rust]
coverage = true              # CI mode: report coverage

# Root threshold (aggregate across all packages)
coverage_min = 75            # Fail if total < 75%

# Per-package thresholds (override root)
[checks.rust.coverage.package.core]
min = 90                     # Core library needs higher coverage

[checks.rust.coverage.package.cli]
min = 60                     # CLI can have lower coverage
exclude_files = ["src/main.rs"]  # Exclude entry point

[checks.rust.coverage.package.experimental]
enforce = false              # Reported but not enforced (under development)

[checks.rust.coverage.package.generated]
enabled = false              # Not reported, not enforced (ignore entirely)
```

### Output

CI mode (reporting):
```
rust: coverage 78.4%
  core: 82.3%
  cli: 68.9%
```

With threshold (fails if below):
```
rust: FAIL
  coverage: 78.4% (min: 75%) - PASS
    core: 82.3% (min: 90%) - FAIL
    cli: 68.9% (min: 60%) - PASS
```

### JSON Output

```json
{
  "adapter": "rust",
  "coverage": {
    "line_percent": 78.4,
    "line_covered": 1245,
    "line_total": 1588,
    "by_package": {
      "core": { "line_percent": 82.3, "min": 90, "passed": false },
      "cli": { "line_percent": 68.9, "min": 60, "passed": true }
    }
  }
}
```

## Shell Adapter

Shell coverage via `kcov` or similar (optional, not default).

```toml
[checks.shell]
coverage = true
coverage_tool = "kcov"       # kcov, bashcov, or shcov
coverage_min = 70
```

## Notes

- Coverage is **reporting only** unless `coverage_min` threshold is set
- Use baseline comparison + ratcheting for gradual improvement
- Multiple test commands share coverage data via LLVM profile merging
- Per-package breakdown uses Cargo workspace structure
