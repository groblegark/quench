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
