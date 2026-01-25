// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Cargo test runner.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::coverage::collect_rust_coverage;
use super::{RunnerContext, TestResult, TestRunResult, TestRunner};
use crate::config::TestSuiteConfig;

/// Cargo test runner for Rust projects.
pub struct CargoRunner;

impl TestRunner for CargoRunner {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn available(&self, ctx: &RunnerContext) -> bool {
        // Check for Cargo.toml in project root
        ctx.root.join("Cargo.toml").exists()
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // Run setup command if specified
        if let Some(setup) = &config.setup
            && let Err(e) = super::run_setup_command(setup, ctx.root)
        {
            return TestRunResult::failed(Duration::ZERO, e);
        }

        let start = Instant::now();

        // Build command - use standard cargo test output (stable Rust compatible)
        let mut cmd = Command::new("cargo");
        cmd.args(["test", "--release"]);

        // Set working directory
        let work_dir = config
            .path
            .as_ref()
            .map(|p| ctx.root.join(p))
            .unwrap_or_else(|| ctx.root.to_path_buf());
        cmd.current_dir(&work_dir);

        // Capture output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = match cmd.output() {
            Ok(out) => out,
            Err(e) => {
                return TestRunResult::failed(start.elapsed(), format!("failed to run cargo: {e}"));
            }
        };

        let total_time = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse human-readable output
        let mut result = parse_cargo_output(&stdout, total_time);

        // Check if cargo command itself failed (compilation error) with no tests parsed
        if !output.status.success() && result.tests.is_empty() && result.passed {
            let msg = stderr.lines().take(10).collect::<Vec<_>>().join("\n");
            return TestRunResult::failed(total_time, format!("cargo test failed:\n{msg}"));
        }

        // Collect coverage if requested
        if ctx.collect_coverage {
            let coverage = collect_rust_coverage(ctx.root, config.path.as_deref());
            if let Some(line_coverage) = coverage.line_coverage {
                let mut cov_map = HashMap::new();
                cov_map.insert("rust".to_string(), line_coverage);
                result = result.with_coverage(cov_map);
            }
            // Add per-package coverage if available
            if !coverage.packages.is_empty() {
                result = result.with_package_coverage(coverage.packages);
            }
        }

        result
    }
}

/// Parse cargo test human-readable output with optimizations.
///
/// Output format (examples):
/// ```text
/// test tests::test_add ... ok
/// test tests::test_fail ... FAILED
/// test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
/// test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
/// ```
///
/// Optimizations:
/// - Pre-allocates test vector based on line count hint
/// - Uses string slices to minimize allocations
/// - Inline helper for test line parsing
pub fn parse_cargo_output(stdout: &str, total_time: Duration) -> TestRunResult {
    // Pre-count lines starting with "test " for capacity hint
    let test_line_count = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with("test "))
        .count();

    let mut tests = Vec::with_capacity(test_line_count);
    let mut suite_passed = true;

    for line in stdout.lines() {
        let line = line.trim();

        // Parse individual test results
        if let Some(rest) = line.strip_prefix("test ")
            && let Some((name, result)) = parse_test_line(rest)
        {
            let passed = result == "ok";
            tests.push(if passed {
                TestResult::passed(name, Duration::ZERO)
            } else {
                suite_passed = false;
                TestResult::failed(name, Duration::ZERO)
            });
        }

        // Parse suite summary: "test result: FAILED. X passed; Y failed; ..."
        // This must be a separate check (not else if) because "test result:"
        // also matches the "test " prefix above.
        if line.starts_with("test result: ") && line.contains("FAILED") {
            suite_passed = false;
        }
    }

    let mut result = if suite_passed {
        TestRunResult::passed(total_time)
    } else {
        TestRunResult::failed(total_time, "tests failed")
    };
    result.tests = tests;
    result
}

/// Parse a test line after "test " prefix.
/// Returns (name, result) where result is "ok" or "FAILED".
#[inline]
fn parse_test_line(rest: &str) -> Option<(&str, &str)> {
    // Format: "<name> ... ok" or "<name> ... FAILED"
    let sep_pos = rest.rfind(" ... ")?;
    let name = &rest[..sep_pos];
    let result = &rest[sep_pos + 5..]; // Skip " ... "
    if result == "ok" || result == "FAILED" {
        Some((name, result))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "cargo_tests.rs"]
mod tests;
