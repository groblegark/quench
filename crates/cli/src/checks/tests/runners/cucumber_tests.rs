#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use super::*;

#[test]
fn parses_passing_scenarios() {
    let output = r#"[
        {
            "uri": "features/math.feature",
            "name": "Math operations",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Adding numbers",
                    "steps": [
                        {
                            "name": "I have entered 50 into the calculator",
                            "result": {
                                "status": "passed",
                                "duration": 1000000
                            }
                        },
                        {
                            "name": "I press add",
                            "result": {
                                "status": "passed",
                                "duration": 500000
                            }
                        }
                    ]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "Adding numbers");
    // Duration is sum of step durations in nanoseconds
    assert_eq!(result.tests[0].duration, Duration::from_nanos(1500000));
}

#[test]
fn parses_failing_scenarios() {
    let output = r#"[
        {
            "uri": "features/math.feature",
            "name": "Math operations",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Division by zero",
                    "steps": [
                        {
                            "name": "I divide by zero",
                            "result": {
                                "status": "failed",
                                "duration": 2000000
                            }
                        }
                    ]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 1);
    assert!(!result.tests[0].passed);
    assert_eq!(result.tests[0].name, "Division by zero");
}

#[test]
fn parses_pending_scenarios() {
    let output = r#"[
        {
            "uri": "features/future.feature",
            "name": "Future features",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Unimplemented feature",
                    "steps": [
                        {
                            "name": "a pending step",
                            "result": {
                                "status": "pending",
                                "duration": 0
                            }
                        }
                    ]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
    assert!(result.tests[0].skipped);
}

#[test]
fn parses_undefined_steps() {
    let output = r#"[
        {
            "uri": "features/new.feature",
            "name": "New feature",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Missing steps",
                    "steps": [
                        {
                            "name": "an undefined step",
                            "result": {
                                "status": "undefined",
                                "duration": 0
                            }
                        }
                    ]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert!(result.tests[0].skipped);
}

#[test]
fn parses_multiple_features() {
    let output = r#"[
        {
            "uri": "features/one.feature",
            "name": "Feature One",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Scenario A",
                    "steps": [{"name": "step", "result": {"status": "passed", "duration": 100}}]
                }
            ]
        },
        {
            "uri": "features/two.feature",
            "name": "Feature Two",
            "elements": [
                {
                    "type": "scenario",
                    "name": "Scenario B",
                    "steps": [{"name": "step", "result": {"status": "passed", "duration": 200}}]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
    assert_eq!(result.tests[0].name, "Scenario A");
    assert_eq!(result.tests[1].name, "Scenario B");
}

#[test]
fn ignores_background_elements() {
    let output = r#"[
        {
            "uri": "features/test.feature",
            "name": "Test",
            "elements": [
                {
                    "type": "background",
                    "name": "Setup",
                    "steps": [{"name": "setup step", "result": {"status": "passed", "duration": 100}}]
                },
                {
                    "type": "scenario",
                    "name": "Actual test",
                    "steps": [{"name": "test step", "result": {"status": "passed", "duration": 200}}]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.tests[0].name, "Actual test");
}

#[test]
fn handles_empty_output() {
    let result = parse_cucumber_json("", Duration::from_secs(0));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_empty_features() {
    let output = r#"[]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn handles_unnamed_scenario() {
    let output = r#"[
        {
            "uri": "features/test.feature",
            "name": "Test Feature",
            "elements": [
                {
                    "type": "scenario",
                    "name": "",
                    "steps": [{"name": "step", "result": {"status": "passed", "duration": 100}}]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert_eq!(result.tests[0].name, "Scenario in Test Feature");
}

#[test]
fn handles_missing_result() {
    let output = r#"[
        {
            "uri": "features/test.feature",
            "name": "Test",
            "elements": [
                {
                    "type": "scenario",
                    "name": "No result",
                    "steps": [
                        {"name": "step without result"}
                    ]
                }
            ]
        }
    ]"#;
    let result = parse_cucumber_json(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.tests[0].duration, Duration::ZERO);
}

#[test]
fn handles_non_json_output() {
    // Without JSON output, we can't reliably determine pass/fail
    let output = "Feature: Test\n  Scenario: example\n    Then it runs";
    let result = parse_cucumber_json(output, Duration::from_secs(1));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn find_json_array_simple() {
    let json = find_json_array(r#"prefix [1, 2, 3] suffix"#);
    assert_eq!(json, Some(r#"[1, 2, 3]"#));
}

#[test]
fn find_json_array_nested() {
    let json = find_json_array(r#"[[1], [2]]"#);
    assert_eq!(json, Some(r#"[[1], [2]]"#));
}

#[test]
fn find_json_array_none() {
    assert!(find_json_array("no array here").is_none());
    assert!(find_json_array("").is_none());
}

#[test]
fn scenario_status_from_steps() {
    // All passed
    let steps = vec![
        CucumberStep {
            name: "step1".to_string(),
            result: Some(CucumberStepResult {
                status: "passed".to_string(),
                duration: 100,
            }),
        },
        CucumberStep {
            name: "step2".to_string(),
            result: Some(CucumberStepResult {
                status: "passed".to_string(),
                duration: 200,
            }),
        },
    ];
    let (status, duration) = scenario_status(&steps);
    assert_eq!(status, "passed");
    assert_eq!(duration, Duration::from_nanos(300));

    // One failed
    let steps = vec![
        CucumberStep {
            name: "step1".to_string(),
            result: Some(CucumberStepResult {
                status: "passed".to_string(),
                duration: 100,
            }),
        },
        CucumberStep {
            name: "step2".to_string(),
            result: Some(CucumberStepResult {
                status: "failed".to_string(),
                duration: 50,
            }),
        },
    ];
    let (status, _) = scenario_status(&steps);
    assert_eq!(status, "failed");

    // One pending
    let steps = vec![CucumberStep {
        name: "step1".to_string(),
        result: Some(CucumberStepResult {
            status: "pending".to_string(),
            duration: 0,
        }),
    }];
    let (status, _) = scenario_status(&steps);
    assert_eq!(status, "pending");
}
