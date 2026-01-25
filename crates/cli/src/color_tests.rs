// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use termcolor::Color;

// NOTE: Environment variable tests for NO_COLOR and COLOR are in
// tests/specs/output/format.rs and tests/specs/config/env.rs
// because env var manipulation is not safe in parallel unit tests.
//
// The resolve_color() function behavior is:
// - NO_COLOR set -> ColorChoice::Never
// - COLOR set -> ColorChoice::Always
// - Neither -> auto-detect based on TTY and agent environment

#[test]
fn scheme_check_name_is_bold() {
    let spec = scheme::check_name();
    assert!(spec.bold());
}

#[test]
fn scheme_fail_is_red_bold() {
    let spec = scheme::fail();
    assert_eq!(spec.fg(), Some(&Color::Red));
    assert!(spec.bold());
}

#[test]
fn scheme_pass_is_green_bold() {
    let spec = scheme::pass();
    assert_eq!(spec.fg(), Some(&Color::Green));
    assert!(spec.bold());
}

#[test]
fn scheme_path_is_cyan() {
    let spec = scheme::path();
    assert_eq!(spec.fg(), Some(&Color::Cyan));
}

#[test]
fn scheme_line_number_is_yellow() {
    let spec = scheme::line_number();
    assert_eq!(spec.fg(), Some(&Color::Yellow));
}

#[test]
fn scheme_advice_has_no_color() {
    let spec = scheme::advice();
    assert!(spec.fg().is_none());
    assert!(!spec.bold());
}

// =============================================================================
// Help text colorization (ANSI 256-color codes)
// =============================================================================

// NOTE: Tests that require specific color output depend on should_colorize()
// which is cached and affected by TTY state. Structure-preserving tests below
// verify the colorization logic works correctly regardless of color state.

#[test]
fn color_codes_match_wok_conventions() {
    assert_eq!(codes::HEADER, 74, "Header should be pastel cyan/steel blue");
    assert_eq!(codes::LITERAL, 250, "Literal should be light grey");
    assert_eq!(codes::CONTEXT, 245, "Context should be medium grey");
}

#[test]
fn fg256_produces_correct_escape_sequence() {
    assert_eq!(fg256(0), "\x1b[38;5;0m");
    assert_eq!(fg256(74), "\x1b[38;5;74m");
    assert_eq!(fg256(245), "\x1b[38;5;245m");
    assert_eq!(fg256(250), "\x1b[38;5;250m");
    assert_eq!(fg256(255), "\x1b[38;5;255m");
}

#[test]
fn reset_sequence_is_correct() {
    assert_eq!(RESET, "\x1b[0m");
}

/// Strip all ANSI escape sequences from a string (for testing)
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm'
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[test]
fn header_contains_text() {
    let result = header("Examples:");
    assert!(result.contains("Examples:"));
    assert_eq!(strip_ansi(&result), "Examples:");
}

#[test]
fn literal_contains_text() {
    let result = literal("quench check");
    assert!(result.contains("quench check"));
    assert_eq!(strip_ansi(&result), "quench check");
}

#[test]
fn context_contains_text() {
    let result = context("value");
    assert!(result.contains("value"));
    assert_eq!(strip_ansi(&result), "value");
}

#[test]
fn find_description_start_with_two_spaces() {
    assert_eq!(find_description_start("cmd  desc"), Some(3));
    assert_eq!(find_description_start("quench check  Run checks"), Some(12));
}

#[test]
fn find_description_start_with_many_spaces() {
    assert_eq!(find_description_start("cmd     desc"), Some(3));
    assert_eq!(
        find_description_start("quench check --all   List all"),
        Some(18)
    );
}

#[test]
fn find_description_start_single_space_returns_none() {
    assert_eq!(find_description_start("cmd desc"), None);
    assert_eq!(find_description_start("quench check"), None);
    assert_eq!(find_description_start("just some words here"), None);
}

#[test]
fn find_description_start_empty_input() {
    assert_eq!(find_description_start(""), None);
}

#[test]
fn find_description_start_only_spaces() {
    assert_eq!(find_description_start("   "), None);
    assert_eq!(find_description_start("      "), None);
}

#[test]
fn colorize_command_simple_command() {
    let result = colorize_command("quench check");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "quench check");
}

#[test]
fn colorize_command_with_quoted_string() {
    let result = colorize_command(r#"quench check "path""#);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, r#"quench check "path""#);
}

#[test]
fn colorize_command_with_flag_and_value() {
    let result = colorize_command("quench check -o json");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "quench check -o json");
}

#[test]
fn colorize_command_with_placeholder() {
    let result = colorize_command("quench check <path>");
    let stripped = strip_ansi(&result);
    assert_eq!(stripped, "quench check <path>");
}

#[test]
fn colorize_command_empty_string() {
    let result = colorize_command("");
    assert_eq!(result, "");
}

#[test]
fn examples_header_line() {
    let input = "Examples:";
    let result = examples(input);
    assert!(result.contains("Examples:"));
    assert_eq!(strip_ansi(&result), "Examples:");
}

#[test]
fn examples_command_line() {
    let input = "  quench check .  Run checks";
    let result = examples(input);
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_documentation_line() {
    let input = "  Syntax: -o FORMAT";
    let result = examples(input);
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_plain_line_no_pattern() {
    let input = "  This is just plain text";
    let result = examples(input);
    // When colors disabled or no pattern matched, should be unchanged
    assert_eq!(strip_ansi(&result), input);
}

#[test]
fn examples_empty_input() {
    let result = examples("");
    assert_eq!(result, "");
}

#[test]
fn examples_blank_lines_preserved() {
    let input = "Examples:\n\n  quench check  Run";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert!(stripped.contains("\n\n"));
}

#[test]
fn examples_multiline_structure() {
    let input = "\
Examples:
  quench check .  Run checks
  quench init     Initialize

Output Formats:
  Syntax: -o FORMAT
  Formats: text, json";

    let result = examples(input);
    let stripped = strip_ansi(&result);

    // Verify structure preserved
    assert_eq!(stripped, input);

    // Verify line count preserved
    assert_eq!(result.lines().count(), input.lines().count());
}

#[test]
fn examples_indentation_preserved() {
    let input = "    deeply indented  desc";
    let result = examples(input);
    let stripped = strip_ansi(&result);
    assert!(stripped.starts_with("    "));
}
