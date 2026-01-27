// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Ruby RuboCop/Standard suppress directive checking.

use std::path::Path;

use crate::adapter::parse_ruby_suppresses;
use crate::check::{CheckContext, Violation};
use crate::config::RubySuppressConfig;

use super::suppress_common::{UnifiedSuppressDirective, check_suppress_violations_generic};

/// Check RuboCop/Standard suppress directives in a Ruby file.
pub(super) fn check_ruby_suppress_violations(
    ctx: &CheckContext,
    path: &Path,
    content: &str,
    config: &RubySuppressConfig,
    is_test_file: bool,
    limit_reached: &mut bool,
) -> Vec<Violation> {
    // Parse RuboCop/Standard suppress directives
    let suppresses = parse_ruby_suppresses(content, None);

    // Convert to unified format
    let unified: Vec<UnifiedSuppressDirective> = suppresses
        .into_iter()
        .map(|s| {
            let code = s.codes.first().map(|c| c.as_str()).unwrap_or("unknown");
            let directive_type = if s.is_todo { "todo" } else { "disable" };
            let pattern = format!("# {}:{} {}", s.kind, directive_type, code);
            UnifiedSuppressDirective {
                line: s.line,
                codes: s.codes,
                has_comment: s.has_comment,
                comment_text: s.comment_text,
                pattern,
            }
        })
        .collect();

    check_suppress_violations_generic(
        ctx,
        path,
        unified,
        config,
        "ruby",
        "suppress",
        is_test_file,
        limit_reached,
    )
}
