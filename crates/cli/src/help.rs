// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Custom help formatting with consolidated --[no-] flags and colorization.
//!
//! Clap displays negatable flags as separate lines:
//!   --color      Force color output
//!   --no-color   Disable color output
//!
//! This module consolidates them into a single line:
//!   --[no-]color   Enable/disable color output
//!
//! It also provides colorized help output using ANSI 256-color codes.

use crate::color;
use clap::Command;
use clap::builder::styling::Styles;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Generate clap Styles for help output with consistent color conventions.
pub fn styles() -> Styles {
    if !color::should_colorize() {
        return Styles::plain();
    }

    use anstyle::{Ansi256Color, Color, Style};

    let header = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(color::codes::HEADER))));
    let literal = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(color::codes::LITERAL))));
    let placeholder =
        Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(color::codes::CONTEXT))));
    let context = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(color::codes::CONTEXT))));

    Styles::styled()
        .header(header)
        .usage(header)
        .literal(literal)
        .placeholder(placeholder)
        .valid(context)
}

/// Regex to match option lines in help output.
/// Captures: (leading_space, short_opt?, long_opt, value?, description)
#[allow(clippy::expect_used)]
static OPTION_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\s+)(?:(-\w),\s+)?(--[\w-]+)(?:\s+<([^>]+)>)?(?:\s+(.*))?$")
        .expect("valid regex")
});

/// Regex to detect --no-X flags.
#[allow(clippy::expect_used)]
static NO_FLAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^--no-(.+)$").expect("valid regex"));

/// Formats help text with consolidated --[no-] flags.
pub fn format_help(cmd: &mut Command) -> String {
    let mut help = Vec::new();
    // write_help only fails on IO errors, which can't happen with Vec<u8>
    let _ = cmd.write_help(&mut help);
    // Help output from clap is always valid UTF-8
    let raw_help = String::from_utf8_lossy(&help);
    consolidate_negatable_flags(&raw_help)
}

/// Parsed option line information.
#[derive(Debug, Clone)]
struct OptionInfo {
    /// Line index in original help
    line_idx: usize,
    /// Leading whitespace
    leading: String,
    /// Short option (e.g., "-o")
    short: Option<String>,
    /// Long option without -- prefix
    long_name: String,
    /// Value placeholder (e.g., "N" for --limit <N>)
    value: Option<String>,
    /// Description text
    description: String,
}

/// Identifies --flag/--no-flag pairs and merges them.
fn consolidate_negatable_flags(help: &str) -> String {
    let lines: Vec<&str> = help.lines().collect();
    let mut options: HashMap<String, OptionInfo> = HashMap::new();
    let mut no_flags: Vec<OptionInfo> = Vec::new();
    let mut lines_to_remove: Vec<usize> = Vec::new();
    let mut replacements: HashMap<usize, String> = HashMap::new();

    // Parse all option lines
    for (idx, line) in lines.iter().enumerate() {
        if let Some(caps) = OPTION_LINE_RE.captures(line) {
            let leading = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let short = caps.get(2).map(|m| m.as_str().to_string());
            let long_with_dashes = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let value = caps.get(4).map(|m| m.as_str().to_string());
            let description = caps.get(5).map(|m| m.as_str()).unwrap_or("").to_string();

            let info = OptionInfo {
                line_idx: idx,
                leading,
                short,
                long_name: long_with_dashes
                    .strip_prefix("--")
                    .unwrap_or(long_with_dashes)
                    .to_string(),
                value,
                description,
            };

            if NO_FLAG_RE.is_match(long_with_dashes) {
                // This is a --no-X flag
                no_flags.push(info);
            } else {
                // This is a regular flag
                options.insert(info.long_name.clone(), info);
            }
        }
    }

    // Process --no-X flags
    for no_info in no_flags {
        let positive_name = no_info
            .long_name
            .strip_prefix("no-")
            .unwrap_or(&no_info.long_name);

        if let Some(positive_info) = options.get(positive_name) {
            // Found a matching positive flag - consolidate!
            let consolidated = build_consolidated_line(positive_info, &no_info);
            replacements.insert(positive_info.line_idx, consolidated);
            lines_to_remove.push(no_info.line_idx);
        }
        // If no positive counterpart, leave --no-X as-is
    }

    // Build output
    let mut result = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if lines_to_remove.contains(&idx) {
            continue;
        }
        if let Some(replacement) = replacements.get(&idx) {
            result.push(replacement.as_str());
        } else {
            result.push(*line);
        }
    }

    result.join("\n")
}

/// Build a consolidated line from positive and negative flag info.
fn build_consolidated_line(positive: &OptionInfo, _negative: &OptionInfo) -> String {
    let mut parts = Vec::new();

    // Leading whitespace
    parts.push(positive.leading.as_str());

    // Short option if present
    if let Some(ref short) = positive.short {
        parts.push(short);
        parts.push(", ");
    }

    // Consolidated long option
    let consolidated_long = format!("--[no-]{}", positive.long_name);
    parts.push(&consolidated_long);

    // Value placeholder (make optional if present)
    let value_str;
    if let Some(ref val) = positive.value {
        value_str = format!(" [{}]", val);
        parts.push(&value_str);
    }

    // Calculate padding to align descriptions
    // Standard clap help uses about 25-30 chars for option names
    let opt_len = parts.iter().map(|s| s.len()).sum::<usize>();
    let target_len = 27; // Approximate column for descriptions
    let padding = if opt_len < target_len {
        " ".repeat(target_len - opt_len)
    } else {
        "  ".to_string()
    };

    // Description (use positive flag's description as primary)
    let desc = if positive.description.is_empty() {
        "Enable/disable this option".to_string()
    } else {
        positive.description.clone()
    };

    format!("{}{}{}", parts.concat(), padding, desc)
}

#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
