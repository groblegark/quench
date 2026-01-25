// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use tempfile::tempdir;

#[test]
fn all_runners_returns_expected_count() {
    let runners = all_runners();
    // cargo, bats, go, pytest, vitest, bun, jest, custom = 8 runners
    assert_eq!(runners.len(), 8);
}

#[test]
fn get_runner_finds_cargo() {
    let runner = get_runner("cargo");
    assert!(runner.is_some());
    assert_eq!(runner.unwrap().name(), "cargo");
}

#[test]
fn get_runner_finds_bats() {
    let runner = get_runner("bats");
    assert!(runner.is_some());
    assert_eq!(runner.unwrap().name(), "bats");
}

#[test]
fn get_runner_returns_none_for_unknown() {
    assert!(get_runner("unknown_runner").is_none());
}

#[test]
fn filter_suites_ci_mode_includes_all() {
    let suites = vec![
        TestSuiteConfig {
            runner: "cargo".to_string(),
            ci: false,
            name: None,
            path: None,
            setup: None,
            command: None,
            targets: vec![],
            max_total: None,
            max_avg: None,
            max_test: None,
        },
        TestSuiteConfig {
            runner: "pytest".to_string(),
            ci: true, // CI-only suite
            name: None,
            path: None,
            setup: None,
            command: None,
            targets: vec![],
            max_total: None,
            max_avg: None,
            max_test: None,
        },
    ];

    // In CI mode, all suites should be included
    let filtered = filter_suites_for_mode(&suites, true);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn filter_suites_fast_mode_excludes_ci_only() {
    let suites = vec![
        TestSuiteConfig {
            runner: "cargo".to_string(),
            ci: false,
            name: None,
            path: None,
            setup: None,
            command: None,
            targets: vec![],
            max_total: None,
            max_avg: None,
            max_test: None,
        },
        TestSuiteConfig {
            runner: "pytest".to_string(),
            ci: true, // CI-only suite
            name: None,
            path: None,
            setup: None,
            command: None,
            targets: vec![],
            max_total: None,
            max_avg: None,
            max_test: None,
        },
    ];

    // In fast mode (not CI), CI-only suites should be excluded
    let filtered = filter_suites_for_mode(&suites, false);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].runner, "cargo");
}

#[test]
fn run_setup_command_succeeds() {
    let temp = tempdir().unwrap();
    let result = run_setup_command("echo hello", temp.path());
    assert!(result.is_ok());
}

#[test]
fn run_setup_command_fails_on_bad_command() {
    let temp = tempdir().unwrap();
    let result = run_setup_command("exit 1", temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("setup command failed"));
}

#[test]
fn run_setup_command_reports_stderr() {
    let temp = tempdir().unwrap();
    let result = run_setup_command("echo 'error message' >&2 && exit 1", temp.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("error message"));
}

#[test]
fn runner_names_contains_expected_entries() {
    assert!(RUNNER_NAMES.contains(&"cargo"));
    assert!(RUNNER_NAMES.contains(&"bats"));
    assert!(RUNNER_NAMES.contains(&"pytest"));
    assert!(RUNNER_NAMES.contains(&"custom"));
}
