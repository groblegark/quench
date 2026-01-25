#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parses_passing_tests() {
    let output = r#"
{"Time":"2024-01-01T00:00:00Z","Action":"run","Package":"example.com/pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:00Z","Action":"output","Package":"example.com/pkg","Test":"TestOne","Output":"=== RUN   TestOne\n"}
{"Time":"2024-01-01T00:00:01Z","Action":"output","Package":"example.com/pkg","Test":"TestOne","Output":"--- PASS: TestOne (0.45s)\n"}
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"example.com/pkg","Test":"TestOne","Elapsed":0.45}
{"Time":"2024-01-01T00:00:01Z","Action":"run","Package":"example.com/pkg","Test":"TestTwo"}
{"Time":"2024-01-01T00:00:02Z","Action":"pass","Package":"example.com/pkg","Test":"TestTwo","Elapsed":0.23}
"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "example.com/pkg/TestOne");
    assert_eq!(result.tests[0].duration, Duration::from_millis(450));
    assert!(result.tests[1].passed);
    assert_eq!(result.tests[1].name, "example.com/pkg/TestTwo");
    assert_eq!(result.tests[1].duration, Duration::from_millis(230));
}

#[test]
fn parses_failing_tests() {
    let output = r#"
{"Time":"2024-01-01T00:00:00Z","Action":"run","Package":"example.com/pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"example.com/pkg","Test":"TestOne","Elapsed":0.45}
{"Time":"2024-01-01T00:00:01Z","Action":"run","Package":"example.com/pkg","Test":"TestTwo"}
{"Time":"2024-01-01T00:00:02Z","Action":"fail","Package":"example.com/pkg","Test":"TestTwo","Elapsed":0.23}
"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 2);
    assert!(result.tests[0].passed);
    assert!(!result.tests[1].passed);
    assert_eq!(result.tests[1].name, "example.com/pkg/TestTwo");
}

#[test]
fn ignores_package_level_results() {
    // Package-level pass/fail events don't have Test field
    let output = r#"
{"Time":"2024-01-01T00:00:00Z","Action":"run","Package":"example.com/pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"example.com/pkg","Test":"TestOne","Elapsed":0.45}
{"Time":"2024-01-01T00:00:02Z","Action":"pass","Package":"example.com/pkg","Elapsed":1.0}
"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
}

#[test]
fn handles_empty_output() {
    let result = parse_go_json("", Duration::from_secs(0));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_malformed_json() {
    let output = r#"
{"Time":"2024-01-01T00:00:00Z","Action":"pass","Package":"pkg","Test":"TestOne","Elapsed":0.45}
not valid json
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"pkg","Test":"TestTwo","Elapsed":0.23}
"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    assert!(result.passed);
    // Should still get the valid test results
    assert_eq!(result.tests.len(), 2);
}

#[test]
fn handles_missing_elapsed() {
    let output =
        r#"{"Time":"2024-01-01T00:00:00Z","Action":"pass","Package":"pkg","Test":"TestOne"}"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.tests[0].duration, Duration::ZERO);
}

#[test]
fn formats_test_name_with_package() {
    let name = format_test_name(Some("example.com/pkg"), "TestOne");
    assert_eq!(name, "example.com/pkg/TestOne");
}

#[test]
fn formats_test_name_without_package() {
    let name = format_test_name(None, "TestOne");
    assert_eq!(name, "TestOne");
}

#[test]
fn ignores_non_terminal_actions() {
    let output = r#"
{"Time":"2024-01-01T00:00:00Z","Action":"run","Package":"pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:00Z","Action":"output","Package":"pkg","Test":"TestOne","Output":"=== RUN   TestOne\n"}
{"Time":"2024-01-01T00:00:01Z","Action":"pause","Package":"pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:01Z","Action":"cont","Package":"pkg","Test":"TestOne"}
{"Time":"2024-01-01T00:00:01Z","Action":"pass","Package":"pkg","Test":"TestOne","Elapsed":0.45}
"#;
    let result = parse_go_json(output, Duration::from_secs(1));

    // Only pass/fail should result in TestResult entries
    assert_eq!(result.tests.len(), 1);
}
