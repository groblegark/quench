//! Shell language adapter.
//!
//! Provides Shell-specific behavior for checks:
//! - File classification (source vs test)
//! - Default patterns for shell scripts
//!
//! See docs/specs/langs/shell.md for specification.

use std::path::Path;

use globset::GlobSet;

use super::glob::build_glob_set;
use super::{Adapter, FileKind};

/// Shell language adapter.
pub struct ShellAdapter {
    source_patterns: GlobSet,
    test_patterns: GlobSet,
}

impl ShellAdapter {
    /// Create a new Shell adapter with default patterns.
    pub fn new() -> Self {
        Self {
            source_patterns: build_glob_set(&["**/*.sh".to_string(), "**/*.bash".to_string()]),
            test_patterns: build_glob_set(&[
                "tests/**/*.bats".to_string(),
                "test/**/*.bats".to_string(),
                "*_test.sh".to_string(),
                "**/*_test.sh".to_string(),
            ]),
        }
    }
}

impl Default for ShellAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for ShellAdapter {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["sh", "bash", "bats"]
    }

    fn classify(&self, path: &Path) -> FileKind {
        // Test patterns take precedence
        if self.test_patterns.is_match(path) {
            return FileKind::Test;
        }

        // Source patterns
        if self.source_patterns.is_match(path) {
            return FileKind::Source;
        }

        FileKind::Other
    }
}

#[cfg(test)]
#[path = "../shell_tests.rs"]
mod tests;
