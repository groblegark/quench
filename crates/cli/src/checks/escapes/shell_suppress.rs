//! Shell shellcheck suppress directive checking.

use std::path::Path;

use crate::adapter::parse_shellcheck_suppresses;
use crate::check::{CheckContext, Violation};
use crate::config::ShellSuppressConfig;

use super::suppress_common::{UnifiedSuppressDirective, check_suppress_violations_generic};

/// Check shellcheck suppress directives in a Shell file.
pub(super) fn check_shell_suppress_violations(
    ctx: &CheckContext,
    path: &Path,
    content: &str,
    config: &ShellSuppressConfig,
    is_test_file: bool,
    limit_reached: &mut bool,
) -> Vec<Violation> {
    // Parse shellcheck suppress directives
    let suppresses = parse_shellcheck_suppresses(content, None);

    // Convert to unified format
    let unified: Vec<UnifiedSuppressDirective> = suppresses
        .into_iter()
        .map(|s| {
            let code = s.codes.first().map(|c| c.as_str()).unwrap_or("unknown");
            let pattern = format!("# shellcheck disable={}", code);
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
        "shell",
        "shellcheck",
        is_test_file,
        limit_reached,
    )
}
