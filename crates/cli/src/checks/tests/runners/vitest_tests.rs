#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parses_passing_tests() {
    let output = r#"{
        "testResults": [
            {
                "name": "src/utils.test.ts",
                "assertionResults": [
                    {"fullName": "adds numbers", "status": "passed", "duration": 45},
                    {"fullName": "multiplies numbers", "status": "passed", "duration": 23}
                ]
            }
        ]
    }"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "adds numbers");
    assert_eq!(result.tests[0].duration, Duration::from_millis(45));
    assert!(result.tests[1].passed);
    assert_eq!(result.tests[1].name, "multiplies numbers");
    assert_eq!(result.tests[1].duration, Duration::from_millis(23));
}

#[test]
fn parses_failing_tests() {
    let output = r#"{
        "testResults": [
            {
                "name": "src/utils.test.ts",
                "assertionResults": [
                    {"fullName": "adds numbers", "status": "passed", "duration": 45},
                    {"fullName": "handles errors", "status": "failed", "duration": 23}
                ]
            }
        ]
    }"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert!(!result.tests[1].passed);
    assert_eq!(result.tests[1].name, "handles errors");
}

#[test]
fn parses_multiple_files() {
    let output = r#"{
        "testResults": [
            {
                "name": "src/utils.test.ts",
                "assertionResults": [
                    {"fullName": "utils > adds", "status": "passed", "duration": 10}
                ]
            },
            {
                "name": "src/math.test.ts",
                "assertionResults": [
                    {"fullName": "math > multiplies", "status": "passed", "duration": 20}
                ]
            }
        ]
    }"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
}

#[test]
fn handles_empty_output() {
    let result = parse_vitest_json("", Duration::from_secs(0));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_empty_test_results() {
    let output = r#"{"testResults": []}"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_missing_duration() {
    let output = r#"{
        "testResults": [
            {
                "name": "test.ts",
                "assertionResults": [
                    {"fullName": "test", "status": "passed"}
                ]
            }
        ]
    }"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));

    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.tests[0].duration, Duration::ZERO);
}

#[test]
fn finds_json_in_output_with_prefix() {
    let output = r#"
some vitest banner text here
stderr output mixed in
{"testResults": [{"name": "test.ts", "assertionResults": [{"fullName": "test", "status": "passed", "duration": 10}]}]}
"#;
    let result = parse_vitest_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
}

#[test]
fn detects_failure_without_json() {
    let output = "FAIL tests/example.test.ts\nError: test failed";
    let result = parse_vitest_json(output, Duration::from_secs(1));
    assert!(!result.passed);
}
