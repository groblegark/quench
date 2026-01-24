// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Git utilities for change detection.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Get list of changed files compared to a git base ref.
pub fn get_changed_files(root: &Path, base: &str) -> anyhow::Result<Vec<PathBuf>> {
    // Get staged/unstaged changes (diffstat against base)
    let output = Command::new("git")
        .args(["diff", "--name-only", base])
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", stderr.trim());
    }

    // Also get staged changes
    let staged_output = Command::new("git")
        .args(["diff", "--name-only", "--cached", base])
        .current_dir(root)
        .output()?;

    let mut files: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.is_empty() {
            files.insert(root.join(line));
        }
    }

    if staged_output.status.success() {
        for line in String::from_utf8_lossy(&staged_output.stdout).lines() {
            if !line.is_empty() {
                files.insert(root.join(line));
            }
        }
    }

    Ok(files.into_iter().collect())
}

/// Get list of staged files (for --staged flag).
pub fn get_staged_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    // Get staged changes
    let output = Command::new("git")
        .args(["diff", "--name-only", "--cached"])
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff --cached failed: {}", stderr.trim());
    }

    let mut files: Vec<PathBuf> = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.is_empty() {
            files.push(root.join(line));
        }
    }

    Ok(files)
}

/// Detect base branch for CI mode (main or master).
pub fn detect_base_branch(root: &Path) -> Option<String> {
    // Check if main branch exists
    let main_check = Command::new("git")
        .args(["rev-parse", "--verify", "main"])
        .current_dir(root)
        .output();

    if let Ok(output) = main_check
        && output.status.success()
    {
        return Some("main".to_string());
    }

    // Fall back to master
    let master_check = Command::new("git")
        .args(["rev-parse", "--verify", "master"])
        .current_dir(root)
        .output();

    if let Ok(output) = master_check
        && output.status.success()
    {
        return Some("master".to_string());
    }

    None
}
