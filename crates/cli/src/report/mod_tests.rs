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
        Vec::new() // Empty means all enabled
    }

    fn disabled_checks(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Test filter that excludes specific checks.
struct ExcludeChecks(Vec<&'static str>);

impl CheckFilter for ExcludeChecks {
    fn enabled_checks(&self) -> Vec<String> {
        Vec::new()
    }

    fn disabled_checks(&self) -> Vec<String> {
        self.0.iter().map(|s| s.to_string()).collect()
    }
}

// --- human_bytes tests ---

#[test]
fn human_bytes_formats_bytes() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(500), "500 B");
    assert_eq!(human_bytes(1023), "1023 B");
}

#[test]
fn human_bytes_formats_kilobytes() {
    assert_eq!(human_bytes(1024), "1.0 KB");
    assert_eq!(human_bytes(1536), "1.5 KB");
    assert_eq!(human_bytes(10240), "10.0 KB");
}

#[test]
fn human_bytes_formats_megabytes() {
    assert_eq!(human_bytes(1048576), "1.0 MB");
    assert_eq!(human_bytes(1572864), "1.5 MB");
    assert_eq!(human_bytes(10485760), "10.0 MB");
}

#[test]
fn human_bytes_boundary_kb() {
    // Just under 1KB
    assert_eq!(human_bytes(1023), "1023 B");
    // Exactly 1KB
    assert_eq!(human_bytes(1024), "1.0 KB");
}

#[test]
fn human_bytes_boundary_mb() {
    // Just under 1MB
    assert_eq!(human_bytes(1048575), "1024.0 KB");
    // Exactly 1MB
    assert_eq!(human_bytes(1048576), "1.0 MB");
}

// --- FilteredMetrics tests ---

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
                source: [("unwrap".to_string(), 10), ("expect".to_string(), 5)]
                    .into_iter()
                    .collect(),
                test: None,
            }),
            build_time: Some(BuildTimeMetrics {
                cold: 45.0,
                hot: 12.5,
            }),
            binary_size: Some(
                [("quench".to_string(), 5_242_880)] // 5 MB
                    .into_iter()
                    .collect(),
            ),
            test_time: Some(TestTimeMetrics {
                total: 30.5,
                avg: 0.5,
                max: 2.0,
            }),
        },
    }
}

#[test]
fn filtered_metrics_includes_all_with_all_checks() {
    let baseline = create_test_baseline();
    let filtered = FilteredMetrics::new(&baseline, &AllChecks);

    assert!(filtered.coverage().is_some());
    assert!(filtered.escapes().is_some());
    assert!(filtered.build_time().is_some());
    assert!(filtered.binary_size().is_some());
    assert!(filtered.test_time().is_some());
}

#[test]
fn filtered_metrics_excludes_tests() {
    let baseline = create_test_baseline();
    let filter = ExcludeChecks(vec!["tests"]);
    let filtered = FilteredMetrics::new(&baseline, &filter);

    // Coverage and test_time are linked to "tests" check
    assert!(filtered.coverage().is_none());
    assert!(filtered.test_time().is_none());

    // Other metrics should still be present
    assert!(filtered.escapes().is_some());
    assert!(filtered.build_time().is_some());
    assert!(filtered.binary_size().is_some());
}

#[test]
fn filtered_metrics_excludes_build() {
    let baseline = create_test_baseline();
    let filter = ExcludeChecks(vec!["build"]);
    let filtered = FilteredMetrics::new(&baseline, &filter);

    // Build time and binary size are linked to "build" check
    assert!(filtered.build_time().is_none());
    assert!(filtered.binary_size().is_none());

    // Other metrics should still be present
    assert!(filtered.coverage().is_some());
    assert!(filtered.escapes().is_some());
    assert!(filtered.test_time().is_some());
}

#[test]
fn filtered_metrics_excludes_escapes() {
    let baseline = create_test_baseline();
    let filter = ExcludeChecks(vec!["escapes"]);
    let filtered = FilteredMetrics::new(&baseline, &filter);

    assert!(filtered.escapes().is_none());

    // Other metrics should still be present
    assert!(filtered.coverage().is_some());
    assert!(filtered.build_time().is_some());
    assert!(filtered.binary_size().is_some());
    assert!(filtered.test_time().is_some());
}

#[test]
fn filtered_metrics_count_all() {
    let baseline = create_test_baseline();
    let filtered = FilteredMetrics::new(&baseline, &AllChecks);

    // coverage (1) + escapes (2 patterns) + build_time (2) + binary_size (1) + test_time (1) = 7
    assert_eq!(filtered.count(), 7);
}

#[test]
fn filtered_metrics_count_with_exclusions() {
    let baseline = create_test_baseline();
    let filter = ExcludeChecks(vec!["tests"]);
    let filtered = FilteredMetrics::new(&baseline, &filter);

    // escapes (2) + build_time (2) + binary_size (1) = 5
    assert_eq!(filtered.count(), 5);
}

#[test]
fn filtered_metrics_handles_empty_baseline() {
    let baseline = Baseline::default();
    let filtered = FilteredMetrics::new(&baseline, &AllChecks);

    assert!(filtered.coverage().is_none());
    assert!(filtered.escapes().is_none());
    assert!(filtered.build_time().is_none());
    assert!(filtered.binary_size().is_none());
    assert!(filtered.test_time().is_none());
    assert_eq!(filtered.count(), 0);
}
