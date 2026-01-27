#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

// =============================================================================
// find_json_object tests
// =============================================================================

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
fn find_json_object_deeply_nested() {
    let json = find_json_object(r#"{"a": {"b": {"c": {"d": 1}}}}"#);
    assert_eq!(json, Some(r#"{"a": {"b": {"c": {"d": 1}}}}"#));
}

#[test]
fn find_json_object_with_array_inside() {
    let json = find_json_object(r#"{"items": [1, 2, 3]}"#);
    assert_eq!(json, Some(r#"{"items": [1, 2, 3]}"#));
}

#[test]
fn find_json_object_none() {
    assert!(find_json_object("no json here").is_none());
    assert!(find_json_object("").is_none());
}

#[test]
fn find_json_object_unclosed() {
    assert!(find_json_object(r#"{"key": "value""#).is_none());
}

#[test]
fn find_json_object_in_runner_output() {
    let output = r#"
Running tests...
Loading dependencies
{"examples": [{"status": "passed"}], "summary": {"total": 1}}
Done.
"#;
    let json = find_json_object(output);
    assert!(json.is_some());
    assert!(json.unwrap().starts_with(r#"{"examples":"#));
}

// =============================================================================
// find_json_array tests
// =============================================================================

#[test]
fn find_json_array_simple() {
    let json = find_json_array(r#"prefix [1, 2, 3] suffix"#);
    assert_eq!(json, Some(r#"[1, 2, 3]"#));
}

#[test]
fn find_json_array_nested() {
    let json = find_json_array(r#"[[1, 2], [3, 4]]"#);
    assert_eq!(json, Some(r#"[[1, 2], [3, 4]]"#));
}

#[test]
fn find_json_array_with_objects() {
    let json = find_json_array(r#"[{"name": "test"}, {"name": "test2"}]"#);
    assert_eq!(json, Some(r#"[{"name": "test"}, {"name": "test2"}]"#));
}

#[test]
fn find_json_array_none() {
    assert!(find_json_array("no json here").is_none());
    assert!(find_json_array("").is_none());
}

#[test]
fn find_json_array_unclosed() {
    assert!(find_json_array(r#"[1, 2, 3"#).is_none());
}

#[test]
fn find_json_array_in_cucumber_output() {
    let output = r#"
Running features...
[{"uri": "features/login.feature", "elements": []}]
Done.
"#;
    let json = find_json_array(output);
    assert!(json.is_some());
    assert!(json.unwrap().starts_with(r#"[{"uri":"#));
}
