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
fn html_format_empty_baseline() {
    let formatter = HtmlFormatter;
    let empty = formatter.format_empty();
    assert!(empty.contains("<!DOCTYPE html>"));
    assert!(empty.contains("No baseline found"));
}

#[test]
fn html_format_includes_doctype() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.starts_with("<!DOCTYPE html>"));
}

#[test]
fn html_format_includes_title() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("<title>Quench Report</title>"));
}

#[test]
fn html_format_includes_css() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("<style>"));
    assert!(output.contains("</style>"));
}

#[test]
fn html_format_includes_header() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("<h1>Quench Report</h1>"));
}

#[test]
fn html_format_includes_commit() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("abc1234"));
}

#[test]
fn html_format_includes_coverage_card() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("Coverage"));
    assert!(output.contains("85.5%"));
}

#[test]
fn html_format_includes_escapes_card() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("Escapes"));
    assert!(output.contains("10"));
}

#[test]
fn html_format_includes_table() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("<table>"));
    assert!(output.contains("<thead>"));
    assert!(output.contains("<tbody>"));
}

#[test]
fn html_format_includes_build_metrics() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("build_time.cold"));
    assert!(output.contains("45.0s"));
    assert!(output.contains("build_time.hot"));
    assert!(output.contains("12.5s"));
}

#[test]
fn html_format_includes_binary_size() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();
    assert!(output.contains("binary_size.quench"));
    assert!(output.contains("5.0 MB"));
}

#[test]
fn html_format_to_matches_format() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;

    let buffered = formatter.format(&baseline, &AllChecks).unwrap();

    let mut streamed = Vec::new();
    formatter
        .format_to(&mut streamed, &baseline, &AllChecks)
        .unwrap();
    let streamed_str = String::from_utf8(streamed).unwrap();

    assert_eq!(buffered, streamed_str);
}

#[test]
fn html_format_empty_to_matches_format_empty() {
    let formatter = HtmlFormatter;

    let buffered = formatter.format_empty();

    let mut streamed = Vec::new();
    formatter.format_empty_to(&mut streamed).unwrap();
    let streamed_str = String::from_utf8(streamed).unwrap();

    assert_eq!(buffered, streamed_str);
}

#[test]
fn html_format_closes_all_tags() {
    let baseline = create_test_baseline();
    let formatter = HtmlFormatter;
    let output = formatter.format(&baseline, &AllChecks).unwrap();

    // Verify key closing tags are present
    assert!(output.contains("</html>"));
    assert!(output.contains("</head>"));
    assert!(output.contains("</body>"));
    assert!(output.contains("</table>"));
}
