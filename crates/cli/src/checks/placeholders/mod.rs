// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Placeholder metrics collection for the tests check.
//!
//! Detects placeholder test patterns and counts them as metrics:
//! - Rust: `#[ignore]` attribute on tests, `todo!()` macro in test bodies
//! - JavaScript/TypeScript: `test.todo()`, `it.todo()`, `test.fixme()`, `it.fixme()`, `test.skip()`
//!
//! These metrics represent test debt and are collected by the tests check.

pub mod javascript;
pub mod rust;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::file_reader::FileContent;

/// Placeholder metrics collected from test files.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlaceholderMetrics {
    pub rust: RustMetrics,
    pub javascript: JsMetrics,
}

/// Rust placeholder metrics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RustMetrics {
    pub ignore: usize,
    pub todo: usize,
}

/// JavaScript placeholder metrics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsMetrics {
    pub todo: usize,
    pub fixme: usize,
    pub skip: usize,
}

impl PlaceholderMetrics {
    /// Convert metrics to JSON format for output.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "rust": {
                "ignore": self.rust.ignore,
                "todo": self.rust.todo,
            },
            "javascript": {
                "todo": self.javascript.todo,
                "fixme": self.javascript.fixme,
                "skip": self.javascript.skip,
            }
        })
    }

    /// Check if any placeholders were detected.
    pub fn has_placeholders(&self) -> bool {
        self.rust.ignore > 0
            || self.rust.todo > 0
            || self.javascript.todo > 0
            || self.javascript.fixme > 0
            || self.javascript.skip > 0
    }
}

/// Default patterns for Rust placeholder detection.
pub fn default_rust_patterns() -> Vec<String> {
    vec!["ignore".to_string(), "todo".to_string()]
}

/// Default patterns for JavaScript placeholder detection.
pub fn default_js_patterns() -> Vec<String> {
    vec!["todo".to_string(), "fixme".to_string(), "skip".to_string()]
}

/// Collect placeholder metrics from a list of test files.
///
/// Scans each file and counts placeholder patterns by type.
pub fn collect_placeholder_metrics<P: AsRef<Path>>(
    test_files: &[P],
    rust_patterns: &[String],
    js_patterns: &[String],
) -> PlaceholderMetrics {
    let mut metrics = PlaceholderMetrics::default();

    for file in test_files {
        let path = file.as_ref();

        // Read file content
        let file_content = match FileContent::read(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let Some(content) = file_content.as_str() else {
            continue; // Skip non-UTF-8 files
        };

        // Detect based on file extension
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "rs" => {
                let placeholders = rust::find_rust_placeholders(content, rust_patterns);
                for p in placeholders {
                    match p.kind {
                        rust::RustPlaceholderKind::Ignore => metrics.rust.ignore += 1,
                        rust::RustPlaceholderKind::Todo => metrics.rust.todo += 1,
                    }
                }
            }
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "mts" => {
                let placeholders = javascript::find_js_placeholders(content, js_patterns);
                for p in placeholders {
                    match p.kind {
                        javascript::JsPlaceholderKind::Todo => metrics.javascript.todo += 1,
                        javascript::JsPlaceholderKind::Fixme => metrics.javascript.fixme += 1,
                        javascript::JsPlaceholderKind::Skip => metrics.javascript.skip += 1,
                    }
                }
            }
            _ => {}
        }
    }

    metrics
}
