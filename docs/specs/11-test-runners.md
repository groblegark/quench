# Test Runners Specification

Test runners execute tests and report timing and coverage information.

Test suites are configured via `[[check.tests.suite]]` and provide:
- **Test time**: Timing metrics (total, avg, max)
- **Coverage**: Code coverage collection (via `targets` field)

## Auto-Discovery

When no `[[check.tests.suite]]` is configured and `auto = true`, quench auto-discovers test runners in CI mode by detecting project files (e.g., `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`).

```toml
[check.tests]
auto = true  # enable auto-discovery (default: false)
```

Auto-discovery is off by default. Users must opt in explicitly.

## Runner Independence

Runners are independent of the code being tested. Any runner can test any project:

- A Rust CLI can have `bats` tests for end-to-end behavior
- A Go service can have `pytest` integration tests
- A shell script project can have `cargo` tests for a Rust helper binary

The runner determines how tests are executed and how output is parsed. Coverage depends on what code the tests exercise (see [Coverage Targets](#coverage-targets)).

## Supported Runners

| Runner | Per-Test Timing | Implicit Coverage |
|--------|-----------------|-------------------|
| `cargo` | Yes | Rust (llvm-cov) |
| `go` | Yes | Go (built-in) |
| `pytest` | Yes | Python (coverage.py) |
| `unittest` | Yes | Python (coverage.py) |
| `vitest` | Yes | JS/TS (built-in) |
| `bun` | Yes | JS/TS (built-in) |
| `jest` | Yes | JS/TS (built-in) |
| `rspec` | Yes | Ruby (SimpleCov) |
| `minitest` | Yes | Ruby (SimpleCov) |
| `bats` | Yes | Via `targets` (kcov, llvm-cov) |
| `cucumber` | No | Via `targets` (instrumented) |
| `custom` | No | None |

## Suite Configuration

```toml
[[check.tests.suite]]
runner = "cargo"
# Implicit: targets Rust code via llvm-cov
# Runs in fast mode and CI mode

[[check.tests.suite]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
targets = ["myapp"]                     # Instrument Rust binary
max_total = "10s"
max_test = "500ms"

[[check.tests.suite]]
runner = "pytest"
path = "tests/integration/"
ci = true                              # Only run in CI mode (slow)
targets = ["myserver"]                  # Also instrument Rust binary
max_total = "60s"

[[check.tests.suite]]
runner = "bats"
path = "tests/scripts/"
targets = ["scripts/*.sh"]              # Shell scripts via kcov
```

### Suite Fields

| Field | Type | Description |
|-------|------|-------------|
| `runner` | string | Runner to use (required) |
| `path` | string | Test directory or file pattern |
| `setup` | string | Command to run before tests |
| `targets` | [string] | Coverage targets (see below) |
| `ci` | bool | Only run in CI mode (default: false) |
| `max_total` | duration | Max total time for this suite |
| `max_avg` | duration | Max average time per test |
| `max_test` | duration | Max time for slowest individual test |

### Custom Commands

For unsupported runners, use custom command:

```toml
[[check.tests.suite]]
name = "custom"
command = "./scripts/run-tests.sh"
# No per-test timing available for custom commands
```

## Coverage Targets

The `targets` field specifies what code a test suite exercises for coverage.

### Implicit Coverage

Runners that test their own language provide implicit coverage:

| Runner | Covers | Tool |
|--------|--------|------|
| `cargo` | Rust | llvm-cov |
| `go` | Go | built-in |
| `pytest` | Python | coverage.py |
| `vitest`/`jest`/`bun` | JS/TS | built-in |

These don't need a `targets` field—coverage just works.

### Explicit Coverage

For integration tests exercising compiled binaries or shell scripts:

```toml
[[check.tests.suite]]
runner = "bats"
path = "tests/cli/"
targets = ["myapp"]                     # Build target name → Rust binary

[[check.tests.suite]]
runner = "pytest"
path = "tests/e2e/"
targets = ["myserver", "scripts/*.sh"]  # Rust binary + shell scripts
```

Coverage targets are resolved:
1. **Build target name** (e.g., `myapp`) → Matches `[rust].targets` → llvm-cov
2. **Glob pattern** (e.g., `scripts/*.sh`) → Matches `[shell].source` → kcov

### No Coverage

For suites that only contribute timing:

```toml
[[check.tests.suite]]
runner = "bats"
path = "tests/smoke/"
targets = []                            # Explicit: timing only
```

Or simply omit the `targets` field.

## Timing Metrics

### Total Time

Wall-clock time for entire test suite. Available for all runners.

### Per-Test Timing

Requires runner support for parsing individual test results.

**Average**: Mean time per test
**Max**: Slowest individual test (with name)

```
tests: time
  total: 12.4s
  avg: 45ms (276 tests)
  max: 2.1s (tests::integration::large_file_parse)
```

## Runner Details

### cargo

```bash
cargo test --all
```

Parses cargo test output for per-test results. Coverage via `cargo llvm-cov`.

### bats

```bash
bats --timing tests/
```

Parses BATS TAP output with timing information.

### pytest

```bash
pytest --durations=0 -v tests/
```

Parses pytest duration report. Coverage via `coverage.py`.

### vitest

```bash
vitest run --reporter=json
```

Parses Vitest JSON reporter output. Built-in coverage support.

### bun

```bash
bun test --reporter=json
```

Parses Bun's JSON test output. Built-in coverage support.

### jest

```bash
jest --json
```

Parses Jest JSON output. Built-in coverage support.

### go

```bash
go test -json ./...
```

Parses Go's JSON test output. Built-in coverage support.

### unittest

```bash
python -m unittest discover
```

Parses Python's unittest output for per-test results. Coverage via `coverage.py`.

### rspec

```bash
rspec --format json
```

Parses RSpec's JSON reporter output for Ruby tests. Coverage via `SimpleCov`.

### minitest

```bash
ruby -Ilib:test -e "require 'minitest/autorun'"
```

Parses Minitest output for Ruby tests. Coverage via `SimpleCov`.

### cucumber

```bash
cucumber --format=json
```

Parses Cucumber JSON output. Provides total duration only (no per-test timing). Coverage via instrumentation for specified targets.

### custom

```toml
[[check.tests.suite]]
name = "custom"
command = "./scripts/run-tests.sh"
```

Executes user-defined shell command. Reports exit code and total time only. No per-test timing or coverage collection.

## Aggregation

When multiple test suites are configured, metrics are aggregated:

- **Total**: Sum of all suite times
- **Average**: Weighted by test count
- **Max**: Slowest test across all suites
- **Coverage**: Merged across suites covering the same language

```
tests: time
  total: 18.6s
  avg: 52ms (358 tests)
  max: 2.1s (tests::integration::large_file_parse)
  suites:
    cargo: 12.4s (276 tests)
    bats: 4.2s (45 tests)
    pytest: 2.0s (37 tests)

tests: coverage 78.4%
  rust: 82.3% (cargo + bats)
  python: 71.2% (pytest)
```

## Thresholds

Time limits are configured per-suite:

```toml
[[check.tests.suite]]
runner = "cargo"
max_total = "30s"
max_avg = "50ms"
max_test = "1s"

[[check.tests.suite]]
runner = "bats"
path = "tests/cli/"
max_total = "10s"
max_test = "500ms"

[[check.tests.suite]]
runner = "pytest"
path = "tests/integration/"
ci = true                              # Slow suite, CI only
max_total = "120s"
max_test = "5s"
```

Configure check level via `[check.tests.time]`:

```toml
[check.tests.time]
check = "warn"                         # error | warn | off
```
