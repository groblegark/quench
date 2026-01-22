# Test Runners Specification

Test runners execute tests and report timing and coverage information.

Test suites configured via `[[checks.<lang>.test_suites]]` are used for both:
- **Test time**: Timing metrics (total, avg, max)
- **Coverage**: Code coverage collection (if enabled)

## Runner Independence

Runners are independent of the code being tested. Any runner can test any project:

- A Rust CLI can have `bats` tests for end-to-end behavior
- A Go service can have `pytest` integration tests
- A shell script project can have `cargo` tests for a Rust helper binary

The adapter (rust, shell, etc.) determines defaults and language-specific features. The runner determines how tests are executed and parsed.

## Supported Runners

| Runner | Per-Test Timing | Auto-Detection |
|--------|-----------------|----------------|
| `cargo` | Yes | `Cargo.toml` |
| `bats` | Yes | `*.bats` files |
| `pytest` | Yes | `pytest.ini`, `pyproject.toml` |
| `vitest` | Yes | `vitest.config.*` |
| `bun` | Yes | `bun.lockb` |
| `jest` | Yes | `jest.config.*` |
| `go` | Yes | `go.mod` |

## Timing Metrics

### Total Time

Wall-clock time for entire test suite. Available for all runners.

### Per-Test Timing

Requires runner support for parsing individual test results.

**Average**: Mean time per test
**Max**: Slowest individual test (with name)

```
rust: test time
  total: 12.4s
  avg: 45ms (276 tests)
  max: 2.1s (tests::integration::large_file_parse)
```

## Runner Configuration

### Auto-Detection

Runners are auto-detected based on project files. Default runner per adapter:

| Adapter | Default Runner |
|---------|----------------|
| `rust` | `cargo` |
| `shell` | `bats` (if `*.bats` exist) |
| `generic` | None |

### Additional Suites

Add test suites beyond the default:

```toml
# Rust project with additional test suites
[[checks.rust.test_suites]]
runner = "bats"
path = "tests/cli/"

[[checks.rust.test_suites]]
runner = "pytest"
path = "tests/integration/"
setup = "cargo build"        # Build binary before running
```

### Custom Commands

For unsupported runners, use custom command:

```toml
[[checks.rust.test_suites]]
name = "custom"
command = "./scripts/run-tests.sh"
# No per-test timing available for custom commands
```

## Runner Details

### cargo

```bash
cargo test --release -- --format json
```

Parses Rust's JSON test output for per-test timing.

### bats

```bash
bats --timing tests/
```

Parses BATS TAP output with timing information.

### pytest

```bash
pytest --durations=0 -v tests/
```

Parses pytest duration report.

### vitest

```bash
vitest run --reporter=json
```

Parses Vitest JSON reporter output.

### bun

```bash
bun test --reporter=json
```

Parses Bun's JSON test output.

### jest

```bash
jest --json
```

Parses Jest JSON output.

### go

```bash
go test -json ./...
```

Parses Go's JSON test output.

## Aggregation

When multiple test suites are configured, metrics are aggregated:

- **Total**: Sum of all suite times
- **Average**: Weighted by test count
- **Max**: Slowest test across all suites

```
rust: test time
  total: 18.6s
  avg: 52ms (358 tests)
  max: 2.1s (tests::integration::large_file_parse)
  suites:
    cargo: 12.4s (276 tests)
    bats: 4.2s (45 tests)
    pytest: 2.0s (37 tests)
```

## Thresholds

Enforce test time limits:

```toml
[checks.rust]
test_time_total_max = "30s"   # Total suite time
test_time_avg_max = "100ms"   # Average per test
test_time_max = "1s"          # Slowest individual test
```

Per-suite thresholds:

```toml
[[checks.rust.test_suites]]
runner = "bats"
path = "tests/cli/"
total_max = "10s"
max = "500ms"
```
