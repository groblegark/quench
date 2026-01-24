//! Behavioral specs for per-language policy check level.
//!
//! Tests that quench correctly:
//! - Respects {lang}.policy.check = "off" to disable policy for that language
//! - Respects {lang}.policy.check = "warn" to report without failing
//! - Allows independent check levels per language
//!
//! Reference: docs/specs/langs/{rust,golang,javascript,shell}.md

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// RUST POLICY CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/rust.md#policy
///
/// > [rust.policy]
/// > check = "off" disables policy for Rust files
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn rust_policy_check_off_disables_policy() {
    // Fixture has lint config + source changes but rust.policy.check = "off"
    // Should pass even though standalone policy would normally fail
    check("escapes").on("policy-lang/rust-off").passes();
}

/// Spec: docs/specs/langs/rust.md#policy
///
/// > [rust.policy]
/// > check = "warn" reports but doesn't fail
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn rust_policy_check_warn_reports_without_failing() {
    check("escapes")
        .on("policy-lang/rust-warn")
        .passes()
        .stdout_has("lint config changes must be standalone");
}

// =============================================================================
// GOLANG POLICY CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/golang.md#policy
///
/// > [golang.policy]
/// > check = "off" disables policy for Go files
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn golang_policy_check_off_disables_policy() {
    check("escapes").on("policy-lang/golang-off").passes();
}

/// Spec: docs/specs/langs/golang.md#policy
///
/// > [golang.policy]
/// > check = "warn" reports but doesn't fail
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn golang_policy_check_warn_reports_without_failing() {
    check("escapes")
        .on("policy-lang/golang-warn")
        .passes()
        .stdout_has("lint config changes must be standalone");
}

// =============================================================================
// JAVASCRIPT POLICY CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/javascript.md#policy
///
/// > [javascript.policy]
/// > check = "off" disables policy for JS/TS files
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn javascript_policy_check_off_disables_policy() {
    check("escapes").on("policy-lang/javascript-off").passes();
}

// =============================================================================
// SHELL POLICY CONFIG SPECS
// =============================================================================

/// Spec: docs/specs/langs/shell.md#policy
///
/// > [shell.policy]
/// > check = "off" disables policy for shell scripts
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn shell_policy_check_off_disables_policy() {
    check("escapes").on("policy-lang/shell-off").passes();
}

// =============================================================================
// INDEPENDENT CHECK LEVEL SPECS
// =============================================================================

/// Spec: docs/specs/10-language-adapters.md
///
/// > Each language can have independent policy check level
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn each_language_can_have_independent_policy_check_level() {
    // Fixture has: rust=error (fails), golang=warn (reports), javascript=off (skipped)
    let result = check("escapes")
        .on("policy-lang/mixed-levels")
        .json()
        .fails();

    // Rust policy violation should cause failure
    let violations = result.require("violations").as_array().unwrap();
    assert!(violations.iter().any(|v| {
        v.get("type")
            .and_then(|t| t.as_str())
            .map(|t| t.contains("lint_config"))
            .unwrap_or(false)
    }));
}

/// Spec: docs/specs/10-language-adapters.md
///
/// > Mixed project: Go policy warns, Rust policy errors
#[test]
#[ignore = "TODO: Phase 1547 - Per-language policy config"]
fn mixed_levels_go_warn_rust_error() {
    // When golang.policy.check = "warn" and rust.policy.check = "error"
    // Go violations should be reported but not cause failure
    // Rust violations should cause failure
    check("escapes")
        .on("policy-lang/mixed-levels")
        .fails()
        .stdout_has("lint config changes must be standalone");
}
