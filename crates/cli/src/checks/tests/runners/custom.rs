// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Custom command test runner.
//!
//! Executes arbitrary test commands and reports pass/fail based on exit code.
//! Does not provide per-test timing information.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{RunnerContext, TestRunResult, TestRunner};
use crate::config::TestSuiteConfig;

/// Custom command runner for arbitrary test commands.
pub struct CustomRunner;

impl TestRunner for CustomRunner {
    fn name(&self) -> &'static str {
        "custom"
    }

    fn available(&self, _ctx: &RunnerContext) -> bool {
        // Always available; command existence is checked at runtime
        true
    }

    fn run(&self, config: &TestSuiteConfig, ctx: &RunnerContext) -> TestRunResult {
        // Custom runner requires a command
        let command = match &config.command {
            Some(cmd) => cmd,
            None => {
                return TestRunResult::failed(
                    Duration::ZERO,
                    "custom runner requires 'command' field",
                );
            }
        };

        // Run setup command if specified
        if let Some(setup) = &config.setup
            && let Err(e) = super::run_setup_command(setup, ctx.root)
        {
            return TestRunResult::failed(Duration::ZERO, e);
        }

        let start = Instant::now();

        // Execute command via shell
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .current_dir(ctx.root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        } else {
            Command::new("sh")
                .args(["-c", command])
                .current_dir(ctx.root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        };

        let total_time = start.elapsed();

        match output {
            Ok(out) if out.status.success() => TestRunResult::passed(total_time),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let truncated: String = stderr.lines().take(10).collect::<Vec<_>>().join("\n");
                let message = if truncated.is_empty() {
                    format!("command failed with exit code {:?}", out.status.code())
                } else {
                    truncated
                };
                TestRunResult::failed(total_time, message)
            }
            Err(e) => TestRunResult::failed(total_time, format!("failed to execute command: {e}")),
        }
    }
}

#[cfg(test)]
#[path = "custom_tests.rs"]
mod tests;
