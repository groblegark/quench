# Checkpoint 11G: Bug Fixes - Tests CI Mode

## Overview

Fix critical bugs in the tests CI mode feature, primarily addressing the timeout configuration being silently ignored by most test runners. Currently, only the Cargo runner implements timeout support while pytest, bats, go, bun, jest, and vitest runners use `cmd.output()` directly, causing tests to hang indefinitely when configured with `timeout`.

**Follows:** checkpoint-11f-quickwins (Quick Wins)

## Project Structure

```
quench/
├── crates/cli/src/checks/tests/runners/
│   ├── mod.rs           # MODIFY: Export timeout utility
│   ├── bats.rs          # MODIFY: Add timeout support
│   ├── pytest.rs        # MODIFY: Add timeout support
│   ├── go.rs            # MODIFY: Add timeout support
│   ├── jest.rs          # MODIFY: Add timeout support
│   ├── vitest.rs        # MODIFY: Add timeout support
│   ├── bun.rs           # MODIFY: Add timeout support
│   └── custom.rs        # MODIFY: Add timeout support
├── tests/specs/checks/tests/
│   └── timeout.rs       # CREATE: Timeout behavior specs
└── tests/fixtures/tests-timeout/
    └── ...              # CREATE: Timeout test fixture
```

## Dependencies

No new dependencies. Uses existing:
- `run_with_timeout()` function in `crates/cli/src/checks/tests/runners/mod.rs:257`
- `std::process::{Command, Stdio}` for process spawning
- `std::io::ErrorKind::TimedOut` for timeout error detection

## Implementation Phases

### Phase 1: Add Timeout Support to All Runners

**Goal:** Make timeout configuration work consistently across all test runners.

**Bug:** Currently, only `cargo.rs` uses `run_with_timeout()`. All other runners call `cmd.output()` directly:

| Runner | Line | Current | Issue |
|--------|------|---------|-------|
| bats.rs | 57 | `cmd.output()` | Timeout ignored |
| pytest.rs | 55 | `cmd.output()` | Timeout ignored |
| go.rs | 59 | `cmd.output()` | Timeout ignored |
| jest.rs | 60 | `cmd.output()` | Timeout ignored |
| vitest.rs | 60 | `cmd.output()` | Timeout ignored |
| bun.rs | 59 | `cmd.output()` | Timeout ignored |

**Fix Pattern:**

The cargo runner shows the correct pattern (cargo.rs:54-77):

```rust
// Before: Direct output call
let output = match cmd.output() { ... }

// After: Spawn + timeout
cmd.stdout(Stdio::piped());
cmd.stderr(Stdio::piped());

let child = match cmd.spawn() {
    Ok(c) => c,
    Err(e) => {
        return TestRunResult::failed(
            start.elapsed(),
            format!("failed to spawn {}: {e}", runner_name),
        );
    }
};

let output = match run_with_timeout(child, config.timeout) {
    Ok(out) => out,
    Err(e) if e.kind() == ErrorKind::TimedOut => {
        let timeout_msg = config
            .timeout
            .map(|t| format!("timed out after {:?}", t))
            .unwrap_or_else(|| "timed out".to_string());
        return TestRunResult::failed(start.elapsed(), timeout_msg);
    }
    Err(e) => {
        return TestRunResult::failed(
            start.elapsed(),
            format!("failed to run {}: {e}", runner_name),
        );
    }
};
```

**Files to modify:**

1. `bats.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method
2. `pytest.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method
3. `go.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method
4. `jest.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method
5. `vitest.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method
6. `bun.rs` - Add import for `run_with_timeout`, `ErrorKind`, update run method

**Verification:**
```bash
cargo test --lib runners
cargo test --test specs timeout
```

### Phase 2: Add Timeout Behavioral Specs

**Goal:** Add specs to verify timeout behavior across runners.

**File:** `tests/specs/checks/tests/timeout.rs`

```rust
//! Behavioral specs for suite timeout configuration.
//!
//! Reference: docs/specs/checks/tests.md#timeout

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

/// Spec: docs/specs/checks/tests.md#timeout
///
/// > Suite timeout kills slow tests and reports failure.
#[test]
fn bats_runner_respects_timeout() {
    let temp = Project::empty();
    temp.config(
        r#"
[[check.tests.suite]]
runner = "bats"
path = "tests"
timeout = "100ms"
"#,
    );
    // Create a bats test that hangs
    temp.file(
        "tests/slow.bats",
        r#"
#!/usr/bin/env bats

@test "hangs forever" {
    sleep 60
}
"#,
    );

    check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("timed out");
}

/// Spec: Suite without timeout runs normally.
#[test]
fn bats_runner_without_timeout_succeeds() {
    let temp = Project::empty();
    temp.config(
        r#"
[[check.tests.suite]]
runner = "bats"
path = "tests"
"#,
    );
    temp.file(
        "tests/quick.bats",
        r#"
#!/usr/bin/env bats

@test "quick test" {
    [ 1 -eq 1 ]
}
"#,
    );

    check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .passes();
}
```

**File:** `tests/specs/checks/tests/mod.rs`

Add module declaration:
```rust
mod timeout;
```

**Verification:**
```bash
cargo test --test specs timeout
```

### Phase 3: Improve Error Messages for Timeout Failures

**Goal:** Provide consistent, actionable error messages for timeout scenarios across all runners.

**File:** `crates/cli/src/checks/tests/runners/mod.rs`

Add a utility function for consistent timeout error formatting:

```rust
/// Format a timeout error message with runner-specific advice.
pub fn format_timeout_error(runner: &str, timeout: Duration) -> String {
    let base = format!("timed out after {:?}", timeout);
    let advice = match runner {
        "cargo" => "check for infinite loops or deadlocks",
        "bats" => "check for infinite loops in shell scripts",
        "pytest" => "check for slow tests or missing mocks",
        "go" => "check for goroutine leaks or infinite loops",
        "jest" | "vitest" | "bun" => "check for unresolved promises or infinite loops",
        _ => "check for slow or hanging tests",
    };
    format!("{} - {}", base, advice)
}
```

Update each runner to use this function instead of a raw timeout message.

**Verification:**
```bash
cargo test --lib timeout
cargo test --test specs timeout
```

### Phase 4: Fix Custom Runner Timeout Support

**Goal:** Ensure the custom runner also supports timeout configuration.

**File:** `crates/cli/src/checks/tests/runners/custom.rs`

The custom runner executes arbitrary commands and must also respect timeout configuration:

```rust
use std::io::ErrorKind;
use super::run_with_timeout;

// In the run method:
cmd.stdout(Stdio::piped());
cmd.stderr(Stdio::piped());

let child = match cmd.spawn() {
    Ok(c) => c,
    Err(e) => {
        return TestRunResult::failed(
            start.elapsed(),
            format!("failed to spawn custom command: {e}"),
        );
    }
};

let output = match run_with_timeout(child, config.timeout) {
    Ok(out) => out,
    Err(e) if e.kind() == ErrorKind::TimedOut => {
        return TestRunResult::failed(
            start.elapsed(),
            format_timeout_error("custom", config.timeout.unwrap()),
        );
    }
    Err(e) => {
        return TestRunResult::failed(
            start.elapsed(),
            format!("custom command failed: {e}"),
        );
    }
};
```

**Verification:**
```bash
cargo test --lib custom
```

### Phase 5: Add Unit Tests for Timeout Integration

**Goal:** Add unit tests verifying timeout behavior in each runner.

**File:** `crates/cli/src/checks/tests/runners/bats_tests.rs` (and similar for other runners)

```rust
#[test]
fn timeout_config_is_passed_to_runner() {
    // Test that the runner properly passes timeout to run_with_timeout
    // This is more of an integration test, but ensures the wiring is correct
}
```

**Note:** The actual timeout behavior is already tested in `mod_tests.rs:374-391` via `run_with_timeout_slow_command_times_out`. Runner tests verify the integration.

**Verification:**
```bash
cargo test --lib runners
make check
```

## Key Implementation Details

### Timeout Configuration

The timeout is configured per-suite in `quench.toml`:

```toml
[[check.tests.suite]]
runner = "bats"
path = "tests"
timeout = "60s"  # Human-readable duration
```

Parsed as `Option<Duration>` in `TestSuiteConfig::timeout`.

### Error Handling Consistency

All runners should:
1. Check for `ErrorKind::TimedOut` specifically
2. Return a `TestRunResult::failed()` with descriptive message
3. Include the configured timeout duration in the error
4. Provide runner-specific advice

### Process Cleanup

The `run_with_timeout()` function in `mod.rs:253-310` already handles:
- Polling the child process every 50ms
- Killing the process on timeout (`child.kill()`)
- Waiting for the killed process (`child.wait()`)
- Returning collected stdout/stderr before killing

No changes needed to the core timeout implementation.

### Affected Runners Summary

| Runner | File | Timeout Support | After Fix |
|--------|------|-----------------|-----------|
| cargo | cargo.rs | Yes | No change |
| bats | bats.rs | **No** | Added |
| pytest | pytest.rs | **No** | Added |
| go | go.rs | **No** | Added |
| jest | jest.rs | **No** | Added |
| vitest | vitest.rs | **No** | Added |
| bun | bun.rs | **No** | Added |
| custom | custom.rs | **No** | Added |

## Verification Plan

| Phase | Command | Expected Result |
|-------|---------|-----------------|
| 1 | `cargo test --lib runners` | All runner unit tests pass |
| 2 | `cargo test --test specs timeout` | Timeout specs pass |
| 3 | `cargo test --lib timeout` | Timeout utility tests pass |
| 4 | `cargo test --lib custom` | Custom runner tests pass |
| 5 | `cargo test --lib runners` | All integration tests pass |
| All | `make check` | Full CI passes |

## Completion Criteria

- [ ] Phase 1: All runners use `run_with_timeout()` for process execution
- [ ] Phase 2: Timeout behavioral specs added and passing
- [ ] Phase 3: Consistent timeout error messages across runners
- [ ] Phase 4: Custom runner timeout support
- [ ] Phase 5: Unit tests for timeout integration
- [ ] All tests pass
- [ ] `make check` passes
- [ ] Changes committed
- [ ] `./done` executed
