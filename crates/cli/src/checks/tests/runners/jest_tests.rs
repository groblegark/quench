#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parses_passing_tests() {
    let output = r#"{
        "success": true,
        "numTotalTests": 2,
        "numPassedTests": 2,
        "numFailedTests": 0,
        "testResults": [
            {
                "name": "/path/to/utils.test.ts",
                "assertionResults": [
                    {"fullName": "adds numbers", "status": "passed", "duration": 45},
                    {"fullName": "multiplies numbers", "status": "passed", "duration": 23}
                ]
            }
        ]
    }"#;
    let result = parse_jest_json(output, Duration::from_secs(1));

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
        "success": false,
        "numTotalTests": 2,
        "numPassedTests": 1,
        "numFailedTests": 1,
        "testResults": [
            {
                "name": "/path/to/utils.test.ts",
                "assertionResults": [
                    {"fullName": "adds numbers", "status": "passed", "duration": 45},
                    {"fullName": "handles errors", "status": "failed", "duration": 23}
                ]
            }
        ]
    }"#;
    let result = parse_jest_json(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert!(!result.tests[1].passed);
    assert_eq!(result.tests[1].name, "handles errors");
}

#[test]
fn respects_success_field() {
    // Even if all assertions pass, success: false should mean failure
    let output = r#"{
        "success": false,
        "testResults": [
            {
                "name": "test.ts",
                "assertionResults": [
                    {"fullName": "test", "status": "passed", "duration": 10}
                ]
            }
        ]
    }"#;
    let result = parse_jest_json(output, Duration::from_secs(1));
    assert!(!result.passed);
}

#[test]
fn parses_multiple_files() {
    let output = r#"{
        "success": true,
        "testResults": [
            {
                "name": "utils.test.ts",
                "assertionResults": [
                    {"fullName": "utils > adds", "status": "passed", "duration": 10}
                ]
            },
            {
                "name": "math.test.ts",
                "assertionResults": [
                    {"fullName": "math > multiplies", "status": "passed", "duration": 20}
                ]
            }
        ]
    }"#;
    let result = parse_jest_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
}

#[test]
fn handles_empty_output() {
    let result = parse_jest_json("", Duration::from_secs(0));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_empty_test_results() {
    let output = r#"{"success": true, "testResults": []}"#;
    let result = parse_jest_json(output, Duration::from_secs(1));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_missing_duration() {
    let output = r#"{
        "success": true,
        "testResults": [
            {
                "name": "test.ts",
                "assertionResults": [
                    {"fullName": "test", "status": "passed"}
                ]
            }
        ]
    }"#;
    let result = parse_jest_json(output, Duration::from_secs(1));

    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.tests[0].duration, Duration::ZERO);
}

#[test]
fn finds_json_in_output_with_prefix() {
    let output = r#"
PASS tests/example.test.ts
some jest banner text
{"success": true, "testResults": [{"name": "test.ts", "assertionResults": [{"fullName": "test", "status": "passed", "duration": 10}]}]}
"#;
    let result = parse_jest_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
}

#[test]
fn detects_failure_without_json() {
    let output = "FAIL tests/example.test.ts\nError: test failed";
    let result = parse_jest_json(output, Duration::from_secs(1));
    assert!(!result.passed);
}

#[test]
fn find_json_object_simple() {
    let json = find_json_object(r#"prefix {"key": "value"} suffix"#);
    assert_eq!(json, Some(r#"{"key": "value"}"#));
}

#[test]
fn find_json_object_nested() {
    let json = find_json_object(r#"{"outer": {"inner": 1}}"#);
    assert_eq!(json, Some(r#"{"outer": {"inner": 1}}"#));
}

#[test]
fn find_json_object_none() {
    assert!(find_json_object("no json here").is_none());
    assert!(find_json_object("").is_none());
}
