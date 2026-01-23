// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Configuration for the agents check.

use serde::Deserialize;

use crate::config::CheckLevel;

/// Configuration for the agents check.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentsConfig {
    /// Check level: error, warn, or off.
    #[serde(default)]
    pub check: CheckLevel,

    /// Agent files to check (default: all recognized).
    #[serde(default = "AgentsConfig::default_files")]
    pub files: Vec<String>,

    /// Files that must exist (root scope).
    #[serde(default)]
    pub required: Vec<String>,

    /// Files checked if present (root scope).
    #[serde(default)]
    pub optional: Vec<String>,

    /// Files that must not exist (root scope).
    #[serde(default)]
    pub forbid: Vec<String>,

    /// Enable file synchronization checking.
    #[serde(default)]
    pub sync: bool,

    /// Source file for synchronization (other files should match this).
    #[serde(default)]
    pub sync_source: Option<String>,

    /// Root scope settings (overrides flat config).
    #[serde(default)]
    pub root: Option<AgentsScopeConfig>,

    /// Package scope settings.
    #[serde(default)]
    pub package: Option<AgentsScopeConfig>,

    /// Module scope settings.
    #[serde(default)]
    pub module: Option<AgentsScopeConfig>,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            check: CheckLevel::default(),
            files: Self::default_files(),
            required: Vec::new(),
            optional: Vec::new(),
            forbid: Vec::new(),
            sync: false,
            sync_source: None,
            root: None,
            package: None,
            module: None,
        }
    }
}

impl AgentsConfig {
    /// Default agent files to detect.
    pub fn default_files() -> Vec<String> {
        vec![
            "CLAUDE.md".to_string(),
            "AGENTS.md".to_string(),
            ".cursorrules".to_string(),
            ".cursorignore".to_string(),
            ".cursor/rules/*.md".to_string(),
            ".cursor/rules/*.mdc".to_string(),
        ]
    }
}

/// Per-scope configuration for agent files.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AgentsScopeConfig {
    /// Files that must exist at this scope.
    #[serde(default)]
    pub required: Vec<String>,

    /// Files checked if present at this scope.
    #[serde(default)]
    pub optional: Vec<String>,

    /// Files that must not exist at this scope.
    #[serde(default)]
    pub forbid: Vec<String>,

    /// Maximum lines per file at this scope.
    #[serde(default)]
    pub max_lines: Option<usize>,

    /// Maximum tokens per file at this scope.
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
