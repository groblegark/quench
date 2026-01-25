#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parses_passing_test() {
    let output = r#"
running 1 test
test tests::add ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
    assert!(result.tests[0].passed);
    assert_eq!(result.tests[0].name, "tests::add");
}

#[test]
fn parses_failing_test() {
    let output = r#"
running 1 test
test tests::fail ... FAILED

failures:

---- tests::fail stdout ----
thread 'tests::fail' panicked at 'assertion failed', src/lib.rs:10:5

failures:
    tests::fail

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 1);
    assert!(!result.tests[0].passed);
}

#[test]
fn handles_mixed_output() {
    // Cargo emits compilation output before test results
    let output = r#"
   Compiling test_project v0.1.0
    Finished release target(s) in 0.1s
     Running tests

running 2 tests
test tests::a ... ok
test tests::b ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 2);
}

#[test]
fn handles_empty_output() {
    let result = parse_cargo_output("", Duration::from_secs(0));
    assert!(result.passed);
    assert!(result.tests.is_empty());
}

#[test]
fn suite_failed_result_marks_result_failed() {
    let output = r#"
running 1 test
test test::a ... ok

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    // Suite failed result should mark result as failed
    assert!(!result.passed);
}

#[test]
fn handles_multiple_test_events() {
    let output = r#"
running 3 tests
test test::one ... ok
test test::two ... ok
test test::three ... FAILED

failures:

---- test::three stdout ----
assertion failed

failures:
    test::three

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(!result.passed);
    assert_eq!(result.tests.len(), 3);
    assert_eq!(result.passed_count(), 2);
    assert_eq!(result.failed_count(), 1);
}

#[test]
fn ignores_non_test_lines() {
    let output = r#"
   Compiling foo v0.1.0
warning: unused variable
  --> src/lib.rs:1:5
   |
1  | let x = 5;
   |     ^ help: if this is intentional, prefix it with an underscore: `_x`

running 1 test
test tests::a ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 1);
}

#[test]
fn parses_ignored_tests() {
    let output = r#"
running 3 tests
test tests::active ... ok
test tests::slow_test ... ignored
test tests::another_ignored ... ignored

test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.tests.len(), 3);
    assert_eq!(result.passed_count(), 1);
    assert_eq!(result.skipped_count(), 2);

    // Verify the individual test results
    assert!(result.tests[0].passed && !result.tests[0].skipped);
    assert!(result.tests[1].passed && result.tests[1].skipped);
    assert!(result.tests[2].passed && result.tests[2].skipped);
}

#[test]
fn skipped_tests_dont_count_as_failed() {
    let output = r#"
running 2 tests
test tests::ok ... ok
test tests::skip ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;
    let result = parse_cargo_output(output, Duration::from_secs(1));

    assert!(result.passed);
    assert_eq!(result.failed_count(), 0);
    assert_eq!(result.skipped_count(), 1);
}

// =============================================================================
// Error Categorization Tests
// =============================================================================

#[test]
fn categorize_compilation_error() {
    let stderr = r#"
error[E0425]: cannot find value `x` in this scope
  --> src/lib.rs:2:5
   |
2  |     x
   |     ^ not found in this scope

error: could not compile `test_project` due to previous error
"#;
    let advice = categorize_cargo_error(stderr, Some(101));
    assert!(advice.contains("compilation failed"));
}

#[test]
fn categorize_no_tests_found() {
    let stderr = "error: no test target matches pattern `nonexistent`";
    let advice = categorize_cargo_error(stderr, Some(101));
    assert!(advice.contains("no tests found"));
}

#[test]
fn categorize_timeout_sigkill() {
    let stderr = "";
    let advice = categorize_cargo_error(stderr, Some(137));
    assert!(advice.contains("timed out"));
}

#[test]
fn categorize_timeout_command() {
    let stderr = "";
    let advice = categorize_cargo_error(stderr, Some(124));
    assert!(advice.contains("timed out"));
}

#[test]
fn categorize_out_of_memory() {
    let stderr = "error: out of memory";
    let advice = categorize_cargo_error(stderr, Some(1));
    assert!(advice.contains("out of memory"));
}

#[test]
fn categorize_segfault_as_oom() {
    let stderr = "";
    let advice = categorize_cargo_error(stderr, Some(139));
    assert!(advice.contains("out of memory"));
}

#[test]
fn categorize_linker_error() {
    let stderr = "error: linker `cc` not found";
    let advice = categorize_cargo_error(stderr, Some(1));
    assert!(advice.contains("linking failed"));
}

#[test]
fn categorize_generic_failure() {
    let stderr = "some random error";
    let advice = categorize_cargo_error(stderr, Some(1));
    assert_eq!(advice, "tests failed");
}
