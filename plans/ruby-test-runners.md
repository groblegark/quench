# Ruby Test Runners Implementation Plan

Implement Ruby test runners (RSpec, Minitest, Cucumber) with JSON output parsing and coverage via SimpleCov. Corresponds to Phase 488-4881 from the Ruby roadmap.

## Overview

Add three test runners for Ruby projects:

1. **RSpec** - Most popular Ruby testing framework, outputs structured JSON
2. **Minitest** - Ruby's built-in testing framework, uses minitest-reporters for JSON
3. **Cucumber** - BDD framework for acceptance tests, outputs JSON

All runners integrate with SimpleCov for coverage collection, reading from `coverage/.resultset.json`.

## Project Structure

```
crates/cli/src/checks/tests/runners/
├── mod.rs                    # Add Ruby runners to exports
├── rspec.rs                  # RSpec runner implementation
├── rspec_tests.rs            # RSpec unit tests
├── minitest.rs               # Minitest runner implementation
├── minitest_tests.rs         # Minitest unit tests
├── cucumber.rs               # Cucumber runner implementation
├── cucumber_tests.rs         # Cucumber unit tests
├── ruby_coverage.rs          # SimpleCov parsing
└── ruby_coverage_tests.rs    # SimpleCov unit tests

tests/
├── fixtures/ruby-gem/        # Existing: add SimpleCov config
└── specs/checks/tests.rs     # Add Ruby runner specs
```

## Dependencies

**Runtime (user's project)**:
- `bundler` - Ruby dependency manager
- `rspec` gem with `--format json` support
- `minitest` gem with optional `minitest-reporters` for JSON
- `cucumber` gem with `--format json` support
- `simplecov` gem for coverage (optional)

**Build-time**:
- `serde` and `serde_json` - Already in use for other runners

## Implementation Phases

### Phase 1: RSpec Runner

RSpec is the most common Ruby testing framework. Implement full JSON parsing.

**Tasks**:
- [ ] Create `rspec.rs` with `RspecRunner` struct implementing `TestRunner` trait
- [ ] Implement `available()`: check for `bundle exec rspec --version` and `spec/` directory
- [ ] Implement `run()`: execute `bundle exec rspec --format json` with timeout
- [ ] Parse RSpec JSON output structure (see Key Implementation Details)
- [ ] Extract per-example timing, status (passed/failed/pending)
- [ ] Create `rspec_tests.rs` with parsing tests

**Verification**:
```bash
cargo test rspec
```

### Phase 2: Minitest Runner

Minitest is Ruby's built-in framework. It requires `minitest-reporters` for structured output.

**Tasks**:
- [ ] Create `minitest.rs` with `MinitestRunner` struct
- [ ] Implement `available()`: check for `test/` directory and Minitest in Gemfile
- [ ] Implement `run()`: execute `bundle exec ruby -Itest -e "..."` with JSON reporter
- [ ] Parse Minitest JSON output (via minitest-reporters)
- [ ] Fallback: parse standard dot/F/E output when no JSON available
- [ ] Create `minitest_tests.rs` with parsing tests

**Verification**:
```bash
cargo test minitest
```

### Phase 3: Cucumber Runner

Cucumber is used for BDD-style acceptance tests.

**Tasks**:
- [ ] Create `cucumber.rs` with `CucumberRunner` struct
- [ ] Implement `available()`: check for `features/` directory
- [ ] Implement `run()`: execute `bundle exec cucumber --format json`
- [ ] Parse Cucumber JSON output (scenario-level, not step-level)
- [ ] Extract scenario timing and status
- [ ] Create `cucumber_tests.rs` with parsing tests

**Verification**:
```bash
cargo test cucumber
```

### Phase 4: Runner Auto-Detection and Registration

Wire up runners and implement auto-detection logic.

**Tasks**:
- [ ] Add `mod rspec;`, `mod minitest;`, `mod cucumber;` to `runners/mod.rs`
- [ ] Add runners to `all_runners()` function
- [ ] Add `"rspec"`, `"minitest"`, `"cucumber"` to `RUNNER_NAMES`
- [ ] Add timeout advice in `format_timeout_error()` for Ruby runners
- [ ] Add `pub ruby: Option<CoverageResult>` to `AggregatedCoverage`
- [ ] Implement `merge_ruby()` method

**Verification**:
```bash
cargo test runners
```

### Phase 5: SimpleCov Coverage Integration

Parse SimpleCov's `.resultset.json` for coverage data.

**Tasks**:
- [ ] Create `ruby_coverage.rs` with `collect_ruby_coverage()` function
- [ ] Parse SimpleCov `.resultset.json` format (see Key Implementation Details)
- [ ] Extract line coverage percentage per file
- [ ] Calculate package-level coverage (group by top-level directory)
- [ ] Add `simplecov_available()` check (look for `coverage/` directory after test run)
- [ ] Integrate coverage collection into Ruby runners
- [ ] Create `ruby_coverage_tests.rs`

**Verification**:
```bash
cargo test ruby_coverage
```

### Phase 6: Integration Testing

End-to-end verification with real Ruby projects.

**Tasks**:
- [ ] Add SimpleCov configuration to `tests/fixtures/ruby-gem/`
- [ ] Add behavioral specs in `tests/specs/checks/tests.rs` for Ruby runners
- [ ] Test runner selection based on directory structure
- [ ] Test coverage aggregation across multiple Ruby suites
- [ ] Run `make check` to verify all tests pass

**Verification**:
```bash
make check
```

## Key Implementation Details

### RSpec JSON Output Structure

```json
{
  "version": "3.12.0",
  "examples": [
    {
      "id": "./spec/math_spec.rb[1:1:1]",
      "description": "adds two numbers",
      "full_description": "Math .add adds two numbers",
      "status": "passed",
      "run_time": 0.001234,
      "pending_message": null
    }
  ],
  "summary": {
    "duration": 0.05,
    "example_count": 10,
    "failure_count": 0,
    "pending_count": 1
  }
}
```

**Rust parsing structs**:
```rust
#[derive(Debug, Deserialize)]
pub(crate) struct RspecOutput {
    pub examples: Vec<RspecExample>,
    pub summary: RspecSummary,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RspecExample {
    pub full_description: String,
    pub status: String,  // "passed", "failed", "pending"
    pub run_time: f64,   // seconds
}

#[derive(Debug, Deserialize)]
pub(crate) struct RspecSummary {
    pub duration: f64,
    pub example_count: u32,
    pub failure_count: u32,
    pub pending_count: u32,
}
```

### Minitest JSON Output (via minitest-reporters)

```json
{
  "status": "pass",
  "tests": [
    {
      "name": "test_adds_two_numbers",
      "classname": "MathTest",
      "time": 0.001,
      "status": "pass"
    }
  ],
  "summary": {
    "total": 10,
    "passed": 9,
    "failed": 0,
    "skipped": 1,
    "time": 0.05
  }
}
```

**Fallback parsing** for standard Minitest output:
```
Run options: --seed 12345

# Running:

..F.S....

Finished in 0.012345s, 100.0000 runs/s, 200.0000 assertions/s.

10 runs, 20 assertions, 1 failures, 0 errors, 1 skips
```

### Cucumber JSON Output

```json
[
  {
    "uri": "features/math.feature",
    "name": "Math operations",
    "elements": [
      {
        "type": "scenario",
        "name": "Adding numbers",
        "steps": [
          {
            "name": "I have entered 50 into the calculator",
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

**Note**: Duration is in nanoseconds. Status can be "passed", "failed", "pending", "skipped".

### SimpleCov .resultset.json Format

Located at `coverage/.resultset.json`:

```json
{
  "RSpec": {
    "coverage": {
      "/path/to/lib/math.rb": {
        "lines": [1, 1, null, 0, 1, null]
      }
    },
    "timestamp": 1234567890
  }
}
```

**Parsing logic**:
```rust
pub fn collect_ruby_coverage(root: &Path) -> CoverageResult {
    let resultset_path = root.join("coverage/.resultset.json");
    if !resultset_path.exists() {
        return CoverageResult::skipped();
    }

    // Parse JSON, calculate line coverage per file:
    // covered = count of non-null, non-zero entries
    // total = count of non-null entries
    // coverage % = (covered / total) * 100
}
```

### Runner Command Patterns

| Runner | Command | Output |
|--------|---------|--------|
| RSpec | `bundle exec rspec --format json` | JSON to stdout |
| Minitest | `bundle exec rake test TESTOPTS="--reporter json"` | JSON to stdout |
| Cucumber | `bundle exec cucumber --format json` | JSON to stdout |

### Auto-Detection Priority

```rust
fn detect_ruby_runner(root: &Path) -> Option<&'static str> {
    // Check directory structure first (fastest)
    if root.join("spec").is_dir() {
        return Some("rspec");
    }
    if root.join("features").is_dir() {
        return Some("cucumber");
    }
    if root.join("test").is_dir() {
        return Some("minitest");
    }

    // Fallback: parse Gemfile for test gems
    if let Some(gemfile) = read_gemfile(root) {
        if gemfile.contains("rspec") { return Some("rspec"); }
        if gemfile.contains("cucumber") { return Some("cucumber"); }
        if gemfile.contains("minitest") { return Some("minitest"); }
    }

    None
}
```

### Test Result Mapping

| RSpec Status | Minitest Status | Cucumber Status | TestResult |
|--------------|-----------------|-----------------|------------|
| passed | pass | passed | `TestResult::passed()` |
| failed | fail/error | failed | `TestResult::failed()` |
| pending | skip | pending/skipped | `TestResult::skipped()` |

## Verification Plan

### Unit Tests

Each runner module has a `*_tests.rs` file with:
- JSON parsing tests with sample output
- Edge cases (empty output, malformed JSON, mixed results)
- Duration extraction accuracy
- Status mapping correctness

### Integration Tests

Add to `tests/specs/checks/tests.rs`:

```rust
#[test]
fn rspec_runner_executes() {
    cli()
        .on("ruby-gem")
        .args(["check", "--tests"])
        .succeeds()
        .stdout_has("rspec");
}

#[test]
fn ruby_coverage_collected() {
    cli()
        .on("ruby-gem")
        .args(["check", "--ci", "--tests"])
        .succeeds()
        .stdout_has("ruby:");
}
```

### Manual Verification

```bash
# Build and run on a Ruby project
cd /path/to/ruby-project
cargo run -- check --tests --ci

# Expected output shows Ruby coverage
# tests: PASS
#   rspec: 10 tests in 0.5s (avg 50ms)
#   coverage:
#     ruby: 85.2%
```

## References

- Existing runner patterns: `crates/cli/src/checks/tests/runners/jest.rs`
- Coverage parsing: `crates/cli/src/checks/tests/runners/coverage.rs`
- Ruby spec: `docs/specs/langs/ruby.md`
- Ruby roadmap: `plans/.4-roadmap-ruby.md` (Phase 488-4881)
