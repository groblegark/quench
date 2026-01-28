// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! JavaScript package manager detection.
//!
//! Detects package manager from lock files and provides command generation.
//! Detection order (first match wins):
//! 1. `bun.lock` / `bun.lockb` (Bun)
//! 2. `pnpm-lock.yaml` (pnpm)
//! 3. `yarn.lock` (Yarn)
//! 4. `package-lock.json` (npm)
//! 5. Fallback to npm

use std::path::Path;

/// JavaScript package manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PackageManager {
    #[default]
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

impl PackageManager {
    /// Detect package manager from lock files in project root.
    ///
    /// Detection order (first match wins):
    /// 1. `bun.lock` (Bun 1.2+ text format)
    /// 2. `bun.lockb` (Bun binary format)
    /// 3. `pnpm-lock.yaml`
    /// 4. `yarn.lock`
    /// 5. `package-lock.json`
    /// 6. Fallback to npm
    pub fn detect(root: &Path) -> Self {
        if root.join("bun.lock").exists() || root.join("bun.lockb").exists() {
            return Self::Bun;
        }
        if root.join("pnpm-lock.yaml").exists() {
            return Self::Pnpm;
        }
        if root.join("yarn.lock").exists() {
            return Self::Yarn;
        }
        // package-lock.json or fallback
        Self::Npm
    }

    /// Package manager executable name.
    pub fn executable(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
            PackageManager::Bun => "bun",
        }
    }

    /// Command to run a package.json script (e.g., "build", "lint").
    ///
    /// Returns the command and arguments as a vector.
    /// Note: Yarn uses `yarn <script>` without "run" for conciseness.
    pub fn run_command(&self, script: &str) -> Vec<String> {
        match self {
            PackageManager::Npm => vec!["npm".into(), "run".into(), script.into()],
            PackageManager::Pnpm => vec!["pnpm".into(), "run".into(), script.into()],
            PackageManager::Yarn => vec!["yarn".into(), script.into()],
            PackageManager::Bun => vec!["bun".into(), "run".into(), script.into()],
        }
    }

    /// Command to run tests.
    ///
    /// Uses the native test command for each package manager.
    pub fn test_command(&self) -> Vec<String> {
        match self {
            PackageManager::Npm => vec!["npm".into(), "test".into()],
            PackageManager::Pnpm => vec!["pnpm".into(), "test".into()],
            PackageManager::Yarn => vec!["yarn".into(), "test".into()],
            PackageManager::Bun => vec!["bun".into(), "test".into()],
        }
    }

    /// Command to execute a package binary (like npx, bunx).
    ///
    /// Used for running tools like vitest, jest without going through scripts.
    pub fn exec_command(&self) -> Vec<String> {
        match self {
            PackageManager::Npm => vec!["npx".into()],
            PackageManager::Pnpm => vec!["pnpm".into(), "exec".into()],
            PackageManager::Yarn => vec!["yarn".into()],
            PackageManager::Bun => vec!["bunx".into()],
        }
    }
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.executable())
    }
}

#[cfg(test)]
#[path = "package_manager_tests.rs"]
mod tests;
