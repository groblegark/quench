# Phase 488: Ruby Test Runners

Add test runners for Ruby projects: RSpec, Minitest, and Cucumber.

## Overview

Implement three Ruby test runners following the established `TestRunner` trait pattern:

1. **RSpec** - `bundle exec rspec --format json`
2. **Minitest** - `bundle exec rake test` with output parsing
3. **Cucumber** - `bundle exec cucumber --format json`

Auto-detection from directory structure (`spec/`, `test/`, `features/`) and Gemfile dependencies.

## Project Structure

```
crates/cli/src/checks/tests/runners/
├── mod.rs                 # Add rspec, minitest, cucumber exports
├── rspec.rs               # RSpec runner implementation
├── rspec_tests.rs         # RSpec unit tests
├── minitest.rs            # Minitest runner implementation
├── minitest_tests.rs      # Minitest unit tests
├── cucumber.rs            # Cucumber runner implementation
├── cucumber_tests.rs      # Cucumber unit tests
└── ruby.rs                # Shared Ruby utilities (Gemfile parsing)

tests/fixtures/
├── ruby-rspec/            # RSpec fixture project
├── ruby-minitest/         # Minitest fixture project
└── ruby-cucumber/         # Cucumber fixture project
```

## Dependencies

- **External**: `bundle` (Bundler), Ruby runtime
- **Internal**: Existing test runner framework (Phase 915)
- **Crates**: `serde_json` (already in use for Jest/Vitest JSON parsing)

## Implementation Phases

### Phase 1: RSpec Runner

**Goal**: Execute RSpec tests and parse JSON output.

**Files**:
- `crates/cli/src/checks/tests/runners/rspec.rs`
- `crates/cli/src/checks/tests/runners/rspec_tests.rs`

**Implementation**:

```rust
// rspec.rs
pub struct RspecRunner;

impl TestRunner for RspecRunner {
    fn name(&self) -> &'static str { "rspec" }

    fn available(&self, ctx: &RunnerContext) -> bool {
        // Check for spec/ directory or Gemfile with rspec
        ctx.root.join("spec").exists()
            || has_gemfile_dep(ctx.root, "rspec")
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // bundle exec rspec --format json <path>
        // Parse JSON for per-test timing, pass/fail/pending
    }
}
```

**RSpec JSON output format**:
```json
{
  "examples": [
    {
      "id": "./spec/foo_spec.rb[1:1]",
      "description": "does something",
      "full_description": "Foo does something",
      "status": "passed",
      "run_time": 0.001234,
      "pending_message": null
    }
  ],
  "summary": {
    "duration": 0.123,
    "example_count": 5,
    "failure_count": 0,
    "pending_count": 1
  }
}
```

**Verification**:
- [ ] Unit tests for JSON parsing
- [ ] Integration test with `tests/fixtures/ruby-rspec/`
- [ ] Handles passed, failed, pending status
- [ ] Extracts per-test timing from `run_time`

### Phase 2: Minitest Runner

**Goal**: Execute Minitest via Rake and parse output.

**Files**:
- `crates/cli/src/checks/tests/runners/minitest.rs`
- `crates/cli/src/checks/tests/runners/minitest_tests.rs`

**Implementation**:

```rust
pub struct MinitestRunner;

impl TestRunner for MinitestRunner {
    fn name(&self) -> &'static str { "minitest" }

    fn available(&self, ctx: &RunnerContext) -> bool {
        // Check for test/ directory or Gemfile with minitest
        ctx.root.join("test").exists()
            || has_gemfile_dep(ctx.root, "minitest")
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // bundle exec rake test
        // Parse dots/F/E output for status
        // Per-test timing requires minitest-reporters
    }
}
```

**Minitest output formats**:

Standard (dots):
```
Run options: --seed 12345

# Running:

..F.E.

Finished in 0.123456s, 48.6 runs/s, 97.2 assertions/s.

6 runs, 12 assertions, 1 failures, 1 errors, 0 skips
```

With minitest-reporters (JSON-like):
```
Finished in 0.012345 seconds.
4 tests, 8 assertions, 0 failures, 0 errors, 0 skips
```

**Verification**:
- [ ] Unit tests for output parsing (dots format)
- [ ] Integration test with `tests/fixtures/ruby-minitest/`
- [ ] Handles passed, failed, error, skip status
- [ ] Extracts summary timing (per-test timing when available)

### Phase 3: Cucumber Runner

**Goal**: Execute Cucumber tests and parse JSON output.

**Files**:
- `crates/cli/src/checks/tests/runners/cucumber.rs`
- `crates/cli/src/checks/tests/runners/cucumber_tests.rs`

**Implementation**:

```rust
pub struct CucumberRunner;

impl TestRunner for CucumberRunner {
    fn name(&self) -> &'static str { "cucumber" }

    fn available(&self, ctx: &RunnerContext) -> bool {
        // Check for features/ directory or Gemfile with cucumber
        ctx.root.join("features").exists()
            || has_gemfile_dep(ctx.root, "cucumber")
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // bundle exec cucumber --format json <path>
        // Parse JSON for per-scenario timing, pass/fail/pending
    }
}
```

**Cucumber JSON output format**:
```json
[
  {
    "uri": "features/login.feature",
    "name": "Login",
    "elements": [
      {
        "name": "Successful login",
        "type": "scenario",
        "steps": [
          {
            "name": "I am on the login page",
            "result": {
              "status": "passed",
              "duration": 1234567
            }
          }
        ]
      }
    ]
  }
]
```

Note: Cucumber durations are in nanoseconds.

**Verification**:
- [ ] Unit tests for JSON parsing
- [ ] Integration test with `tests/fixtures/ruby-cucumber/`
- [ ] Handles passed, failed, pending, skipped status
- [ ] Extracts per-scenario timing (sum of step durations)

### Phase 4: Auto-Detection & Integration

**Goal**: Wire up runners, add detection logic, update RUNNER_NAMES.

**Files**:
- `crates/cli/src/checks/tests/runners/mod.rs`
- `crates/cli/src/checks/tests/runners/ruby.rs` (shared utilities)

**Changes to mod.rs**:
```rust
mod cucumber;
mod minitest;
mod rspec;
mod ruby;

pub use cucumber::CucumberRunner;
pub use minitest::MinitestRunner;
pub use rspec::RspecRunner;

pub const RUNNER_NAMES: &[&str] = &[
    "cargo", "go", "pytest", "vitest", "bun", "jest", "bats",
    "rspec", "minitest", "cucumber",  // Add Ruby runners
    "custom",
];

pub fn all_runners() -> Vec<Arc<dyn TestRunner>> {
    vec![
        // ... existing runners ...
        Arc::new(RspecRunner),
        Arc::new(MinitestRunner),
        Arc::new(CucumberRunner),
        Arc::new(CustomRunner),  // Custom always last
    ]
}
```

**Shared ruby.rs utilities**:
```rust
/// Check if Gemfile contains a dependency.
pub fn has_gemfile_dep(root: &Path, gem: &str) -> bool {
    let gemfile = root.join("Gemfile");
    if !gemfile.exists() {
        return false;
    }
    // Simple pattern match: gem 'rspec' or gem "rspec"
    std::fs::read_to_string(&gemfile)
        .ok()
        .map(|content| {
            content.lines().any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("gem ")
                    && (trimmed.contains(&format!("'{gem}'"))
                        || trimmed.contains(&format!("\"{gem}\"")))
            })
        })
        .unwrap_or(false)
}

/// Check if bundle is available.
pub fn bundle_available() -> bool {
    Command::new("bundle")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
```

**Verification**:
- [ ] Auto-detect rspec from `spec/` directory
- [ ] Auto-detect minitest from `test/` directory
- [ ] Auto-detect cucumber from `features/` directory
- [ ] Gemfile dependency detection as fallback
- [ ] Integration tests for detection logic

### Phase 5: Test Fixtures

**Goal**: Create minimal Ruby fixtures for integration testing.

**Files**:
```
tests/fixtures/ruby-rspec/
├── Gemfile
├── spec/
│   ├── spec_helper.rb
│   └── example_spec.rb

tests/fixtures/ruby-minitest/
├── Gemfile
├── Rakefile
├── test/
│   ├── test_helper.rb
│   └── example_test.rb

tests/fixtures/ruby-cucumber/
├── Gemfile
├── features/
│   ├── support/env.rb
│   ├── step_definitions/steps.rb
│   └── example.feature
```

**Verification**:
- [ ] `bundle install` works in each fixture
- [ ] Tests actually run and produce expected output
- [ ] CI can run fixtures (Ruby installed)

### Phase 6: Spec Tests & Documentation

**Goal**: Add behavioral tests and update documentation.

**Files**:
- `tests/specs/runners/ruby_tests.rs` - Behavioral spec tests
- `docs/specs/11-test-runners.md` - Update with Ruby runners

**Spec test examples**:
```rust
#[test]
fn rspec_runner_parses_json() {
    let json = r#"{"examples":[...],"summary":{...}}"#;
    let result = parse_rspec_json(json, Duration::from_secs(1));
    assert!(result.passed);
    assert_eq!(result.tests.len(), 3);
}

#[test]
fn minitest_runner_parses_dots() {
    let output = "..F.E.\n\n6 runs, 12 assertions, 1 failures, 1 errors, 0 skips";
    let result = parse_minitest_output(output, Duration::from_secs(1));
    assert!(!result.passed);
    assert_eq!(result.failed_count(), 2); // 1 failure + 1 error
}
```

**Documentation updates**:
```markdown
| Runner | Per-Test Timing | Implicit Coverage |
|--------|-----------------|-------------------|
| `rspec` | Yes | Ruby (SimpleCov) |
| `minitest` | Via reporters | Ruby (SimpleCov) |
| `cucumber` | Yes | Ruby (SimpleCov) |
```

**Verification**:
- [ ] All unit tests pass
- [ ] Behavioral specs pass
- [ ] `make check` passes
- [ ] Documentation accurate

## Key Implementation Details

### Command Execution Pattern

All Ruby runners use `bundle exec` for consistency with Bundler:

```rust
let mut cmd = Command::new("bundle");
cmd.args(["exec", "rspec", "--format", "json"]);
if let Some(path) = &config.path {
    cmd.arg(path);
}
cmd.current_dir(ctx.root);
```

### JSON Parsing Pattern

Follow the Jest runner pattern for JSON parsing:

```rust
#[derive(Debug, Deserialize)]
struct RspecOutput {
    examples: Vec<RspecExample>,
    summary: RspecSummary,
}

fn parse_rspec_json(stdout: &str, total_time: Duration) -> TestRunResult {
    // Find JSON in output (may have other output before it)
    let json_str = find_json_object(stdout);

    let output: RspecOutput = match json_str.and_then(|s| serde_json::from_str(s).ok()) {
        Some(o) => o,
        None => return TestRunResult::failed(total_time, "failed to parse rspec JSON"),
    };

    // ... convert to TestResult/TestRunResult
}
```

### Timeout Handling

All runners use the existing `run_with_timeout` function:

```rust
let child = cmd.spawn()?;
let output = match run_with_timeout(child, config.timeout) {
    Ok(out) => out,
    Err(e) if e.kind() == ErrorKind::TimedOut => {
        return TestRunResult::failed(
            start.elapsed(),
            format_timeout_error("rspec", config.timeout.unwrap())
        );
    }
    Err(e) => return TestRunResult::failed(start.elapsed(), e.to_string()),
};
```

### Error Categorization

Add Ruby-specific error messages:

```rust
fn categorize_ruby_error(stderr: &str, exit_code: Option<i32>) -> String {
    if stderr.contains("Could not find gem") {
        return "missing dependencies - run bundle install".to_string();
    }
    if stderr.contains("LoadError") {
        return "load error - check require statements".to_string();
    }
    if stderr.contains("SyntaxError") {
        return "syntax error - fix Ruby syntax first".to_string();
    }
    "tests failed".to_string()
}
```

## Verification Plan

### Unit Tests

Each parser function has unit tests with sample output:

```rust
// rspec_tests.rs
#[test]
fn parses_passing_suite() {
    let json = include_str!("../../../tests/fixtures/rspec_passing.json");
    let result = parse_rspec_json(json, Duration::from_secs(1));
    assert!(result.passed);
}

#[test]
fn parses_failing_suite() {
    let json = include_str!("../../../tests/fixtures/rspec_failing.json");
    let result = parse_rspec_json(json, Duration::from_secs(1));
    assert!(!result.passed);
    assert_eq!(result.failed_count(), 2);
}

#[test]
fn extracts_per_test_timing() {
    let json = r#"{"examples":[{"run_time":0.123,...}],...}"#;
    let result = parse_rspec_json(json, Duration::from_secs(1));
    assert_eq!(result.tests[0].duration, Duration::from_secs_f64(0.123));
}
```

### Integration Tests

Run actual Ruby tests in CI:

```rust
#[test]
#[ignore = "requires Ruby and bundler"]
fn rspec_integration() {
    let fixture = Path::new("tests/fixtures/ruby-rspec");
    let ctx = RunnerContext { root: fixture, ci_mode: false, collect_coverage: false };
    let config = TestSuiteConfig { runner: "rspec".into(), ..Default::default() };

    let runner = RspecRunner;
    assert!(runner.available(&ctx));

    let result = runner.run(&config, &ctx);
    assert!(result.passed);
    assert!(!result.tests.is_empty());
}
```

### Verification Checklist

- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `make check` passes
- [ ] Ruby integration tests pass (when Ruby available)
- [ ] Documentation updated

## Configuration Examples

```toml
# quench.toml - RSpec suite
[[check.tests.suite]]
runner = "rspec"
path = "spec/"
max_total = "60s"
max_test = "5s"

# Minitest suite
[[check.tests.suite]]
runner = "minitest"
path = "test/"
setup = "bundle install"

# Cucumber suite (CI only - slow)
[[check.tests.suite]]
runner = "cucumber"
path = "features/"
ci = true
max_total = "120s"
```
