// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::time::Duration;

use super::*;

#[test]
fn parse_go_test_output_extracts_timing() {
    let stdout = r#"
=== RUN   TestFoo
--- PASS: TestFoo (0.50s)
=== RUN   TestBar
--- PASS: TestBar (1.20s)
=== RUN   TestBaz
--- FAIL: TestBaz (0.30s)
FAIL
"#;

    let (count, max, slowest) = parse_go_test_output(stdout, "");

    assert_eq!(count, 3);
    assert_eq!(max, Duration::from_secs_f64(1.20));
    assert_eq!(slowest, Some("TestBar".to_string()));
}

#[test]
fn parse_rust_test_output_counts_tests() {
    let stderr = r#"
   Compiling myapp v0.1.0
    Finished test
     Running unittests src/lib.rs
test foo::test_one ... ok
test foo::test_two ... ok
test bar::test_three ... FAILED
"#;

    let (count, _max, _slowest) = parse_rust_test_output("", stderr);

    assert_eq!(count, 3);
}

#[test]
fn test_timing_metrics_to_json() {
    let metrics = TestTimingMetrics {
        total: Duration::from_secs(30),
        avg: Duration::from_secs(1),
        max: Duration::from_secs(5),
        test_count: 30,
        slowest_test: Some("TestSlow".to_string()),
    };

    let json = metrics.to_json();

    assert_eq!(json["total"], 30.0);
    assert_eq!(json["avg"], 1.0);
    assert_eq!(json["max"], 5.0);
    assert_eq!(json["test_count"], 30);
    assert_eq!(json["slowest_test"], "TestSlow");
}

#[test]
fn estimate_from_output_counts_pass_fail() {
    let stdout = "test1 ok\ntest2 PASS\ntest3 FAIL\nsome other line\n";

    let (count, _max, _slowest) = estimate_from_output(stdout);

    assert_eq!(count, 3);
}
