// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Parse helper functions for configuration.

use std::path::Path;

use super::{
    AgentsConfig, AgentsScopeConfig, CfgTestSplitMode, CheckLevel, ClocConfig, DocsConfig,
    EscapeAction, EscapePattern, EscapesConfig, GoConfig, GoPolicyConfig, GoSuppressConfig,
    LineMetric, LintChangesPolicy, RustConfig, RustPolicyConfig, ShellConfig, ShellPolicyConfig,
    ShellSuppressConfig, SuppressConfig, SuppressLevel, SuppressScopeConfig, TocConfig,
};
use crate::checks::agents::config::{ContentRule, RequiredSection, SectionsConfig};

/// Parse a TOML array of strings into a Vec<String>.
pub(super) fn parse_string_array(value: Option<&toml::Value>) -> Option<Vec<String>> {
    value?.as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
}

/// Parse a TOML array of strings with a default function.
pub(super) fn parse_string_array_or_else<F>(value: Option<&toml::Value>, default: F) -> Vec<String>
where
    F: FnOnce() -> Vec<String>,
{
    parse_string_array(value).unwrap_or_else(default)
}

/// Parse a TOML array of strings, returning empty vec if not found.
pub(super) fn parse_string_array_or_empty(value: Option<&toml::Value>) -> Vec<String> {
    parse_string_array(value).unwrap_or_default()
}

/// Parse a TOML string value with a default function.
fn parse_string_or_else<F>(value: Option<&toml::Value>, default: F) -> String
where
    F: FnOnce() -> String,
{
    value
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(default)
}

/// Parse a TOML string value as Option<String>.
pub(super) fn parse_string_option(value: Option<&toml::Value>) -> Option<String> {
    value.and_then(|v| v.as_str()).map(String::from)
}

/// Parse a TOML integer value as usize with a default function.
fn parse_usize_or_else<F>(value: Option<&toml::Value>, default: F) -> usize
where
    F: FnOnce() -> usize,
{
    value
        .and_then(|v| v.as_integer())
        .map(|v| v as usize)
        .unwrap_or_else(default)
}

/// Parse a TOML integer value as Option<usize>.
pub(super) fn parse_usize_option(value: Option<&toml::Value>) -> Option<usize> {
    value.and_then(|v| v.as_integer()).map(|v| v as usize)
}

/// Parse check level from TOML value.
fn parse_check_level(value: Option<&toml::Value>) -> CheckLevel {
    match value.and_then(|v| v.as_str()) {
        Some("error") => CheckLevel::Error,
        Some("warn") => CheckLevel::Warn,
        Some("off") => CheckLevel::Off,
        _ => CheckLevel::default(),
    }
}

/// Parse suppress level from TOML value with a default.
fn parse_suppress_level(value: Option<&toml::Value>, default: SuppressLevel) -> SuppressLevel {
    match value.and_then(|v| v.as_str()) {
        Some("forbid") => SuppressLevel::Forbid,
        Some("comment") => SuppressLevel::Comment,
        Some("allow") => SuppressLevel::Allow,
        _ => default,
    }
}

/// Parse lint changes policy from TOML value.
fn parse_lint_changes_policy(value: Option<&toml::Value>) -> LintChangesPolicy {
    match value.and_then(|v| v.as_str()) {
        Some("standalone") => LintChangesPolicy::Standalone,
        Some("none") | None => LintChangesPolicy::None,
        _ => LintChangesPolicy::None,
    }
}

/// Parse cfg_test_split from TOML value.
/// Supports both legacy boolean and new string modes.
fn parse_cfg_test_split(value: Option<&toml::Value>) -> CfgTestSplitMode {
    match value {
        // Legacy boolean support
        Some(toml::Value::Boolean(true)) => CfgTestSplitMode::Count,
        Some(toml::Value::Boolean(false)) => CfgTestSplitMode::Off,
        // New string modes
        Some(toml::Value::String(s)) => match s.as_str() {
            "count" => CfgTestSplitMode::Count,
            "require" => CfgTestSplitMode::Require,
            "off" => CfgTestSplitMode::Off,
            _ => CfgTestSplitMode::Count, // Default on unknown
        },
        None => CfgTestSplitMode::Count,
        _ => CfgTestSplitMode::Count,
    }
}

/// Parse Rust-specific configuration from TOML value.
pub(super) fn parse_rust_config(value: Option<&toml::Value>) -> RustConfig {
    let Some(toml::Value::Table(t)) = value else {
        return RustConfig::default();
    };

    RustConfig {
        cfg_test_split: parse_cfg_test_split(t.get("cfg_test_split")),
        suppress: parse_suppress_config(t.get("suppress")),
        policy: parse_policy_config(t.get("policy")),
        cloc_advice: parse_string_option(t.get("cloc_advice")),
    }
}

/// Parse Shell-specific configuration from TOML value.
pub(super) fn parse_shell_config(value: Option<&toml::Value>) -> ShellConfig {
    let Some(toml::Value::Table(t)) = value else {
        return ShellConfig::default();
    };

    ShellConfig {
        source: parse_string_array_or_else(t.get("source"), ShellConfig::default_source),
        tests: parse_string_array_or_else(t.get("tests"), ShellConfig::default_tests),
        suppress: parse_shell_suppress_config(t.get("suppress")),
        policy: parse_shell_policy_config(t.get("policy")),
        cloc_advice: parse_string_option(t.get("cloc_advice")),
    }
}

/// Parse shell suppress configuration.
fn parse_shell_suppress_config(value: Option<&toml::Value>) -> ShellSuppressConfig {
    let Some(toml::Value::Table(t)) = value else {
        return ShellSuppressConfig::default();
    };

    ShellSuppressConfig {
        check: parse_suppress_level(t.get("check"), ShellSuppressConfig::default_check()),
        comment: parse_string_option(t.get("comment")),
        // Shell uses empty defaults (forbid level doesn't need patterns)
        source: parse_suppress_scope_with_defaults(
            t.get("source"),
            ShellSuppressConfig::default_source(),
        ),
        test: t
            .get("test")
            .map(|v| {
                parse_suppress_scope_with_defaults(Some(v), ShellSuppressConfig::default_test())
            })
            .unwrap_or_else(ShellSuppressConfig::default_test),
    }
}

/// Parse shell policy configuration.
fn parse_shell_policy_config(value: Option<&toml::Value>) -> ShellPolicyConfig {
    let Some(toml::Value::Table(t)) = value else {
        return ShellPolicyConfig::default();
    };

    ShellPolicyConfig {
        lint_changes: parse_lint_changes_policy(t.get("lint_changes")),
        lint_config: parse_string_array_or_else(
            t.get("lint_config"),
            ShellPolicyConfig::default_lint_config,
        ),
    }
}

/// Parse Go-specific configuration from TOML value.
pub(super) fn parse_go_config(value: Option<&toml::Value>) -> GoConfig {
    let Some(toml::Value::Table(t)) = value else {
        return GoConfig::default();
    };

    GoConfig {
        source: parse_string_array_or_else(t.get("source"), GoConfig::default_source),
        tests: parse_string_array_or_else(t.get("tests"), GoConfig::default_tests),
        suppress: parse_go_suppress_config(t.get("suppress")),
        policy: parse_go_policy_config(t.get("policy")),
        cloc_advice: parse_string_option(t.get("cloc_advice")),
    }
}

/// Parse Go suppress configuration.
fn parse_go_suppress_config(value: Option<&toml::Value>) -> GoSuppressConfig {
    let Some(toml::Value::Table(t)) = value else {
        return GoSuppressConfig::default();
    };

    GoSuppressConfig {
        check: parse_suppress_level(t.get("check"), GoSuppressConfig::default_check()),
        comment: parse_string_option(t.get("comment")),
        // Go uses empty defaults (no per-lint patterns yet)
        source: parse_suppress_scope_with_defaults(
            t.get("source"),
            GoSuppressConfig::default_source(),
        ),
        test: t
            .get("test")
            .map(|v| parse_suppress_scope_with_defaults(Some(v), GoSuppressConfig::default_test()))
            .unwrap_or_else(GoSuppressConfig::default_test),
    }
}

/// Parse Go policy configuration.
fn parse_go_policy_config(value: Option<&toml::Value>) -> GoPolicyConfig {
    let Some(toml::Value::Table(t)) = value else {
        return GoPolicyConfig::default();
    };

    GoPolicyConfig {
        lint_changes: parse_lint_changes_policy(t.get("lint_changes")),
        lint_config: parse_string_array_or_else(
            t.get("lint_config"),
            GoPolicyConfig::default_lint_config,
        ),
    }
}

/// Parse lint policy configuration from TOML value.
fn parse_policy_config(value: Option<&toml::Value>) -> RustPolicyConfig {
    let Some(toml::Value::Table(t)) = value else {
        return RustPolicyConfig::default();
    };

    RustPolicyConfig {
        lint_changes: parse_lint_changes_policy(t.get("lint_changes")),
        lint_config: parse_string_array_or_else(
            t.get("lint_config"),
            RustPolicyConfig::default_lint_config,
        ),
    }
}

/// Parse suppress configuration from TOML value.
fn parse_suppress_config(value: Option<&toml::Value>) -> SuppressConfig {
    let Some(toml::Value::Table(t)) = value else {
        return SuppressConfig::default();
    };

    SuppressConfig {
        check: parse_suppress_level(t.get("check"), SuppressConfig::default_check()),
        comment: parse_string_option(t.get("comment")),
        source: parse_suppress_scope(t.get("source"), false),
        test: parse_suppress_scope(t.get("test"), true),
    }
}

/// Parse scope-specific suppress configuration with language-specific defaults.
fn parse_suppress_scope(value: Option<&toml::Value>, is_test: bool) -> SuppressScopeConfig {
    let defaults = if is_test {
        SuppressScopeConfig::default_for_test()
    } else {
        SuppressScopeConfig::default_for_source()
    };
    parse_suppress_scope_with_defaults(value, defaults)
}

/// Parse scope-specific suppress configuration with explicit defaults.
fn parse_suppress_scope_with_defaults(
    value: Option<&toml::Value>,
    defaults: SuppressScopeConfig,
) -> SuppressScopeConfig {
    let Some(toml::Value::Table(t)) = value else {
        return defaults;
    };

    let is_test = defaults.check == Some(SuppressLevel::Allow);

    let check = match t.get("check").and_then(|v| v.as_str()) {
        Some("forbid") => Some(SuppressLevel::Forbid),
        Some("comment") => Some(SuppressLevel::Comment),
        Some("allow") => Some(SuppressLevel::Allow),
        _ if is_test => Some(SuppressLevel::Allow),
        _ => None,
    };

    let allow = parse_string_array_or_empty(t.get("allow"));
    let forbid = parse_string_array_or_empty(t.get("forbid"));

    // Parse per-lint-code comment patterns.
    // Supports both:
    //   - Table form: [rust.suppress.source.dead_code] comment = "..."
    //   - Inline form: dead_code = ["// KEEP:", "// NOTE:"] or dead_code = "// KEEP:"
    let mut patterns = defaults.patterns;

    for (key, val) in t.iter() {
        // Skip known fields
        if matches!(key.as_str(), "check" | "allow" | "forbid") {
            continue;
        }

        match val {
            // Table form: [rust.suppress.source.dead_code] comment = "..."
            toml::Value::Table(lint_table) => {
                if let Some(comment_val) = lint_table.get("comment") {
                    let comment_patterns = parse_pattern_value(comment_val);
                    if !comment_patterns.is_empty() {
                        patterns.insert(key.clone(), comment_patterns);
                    }
                }
            }
            // Inline form: dead_code = "..." or dead_code = ["...", "..."]
            _ => {
                let comment_patterns = parse_pattern_value(val);
                if !comment_patterns.is_empty() {
                    patterns.insert(key.clone(), comment_patterns);
                }
            }
        }
    }

    SuppressScopeConfig {
        check,
        allow,
        forbid,
        patterns,
    }
}

/// Parse a pattern value that can be either a string or array of strings.
fn parse_pattern_value(value: &toml::Value) -> Vec<String> {
    match value {
        toml::Value::String(s) => vec![s.clone()],
        toml::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse cloc configuration from TOML value.
pub(super) fn parse_cloc_config(value: Option<&toml::Value>) -> ClocConfig {
    let Some(toml::Value::Table(t)) = value else {
        return ClocConfig::default();
    };

    ClocConfig {
        max_lines: parse_usize_or_else(t.get("max_lines"), ClocConfig::default_max_lines),
        max_lines_test: parse_usize_or_else(
            t.get("max_lines_test"),
            ClocConfig::default_max_lines_test,
        ),
        metric: match t.get("metric").and_then(|v| v.as_str()) {
            Some("nonblank") => LineMetric::Nonblank,
            _ => LineMetric::Lines,
        },
        check: parse_check_level(t.get("check")),
        test_patterns: parse_string_array_or_else(
            t.get("test_patterns"),
            ClocConfig::default_test_patterns,
        ),
        exclude: parse_string_array_or_empty(t.get("exclude")),
        max_tokens: t
            .get("max_tokens")
            .map(|v| {
                if v.as_bool() == Some(false) {
                    None // max_tokens = false disables the check
                } else {
                    v.as_integer().map(|n| n as usize)
                }
            })
            .unwrap_or_else(ClocConfig::default_max_tokens),
        advice: parse_string_or_else(t.get("advice"), ClocConfig::default_advice),
        advice_test: parse_string_or_else(t.get("advice_test"), ClocConfig::default_advice_test),
    }
}

/// Parse escapes configuration from TOML value.
pub(super) fn parse_escapes_config(value: Option<&toml::Value>) -> EscapesConfig {
    let Some(toml::Value::Table(t)) = value else {
        return EscapesConfig::default();
    };

    EscapesConfig {
        check: parse_check_level(t.get("check")),
        patterns: t
            .get("patterns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_escape_pattern).collect())
            .unwrap_or_default(),
    }
}

/// Parse a single escape pattern from TOML value.
fn parse_escape_pattern(value: &toml::Value) -> Option<EscapePattern> {
    let t = value.as_table()?;

    let name = t.get("name")?.as_str()?.to_string();
    let pattern = t.get("pattern")?.as_str()?.to_string();

    let action = match t.get("action").and_then(|v| v.as_str()) {
        Some("forbid") => EscapeAction::Forbid,
        Some("comment") => EscapeAction::Comment,
        Some("count") => EscapeAction::Count,
        _ => EscapeAction::default(),
    };

    let comment = parse_string_option(t.get("comment"));

    let threshold = t
        .get("threshold")
        .and_then(|v| v.as_integer())
        .map(|v| v as usize)
        .unwrap_or(0);

    let advice = parse_string_option(t.get("advice"));

    Some(EscapePattern {
        name,
        pattern,
        action,
        comment,
        threshold,
        advice,
    })
}

/// Warn about unknown configuration key.
pub(super) fn warn_unknown_key(path: &Path, key: &str) {
    eprintln!(
        "quench: warning: {}: unrecognized field `{}` (ignored)",
        path.display(),
        key
    );
}

/// Parse agents configuration from TOML value.
pub(super) fn parse_agents_config(value: Option<&toml::Value>) -> AgentsConfig {
    let Some(toml::Value::Table(t)) = value else {
        return AgentsConfig::default();
    };

    AgentsConfig {
        check: parse_check_level(t.get("check")),
        files: parse_string_array_or_else(t.get("files"), AgentsConfig::default_files),
        required: parse_string_array_or_empty(t.get("required")),
        optional: parse_string_array_or_empty(t.get("optional")),
        forbid: parse_string_array_or_empty(t.get("forbid")),
        sync: t.get("sync").and_then(|v| v.as_bool()).unwrap_or(false),
        sync_source: parse_string_option(t.get("sync_source")),
        sections: parse_sections_config(t.get("sections")),
        tables: parse_content_rule(t.get("tables")).unwrap_or_default(),
        box_diagrams: parse_content_rule(t.get("box_diagrams")).unwrap_or_else(ContentRule::allow),
        mermaid: parse_content_rule(t.get("mermaid")).unwrap_or_else(ContentRule::allow),
        max_lines: parse_usize_option(t.get("max_lines")),
        max_tokens: parse_usize_option(t.get("max_tokens")),
        root: t.get("root").map(parse_agents_scope_config),
        package: t.get("package").map(parse_agents_scope_config),
        module: t.get("module").map(parse_agents_scope_config),
    }
}

/// Parse a content rule from TOML value.
fn parse_content_rule(value: Option<&toml::Value>) -> Option<ContentRule> {
    let s = value?.as_str()?;
    match s {
        "allow" => Some(ContentRule::Allow),
        "forbid" => Some(ContentRule::Forbid),
        _ => None,
    }
}

/// Parse a scope-specific agents configuration.
fn parse_agents_scope_config(value: &toml::Value) -> AgentsScopeConfig {
    let Some(t) = value.as_table() else {
        return AgentsScopeConfig::default();
    };

    AgentsScopeConfig {
        required: parse_string_array_or_empty(t.get("required")),
        optional: parse_string_array_or_empty(t.get("optional")),
        forbid: parse_string_array_or_empty(t.get("forbid")),
        max_lines: parse_usize_option(t.get("max_lines")),
        max_tokens: parse_usize_option(t.get("max_tokens")),
    }
}

/// Parse sections configuration from TOML value.
fn parse_sections_config(value: Option<&toml::Value>) -> SectionsConfig {
    let Some(toml::Value::Table(t)) = value else {
        return SectionsConfig::default();
    };

    SectionsConfig {
        required: t
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_required_section).collect())
            .unwrap_or_default(),
        forbid: parse_string_array_or_empty(t.get("forbid")),
    }
}

/// Parse a single required section from TOML value.
fn parse_required_section(value: &toml::Value) -> Option<RequiredSection> {
    match value {
        // Simple form: just a string name
        toml::Value::String(name) => Some(RequiredSection {
            name: name.clone(),
            advice: None,
        }),
        // Extended form: table with name and advice
        toml::Value::Table(t) => {
            let name = t.get("name")?.as_str()?.to_string();
            let advice = parse_string_option(t.get("advice"));
            Some(RequiredSection { name, advice })
        }
        _ => None,
    }
}

/// Parse docs configuration from TOML value.
pub(super) fn parse_docs_config(value: Option<&toml::Value>) -> DocsConfig {
    let Some(toml::Value::Table(t)) = value else {
        return DocsConfig::default();
    };

    DocsConfig {
        check: parse_string_option(t.get("check")),
        toc: parse_toc_config(t.get("toc")),
    }
}

/// Parse TOC configuration from TOML value.
fn parse_toc_config(value: Option<&toml::Value>) -> TocConfig {
    let Some(toml::Value::Table(t)) = value else {
        return TocConfig::default();
    };

    TocConfig {
        check: parse_string_option(t.get("check")),
        include: parse_string_array_or_else(t.get("include"), TocConfig::default_include),
        exclude: parse_string_array_or_else(t.get("exclude"), TocConfig::default_exclude),
    }
}
