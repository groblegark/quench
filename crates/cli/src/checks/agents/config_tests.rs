// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn default_files_includes_claude_md() {
    let config = AgentsConfig::default();
    assert!(config.files.contains(&"CLAUDE.md".to_string()));
}

#[test]
fn default_files_includes_cursorrules() {
    let config = AgentsConfig::default();
    assert!(config.files.contains(&".cursorrules".to_string()));
}

#[test]
fn default_files_includes_cursor_rules_glob() {
    let config = AgentsConfig::default();
    assert!(config.files.contains(&".cursor/rules/*.md".to_string()));
}

#[test]
fn default_config_has_no_required_files() {
    let config = AgentsConfig::default();
    assert!(config.required.is_empty());
}

#[test]
fn default_sync_is_disabled() {
    let config = AgentsConfig::default();
    assert!(!config.sync);
    assert!(config.sync_source.is_none());
}

#[test]
fn default_tables_is_forbid() {
    let config = AgentsConfig::default();
    assert_eq!(config.tables, ContentRule::Forbid);
}

#[test]
fn default_box_diagrams_is_allow() {
    let config = AgentsConfig::default();
    assert_eq!(config.box_diagrams, ContentRule::Allow);
}

#[test]
fn default_mermaid_is_allow() {
    let config = AgentsConfig::default();
    assert_eq!(config.mermaid, ContentRule::Allow);
}

#[test]
fn default_max_lines_is_none() {
    let config = AgentsConfig::default();
    assert!(config.max_lines.is_none());
}

#[test]
fn default_max_tokens_is_none() {
    let config = AgentsConfig::default();
    assert!(config.max_tokens.is_none());
}

#[test]
fn content_rule_deserialize_allow() {
    let json = r#""allow""#;
    let rule: ContentRule = serde_json::from_str(json).unwrap();
    assert_eq!(rule, ContentRule::Allow);
}

#[test]
fn content_rule_deserialize_forbid() {
    let json = r#""forbid""#;
    let rule: ContentRule = serde_json::from_str(json).unwrap();
    assert_eq!(rule, ContentRule::Forbid);
}

#[test]
fn content_rule_deserialize_invalid() {
    let json = r#""invalid""#;
    let result: Result<ContentRule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
