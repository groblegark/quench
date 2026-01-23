//! Shell shellcheck suppress directive checking.

use std::path::Path;

use crate::adapter::parse_shellcheck_suppresses;
use crate::check::{CheckContext, Violation};
use crate::config::{ShellSuppressConfig, SuppressLevel};

use super::try_create_violation;

/// Check shellcheck suppress directives in a Shell file.
pub(super) fn check_shell_suppress_violations(
    ctx: &CheckContext,
    path: &Path,
    content: &str,
    config: &ShellSuppressConfig,
    is_test_file: bool,
    limit_reached: &mut bool,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Determine effective check level based on source vs test
    let scope_config = if is_test_file {
        &config.test
    } else {
        &config.source
    };

    let effective_check = if is_test_file {
        config.test.check.unwrap_or(SuppressLevel::Allow)
    } else {
        scope_config.check.unwrap_or(config.check)
    };

    // If allow, no checking needed
    if effective_check == SuppressLevel::Allow {
        return violations;
    }

    // Parse shellcheck suppress directives
    let suppresses = parse_shellcheck_suppresses(content, config.comment.as_deref());

    for suppress in suppresses {
        if *limit_reached {
            break;
        }

        // Check each shellcheck code
        for code in &suppress.codes {
            if *limit_reached {
                break;
            }

            let pattern = format!("# shellcheck disable={}", code);

            // Check forbid list first
            if scope_config.forbid.contains(code) {
                let advice = format!(
                    "Suppressing shellcheck {} is forbidden. Remove the suppression or fix the issue.",
                    code
                );
                if let Some(v) = try_create_violation(
                    ctx,
                    path,
                    (suppress.line + 1) as u32,
                    "shellcheck_forbidden",
                    &advice,
                    &pattern,
                ) {
                    violations.push(v);
                } else {
                    *limit_reached = true;
                }
                continue;
            }

            // Check allow list (skip check)
            if scope_config.allow.contains(code) {
                continue;
            }

            // Check if comment is required
            if effective_check == SuppressLevel::Comment && !suppress.has_comment {
                let advice = if let Some(ref pat) = config.comment {
                    format!(
                        "Shellcheck suppression requires justification. Add a {} comment.",
                        pat
                    )
                } else {
                    "Shellcheck suppression requires justification. Add a comment above explaining why."
                        .into()
                };
                if let Some(v) = try_create_violation(
                    ctx,
                    path,
                    (suppress.line + 1) as u32,
                    "shellcheck_missing_comment",
                    &advice,
                    &pattern,
                ) {
                    violations.push(v);
                } else {
                    *limit_reached = true;
                }
            }

            // Forbid level means all suppressions fail
            if effective_check == SuppressLevel::Forbid {
                let advice = format!(
                    "Shellcheck suppressions are forbidden. Fix the underlying issue {} instead of disabling it.",
                    code
                );
                if let Some(v) = try_create_violation(
                    ctx,
                    path,
                    (suppress.line + 1) as u32,
                    "shellcheck_forbidden",
                    &advice,
                    &pattern,
                ) {
                    violations.push(v);
                } else {
                    *limit_reached = true;
                }
            }
        }
    }

    violations
}
