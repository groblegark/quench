// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use super::*;
use yare::parameterized;

// =============================================================================
// SIMPLECOV FORMAT PARSING TESTS
// =============================================================================

#[test]
fn parses_modern_simplecov_format() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/project/lib/math.rb": {
                    "lines": [1, 1, null, 0, 1, null]
                },
                "/project/lib/calculator.rb": {
                    "lines": [1, 1, 1, 1]
                }
            },
            "timestamp": 1234567890
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    assert!(result.success);
    assert!(result.line_coverage.is_some());
    assert_eq!(result.files.len(), 2);

    // math.rb: lines [1, 1, null, 0, 1, null]
    //   4 relevant lines (non-null), 3 covered = 75%
    let math_coverage = result.files.get("lib/math.rb").unwrap();
    assert!((math_coverage - 75.0).abs() < 0.01);

    // calculator.rb: lines [1, 1, 1, 1]
    //   4 relevant lines, 4 covered = 100%
    let calc_coverage = result.files.get("lib/calculator.rb").unwrap();
    assert!((calc_coverage - 100.0).abs() < 0.01);
}

#[test]
fn parses_legacy_simplecov_format() {
    let json = r#"{
        "Minitest": {
            "coverage": {
                "/project/lib/utils.rb": [1, 0, null, 1, 1]
            },
            "timestamp": 1234567890
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    assert!(result.success);
    assert_eq!(result.files.len(), 1);

    // utils.rb: 4 relevant lines, 3 covered = 75%
    let coverage = result.files.get("lib/utils.rb").unwrap();
    assert!((coverage - 75.0).abs() < 0.01);
}

#[test]
fn merges_multiple_suites() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/project/lib/a.rb": {"lines": [1, 1, 0, 0]}
            },
            "timestamp": 100
        },
        "Minitest": {
            "coverage": {
                "/project/lib/b.rb": {"lines": [1, 1, 1, 1]}
            },
            "timestamp": 200
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    assert!(result.success);
    assert_eq!(result.files.len(), 2);
    assert!(result.files.contains_key("lib/a.rb"));
    assert!(result.files.contains_key("lib/b.rb"));
}

#[test]
fn calculates_overall_coverage() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/project/lib/a.rb": {"lines": [1, 1, 0, 0]},
                "/project/lib/b.rb": {"lines": [1, 1, 1, 1]}
            },
            "timestamp": 100
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    // a.rb: 50%, b.rb: 100%, average = 75%
    let overall = result.line_coverage.unwrap();
    assert!((overall - 75.0).abs() < 0.01);
}

#[test]
fn extracts_package_from_lib() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/project/lib/myapp/models/user.rb": {"lines": [1, 1]},
                "/project/lib/myapp/controllers/home.rb": {"lines": [1, 0]}
            },
            "timestamp": 100
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    assert_eq!(result.packages.len(), 1);
    assert!(result.packages.contains_key("myapp"));
}

#[test]
fn extracts_package_from_app() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/rails/app/models/user.rb": {"lines": [1, 1]},
                "/rails/app/controllers/home.rb": {"lines": [1, 0]}
            },
            "timestamp": 100
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    // Should extract "models" and "controllers" as packages
    assert_eq!(result.packages.len(), 2);
    assert!(result.packages.contains_key("models"));
    assert!(result.packages.contains_key("controllers"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[parameterized(
    empty_coverage = { r#"{"RSpec": {"coverage": {}, "timestamp": 100}}"# },
)]
fn handles_empty_coverage(json: &str) {
    let result = parse_simplecov_json(json, Duration::from_secs(1));
    assert!(result.success);
    assert!(result.line_coverage.is_none());
    assert!(result.files.is_empty());
}

#[test]
fn handles_all_null_lines() {
    let json = r#"{
        "RSpec": {
            "coverage": {
                "/project/lib/empty.rb": {"lines": [null, null, null]}
            },
            "timestamp": 100
        }
    }"#;
    let result = parse_simplecov_json(json, Duration::from_secs(1));

    // File with no relevant lines should not affect coverage
    assert!(result.files.is_empty());
}

#[parameterized(
    not_json = { "not json" },
    empty_string = { "" },
)]
fn handles_invalid_json(json: &str) {
    let result = parse_simplecov_json(json, Duration::from_secs(1));
    assert!(!result.success);
}

// =============================================================================
// LINE COVERAGE CALCULATION TESTS
// =============================================================================

#[parameterized(
    basic = { vec![Some(1), Some(1), Some(1), Some(0)], Some(75.0) },
    with_nulls = { vec![Some(1), None, Some(0), None], Some(50.0) },
    all_nulls = { vec![None, None, None], None },
    empty = { vec![], None },
)]
fn calculate_line_coverage_cases(lines: Vec<Option<u64>>, expected: Option<f64>) {
    let coverage = calculate_line_coverage(&lines);
    match expected {
        Some(e) => assert!((coverage.unwrap() - e).abs() < 0.01),
        None => assert!(coverage.is_none()),
    }
}

// =============================================================================
// PATH NORMALIZATION TESTS
// =============================================================================

#[parameterized(
    lib_path = { "/home/user/project/lib/myapp/model.rb", "lib/myapp/model.rb" },
    app_path = { "/rails/app/models/user.rb", "app/models/user.rb" },
    fallback = { "/some/random/path/file.rb", "file.rb" },
)]
fn normalize_ruby_path_cases(path: &str, expected: &str) {
    assert_eq!(normalize_ruby_path(path), expected);
}

// =============================================================================
// PACKAGE EXTRACTION TESTS
// =============================================================================

#[parameterized(
    lib_package = { "lib/myapp/model.rb", "myapp" },
    app_package = { "app/models/user.rb", "models" },
    root = { "some_file.rb", "root" },
)]
fn extract_ruby_package_cases(path: &str, expected: &str) {
    assert_eq!(extract_ruby_package(path), expected);
}
