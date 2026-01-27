// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::time::Duration;

use super::*;
use yare::parameterized;

// =============================================================================
// JSON TEST DATA
// =============================================================================

const PASSING_JSON: &str = r#"{
    "status": "pass",
    "tests": [
        {
            "name": "test_adds_two_numbers",
            "classname": "MathTest",
            "time": 0.001,
            "status": "pass"
        },
        {
            "name": "test_multiplies",
            "classname": "MathTest",
            "time": 0.002,
            "status": "pass"
        }
    ],
    "summary": {
        "total": 2,
        "passed": 2,
        "failed": 0,
        "skipped": 0,
        "time": 0.05
    }
}"#;

const FAILING_JSON: &str = r#"{
    "status": "fail",
    "tests": [
        {
            "name": "test_passes",
            "classname": "MathTest",
            "time": 0.001,
            "status": "pass"
        },
        {
            "name": "test_fails",
            "classname": "MathTest",
            "time": 0.002,
            "status": "fail"
        }
    ],
    "summary": {
        "total": 2,
        "passed": 1,
        "failed": 1,
        "skipped": 0,
        "time": 0.05
    }
}"#;

const SKIPPED_JSON: &str = r#"{
    "status": "pass",
    "tests": [
        {
            "name": "test_works",
            "classname": "MathTest",
            "time": 0.001,
            "status": "pass"
        },
        {
            "name": "test_skipped",
            "classname": "MathTest",
            "time": 0.0,
            "status": "skip"
        }
    ],
    "summary": {
        "total": 2,
        "passed": 1,
        "failed": 0,
        "skipped": 1,
        "time": 0.03
    }
}"#;

// =============================================================================
// PARAMETERIZED JSON PARSING TESTS
// =============================================================================

#[parameterized(
    passing = { PASSING_JSON, true, 2, 0, 0 },
    failing = { FAILING_JSON, false, 2, 1, 0 },
    skipped = { SKIPPED_JSON, true, 2, 0, 1 },
)]
fn parses_json_test_results(
    json: &str,
    expect_passed: bool,
    test_count: usize,
    fail_count: usize,
    skip_count: usize,
) {
    let result = try_parse_minitest_json(json, Duration::from_secs(1)).unwrap();
    assert_eq!(result.passed, expect_passed, "passed mismatch");
    assert_eq!(result.tests.len(), test_count, "test count mismatch");
    let actual_fails = result.tests.iter().filter(|t| !t.passed).count();
    assert_eq!(actual_fails, fail_count, "failure count mismatch");
    let actual_skips = result.tests.iter().filter(|t| t.skipped).count();
    assert_eq!(actual_skips, skip_count, "skip count mismatch");
}

#[test]
fn parses_json_passing_test_details() {
    let result = try_parse_minitest_json(PASSING_JSON, Duration::from_secs(1)).unwrap();

    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "MathTest#test_adds_two_numbers");
    assert_eq!(result.tests[0].duration, Duration::from_secs_f64(0.001));
    assert!(result.tests[1].passed);
}

#[test]
fn parses_json_without_classname() {
    let output = r#"{
        "status": "pass",
        "tests": [
            {
                "name": "test_something",
                "classname": "",
                "time": 0.001,
                "status": "pass"
            }
        ],
        "summary": {"total": 1, "passed": 1, "failed": 0, "skipped": 0, "time": 0.01}
    }"#;
    let result = try_parse_minitest_json(output, Duration::from_secs(1)).unwrap();

    assert_eq!(result.tests[0].name, "test_something");
}

#[test]
fn returns_none_for_non_json() {
    let result = try_parse_minitest_json("not json", Duration::from_secs(1));
    assert!(result.is_none());
}

// =============================================================================
// TEXT OUTPUT PARSING TESTS
// =============================================================================

const TEXT_PASSING: &str = r#"
Run options: --seed 12345

# Running:

..

Finished in 0.012345s, 100.0000 runs/s, 200.0000 assertions/s.

2 runs, 4 assertions, 0 failures, 0 errors, 0 skips
"#;

const TEXT_FAILING: &str = r#"
Run options: --seed 54321

# Running:

.F.

Finished in 0.023456s, 80.0000 runs/s, 160.0000 assertions/s.

3 runs, 6 assertions, 1 failures, 0 errors, 0 skips
"#;

const TEXT_ERRORS: &str = r#"
# Running:

.E.

3 runs, 4 assertions, 0 failures, 1 errors, 0 skips
"#;

const TEXT_SKIPS: &str = r#"
# Running:

.S.

3 runs, 4 assertions, 0 failures, 0 errors, 1 skips
"#;

#[parameterized(
    passing = { TEXT_PASSING, true, 2, 0, 0 },
    failing = { TEXT_FAILING, false, 3, 1, 0 },
    with_errors = { TEXT_ERRORS, false, 3, 1, 0 },
    with_skips = { TEXT_SKIPS, true, 3, 0, 1 },
)]
fn parses_text_output(
    stdout: &str,
    passed: bool,
    test_count: usize,
    fail_count: usize,
    skip_count: usize,
) {
    let result = parse_minitest_output(stdout, "", Duration::from_secs(1), passed);

    assert_eq!(result.passed, passed, "passed mismatch");
    assert_eq!(result.tests.len(), test_count, "test count mismatch");
    let actual_fails = result.tests.iter().filter(|t| !t.passed).count();
    assert_eq!(actual_fails, fail_count, "failure count mismatch");
    assert_eq!(
        result.tests.iter().filter(|t| t.skipped).count(),
        skip_count,
        "skip count mismatch"
    );
}

// =============================================================================
// SUMMARY LINE PARSING TESTS
// =============================================================================

#[parameterized(
    standard = { "10 runs, 20 assertions, 1 failures, 0 errors, 2 skips", Some((10, 1, 0, 2)) },
    singular = { "1 run, 2 assertions, 0 failure, 0 error, 0 skip", Some((1, 0, 0, 0)) },
    no_skips = { "5 runs, 10 assertions, 0 failures, 0 errors, 0 skips", Some((5, 0, 0, 0)) },
    with_errors = { "8 runs, 15 assertions, 2 failures, 1 errors, 0 skips", Some((8, 2, 1, 0)) },
    finished_line = { "Finished in 0.01s", None },
    running_line = { "# Running:", None },
    empty = { "", None },
)]
fn parses_summary_line_cases(line: &str, expected: Option<(u32, u32, u32, u32)>) {
    let summary = parse_summary_line(line);
    match expected {
        Some((runs, failures, errors, skips)) => {
            let s = summary.unwrap();
            assert_eq!(s.runs, runs, "runs mismatch");
            assert_eq!(s.failures, failures, "failures mismatch");
            assert_eq!(s.errors, errors, "errors mismatch");
            assert_eq!(s.skips, skips, "skips mismatch");
        }
        None => assert!(summary.is_none()),
    }
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn handles_empty_output() {
    let result = parse_minitest_output("", "", Duration::from_secs(0), true);
    assert!(result.passed);
    assert!(result.tests.is_empty());
}
