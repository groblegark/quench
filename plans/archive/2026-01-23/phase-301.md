# Phase 301: Rust Adapter - Specs

**Root Feature:** `quench-a0ea`

## Overview

Write behavioral specifications (tests) for the Rust language adapter. This phase defines expected behavior through ignored specs that will drive subsequent implementation phases. The Rust adapter provides:

- **Auto-detection** via `Cargo.toml`
- **Default patterns** for Rust source files
- **Inline test detection** via `#[cfg(test)]` blocks
- **Rust-specific escape patterns** (unsafe, unwrap, allow)
- **Lint config policy** enforcement

Reference docs:
- `docs/specs/langs/rust.md`
- `docs/specs/10-language-adapters.md`
- `docs/specs/checks/escape-hatches.md`

## Project Structure

```
quench/
├── tests/
│   ├── specs/
│   │   ├── adapters/           # NEW: Adapter specs
│   │   │   ├── mod.rs
│   │   │   └── rust.rs         # Rust adapter behavioral specs
│   │   └── main.rs             # Add adapters module
│   └── fixtures/
│       └── rust/               # NEW: Rust-specific fixtures
│           ├── auto-detect/    # Has Cargo.toml, no quench.toml
│           ├── cfg-test/       # #[cfg(test)] inline tests
│           ├── unsafe-ok/      # SAFETY comments present
│           ├── unsafe-fail/    # Missing SAFETY comments
│           ├── unwrap-source/  # .unwrap() in source (fails)
│           ├── unwrap-test/    # .unwrap() in tests (passes)
│           ├── allow-ok/       # #[allow] with comment
│           ├── allow-fail/     # #[allow] without comment
│           ├── workspace-auto/ # Auto-detect workspace packages
│           └── lint-policy/    # Lint config with source changes
└── plans/
    └── phase-301.md
```

## Dependencies

No new external dependencies. Uses existing:
- Test harness from `tests/specs/prelude.rs`
- Fixture infrastructure from `tests/fixtures/`
- Adapter trait from `crates/cli/src/adapter/mod.rs`

## Implementation Phases

### Phase 1: Spec Module Setup

Create the specs module structure for adapter tests.

**Create `tests/specs/adapters/mod.rs`:**

```rust
//! Behavioral specs for language adapters.
//!
//! Tests that quench correctly detects and applies language-specific behavior.
//!
//! Reference: docs/specs/10-language-adapters.md

pub mod rust;
```

**Update `tests/specs/main.rs`** to include adapters:

```rust
mod adapters;
```

**Milestone:** Module compiles, no tests yet.

**Verification:**
```bash
cargo test --test specs -- adapters
```

---

### Phase 2: Auto-Detection Specs

Write specs for Rust project detection.

**Create `tests/specs/adapters/rust.rs`:**

```rust
//! Behavioral specs for the Rust language adapter.
//!
//! Tests that quench correctly:
//! - Detects Rust projects via Cargo.toml
//! - Applies default source/test patterns
//! - Handles inline #[cfg(test)] blocks
//! - Applies Rust-specific escape patterns
//!
//! Reference: docs/specs/langs/rust.md

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// AUTO-DETECTION SPECS
// =============================================================================

/// Spec: docs/specs/10-language-adapters.md#adapter-selection
///
/// > rust | Cargo.toml exists | **/*.rs
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_auto_detected_when_cargo_toml_present() {
    // Project has Cargo.toml but no quench.toml [rust] section
    // Should still apply Rust defaults
    let result = cli().on("rust/auto-detect").json().passes();
    let checks = result.checks();

    // escapes check should have rust-specific patterns active
    // (will verify by checking that .unwrap() is detected)
    assert!(checks.iter().any(|c| c.get("name").and_then(|n| n.as_str()) == Some("escapes")));
}

/// Spec: docs/specs/langs/rust.md#default-patterns
///
/// > source = ["**/*.rs"]
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_default_source_pattern_matches_rs_files() {
    let cloc = check("cloc").on("rust/auto-detect").json().passes();
    let metrics = cloc.require("metrics");

    // Should count .rs files as source
    let source_loc = metrics.get("source_loc").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(source_loc > 0, "should count .rs files as source");
}

/// Spec: docs/specs/langs/rust.md#default-patterns
///
/// > ignore = ["target/"]
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_default_ignores_target_directory() {
    // Fixture has files in target/ that should be ignored
    let cloc = check("cloc").on("rust/auto-detect").json().passes();
    let files = cloc.get("files").and_then(|f| f.as_array());

    if let Some(files) = files {
        assert!(
            !files.iter().any(|f| {
                f.as_str().map(|s| s.contains("target/")).unwrap_or(false)
            }),
            "target/ directory should be ignored"
        );
    }
}
```

**Milestone:** Auto-detection specs compile and are ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter -- --ignored 2>&1 | grep "3 ignored"
```

---

### Phase 3: Workspace Detection Specs

Write specs for Cargo workspace package detection.

**Add to `tests/specs/adapters/rust.rs`:**

```rust
// =============================================================================
// WORKSPACE DETECTION SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#default-patterns
///
/// > Detected when Cargo.toml exists in project root.
/// > Auto-detects workspace packages from Cargo.toml [workspace] members.
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_detects_workspace_packages_from_cargo_toml() {
    // Fixture has Cargo.toml with [workspace] members = ["crates/*"]
    let cloc = check("cloc").on("rust/workspace-auto").json().passes();
    let by_package = cloc.get("by_package");

    assert!(by_package.is_some(), "should have by_package breakdown");
    let by_package = by_package.unwrap();

    // Should detect packages from workspace members
    assert!(by_package.get("core").is_some(), "should detect 'core' package");
    assert!(by_package.get("cli").is_some(), "should detect 'cli' package");
}
```

**Milestone:** Workspace spec compiles and is ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter_detects -- --ignored
```

---

### Phase 4: Test Code Detection Specs

Write specs for `#[cfg(test)]` inline test detection.

**Add to `tests/specs/adapters/rust.rs`:**

```rust
// =============================================================================
// TEST CODE DETECTION SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#test-code-detection
///
/// > Lines inside #[cfg(test)] blocks are counted as test LOC
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_cfg_test_blocks_counted_as_test_loc() {
    let cloc = check("cloc").on("rust/cfg-test").json().passes();
    let metrics = cloc.require("metrics");

    // Source file has both source and #[cfg(test)] code
    let source_loc = metrics.get("source_loc").and_then(|v| v.as_u64()).unwrap_or(0);
    let test_loc = metrics.get("test_loc").and_then(|v| v.as_u64()).unwrap_or(0);

    assert!(source_loc > 0, "should have source LOC");
    assert!(test_loc > 0, "should have test LOC from #[cfg(test)]");
}

/// Spec: docs/specs/langs/rust.md#test-code-detection
///
/// > Configurable: cfg_test_split = true (default)
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_cfg_test_split_can_be_disabled() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust]
cfg_test_split = false
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    #[test]
    fn test_add() { assert_eq!(super::add(1, 2), 3); }
}
"#,
    ).unwrap();

    let cloc = check("cloc").pwd(dir.path()).json().passes();
    let metrics = cloc.require("metrics");

    // With cfg_test_split = false, all lines should be counted as source
    let test_loc = metrics.get("test_loc").and_then(|v| v.as_u64()).unwrap_or(0);
    assert_eq!(test_loc, 0, "should not split #[cfg(test)] when disabled");
}
```

**Milestone:** Test detection specs compile and are ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter -- cfg_test --ignored
```

---

### Phase 5: Rust Escape Pattern Specs

Write specs for Rust-specific escape patterns.

**Add to `tests/specs/adapters/rust.rs`:**

```rust
// =============================================================================
// ESCAPE PATTERN SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#default-escape-patterns
///
/// > unsafe { } | comment | // SAFETY:
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_unsafe_without_safety_comment_fails() {
    check("escapes")
        .on("rust/unsafe-fail")
        .fails()
        .stdout_has("escapes: FAIL")
        .stdout_has("// SAFETY:");
}

/// Spec: docs/specs/langs/rust.md#default-escape-patterns
///
/// > unsafe { } | comment | // SAFETY:
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_unsafe_with_safety_comment_passes() {
    check("escapes").on("rust/unsafe-ok").passes();
}

/// Spec: docs/specs/langs/rust.md#default-escape-patterns
///
/// > .unwrap() | forbid | -
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_unwrap_in_source_code_fails() {
    let escapes = check("escapes").on("rust/unwrap-source").json().fails();
    let violations = escapes.require("violations").as_array().unwrap();

    assert!(
        violations.iter().any(|v| {
            v.get("pattern").and_then(|p| p.as_str()) == Some("unwrap")
        }),
        "should have unwrap violation"
    );
}

/// Spec: docs/specs/checks/escape-hatches.md#forbid
///
/// > Always allowed in test code.
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_unwrap_in_test_code_allowed() {
    // .unwrap() only appears in test files or #[cfg(test)] blocks
    check("escapes").on("rust/unwrap-test").passes();
}

/// Spec: docs/specs/langs/rust.md#default-escape-patterns
///
/// > .expect( | forbid | -
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_expect_in_source_code_fails() {
    let dir = temp_project();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn f() { Some(1).expect(\"should have value\"); }",
    ).unwrap();

    check("escapes").pwd(dir.path()).fails();
}

/// Spec: docs/specs/langs/rust.md#default-escape-patterns
///
/// > mem::transmute | comment | // SAFETY:
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_transmute_without_safety_comment_fails() {
    let dir = temp_project();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "use std::mem; pub fn f() -> u64 { unsafe { mem::transmute(1i64) } }",
    ).unwrap();

    check("escapes")
        .pwd(dir.path())
        .fails()
        .stdout_has("// SAFETY:");
}
```

**Milestone:** Escape pattern specs compile and are ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter -- unsafe --ignored
cargo test --test specs rust_adapter -- unwrap --ignored
```

---

### Phase 6: Suppress (Allow/Expect) Specs

Write specs for `#[allow(...)]` and `#[expect(...)]` attribute handling.

**Add to `tests/specs/adapters/rust.rs`:**

```rust
// =============================================================================
// SUPPRESS ATTRIBUTE SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#suppress
///
/// > "comment" - Requires justification comment (default for source)
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_allow_without_comment_fails_when_configured() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.suppress]
check = "comment"
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "#[allow(dead_code)]\nfn unused() {}",
    ).unwrap();

    check("escapes")
        .pwd(dir.path())
        .fails()
        .stdout_has("#[allow");
}

/// Spec: docs/specs/langs/rust.md#suppress
///
/// > "comment" - Requires justification comment
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_allow_with_comment_passes() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.suppress]
check = "comment"
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "// This function is reserved for future use\n#[allow(dead_code)]\nfn unused() {}",
    ).unwrap();

    check("escapes").pwd(dir.path()).passes();
}

/// Spec: docs/specs/langs/rust.md#suppress
///
/// > [rust.suppress.test] check = "allow" - tests can suppress freely
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_allow_in_test_code_always_passes() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.suppress]
check = "comment"
[rust.suppress.test]
check = "allow"
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("tests")).unwrap();
    std::fs::write(
        dir.path().join("tests/test.rs"),
        "#[allow(unused)]\n#[test]\nfn test_something() {}",
    ).unwrap();

    check("escapes").pwd(dir.path()).passes();
}

/// Spec: docs/specs/langs/rust.md#suppress
///
/// > allow = ["dead_code"] - no comment needed for specific codes
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_allow_list_skips_comment_check() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.suppress]
check = "comment"
[rust.suppress.source]
allow = ["dead_code"]
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "#[allow(dead_code)]\nfn unused() {}",  // No comment, but dead_code is in allow list
    ).unwrap();

    check("escapes").pwd(dir.path()).passes();
}

/// Spec: docs/specs/langs/rust.md#suppress
///
/// > forbid = ["unsafe_code"] - never allowed
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_forbid_list_always_fails() {
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.suppress.source]
forbid = ["unsafe_code"]
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "// Even with comment, forbidden\n#[allow(unsafe_code)]\nfn allow_unsafe() {}",
    ).unwrap();

    check("escapes").pwd(dir.path()).fails();
}
```

**Milestone:** Suppress specs compile and are ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter -- allow --ignored
```

---

### Phase 7: Lint Config Policy Specs

Write specs for lint configuration change policy.

**Add to `tests/specs/adapters/rust.rs`:**

```rust
// =============================================================================
// LINT CONFIG POLICY SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#policy
///
/// > lint_changes = "standalone" - lint config changes must be standalone PRs
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_lint_config_changes_with_source_fails_standalone_policy() {
    // This requires git state - fixture has both rustfmt.toml and src changes staged
    check("escapes")
        .on("rust/lint-policy")
        .args(&["--base", "HEAD~1"])
        .fails()
        .stdout_has("lint config changes must be standalone");
}

/// Spec: docs/specs/langs/rust.md#policy
///
/// > lint_config = ["rustfmt.toml", ".rustfmt.toml", "clippy.toml", ".clippy.toml"]
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_lint_config_standalone_passes() {
    // Only lint config changed, no source files
    let dir = temp_project();
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[rust.policy]
lint_changes = "standalone"
lint_config = ["rustfmt.toml"]
"#,
    ).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::write(dir.path().join("rustfmt.toml"), "max_width = 100\n").unwrap();

    // Initialize git repo with initial commit
    // Then add only rustfmt.toml change
    // This would need git setup - may need fixture

    check("escapes").pwd(dir.path()).passes();
}
```

**Milestone:** Policy specs compile and are ignored.

**Verification:**
```bash
cargo test --test specs rust_adapter -- lint --ignored
```

---

### Phase 8: Create Test Fixtures

Create fixtures for the specs.

**Create `tests/fixtures/rust/auto-detect/`:**
```
rust/auto-detect/
├── Cargo.toml          # [package] name = "auto-detect"
├── src/
│   └── lib.rs          # Simple function, no escapes
└── target/
    └── debug/
        └── should_ignore.rs  # Should be ignored
```

**Create `tests/fixtures/rust/cfg-test/`:**
```
rust/cfg-test/
├── Cargo.toml
├── quench.toml         # version = 1
└── src/
    └── lib.rs          # Has #[cfg(test)] mod tests { ... }
```

**Create `tests/fixtures/rust/unsafe-ok/`:**
```
rust/unsafe-ok/
├── Cargo.toml
├── quench.toml
└── src/
    └── lib.rs          # // SAFETY: reason \n unsafe { }
```

**Create `tests/fixtures/rust/unsafe-fail/`:**
```
rust/unsafe-fail/
├── Cargo.toml
├── quench.toml
└── src/
    └── lib.rs          # unsafe { } without comment
```

**Create `tests/fixtures/rust/unwrap-source/`:**
```
rust/unwrap-source/
├── Cargo.toml
├── quench.toml
└── src/
    └── lib.rs          # Some(1).unwrap() in source
```

**Create `tests/fixtures/rust/unwrap-test/`:**
```
rust/unwrap-test/
├── Cargo.toml
├── quench.toml
└── src/
│   └── lib.rs          # Clean source, no unwrap
└── tests/
    └── test.rs         # Some(1).unwrap() in test (allowed)
```

**Create `tests/fixtures/rust/workspace-auto/`:**
```
rust/workspace-auto/
├── Cargo.toml          # [workspace] members = ["crates/*"]
├── quench.toml         # version = 1
└── crates/
    ├── core/
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── cli/
        ├── Cargo.toml
        └── src/main.rs
```

**Milestone:** All fixtures created and valid.

**Verification:**
```bash
ls tests/fixtures/rust/
# Should show: auto-detect cfg-test unsafe-ok unsafe-fail unwrap-source unwrap-test workspace-auto
```

---

### Phase 9: Verify All Specs Compile

Run all specs to ensure they compile and are properly ignored.

**Verification:**
```bash
# Compile check
cargo build --tests

# Run specs (all should be ignored)
cargo test --test specs rust_adapter 2>&1 | grep "ignored"

# Should show: ~15 ignored tests

# Full quality gates
make check
```

---

## Key Implementation Details

### Spec Naming Convention

All specs follow the pattern:
```
rust_adapter_{feature}_{condition}_{expected_result}
```

Examples:
- `rust_adapter_auto_detected_when_cargo_toml_present`
- `rust_adapter_unwrap_in_source_code_fails`
- `rust_adapter_cfg_test_blocks_counted_as_test_loc`

### Fixture Design

Fixtures are minimal but complete:
- Each fixture tests one specific behavior
- `Cargo.toml` is always present (required for Rust detection)
- `quench.toml` only included when testing config-specific behavior
- Source files are small (< 10 lines) to keep fixtures readable

### Test Code Boundaries

Per the spec, test code includes:
1. Files in `tests/` directory
2. Files matching `*_test.rs` or `*_tests.rs`
3. Lines inside `#[cfg(test)]` blocks (when `cfg_test_split = true`)

### Escape Pattern Defaults

When Rust adapter is active, these patterns are applied by default:

| Pattern | Action | Comment |
|---------|--------|---------|
| `unsafe\\s*\\{` | comment | `// SAFETY:` |
| `\\.unwrap\\(\\)` | forbid | - |
| `\\.expect\\(` | forbid | - |
| `mem::transmute` | comment | `// SAFETY:` |

### Policy Enforcement

The `lint_changes = "standalone"` policy requires:
1. Git diff detection (`--base` flag)
2. File categorization (lint config vs source)
3. Policy violation when both change in same commit

---

## Verification Plan

### After Each Phase

```bash
# Compile check
cargo build --tests

# Run relevant specs (should all be ignored)
cargo test --test specs rust_adapter -- --ignored

# Check for clippy warnings
cargo clippy --all-targets --all-features -- -D warnings
```

### End-to-End Verification

```bash
# All rust adapter specs (should be ~15 ignored)
cargo test --test specs rust_adapter 2>&1 | grep -E "^test.*ignored"

# Count total ignored
cargo test --test specs -- --ignored 2>&1 | tail -1

# Full quality gates
make check
```

### Test Matrix

| Spec Category | Count | Fixture Required |
|--------------|-------|------------------|
| Auto-detection | 3 | rust/auto-detect |
| Workspace | 1 | rust/workspace-auto |
| Test detection | 2 | rust/cfg-test |
| Unsafe pattern | 2 | rust/unsafe-ok, rust/unsafe-fail |
| Unwrap pattern | 3 | rust/unwrap-source, rust/unwrap-test |
| Suppress | 4 | temp_project() |
| Lint policy | 2 | rust/lint-policy |

---

## Summary

| Phase | Task | Key Files | Status |
|-------|------|-----------|--------|
| 1 | Spec module setup | `tests/specs/adapters/` | [ ] Pending |
| 2 | Auto-detection specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 3 | Workspace detection specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 4 | Test code detection specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 5 | Rust escape pattern specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 6 | Suppress attribute specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 7 | Lint config policy specs | `tests/specs/adapters/rust.rs` | [ ] Pending |
| 8 | Create test fixtures | `tests/fixtures/rust/` | [ ] Pending |
| 9 | Verify all specs compile | - | [ ] Pending |

## Future Phases

- **Phase 302**: Rust Adapter Implementation (remove `#[ignore]` attributes)
- **Phase 303**: Rust Build Metrics (binary size, build time)
- **Phase 304**: Rust Coverage Integration (cargo llvm-cov)
