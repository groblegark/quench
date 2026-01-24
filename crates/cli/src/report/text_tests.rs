// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use crate::baseline::{
    BaselineMetrics, BuildTimeMetrics, CoverageMetrics, EscapesMetrics, TestTimeMetrics,
};

/// Test filter that includes all checks.
struct AllChecks;

impl CheckFilter for AllChecks {
    fn enabled_checks(&self) -> Vec<String> {
        Vec::new()
    }

    fn disabled_checks(&self) -> Vec<String> {
        Vec::new()
    }
}

fn create_test_baseline() -> Baseline {
    Baseline {
        version: 1,
        updated: chrono::Utc::now(),
        commit: Some("abc1234".to_string()),
        metrics: BaselineMetrics {
            coverage: Some(CoverageMetrics {
                total: 85.5,
                by_package: None,
            }),
            escapes: Some(EscapesMetrics {
                source: [("unwrap".to_string(), 10)].into_iter().collect(),
                test: None,
            }),
            build_time: Some(BuildTimeMetrics {
                cold: 45.0,
                hot: 12.5,
            }),
            binary_size: Some([("quench".to_string(), 5_242_880)].into_iter().collect()),
            test_time: Some(TestTimeMetrics {
                total: 30.5,
                avg: 0.5,
                max: 2.0,
            }),
        },
    }
}

#[test]
fn text_format_empty_baseline() {
    let formatter = TextFormatter;
    assert_eq!(formatter.format_empty(), "No baseline found.\n");
}

#[test]
fn text_format_includes_header() {
    let baseline = Baseline::default();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("Quench Report"));
    assert!(output.contains("============="));
}

#[test]
fn text_format_includes_baseline_date() {
    let baseline = Baseline::default();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("Baseline:"));
}

#[test]
fn text_format_includes_commit_when_present() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("abc1234"));
}

#[test]
fn text_format_includes_coverage() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("coverage: 85.5%"));
}

#[test]
fn text_format_includes_escapes() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("escapes.unwrap: 10"));
}

#[test]
fn text_format_includes_build_time() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("build_time.cold: 45.0s"));
    assert!(output.contains("build_time.hot: 12.5s"));
}

#[test]
fn text_format_includes_binary_size() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("binary_size.quench: 5.0 MB"));
}

#[test]
fn text_format_includes_test_time() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("test_time.total: 30.5s"));
}

#[test]
fn text_format_to_matches_format() {
    let baseline = create_test_baseline();
    let formatter = TextFormatter;

    let buffered = formatter.format(&baseline, &AllChecks).unwrap();

    let mut streamed = Vec::new();
    formatter
        .format_to(&mut streamed, &baseline, &AllChecks)
        .unwrap();
    let streamed_str = String::from_utf8(streamed).unwrap();

    assert_eq!(buffered, streamed_str);
}

#[test]
fn text_format_empty_to_matches_format_empty() {
    let formatter = TextFormatter;

    let buffered = formatter.format_empty();

    let mut streamed = Vec::new();
    formatter.format_empty_to(&mut streamed).unwrap();
    let streamed_str = String::from_utf8(streamed).unwrap();

    assert_eq!(buffered, streamed_str);
}

#[test]
fn text_format_escapes_sorted_alphabetically() {
    let mut baseline = create_test_baseline();
    baseline.metrics.escapes = Some(EscapesMetrics {
        source: [
            ("zebra".to_string(), 1),
            ("alpha".to_string(), 2),
            ("middle".to_string(), 3),
        ]
        .into_iter()
        .collect(),
        test: None,
    });

    let formatter = TextFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();

    let alpha_pos = output.find("escapes.alpha").unwrap();
    let middle_pos = output.find("escapes.middle").unwrap();
    let zebra_pos = output.find("escapes.zebra").unwrap();

    assert!(alpha_pos < middle_pos);
    assert!(middle_pos < zebra_pos);
}
