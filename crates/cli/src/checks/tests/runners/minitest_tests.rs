#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use super::*;

#[test]
fn parses_json_passing_tests() {
    let output = r#"{
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
    let result = try_parse_minitest_json(output, Duration::from_secs(1)).unwrap();

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "MathTest#test_adds_two_numbers");
    assert_eq!(result.tests[0].duration, Duration::from_secs_f64(0.001));
    assert!(result.tests[1].passed);
}

#[test]
fn parses_json_failing_tests() {
    let output = r#"{
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
    let result = try_parse_minitest_json(output, Duration::from_secs(1)).unwrap();

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert!(!result.tests[1].passed);
}

#[test]
fn parses_json_skipped_tests() {
    let output = r#"{
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
    let result = try_parse_minitest_json(output, Duration::from_secs(1)).unwrap();

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[1].skipped);
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

#[test]
fn parses_text_output_passing() {
    let stdout = r#"
Run options: --seed 12345

# Running:

..

Finished in 0.012345s, 100.0000 runs/s, 200.0000 assertions/s.

2 runs, 4 assertions, 0 failures, 0 errors, 0 skips
"#;
    let result = parse_minitest_output(stdout, "", Duration::from_secs(1), true);

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
}

#[test]
fn parses_text_output_with_failures() {
    let stdout = r#"
Run options: --seed 54321

# Running:

.F.

Finished in 0.023456s, 80.0000 runs/s, 160.0000 assertions/s.

3 runs, 6 assertions, 1 failures, 0 errors, 0 skips
"#;
    let result = parse_minitest_output(stdout, "", Duration::from_secs(1), false);

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 3);
    // 2 passes + 1 failure
    assert_eq!(result.tests.iter().filter(|t| t.passed).count(), 2);
    assert_eq!(result.tests.iter().filter(|t| !t.passed).count(), 1);
}

#[test]
fn parses_text_output_with_errors() {
    let stdout = r#"
# Running:

.E.

3 runs, 4 assertions, 0 failures, 1 errors, 0 skips
"#;
    let result = parse_minitest_output(stdout, "", Duration::from_secs(1), false);

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 3);
}

#[test]
fn parses_text_output_with_skips() {
    let stdout = r#"
# Running:

.S.

3 runs, 4 assertions, 0 failures, 0 errors, 1 skips
"#;
    let result = parse_minitest_output(stdout, "", Duration::from_secs(1), true);

    assert!(result.passed);
    assert_eq!(result.tests.len(), 3);
    assert_eq!(result.tests.iter().filter(|t| t.skipped).count(), 1);
}

#[test]
fn parses_summary_line_correctly() {
    let summary = parse_summary_line("10 runs, 20 assertions, 1 failures, 0 errors, 2 skips");
    assert!(summary.is_some());
    let s = summary.unwrap();
    assert_eq!(s.runs, 10);
    assert_eq!(s.failures, 1);
    assert_eq!(s.errors, 0);
    assert_eq!(s.skips, 2);
}

#[test]
fn parses_summary_line_singular() {
    let summary = parse_summary_line("1 run, 2 assertions, 0 failure, 0 error, 0 skip");
    assert!(summary.is_some());
    let s = summary.unwrap();
    assert_eq!(s.runs, 1);
}

#[test]
fn rejects_non_summary_line() {
    assert!(parse_summary_line("Finished in 0.01s").is_none());
    assert!(parse_summary_line("# Running:").is_none());
    assert!(parse_summary_line("").is_none());
}

#[test]
fn handles_empty_output() {
    let result = parse_minitest_output("", "", Duration::from_secs(0), true);
    assert!(result.passed);
    assert!(result.tests.is_empty());
}
