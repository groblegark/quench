# JavaScript Test Runner Auto-Detection

Complete auto-detection of JavaScript test runners from `package.json` scripts and config files (Phase 498 remaining items).

## Overview

When a JavaScript/TypeScript project is detected but no `[[check.tests.suite]]` is configured, quench should automatically detect and configure the appropriate test runner. This enables zero-config test running for most JS/TS projects.

**Items to complete from Phase 498:**
- [x] Auto-detect runner from package.json scripts
- [x] Auto-detect runner from config files (vitest.config.ts, jest.config.js)
- [x] Integration test: run tests on fixtures/js-simple

## Project Structure

```
crates/cli/src/
├── checks/tests/
│   └── runners/
│       ├── mod.rs         # Add detect_js_runner() export
│       └── js_detect.rs   # NEW: JS runner detection logic
└── adapter/javascript/
    └── mod.rs             # Already has JsWorkspace for package.json parsing
```

**Test files:**
```
tests/
├── specs/checks/tests/
│   └── js_runners.rs      # NEW: Integration specs for JS runner auto-detection
└── fixtures/js-simple/
    ├── package.json       # UPDATE: Add actual test script and devDependencies
    └── vitest.config.ts   # NEW: Vitest config for testing
```

## Dependencies

No new external dependencies required. Uses existing:
- `serde_json` - package.json parsing (already used in workspace.rs)
- `std::fs` - config file existence checks

## Implementation Phases

### Phase 1: Detection Module (js_detect.rs)

Create the core detection logic in a new module.

**File:** `crates/cli/src/checks/tests/runners/js_detect.rs`

```rust
//! JavaScript test runner auto-detection.
//!
//! Detection priority (first match wins):
//! 1. Config files (most specific signal)
//! 2. package.json devDependencies
//! 3. package.json scripts.test command

use std::path::Path;

/// Detected JavaScript test runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsRunner {
    Vitest,
    Jest,
    Bun,
}

impl JsRunner {
    /// Convert to runner name string used in TestSuiteConfig.
    pub fn name(&self) -> &'static str {
        match self {
            JsRunner::Vitest => "vitest",
            JsRunner::Jest => "jest",
            JsRunner::Bun => "bun",
        }
    }
}

/// Detection result with confidence signal.
#[derive(Debug)]
pub struct DetectionResult {
    pub runner: JsRunner,
    pub source: DetectionSource,
}

/// How the runner was detected.
#[derive(Debug)]
pub enum DetectionSource {
    ConfigFile(String),      // e.g., "vitest.config.ts"
    DevDependency(String),   // e.g., "vitest"
    TestScript(String),      // e.g., "vitest run"
}

/// Detect JavaScript test runner for a project.
///
/// Returns None if no runner can be detected.
pub fn detect_js_runner(root: &Path) -> Option<DetectionResult> {
    // 1. Check config files (highest priority)
    if let Some(result) = detect_from_config_files(root) {
        return Some(result);
    }

    // 2. Check package.json
    let package_json = root.join("package.json");
    if !package_json.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&package_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    // 2a. Check devDependencies
    if let Some(result) = detect_from_dependencies(&json) {
        return Some(result);
    }

    // 2b. Check scripts.test
    detect_from_test_script(&json)
}

fn detect_from_config_files(root: &Path) -> Option<DetectionResult> {
    // Vitest config files
    const VITEST_CONFIGS: &[&str] = &[
        "vitest.config.ts", "vitest.config.js", "vitest.config.mts", "vitest.config.mjs",
    ];
    for config in VITEST_CONFIGS {
        if root.join(config).exists() {
            return Some(DetectionResult {
                runner: JsRunner::Vitest,
                source: DetectionSource::ConfigFile(config.to_string()),
            });
        }
    }

    // Jest config files
    const JEST_CONFIGS: &[&str] = &[
        "jest.config.ts", "jest.config.js", "jest.config.mjs", "jest.config.json",
    ];
    for config in JEST_CONFIGS {
        if root.join(config).exists() {
            return Some(DetectionResult {
                runner: JsRunner::Jest,
                source: DetectionSource::ConfigFile(config.to_string()),
            });
        }
    }

    // Bun config (bun.toml with [test] section - less common)
    // Bun detection primarily via dependencies/scripts

    None
}

fn detect_from_dependencies(json: &serde_json::Value) -> Option<DetectionResult> {
    let dev_deps = json.get("devDependencies")?;

    // Check in priority order
    if dev_deps.get("vitest").is_some() {
        return Some(DetectionResult {
            runner: JsRunner::Vitest,
            source: DetectionSource::DevDependency("vitest".to_string()),
        });
    }

    if dev_deps.get("jest").is_some() {
        return Some(DetectionResult {
            runner: JsRunner::Jest,
            source: DetectionSource::DevDependency("jest".to_string()),
        });
    }

    // Bun is typically used as a runtime, not a devDependency
    // Check dependencies too for bun-specific test setup
    let deps = json.get("dependencies");
    if dev_deps.get("bun-types").is_some() || deps.and_then(|d| d.get("bun-types")).is_some() {
        return Some(DetectionResult {
            runner: JsRunner::Bun,
            source: DetectionSource::DevDependency("bun-types".to_string()),
        });
    }

    None
}

fn detect_from_test_script(json: &serde_json::Value) -> Option<DetectionResult> {
    let test_script = json
        .get("scripts")?
        .get("test")?
        .as_str()?;

    // Parse the test script command
    if test_script.contains("vitest") {
        return Some(DetectionResult {
            runner: JsRunner::Vitest,
            source: DetectionSource::TestScript(test_script.to_string()),
        });
    }

    if test_script.contains("jest") {
        return Some(DetectionResult {
            runner: JsRunner::Jest,
            source: DetectionSource::TestScript(test_script.to_string()),
        });
    }

    if test_script.contains("bun test") {
        return Some(DetectionResult {
            runner: JsRunner::Bun,
            source: DetectionSource::TestScript(test_script.to_string()),
        });
    }

    None
}
```

**Update:** `crates/cli/src/checks/tests/runners/mod.rs`

```rust
mod js_detect;
pub use js_detect::{JsRunner, DetectionResult, DetectionSource, detect_js_runner};
```

**Milestone:** Unit tests pass for detection logic.

---

### Phase 2: Tests Check Integration

Integrate auto-detection into the tests check when no suites are configured.

**Update:** `crates/cli/src/checks/tests/mod.rs`

Add auto-detection in `run_test_suites()`:

```rust
use self::runners::{detect_js_runner, JsRunner};

impl TestsCheck {
    fn run_test_suites(&self, ctx: &CheckContext) -> CheckResult {
        // If suites are explicitly configured, use them
        if !ctx.config.check.tests.suite.is_empty() {
            return self.run_configured_suites(ctx);
        }

        // Auto-detect JavaScript runner if JS project detected
        if let Some(detected) = self.auto_detect_js_suite(ctx) {
            return self.run_auto_detected_suite(ctx, detected);
        }

        // No suites to run
        CheckResult::passed(self.name())
    }

    /// Auto-detect JavaScript test runner.
    fn auto_detect_js_suite(&self, ctx: &CheckContext) -> Option<TestSuiteConfig> {
        // Only auto-detect if package.json exists
        if !ctx.root.join("package.json").exists() {
            return None;
        }

        let detection = detect_js_runner(ctx.root)?;

        Some(TestSuiteConfig {
            runner: detection.runner.name().to_string(),
            name: Some(format!("{} (auto-detected)", detection.runner.name())),
            path: None,
            setup: None,
            command: None,
            targets: vec![],
            ci: false,
            max_total: None,
            max_avg: None,
            max_test: None,
            timeout: None,
        })
    }

    fn run_auto_detected_suite(&self, ctx: &CheckContext, suite: TestSuiteConfig) -> CheckResult {
        // Run with single auto-detected suite
        let runner_ctx = RunnerContext {
            root: ctx.root,
            ci_mode: ctx.ci_mode,
            collect_coverage: ctx.ci_mode,
        };

        let result = Self::run_single_suite(&suite, &runner_ctx);

        // Build metrics similar to run_configured_suites
        let metrics = json!({
            "test_count": result.test_count,
            "total_ms": result.total_ms,
            "auto_detected": true,
            "runner": suite.runner,
            "suites": [/* ... */]
        });

        if result.passed || result.skipped {
            CheckResult::passed(self.name()).with_metrics(metrics)
        } else {
            let violation = Violation::file_only(
                format!("<suite:{}>", result.name),
                "test_suite_failed",
                result.error.unwrap_or_else(|| "test suite failed".to_string()),
            );
            CheckResult::failed(self.name(), vec![violation]).with_metrics(metrics)
        }
    }
}
```

**Milestone:** Auto-detection triggers when package.json present and no suites configured.

---

### Phase 3: Test Fixture Update

Update `fixtures/js-simple` to have a real test runner for integration testing.

**Update:** `tests/fixtures/js-simple/package.json`

```json
{
  "name": "js-simple",
  "version": "1.0.0",
  "type": "module",
  "main": "src/index.ts",
  "scripts": {
    "test": "vitest run"
  },
  "devDependencies": {
    "vitest": "^2.0.0",
    "typescript": "^5.0.0"
  }
}
```

**Create:** `tests/fixtures/js-simple/vitest.config.ts`

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/**/*.test.ts'],
  },
});
```

**Create:** `tests/fixtures/js-simple/tests/index.test.ts`

```typescript
import { describe, it, expect } from 'vitest';
import { greet } from '../src/index';

describe('greet', () => {
  it('returns greeting message', () => {
    expect(greet('World')).toBe('Hello, World!');
  });
});
```

**Update:** `tests/fixtures/js-simple/src/index.ts`

```typescript
export function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

**Milestone:** `npm install && npm test` passes in js-simple fixture.

---

### Phase 4: Integration Tests

Add behavioral specs for JS runner auto-detection.

**Create:** `tests/specs/checks/tests/js_runners.rs`

```rust
//! JavaScript test runner auto-detection specs.

use crate::prelude::*;

#[test]
fn auto_detects_vitest_from_config_file() {
    check("tests")
        .on("js-simple")
        .passes()
        .json_has(".auto_detected", "true")
        .json_has(".runner", "vitest");
}

#[test]
fn auto_detects_jest_from_devdependencies() {
    check("tests")
        .on("js-jest-project")  // Need to create this fixture
        .passes()
        .json_has(".runner", "jest");
}

#[test]
fn explicit_config_takes_precedence() {
    // When quench.toml has [[check.tests.suite]], don't auto-detect
    check("tests")
        .on("js-configured")
        .passes()
        .json_missing(".auto_detected");
}

#[test]
#[ignore = "TODO: requires npm install"]
fn runs_vitest_and_collects_metrics() {
    check("tests")
        .on("js-simple")
        .with_setup("npm install")
        .passes()
        .json_has(".test_count");
}
```

**Update:** `tests/specs/checks/tests/mod.rs`

```rust
mod js_runners;
```

**Milestone:** All integration tests pass.

---

## Key Implementation Details

### Detection Priority

1. **Config files** (highest confidence) - User explicitly configured the runner
2. **devDependencies** - Runner is installed in the project
3. **scripts.test** - Last resort, parses the test command

### Config File Patterns

| Runner | Config Files |
|--------|--------------|
| Vitest | `vitest.config.{ts,js,mts,mjs}` |
| Jest | `jest.config.{ts,js,mjs,json}` |
| Bun | Primary detection via dependencies/scripts |

### Edge Cases

- **Monorepo:** Auto-detect per-package if workspace detected
- **No runner found:** Fall back to commit correlation check (existing behavior)
- **Multiple signals:** Config file wins over dependencies, dependencies win over scripts
- **Bun runtime:** Detect via `bun test` in scripts or `bun-types` dependency

### Metrics Output

Auto-detected suites include an `auto_detected: true` field in metrics:

```json
{
  "test_count": 5,
  "total_ms": 250,
  "auto_detected": true,
  "runner": "vitest",
  "detection_source": "config_file:vitest.config.ts"
}
```

## Verification Plan

### Unit Tests

1. `js_detect_tests.rs` - Detection logic tests
   - Detects vitest from config file
   - Detects jest from devDependencies
   - Detects bun from scripts.test
   - Returns None when no runner detected
   - Prioritizes config file over dependencies

### Integration Tests

2. `tests/specs/checks/tests/js_runners.rs`
   - Auto-detection triggers on JS project
   - Explicit config takes precedence
   - Metrics include auto_detected flag

### Manual Testing

3. Verify on real projects:
   ```bash
   # Test on js-simple fixture
   cd tests/fixtures/js-simple
   npm install
   cd ../../..
   cargo run -- check --tests tests/fixtures/js-simple

   # Should see: "Running vitest (auto-detected)"
   ```

### CI Verification

4. Run full test suite:
   ```bash
   make check
   ```

## Summary

| Phase | Deliverable | Verification |
|-------|-------------|--------------|
| 1 | Detection module | Unit tests pass |
| 2 | Tests check integration | Auto-detection triggers |
| 3 | Updated fixture | `npm test` passes |
| 4 | Integration tests | All specs pass |
