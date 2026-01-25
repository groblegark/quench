# Phase 935: Test Runners - Other Runners

## Overview

Implement the remaining test runners: pytest, vitest, jest, bun, go test, and a custom command runner. Each runner parses tool-specific output to extract per-test timing and pass/fail status. The custom runner provides a fallback for unsupported tools (no per-test timing).

## Project Structure

```
crates/cli/src/checks/tests/runners/
├── mod.rs              # Update: register new runners
├── bats.rs             # Reference: TAP parsing pattern
├── cargo.rs            # Reference: JSON parsing pattern
├── pytest.rs           # NEW: pytest --durations output
├── pytest_tests.rs     # NEW: unit tests
├── vitest.rs           # NEW: vitest JSON reporter
├── vitest_tests.rs     # NEW: unit tests
├── jest.rs             # NEW: jest JSON output
├── jest_tests.rs       # NEW: unit tests
├── bun.rs              # NEW: bun JSON output
├── bun_tests.rs        # NEW: unit tests
├── go.rs               # NEW: go test -json output
├── go_tests.rs         # NEW: unit tests
├── custom.rs           # NEW: custom command (no per-test timing)
├── custom_tests.rs     # NEW: unit tests
├── result.rs           # Existing
├── coverage.rs         # Existing
└── stub.rs             # Delete after all runners implemented
```

## Dependencies

No new dependencies required. All runners use:
- `std::process::Command` for execution
- `serde_json` (already in deps) for JSON parsing
- Existing `TestRunner` trait and `TestRunResult` types

## Implementation Phases

### Phase 1: pytest Runner

**Command:** `pytest --durations=0 -v <path>`

**Output Format:**
```
============================= slowest durations =============================
0.45s call     test_module.py::test_one
0.23s call     test_module.py::test_two
0.01s setup    test_module.py::test_one
...
============================= 2 passed in 0.68s =============================
```

**Implementation:**
```rust
// crates/cli/src/checks/tests/runners/pytest.rs
pub struct PytestRunner;

impl TestRunner for PytestRunner {
    fn name(&self) -> &'static str { "pytest" }

    fn available(&self, ctx: &RunnerContext) -> bool {
        // Check: pytest --version succeeds
        Command::new("pytest").arg("--version")...
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // 1. Run setup command if specified
        // 2. Execute: pytest --durations=0 -v <path>
        // 3. Parse durations section for per-test timing
        // 4. Parse summary line for pass/fail counts
    }
}

fn parse_pytest_output(stdout: &str, total_time: Duration) -> TestRunResult {
    // Find "slowest durations" section
    // Parse lines: "0.45s call test_module.py::test_one"
    // Find summary: "2 passed, 1 failed in 0.68s"
}
```

**Verification:** Unit tests for output parsing, integration test with fixture.

### Phase 2: Go Test Runner

**Command:** `go test -json ./...`

**Output Format (NDJSON):**
```json
{"Time":"2024-01-01T00:00:00Z","Action":"run","Package":"pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"pkg","Test":"TestOne","Elapsed":0.45}
{"Time":"2024-01-01T00:00:02Z","Action":"fail","Package":"pkg","Test":"TestTwo","Elapsed":0.23}
```

**Implementation:**
```rust
// crates/cli/src/checks/tests/runners/go.rs
pub struct GoRunner;

#[derive(Deserialize)]
struct GoTestEvent {
    #[serde(rename = "Action")]
    action: String,
    #[serde(rename = "Package")]
    package: Option<String>,
    #[serde(rename = "Test")]
    test: Option<String>,
    #[serde(rename = "Elapsed")]
    elapsed: Option<f64>,
}

fn parse_go_json(stdout: &str, total_time: Duration) -> TestRunResult {
    // Parse NDJSON lines
    // Collect "pass" and "fail" actions with Test names
    // Extract elapsed time in seconds
}
```

**Verification:** Unit tests for NDJSON parsing, fixture with Go project.

### Phase 3: vitest Runner

**Command:** `vitest run --reporter=json`

**Output Format:**
```json
{
  "testResults": [
    {
      "name": "src/utils.test.ts",
      "assertionResults": [
        {"fullName": "adds numbers", "status": "passed", "duration": 45},
        {"fullName": "handles errors", "status": "failed", "duration": 23}
      ]
    }
  ]
}
```

**Implementation:**
```rust
// crates/cli/src/checks/tests/runners/vitest.rs
pub struct VitestRunner;

#[derive(Deserialize)]
struct VitestOutput {
    #[serde(rename = "testResults")]
    test_results: Vec<VitestTestFile>,
}

#[derive(Deserialize)]
struct VitestTestFile {
    name: String,
    #[serde(rename = "assertionResults")]
    assertion_results: Vec<VitestAssertion>,
}

#[derive(Deserialize)]
struct VitestAssertion {
    #[serde(rename = "fullName")]
    full_name: String,
    status: String,           // "passed" | "failed"
    duration: Option<u64>,    // milliseconds
}
```

**Verification:** Unit tests for JSON parsing, fixture with vitest project.

### Phase 4: jest and bun Runners

Jest and bun share identical JSON output format.

**Command (jest):** `jest --json`
**Command (bun):** `bun test --reporter=json`

**Output Format:**
```json
{
  "success": true,
  "numTotalTests": 10,
  "numPassedTests": 9,
  "numFailedTests": 1,
  "testResults": [
    {
      "name": "/path/to/test.ts",
      "assertionResults": [
        {"fullName": "test name", "status": "passed", "duration": 45}
      ]
    }
  ]
}
```

**Implementation:**
```rust
// crates/cli/src/checks/tests/runners/jest.rs
pub struct JestRunner;

// crates/cli/src/checks/tests/runners/bun.rs
pub struct BunRunner;

// Shared types (can be in a common module or duplicated)
#[derive(Deserialize)]
struct JestOutput {
    success: bool,
    #[serde(rename = "testResults")]
    test_results: Vec<JestTestFile>,
}

#[derive(Deserialize)]
struct JestTestFile {
    name: String,
    #[serde(rename = "assertionResults")]
    assertion_results: Vec<JestAssertion>,
}

#[derive(Deserialize)]
struct JestAssertion {
    #[serde(rename = "fullName")]
    full_name: String,
    status: String,
    duration: Option<u64>,
}
```

**Verification:** Unit tests for JSON parsing, fixtures for jest/bun projects.

### Phase 5: Custom Command Runner

**Configuration:**
```toml
[[check.tests.suite]]
name = "custom"
command = "./scripts/run-tests.sh"
```

**Implementation:**
```rust
// crates/cli/src/checks/tests/runners/custom.rs
pub struct CustomRunner;

impl TestRunner for CustomRunner {
    fn name(&self) -> &'static str { "custom" }

    fn available(&self, _ctx: &RunnerContext) -> bool {
        true  // Always available (command existence checked at runtime)
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        let command = config.command.as_ref()
            .ok_or("custom runner requires 'command' field")?;

        // 1. Run setup if specified
        // 2. Execute command via shell
        // 3. Return pass/fail based on exit code only
        // 4. No per-test timing (tests vec is empty)

        let start = Instant::now();
        let output = Command::new("sh")
            .args(["-c", command])
            .current_dir(ctx.root)
            .output()?;

        if output.status.success() {
            TestRunResult::passed(start.elapsed())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            TestRunResult::failed(start.elapsed(), stderr.lines().take(10).join("\n"))
        }
    }
}
```

**Verification:** Unit tests for command execution, fixture with custom script.

### Phase 6: Registry Update and Cleanup

1. **Update `mod.rs`** to register all new runners:
```rust
mod bats;
mod bun;
mod cargo;
mod coverage;
mod custom;
mod go;
mod jest;
mod pytest;
mod result;
mod vitest;

pub use bats::BatsRunner;
pub use bun::BunRunner;
pub use cargo::CargoRunner;
pub use custom::CustomRunner;
pub use go::GoRunner;
pub use jest::JestRunner;
pub use pytest::PytestRunner;
pub use vitest::VitestRunner;

pub fn all_runners() -> Vec<Arc<dyn TestRunner>> {
    vec![
        Arc::new(CargoRunner),
        Arc::new(BatsRunner),
        Arc::new(GoRunner),
        Arc::new(PytestRunner),
        Arc::new(VitestRunner),
        Arc::new(BunRunner),
        Arc::new(JestRunner),
        Arc::new(CustomRunner),
    ]
}
```

2. **Delete `stub.rs`** - no longer needed.

3. **Update `RUNNER_NAMES`** constant if needed.

4. **Update tests in `mod_tests.rs`** for new runner count.

## Key Implementation Details

### Output Parsing Patterns

All runners follow the same pattern from `bats.rs`:

1. **Execute command** with timing flags/JSON output
2. **Parse stdout** for test results
3. **Build `TestResult` list** with name, passed, duration
4. **Return `TestRunResult`** with overall pass/fail and test list

### Availability Checks

Each runner checks:
1. Tool is installed: `<tool> --version` succeeds
2. Project has relevant files (optional, depends on runner)

```rust
fn available(&self, ctx: &RunnerContext) -> bool {
    let installed = Command::new("pytest")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    installed
}
```

### Duration Handling

- **Milliseconds:** Jest, Bun, Vitest report in ms as integers
- **Seconds:** Go, pytest report in seconds as floats
- **Convert to `Duration`:** Use `Duration::from_millis()` or `Duration::from_secs_f64()`

### Error Handling

Follow the bats pattern:
- Setup failure → `TestRunResult::failed(Duration::ZERO, error)`
- Command execution failure → `TestRunResult::failed(elapsed, error)`
- Parse failure → Still return results with available data

## Verification Plan

### Unit Tests (per runner)

Each `<runner>_tests.rs` should test:
1. **Parsing success output** - verify test names and durations extracted
2. **Parsing failure output** - verify failed tests detected
3. **Parsing empty output** - graceful handling
4. **Duration extraction** - milliseconds and seconds formats
5. **Edge cases** - malformed lines, missing fields

Example test structure:
```rust
// pytest_tests.rs
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;

#[test]
fn parses_passing_tests() {
    let output = r#"
============================= slowest durations =============================
0.45s call     test_module.py::test_one
0.23s call     test_module.py::test_two
============================= 2 passed in 0.68s =============================
"#;
    let result = parse_pytest_output(output, Duration::from_secs(1));
    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert_eq!(result.tests[0].name, "test_module.py::test_one");
}

#[test]
fn parses_failing_tests() {
    let output = r#"
============================= 1 passed, 1 failed in 0.68s =============================
"#;
    let result = parse_pytest_output(output, Duration::from_secs(1));
    assert!(!result.passed);
}
```

### Integration Tests

Add spec tests in `tests/specs/` for each runner (where practical):
- Create minimal fixtures in `tests/fixtures/`
- Test that `quench check --tests` produces expected output

### Manual Verification

For runners requiring external tools:
1. Install tool locally
2. Create sample project with tests
3. Run `quench check --tests` and verify output

### CI Considerations

Some runners may not be available in CI:
- Mark integration tests with `#[ignore = "requires <tool>"]` if CI doesn't have tool
- Ensure unit tests (parsing) always run
- Consider adding tool installation to CI matrix

## Commit Strategy

One commit per phase:
1. `feat(tests): implement pytest runner`
2. `feat(tests): implement go test runner`
3. `feat(tests): implement vitest runner`
4. `feat(tests): implement jest and bun runners`
5. `feat(tests): implement custom command runner`
6. `refactor(tests): remove stub runner, update registry`
