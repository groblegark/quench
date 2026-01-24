// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Documentation validation check.
//!
//! Validates:
//! - TOC entries reference existing files
//! - Markdown links point to existing files
//! - Specs have required sections
//! - Feature commits have documentation (CI mode)

mod commit;
mod content;
mod links;
mod specs;
mod toc;

use std::path::Path;

use crate::adapter::build_glob_set;
use crate::check::{Check, CheckContext, CheckResult, Violation};

/// Check if a docs subcheck is enabled.
///
/// Returns `true` if the check should run, `false` if disabled.
/// Checks subcheck config first, then falls back to parent config.
pub(super) fn is_check_enabled(subcheck: Option<&str>, parent: Option<&str>) -> bool {
    matches!(subcheck.or(parent).unwrap_or("error"), "error" | "warn")
}

/// Process markdown files matching include/exclude patterns.
///
/// Calls the validator closure for each matching file with:
/// - relative_path: Path relative to ctx.root
/// - content: File contents
pub(super) fn process_markdown_files<F>(
    ctx: &CheckContext,
    include: &[String],
    exclude: &[String],
    violations: &mut Vec<Violation>,
    mut validator: F,
) where
    F: FnMut(&CheckContext, &Path, &str, &mut Vec<Violation>),
{
    let include_set = build_glob_set(include);
    let exclude_set = build_glob_set(exclude);

    for walked in ctx.files {
        let relative_path = walked.path.strip_prefix(ctx.root).unwrap_or(&walked.path);
        let path_str = relative_path.to_string_lossy();

        if !include_set.is_match(&*path_str) {
            continue;
        }
        if exclude_set.is_match(&*path_str) {
            continue;
        }

        let content = match std::fs::read_to_string(&walked.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        validator(ctx, relative_path, &content, violations);
    }
}

pub struct DocsCheck;

impl Check for DocsCheck {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn description(&self) -> &'static str {
        "Documentation validation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let mut violations = Vec::new();

        // Check if docs check is disabled
        if !is_check_enabled(None, ctx.config.check.docs.check.as_deref()) {
            return CheckResult::passed("docs");
        }

        // Run TOC validation
        toc::validate_toc(ctx, &mut violations);

        // Run link validation
        links::validate_links(ctx, &mut violations);

        // Run specs validation
        specs::validate_specs(ctx, &mut violations);

        // Run commit validation (CI mode only)
        if ctx.ci_mode {
            commit::validate_commit_docs(ctx, &mut violations);
        }

        // Respect violation limit
        if let Some(limit) = ctx.limit {
            violations.truncate(limit);
        }

        // Collect metrics for JSON output
        let metrics =
            specs::collect_metrics(ctx).map(|m| serde_json::to_value(m).unwrap_or_default());

        let result = if violations.is_empty() {
            CheckResult::passed("docs")
        } else {
            CheckResult::failed("docs", violations)
        };

        if let Some(m) = metrics {
            result.with_metrics(m)
        } else {
            result
        }
    }

    fn default_enabled(&self) -> bool {
        true
    }
}
