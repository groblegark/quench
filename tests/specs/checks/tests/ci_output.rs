//! Behavioral specs for CI mode exact output format.
//!
//! Reference: docs/specs/11-test-runners.md#ci-output-format

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// TEXT OUTPUT FORMAT
// =============================================================================

/// Spec: CI mode text output shows test results summary.
///
/// > CI mode should output "PASS: tests" on success.
#[test]
fn tests_ci_text_output_passes() {
    check("tests")
        .on("tests-ci")
        .args(&["--ci"])
        .passes()
        .stdout_has("PASS: tests");
}

// =============================================================================
// JSON OUTPUT STRUCTURE
// =============================================================================

/// Spec: CI mode JSON output includes timing metrics structure.
///
/// > CI mode JSON should include test_count, total_ms, and suites array
/// > with required fields.
#[test]
fn tests_ci_json_output_timing_structure() {
    let result = check("tests")
        .on("tests-ci")
        .args(&["--ci"])
        .json()
        .passes();
    let metrics = result.require("metrics");

    // Verify required fields
    assert!(metrics.get("test_count").is_some());
    assert!(metrics.get("total_ms").is_some());
    let suites = metrics.get("suites").unwrap().as_array().unwrap();
    assert!(!suites.is_empty());

    // Suite should have required fields
    let suite = &suites[0];
    assert!(suite.get("name").is_some());
    assert!(suite.get("runner").is_some());
    assert!(suite.get("passed").is_some());
    assert!(suite.get("test_count").is_some());
    assert!(suite.get("total_ms").is_some());
}

// =============================================================================
// THRESHOLD VIOLATION OUTPUT
// =============================================================================

/// Spec: CI mode text output shows threshold violations.
///
/// > Timing violations should display the violation type and exceeded limit.
#[test]
fn tests_ci_text_output_timing_violation() {
    let temp = Project::empty();
    temp.config(
        r#"
[[check.tests.suite]]
runner = "cargo"
max_total = "1ms"

[check.tests.time]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    temp.file(
        "tests/basic.rs",
        r#"
#[test]
fn test_add() { assert_eq!(test_project::add(1, 2), 3); }
"#,
    );

    check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .fails()
        .stdout_has("time_total_exceeded")
        .stdout_has("exceeds max_total");
}

/// Spec: CI violation has threshold and value fields.
///
/// > JSON violations for CI thresholds should include threshold and value.
#[test]
fn tests_ci_json_violation_has_threshold_and_value() {
    let temp = Project::empty();
    temp.config(
        r#"
[[check.tests.suite]]
runner = "cargo"
max_total = "1ms"

[check.tests.time]
check = "error"
"#,
    );
    temp.file(
        "Cargo.toml",
        r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
    );
    temp.file("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    temp.file(
        "tests/basic.rs",
        r#"
#[test]
fn test_add() { assert_eq!(test_project::add(1, 2), 3); }
"#,
    );

    let result = check("tests")
        .pwd(temp.path())
        .args(&["--ci"])
        .json()
        .fails();

    let v = result.require_violation("time_total_exceeded");
    assert!(v.get("threshold").is_some());
    assert!(v.get("value").is_some());
}
