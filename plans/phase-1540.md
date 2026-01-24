# Phase 1540: [lang.cloc] Behavioral Specs

**Root Feature:** `quench-init` / Config Schema

## Overview

Write behavioral specs for per-language cloc configuration. Each language adapter (rust, golang, javascript, shell) supports its own `[lang.cloc]` section with `check` and `advice` fields that can override global cloc settings.

These specs document the expected behavior. Implementation is Phase 1542.

**Prerequisite:** Phase 1535 (Agent Config Output) completed.

## Project Structure

Files to create/modify:

```
tests/specs/checks/
├── cloc.rs              # Existing global cloc specs
└── cloc_lang.rs         # NEW: Per-language cloc specs

tests/specs/checks/mod.rs  # Add mod cloc_lang

tests/fixtures/
├── cloc-lang/
│   ├── rust-off/        # Rust cloc disabled
│   ├── rust-warn/       # Rust cloc warns only
│   ├── rust-advice/     # Rust custom advice
│   ├── golang-off/      # Go cloc disabled
│   ├── golang-warn/     # Go cloc warns only
│   ├── javascript-off/  # JS cloc disabled
│   ├── shell-off/       # Shell cloc disabled
│   ├── mixed-levels/    # Different languages with different levels
│   └── inherits/        # Unset lang.cloc inherits from check.cloc
```

Reference files:

```
docs/specs/langs/rust.md#configuration ([rust.cloc] section)
docs/specs/langs/golang.md#configuration ([golang.cloc] section)
docs/specs/langs/javascript.md#configuration ([javascript.cloc] section)
docs/specs/langs/shell.md#configuration ([shell.cloc] section)
docs/specs/10-language-adapters.md
```

## Dependencies

No new dependencies. Uses existing test infrastructure.

## Implementation Phases

### Phase 1: Create Test Fixtures

Create fixtures for per-language cloc testing.

**`tests/fixtures/cloc-lang/rust-off/`**

Rust project where `rust.cloc.check = "off"` should skip Rust files:

```toml
# quench.toml
version = 1
[check.cloc]
max_lines = 5

[rust.cloc]
check = "off"
```

```rust
// src/big.rs (10 lines - would fail if checked)
fn a() {}
fn b() {}
fn c() {}
fn d() {}
fn e() {}
fn f() {}
fn g() {}
fn h() {}
fn i() {}
fn j() {}
```

```toml
# Cargo.toml
[package]
name = "test"
version = "0.1.0"
```

**`tests/fixtures/cloc-lang/rust-warn/`**

Rust project where `rust.cloc.check = "warn"` reports but passes:

```toml
# quench.toml
version = 1
[check.cloc]
max_lines = 5

[rust.cloc]
check = "warn"
```

```rust
// src/big.rs (10 lines - should warn but not fail)
fn a() {}
fn b() {}
fn c() {}
fn d() {}
fn e() {}
fn f() {}
fn g() {}
fn h() {}
fn i() {}
fn j() {}
```

**`tests/fixtures/cloc-lang/rust-advice/`**

Rust project with custom advice:

```toml
# quench.toml
version = 1
[check.cloc]
max_lines = 5

[rust.cloc]
check = "error"
advice = "Rust files should be split into smaller modules."
```

```rust
// src/big.rs (10 lines - triggers custom advice)
fn a() {}
fn b() {}
fn c() {}
fn d() {}
fn e() {}
fn f() {}
fn g() {}
fn h() {}
fn i() {}
fn j() {}
```

**`tests/fixtures/cloc-lang/mixed-levels/`**

Project with Rust (error), Go (warn), JS (off):

```toml
# quench.toml
version = 1
[check.cloc]
max_lines = 5

[rust.cloc]
check = "error"

[golang.cloc]
check = "warn"

[javascript.cloc]
check = "off"
```

With oversized files in each language.

**`tests/fixtures/cloc-lang/inherits/`**

Project where language sections are unset - should inherit from global:

```toml
# quench.toml
version = 1
[check.cloc]
check = "error"
max_lines = 5

# No [rust.cloc] section - inherits check = "error"
```

### Phase 2: Create Spec Module

Create `tests/specs/checks/cloc_lang.rs`:

```rust
//! Behavioral specs for per-language CLOC configuration.
//!
//! Tests that quench correctly:
//! - Respects {lang}.cloc.check = "off" to disable cloc for that language
//! - Respects {lang}.cloc.check = "warn" to report without failing
//! - Uses {lang}.cloc.advice for custom violation advice
//! - Allows independent check levels per language
//! - Falls back to check.cloc.check when {lang}.cloc.check is unset
//!
//! Reference: docs/specs/langs/{rust,golang,javascript,shell}.md

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// RUST CLOC CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#configuration
///
/// > [rust.cloc]
/// > check = "error" | "warn" | "off"
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn rust_cloc_check_off_skips_rust_files() {
    // Fixture has oversized Rust file but rust.cloc.check = "off"
    check("cloc").on("cloc-lang/rust-off").passes();
}

/// Spec: docs/specs/langs/rust.md#configuration
///
/// > [rust.cloc]
/// > check = "warn" reports but doesn't fail
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn rust_cloc_check_warn_reports_without_failing() {
    check("cloc")
        .on("cloc-lang/rust-warn")
        .passes()
        .stdout_has("big.rs")
        .stdout_has("file_too_large");
}

/// Spec: docs/specs/langs/rust.md#configuration
///
/// > [rust.cloc]
/// > advice = "..." - Custom advice for oversized Rust files
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn rust_cloc_advice_overrides_default() {
    let cloc = check("cloc").on("cloc-lang/rust-advice").json().fails();
    let violations = cloc.require("violations").as_array().unwrap();

    let advice = violations[0]
        .get("advice")
        .and_then(|a| a.as_str())
        .unwrap();
    assert_eq!(advice, "Rust files should be split into smaller modules.");
}

// =============================================================================
// GOLANG CLOC CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/golang.md#configuration
///
/// > [golang.cloc]
/// > check = "off" disables cloc for Go files
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn golang_cloc_check_off_skips_go_files() {
    check("cloc").on("cloc-lang/golang-off").passes();
}

/// Spec: docs/specs/langs/golang.md#configuration
///
/// > [golang.cloc]
/// > check = "warn" reports but doesn't fail
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn golang_cloc_check_warn_reports_without_failing() {
    check("cloc")
        .on("cloc-lang/golang-warn")
        .passes()
        .stdout_has("big.go")
        .stdout_has("file_too_large");
}

// =============================================================================
// JAVASCRIPT CLOC CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/javascript.md#configuration
///
/// > [javascript.cloc]
/// > check = "off" disables cloc for JS/TS files
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn javascript_cloc_check_off_skips_js_files() {
    check("cloc").on("cloc-lang/javascript-off").passes();
}

// =============================================================================
// SHELL CLOC CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/shell.md#configuration
///
/// > [shell.cloc]
/// > check = "off" disables cloc for shell scripts
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn shell_cloc_check_off_skips_shell_files() {
    check("cloc").on("cloc-lang/shell-off").passes();
}

// =============================================================================
// INDEPENDENT CHECK LEVEL SPECS
// =============================================================================

/// Spec: docs/specs/10-language-adapters.md
///
/// > Each language can have independent cloc check level
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn each_language_can_have_independent_cloc_check_level() {
    // Fixture has: rust=error (fails), golang=warn (reports), javascript=off (skipped)
    let result = check("cloc").on("cloc-lang/mixed-levels").json().fails();

    // Rust file should cause failure
    let violations = result.require("violations").as_array().unwrap();
    assert!(violations.iter().any(|v| {
        v.get("file")
            .and_then(|f| f.as_str())
            .map(|f| f.ends_with(".rs"))
            .unwrap_or(false)
    }));

    // Go file should not appear in violations (only warned)
    // JS file should not appear at all (skipped)
    assert!(!violations.iter().any(|v| {
        v.get("file")
            .and_then(|f| f.as_str())
            .map(|f| f.ends_with(".js"))
            .unwrap_or(false)
    }));
}

/// Spec: docs/specs/10-language-adapters.md
///
/// > Mixed project: Go file over limit warns, Rust file over limit fails
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn mixed_levels_go_warn_rust_error() {
    // When golang.cloc.check = "warn" and rust.cloc.check = "error"
    // Go violations should be reported but not cause failure
    // Rust violations should cause failure
    check("cloc")
        .on("cloc-lang/mixed-levels")
        .fails()
        .stdout_has("big.rs")
        .stdout_has("file_too_large");
}

// =============================================================================
// INHERITANCE SPECS
// =============================================================================

/// Spec: docs/specs/10-language-adapters.md
///
/// > Unset {lang}.cloc.check inherits from check.cloc.check
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn unset_lang_cloc_inherits_from_global() {
    // Fixture has check.cloc.check = "error" but no [rust.cloc] section
    // Rust files should still be checked with error level
    check("cloc").on("cloc-lang/inherits").fails();
}

/// Spec: docs/specs/10-language-adapters.md
///
/// > Global check.cloc.check = "off" disables all languages unless overridden
#[test]
#[ignore = "TODO: Phase 1542 - Per-language cloc config"]
fn global_off_disables_all_unless_overridden() {
    let temp = Project::empty();
    temp.config(
        r#"[check.cloc]
check = "off"
max_lines = 5

[rust.cloc]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    );
    // Oversized Rust file should fail (rust.cloc overrides global off)
    temp.file(
        "src/big.rs",
        "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\nfn f() {}\n",
    );
    // Oversized Go file should pass (inherits global off)
    temp.file("go.mod", "module test\n");
    temp.file(
        "big.go",
        "package main\nfunc a() {}\nfunc b() {}\nfunc c() {}\nfunc d() {}\nfunc e() {}\nfunc f() {}\n",
    );

    // Should fail only because of Rust file
    let result = check("cloc").pwd(temp.path()).json().fails();
    let violations = result.require("violations").as_array().unwrap();

    // Only Rust violation, not Go
    assert!(violations.iter().all(|v| {
        v.get("file")
            .and_then(|f| f.as_str())
            .map(|f| f.ends_with(".rs"))
            .unwrap_or(false)
    }));
}
```

### Phase 3: Update Module Declarations

Add to `tests/specs/checks/mod.rs`:

```rust
mod cloc_lang;
```

### Phase 4: Create All Fixtures

Create remaining fixture directories with appropriate content:

**`tests/fixtures/cloc-lang/golang-off/`**
- `quench.toml` with `[golang.cloc] check = "off"`
- `go.mod`
- Oversized `big.go`

**`tests/fixtures/cloc-lang/golang-warn/`**
- `quench.toml` with `[golang.cloc] check = "warn"`
- `go.mod`
- Oversized `big.go`

**`tests/fixtures/cloc-lang/javascript-off/`**
- `quench.toml` with `[javascript.cloc] check = "off"`
- `package.json`
- Oversized `src/big.js`

**`tests/fixtures/cloc-lang/shell-off/`**
- `quench.toml` with `[shell.cloc] check = "off"`
- Oversized `scripts/big.sh`

**`tests/fixtures/cloc-lang/mixed-levels/`**
- `quench.toml` with rust=error, golang=warn, javascript=off
- `Cargo.toml`, `go.mod`, `package.json`
- Oversized files in each language

**`tests/fixtures/cloc-lang/inherits/`**
- `quench.toml` with `[check.cloc] check = "error"` only
- `Cargo.toml`
- Oversized `src/big.rs`

### Phase 5: Run Verification

```bash
# Compile specs (should pass - all are ignored)
cargo test --test specs cloc_lang -- --list

# Show unimplemented count
cargo test --test specs cloc_lang -- --ignored

# Full check
make check
```

## Key Implementation Details

### Config Hierarchy

The cloc check level follows this precedence:
1. `{lang}.cloc.check` - Most specific (per-language)
2. `check.cloc.check` - Global default
3. Default (`"error"`) - When nothing is configured

### Advice Override

Per-language advice (`{lang}.cloc.advice`) completely replaces the default advice for that language's files. It does not append or merge.

### Warn Level Behavior

When `check = "warn"`:
- Violations are still reported in output
- Exit code is 0 (pass)
- Metrics are still collected
- JSON output includes violations with level: "warn"

### Per-Language File Classification

Files are classified to a language adapter based on:
1. Extension (`.rs` → rust, `.go` → golang, etc.)
2. Detection markers (Cargo.toml → rust, go.mod → golang)

A file belongs to exactly one language adapter. The cloc check level for that adapter applies.

## Verification Plan

### 1. Compile Check

```bash
cargo test --test specs cloc_lang -- --list
```

Expected: All tests listed, none fail to compile.

### 2. Ignored Count

```bash
cargo test --test specs cloc_lang -- --ignored 2>&1 | grep -c "ignored"
```

Expected: 12 ignored tests (the full spec count for Phase 1542).

### 3. Fixtures Exist

```bash
ls tests/fixtures/cloc-lang/
```

Expected: All fixture directories present.

### 4. Full Check

```bash
make check
```

Expected: All checks pass.

### Spec Coverage

| Roadmap Item | Test Function | Status |
|--------------|---------------|--------|
| `{lang}.cloc.check = "off"` disables | `*_cloc_check_off_skips_*` (4 tests) | Spec written |
| `{lang}.cloc.check = "warn"` reports | `*_cloc_check_warn_*` (2 tests) | Spec written |
| `{lang}.cloc.advice` overrides | `rust_cloc_advice_overrides_default` | Spec written |
| Independent check levels | `each_language_can_have_independent_*` | Spec written |
| Unset inherits from global | `unset_lang_cloc_inherits_*` | Spec written |
