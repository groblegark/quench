// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared suppress checking logic for all language suppress checkers.
//!
//! Provides common traits and functions to eliminate duplication across
//! Go, JavaScript, Shell, Ruby, and Rust suppress checkers.

use std::path::Path;

use crate::check::{CheckContext, Violation};
use crate::config::{
    GoSuppressConfig, RubySuppressConfig, ShellSuppressConfig, SuppressConfig, SuppressLevel,
    SuppressScopeConfig,
};

use super::violations::try_create_violation;

// =============================================================================
// Traits and Types for Generic Suppress Checking
// =============================================================================

/// Trait for accessing suppress configuration fields uniformly.
///
/// Implemented by GoSuppressConfig, ShellSuppressConfig, RubySuppressConfig,
/// and SuppressConfig (used by JavaScript and Rust).
pub trait SuppressConfigAccess {
    /// Get the base check level.
    fn check(&self) -> SuppressLevel;
    /// Get the global comment pattern (optional).
    fn comment(&self) -> Option<&str>;
    /// Get the source scope configuration.
    fn source(&self) -> &SuppressScopeConfig;
    /// Get the test scope configuration.
    fn test(&self) -> &SuppressScopeConfig;
}

/// A unified suppress directive for checking.
///
/// Each language parser converts its specific directive type to this unified
/// format before calling the generic checking function.
pub struct UnifiedSuppressDirective {
    /// Line number (0-indexed).
    pub line: usize,
    /// Lint codes being suppressed.
    pub codes: Vec<String>,
    /// Whether a justification comment was found.
    pub has_comment: bool,
    /// The comment text if found.
    pub comment_text: Option<String>,
    /// Pre-formatted pattern string for violation messages.
    pub pattern: String,
}

// =============================================================================
// SuppressConfigAccess Implementations
// =============================================================================

impl SuppressConfigAccess for GoSuppressConfig {
    fn check(&self) -> SuppressLevel {
        self.check
    }
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    fn source(&self) -> &SuppressScopeConfig {
        &self.source
    }
    fn test(&self) -> &SuppressScopeConfig {
        &self.test
    }
}

impl SuppressConfigAccess for ShellSuppressConfig {
    fn check(&self) -> SuppressLevel {
        self.check
    }
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    fn source(&self) -> &SuppressScopeConfig {
        &self.source
    }
    fn test(&self) -> &SuppressScopeConfig {
        &self.test
    }
}

impl SuppressConfigAccess for RubySuppressConfig {
    fn check(&self) -> SuppressLevel {
        self.check
    }
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    fn source(&self) -> &SuppressScopeConfig {
        &self.source
    }
    fn test(&self) -> &SuppressScopeConfig {
        &self.test
    }
}

impl SuppressConfigAccess for SuppressConfig {
    fn check(&self) -> SuppressLevel {
        self.check
    }
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    fn source(&self) -> &SuppressScopeConfig {
        &self.source
    }
    fn test(&self) -> &SuppressScopeConfig {
        &self.test
    }
}

/// Check suppress violations from a list of directives.
///
/// This is the main entry point for the generic suppress checking logic.
/// Each language-specific checker parses directives, converts them to
/// UnifiedSuppressDirective, then calls this function.
// TODO(refactor): Consider grouping ctx+path+limit_reached into a CheckState struct
#[allow(clippy::too_many_arguments)]
pub fn check_suppress_violations_generic<C: SuppressConfigAccess>(
    ctx: &CheckContext,
    path: &Path,
    directives: Vec<UnifiedSuppressDirective>,
    config: &C,
    language: &str,
    violation_type_prefix: &str,
    is_test_file: bool,
    limit_reached: &mut bool,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Get scope config and check level
    let (scope_config, scope_check) = if is_test_file {
        (
            config.test(),
            config.test().check.unwrap_or(SuppressLevel::Allow),
        )
    } else {
        (
            config.source(),
            config.source().check.unwrap_or(config.check()),
        )
    };

    // If allow, no checking needed
    if scope_check == SuppressLevel::Allow {
        return violations;
    }

    for directive in directives {
        if *limit_reached {
            break;
        }

        // Build params for shared checking logic
        let params = SuppressCheckParams {
            scope_config,
            scope_check,
            global_comment: config.comment(),
        };

        let attr_info = SuppressAttrInfo {
            codes: &directive.codes,
            has_comment: directive.has_comment,
            comment_text: directive.comment_text.as_deref(),
        };

        // Use shared checking logic
        if let Some(violation_kind) = check_suppress_attr(&params, &attr_info) {
            let (violation_type, advice) = match violation_kind {
                SuppressViolationKind::Forbidden { ref code } => {
                    let advice = format!(
                        "Suppressing `{}` is forbidden. Remove the suppression or address the issue.",
                        code
                    );
                    (format!("{}_forbidden", violation_type_prefix), advice)
                }
                SuppressViolationKind::MissingComment {
                    ref lint_code,
                    ref required_patterns,
                } => {
                    let advice = build_suppress_missing_comment_advice(
                        language,
                        lint_code.as_deref(),
                        required_patterns,
                    );
                    (format!("{}_missing_comment", violation_type_prefix), advice)
                }
                SuppressViolationKind::AllForbidden => {
                    let advice =
                        "Lint suppressions are forbidden. Remove and fix the underlying issue.";
                    (
                        format!("{}_forbidden", violation_type_prefix),
                        advice.to_string(),
                    )
                }
            };

            if let Some(v) = try_create_violation(
                ctx,
                path,
                (directive.line + 1) as u32,
                &violation_type,
                &advice,
                &directive.pattern,
            ) {
                violations.push(v);
            } else {
                *limit_reached = true;
            }
        }
    }

    violations
}

// =============================================================================
// Original Shared Types and Functions
// =============================================================================

/// Parameters for checking suppress attributes.
pub struct SuppressCheckParams<'a> {
    /// The scope-specific config (source or test).
    pub scope_config: &'a SuppressScopeConfig,
    /// Effective check level for this scope.
    pub scope_check: SuppressLevel,
    /// Global comment pattern (fallback when no per-lint pattern).
    pub global_comment: Option<&'a str>,
}

/// Information about a suppress attribute being checked.
pub struct SuppressAttrInfo<'a> {
    /// Lint codes being suppressed.
    pub codes: &'a [String],
    /// Whether a justification comment was found.
    pub has_comment: bool,
    /// The actual comment text if found.
    pub comment_text: Option<&'a str>,
}

/// Type of suppress violation detected.
#[derive(Debug, PartialEq, Eq)]
pub enum SuppressViolationKind {
    /// A forbidden lint code was suppressed.
    Forbidden { code: String },
    /// Missing required justification comment.
    MissingComment {
        /// The lint code being suppressed (for lint-specific guidance).
        lint_code: Option<String>,
        /// The required comment patterns (if any).
        required_patterns: Vec<String>,
    },
    /// All suppressions are forbidden at this scope level.
    AllForbidden,
}

/// Get lint-specific guidance for Rust lints.
fn get_rust_lint_guidance(lint_code: &str) -> &'static str {
    match lint_code {
        "dead_code" => "\nIs this code still needed?\nIt is usually best to remove dead code.\n",
        "clippy::too_many_arguments" => "Can this function be refactored?",
        "clippy::cast_possible_truncation" => "Is this cast safe?",
        "deprecated" => "Can this deprecated API be replaced?",
        _ => "Is this suppression necessary?",
    }
}

/// Get lint-specific guidance for Shell lints.
fn get_shell_lint_guidance(lint_code: &str) -> &'static str {
    match lint_code {
        "SC2034" => "Is this unused variable needed?",
        "SC2086" => "Is unquoted expansion intentional here?",
        "SC2154" => "Is this variable defined externally?",
        _ => "Is this ShellCheck finding a false positive?",
    }
}

/// Get lint-specific guidance for Go lints.
fn get_go_lint_guidance(lint_code: &str) -> &'static str {
    match lint_code {
        "errcheck" => "Is this error handling necessary to skip?",
        "gosec" => "Is this security finding a false positive?",
        _ => "Is this suppression necessary?",
    }
}

/// Get lint-specific guidance for JavaScript/TypeScript lints.
fn get_js_lint_guidance(lint_code: &str) -> &'static str {
    match lint_code {
        "no-console" => "Is this console output needed in production?",
        "no-explicit-any"
        | "@typescript-eslint/no-explicit-any"
        | "lint/suspicious/noExplicitAny" => "Can this be properly typed instead?",
        "no-unused-vars" | "@typescript-eslint/no-unused-vars" => "Is this variable still needed?",
        _ => "Is this suppression necessary?",
    }
}

/// Format pattern instructions based on number of patterns and lint guidance.
///
/// The conditional phrase ("If so", "If not", "If it should be kept") is determined
/// by the lint guidance question type.
fn format_pattern_instructions(patterns: &[String], guidance: &str) -> String {
    if patterns.is_empty() {
        return String::new();
    }

    // Determine conditional phrase from the guidance question
    let condition = if guidance.starts_with("Can this function be refactored") {
        "not"
    } else if guidance.contains("still needed") || guidance.contains("unused variable needed") {
        // "Is this code still needed?\nIt is usually best to remove dead code.", "Is this unused variable needed?"
        "it should be kept"
    } else if guidance.starts_with("Is this") || guidance.starts_with("Is unquoted") {
        // Questions like "Is this cast safe?", "Is this variable defined externally?"
        "so"
    } else {
        // Default
        "it should be kept"
    };

    if patterns.len() == 1 {
        // Single pattern
        format!("If {}, add:\n  {} ...", condition, patterns[0])
    } else {
        // Multiple patterns
        let formatted_patterns = patterns
            .iter()
            .map(|p| format!("  {} ...", p))
            .collect::<Vec<_>>()
            .join("\n");
        format!("If {}, add one of:\n{}", condition, formatted_patterns)
    }
}

/// Build the three-part suppress missing comment advice message.
///
/// Format:
/// 1. General statement: "Lint suppression requires justification."
/// 2. Lint-specific guidance: A question tailored to the specific lint
/// 3. Pattern instructions: How to add the required comment
pub fn build_suppress_missing_comment_advice(
    language: &str,
    lint_code: Option<&str>,
    required_patterns: &[String],
) -> String {
    let mut parts = Vec::new();

    // Part 1: General statement
    parts.push("Lint suppression requires justification.".to_string());

    // Part 2: Lint-specific guidance
    let guidance = if let Some(code) = lint_code {
        match language {
            "rust" => get_rust_lint_guidance(code),
            "shell" => get_shell_lint_guidance(code),
            "go" => get_go_lint_guidance(code),
            "javascript" => get_js_lint_guidance(code),
            _ => "Is this suppression necessary?",
        }
    } else {
        "Is this suppression necessary?"
    };
    parts.push(guidance.to_string());

    // Part 3: Pattern instructions
    if !required_patterns.is_empty() {
        parts.push(format_pattern_instructions(required_patterns, guidance));
    } else {
        // No specific patterns - generic guidance
        let msg = match language {
            "rust" => "Add a comment above the attribute.",
            "shell" => "Add a comment above the directive.",
            "go" => "Add a comment above the directive or inline (//nolint:code // reason).",
            "javascript" => "Add a comment above the directive or use inline reason (-- reason).",
            _ => "Add a comment above the directive.",
        };
        parts.push(msg.to_string());
    }

    parts.join("\n")
}

/// Check a suppress attribute against scope config.
///
/// Returns `None` if no violation, `Some(kind)` if violation detected.
/// Stops at the first violation found.
pub fn check_suppress_attr(
    params: &SuppressCheckParams,
    attr: &SuppressAttrInfo,
) -> Option<SuppressViolationKind> {
    // 1. Check forbid list first
    for code in attr.codes {
        if is_code_in_list(code, &params.scope_config.forbid) {
            return Some(SuppressViolationKind::Forbidden { code: code.clone() });
        }
    }

    // 2. Check allow list - if any code matches, skip remaining checks
    for code in attr.codes {
        if is_code_in_list(code, &params.scope_config.allow) {
            return None;
        }
    }

    // 3. Check if all suppressions are forbidden at this level
    if params.scope_check == SuppressLevel::Forbid {
        return Some(SuppressViolationKind::AllForbidden);
    }

    // 4. Check comment requirement
    if params.scope_check == SuppressLevel::Comment {
        let (lint_code, required_patterns) = find_required_patterns(params, attr);
        if !has_valid_comment(attr, &required_patterns) {
            return Some(SuppressViolationKind::MissingComment {
                lint_code,
                required_patterns,
            });
        }
    }

    None
}

/// Find the required comment patterns for an attribute.
/// Checks per-lint patterns first, then falls back to global.
/// Returns the lint code (if found) and a list of valid patterns (any match is acceptable).
fn find_required_patterns(
    params: &SuppressCheckParams,
    attr: &SuppressAttrInfo,
) -> (Option<String>, Vec<String>) {
    // Check per-lint patterns first (first matching code wins)
    for code in attr.codes {
        if let Some(patterns) = params.scope_config.patterns.get(code) {
            return (Some(code.clone()), patterns.clone());
        }
    }
    // Fall back to global pattern
    let patterns = params
        .global_comment
        .map(|p| vec![p.to_string()])
        .unwrap_or_default();
    (attr.codes.first().cloned(), patterns)
}

/// Check if the attribute has a valid justification comment.
/// If required_patterns is non-empty, comment must match one of them.
/// If required_patterns is empty, any non-empty comment is valid.
fn has_valid_comment(attr: &SuppressAttrInfo, required_patterns: &[String]) -> bool {
    if !attr.has_comment {
        return false;
    }

    // If no specific patterns required, any comment is valid
    if required_patterns.is_empty() {
        return true;
    }

    // Need to match one of the patterns
    let Some(text) = &attr.comment_text else {
        return false;
    };

    let norm_text = normalize_comment_text(text);
    required_patterns.iter().any(|pattern| {
        let norm_pattern = normalize_comment_pattern(pattern);
        norm_text.starts_with(&norm_pattern)
    })
}

/// Normalize a comment pattern by stripping common prefixes.
fn normalize_comment_pattern(pattern: &str) -> String {
    pattern
        .trim()
        .trim_start_matches("//")
        .trim_start_matches('#')
        .trim()
        .to_string()
}

/// Normalize comment text by stripping common prefixes.
fn normalize_comment_text(text: &str) -> String {
    text.trim()
        .trim_start_matches("//")
        .trim_start_matches('#')
        .trim()
        .to_string()
}

/// Check if a lint code matches any pattern in a list.
/// Supports exact match and prefix match (e.g., "clippy" matches "clippy::unwrap_used").
fn is_code_in_list(code: &str, list: &[String]) -> bool {
    list.iter().any(|pattern| code_matches(code, pattern))
}

/// Check if a code matches a pattern.
/// Supports exact match and prefix match with `::` separator.
fn code_matches(code: &str, pattern: &str) -> bool {
    code == pattern || code.starts_with(&format!("{}::", pattern))
}

#[cfg(test)]
#[path = "suppress_common_tests.rs"]
mod tests;
