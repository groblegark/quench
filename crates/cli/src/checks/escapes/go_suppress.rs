// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Go nolint directive checking for the escapes check.
//!
//! Checks `//nolint` directives and enforces comment requirements.

use std::path::Path;

use crate::adapter::parse_nolint_directives;
use crate::check::{CheckContext, Violation};
use crate::config::GoSuppressConfig;

use super::suppress_common::{UnifiedSuppressDirective, check_suppress_violations_generic};

/// Check Go nolint directives and return violations.
pub fn check_go_suppress_violations(
    ctx: &CheckContext,
    path: &Path,
    content: &str,
    config: &GoSuppressConfig,
    is_test_file: bool,
    limit_reached: &mut bool,
) -> Vec<Violation> {
    // Parse nolint directives
    let directives = parse_nolint_directives(content, config.comment.as_deref());

    // Convert to unified format
    let unified: Vec<UnifiedSuppressDirective> = directives
        .into_iter()
        .map(|d| {
            let pattern = if d.codes.is_empty() {
                "//nolint".to_string()
            } else {
                format!("//nolint:{}", d.codes.join(","))
            };
            UnifiedSuppressDirective {
                line: d.line,
                codes: d.codes,
                has_comment: d.has_comment,
                comment_text: d.comment_text,
                pattern,
            }
        })
        .collect();

    check_suppress_violations_generic(
        ctx,
        path,
        unified,
        config,
        "go",
        "suppress",
        is_test_file,
        limit_reached,
    )
}
