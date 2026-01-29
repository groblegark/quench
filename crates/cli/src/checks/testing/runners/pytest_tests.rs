// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use super::*;

#[test]
fn parses_passing_tests_with_durations() {
    let output = r#"
============================= test session starts ==============================
collected 2 items

test_example.py::test_one PASSED                                         [ 50%]
test_example.py::test_two PASSED                                         [100%]

============================= slowest durations =============================
0.45s call     test_example.py::test_one
0.23s call     test_example.py::test_two
0.01s setup    test_example.py::test_one
0.00s setup    test_example.py::test_two
============================= 2 passed in 0.68s =============================
"#;
    let result = parse_pytest_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "test_example.py::test_one");
    assert_eq!(result.tests[0].duration, Duration::from_millis(450));
    assert_eq!(result.tests[1].name, "test_example.py::test_two");
    assert_eq!(result.tests[1].duration, Duration::from_millis(230));
}

#[test]
fn parses_failing_tests() {
    let output = r#"
============================= test session starts ==============================
collected 2 items

test_example.py::test_one PASSED                                         [ 50%]
test_example.py::test_two FAILED                                         [100%]

============================= FAILURES ===============================
____________________________ test_two ____________________________

    def test_two():
>       assert False
E       AssertionError

============================= slowest durations =============================
0.45s call     test_example.py::test_one
============================= 1 passed, 1 failed in 0.68s =============================
"#;
    let result = parse_pytest_output(output, Duration::from_secs(1));

    assert!(!result.passed);
}

#[test]
fn parses_summary_passed_only() {
    let line = "============================= 5 passed in 1.23s =============================";
    let (passed, failed) = parse_summary_line(line).unwrap();
    assert_eq!(passed, 5);
    assert_eq!(failed, 0);
}

#[test]
fn parses_summary_passed_and_failed() {
    let line =
        "============================= 2 passed, 3 failed in 1.00s =============================";
    let (passed, failed) = parse_summary_line(line).unwrap();
    assert_eq!(passed, 2);
    assert_eq!(failed, 3);
}

#[test]
fn parses_summary_complex() {
    let line = "============================= 1 failed, 2 passed, 1 skipped in 1.00s =============================";
    let (passed, failed) = parse_summary_line(line).unwrap();
    assert_eq!(passed, 2);
    assert_eq!(failed, 1);
}

#[test]
fn parses_duration_seconds() {
    assert_eq!(parse_duration("0.45s"), Some(Duration::from_millis(450)));
    assert_eq!(parse_duration("1.0s"), Some(Duration::from_secs(1)));
    assert_eq!(parse_duration("0.001s"), Some(Duration::from_micros(1000)));
}

#[test]
fn parses_duration_invalid() {
    assert!(parse_duration("invalid").is_none());
    assert!(parse_duration("45ms").is_none());
    assert!(parse_duration("").is_none());
}

#[test]
fn parses_duration_line_call_phase() {
    let result = parse_duration_line("0.45s call     test_example.py::test_one").unwrap();
    assert!(result.passed);
    assert_eq!(result.name, "test_example.py::test_one");
    assert_eq!(result.duration, Duration::from_millis(450));
}

#[test]
fn ignores_setup_and_teardown_phases() {
    assert!(parse_duration_line("0.01s setup    test_example.py::test_one").is_none());
    assert!(parse_duration_line("0.00s teardown test_example.py::test_one").is_none());
}

#[test]
fn handles_empty_output() {
    let result = parse_pytest_output("", Duration::from_secs(0));
    // Empty output is considered passed (no failures detected)
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_no_durations_section() {
    let output = r#"
============================= test session starts ==============================
collected 1 item

test_example.py::test_one PASSED                                         [100%]

============================= 1 passed in 0.01s =============================
"#;
    let result = parse_pytest_output(output, Duration::from_secs(1));
    assert!(result.passed);
    // No tests parsed from durations, but summary says passed
    assert!(result.tests.is_empty());
}

#[test]
fn summary_returns_none_for_non_summary() {
    assert!(parse_summary_line("test_example.py::test_one PASSED").is_none());
    assert!(parse_summary_line("collected 2 items").is_none());
    assert!(parse_summary_line("").is_none());
}
