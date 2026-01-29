// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared test file naming patterns and utilities.
//!
//! This module provides shared constants for identifying test files
//! and correlating them with source files across languages.
//!
//! # Supported Patterns
//!
//! - Rust: `*_test.rs`, `*_tests.rs`, `test_*.rs`
//! - Go: `*_test.go`
//! - JavaScript/TypeScript: `*.test.ts`, `*.spec.ts`, `__tests__/*.test.ts`
//! - Python: `test_*.py`, `*_test.py`

use std::path::Path;

/// Suffixes that identify test files (Rust/Go style).
pub const TEST_SUFFIXES: &[&str] = &["_tests", "_test", "_spec"];

/// Prefixes that identify test files.
pub const TEST_PREFIXES: &[&str] = &["test_"];

/// Suffixes for JS/TS style test files (part of stem, e.g., "parser.test.ts").
pub const JS_TEST_SUFFIXES: &[&str] = &[".test", ".spec"];

/// All suffix patterns combined for extraction.
pub const ALL_TEST_SUFFIXES: &[&str] = &["_tests", "_test", ".test", ".spec", "_spec"];

/// Check if a test base name correlates with a source base name.
///
/// Matching rules:
/// 1. Direct: "parser" matches "parser"
/// 2. Source with suffix: "parser" matches "parser_test", "parser_tests"
/// 3. Source with prefix: "parser" matches "test_parser"
///
/// Note: This function assumes `test_base` is already stripped of test affixes
/// via `strip_test_affixes()`. If not, direct matching will still work but
/// suffix/prefix matching may produce false negatives.
#[inline]
pub fn matches_base_name(test_base: &str, source_base: &str) -> bool {
    // Direct match (most common case, check first)
    if test_base == source_base {
        return true;
    }

    // Test has suffix matching source (e.g., "parser_test" matches source "parser")
    for suffix in TEST_SUFFIXES {
        if test_base == format!("{}{}", source_base, suffix) {
            return true;
        }
    }

    // Test has prefix matching source (e.g., "test_parser" matches source "parser")
    for prefix in TEST_PREFIXES {
        if test_base == format!("{}{}", prefix, source_base) {
            return true;
        }
    }

    false
}

/// Detected language of a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Go,
    JavaScript,
    Python,
    Unknown,
}

/// Detect the language of a source file from its extension.
#[inline]
pub fn detect_language(path: &Path) -> Language {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Language::Rust,
        Some("go") => Language::Go,
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "mts") => Language::JavaScript,
        Some("py") => Language::Python,
        _ => Language::Unknown,
    }
}

/// Generate candidate test file paths for a given source file.
///
/// Returns a list of potential test file paths based on the detected language.
pub fn candidate_test_paths_for(source_path: &Path) -> Vec<String> {
    let base = match source_path.file_stem().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return vec![],
    };

    match detect_language(source_path) {
        Language::Rust => candidate_rust_test_paths(base),
        Language::Go => candidate_go_test_paths(base),
        Language::JavaScript => candidate_js_test_paths(base),
        Language::Python => candidate_python_test_paths(base),
        Language::Unknown => vec![],
    }
}

/// Get candidate test paths for Rust files.
fn candidate_rust_test_paths(base: &str) -> Vec<String> {
    vec![
        format!("tests/{}_tests.rs", base),
        format!("tests/{}_test.rs", base),
        format!("tests/{}.rs", base),
        format!("test/{}_tests.rs", base),
        format!("test/{}_test.rs", base),
        format!("test/{}.rs", base),
    ]
}

/// Get candidate test paths for Go files.
fn candidate_go_test_paths(base: &str) -> Vec<String> {
    vec![format!("{}_test.go", base)]
}

/// Get candidate test paths for JavaScript/TypeScript files.
fn candidate_js_test_paths(base: &str) -> Vec<String> {
    let exts = ["ts", "js"];
    let mut paths = Vec::with_capacity(16);
    for ext in &exts {
        paths.push(format!("{}.test.{}", base, ext));
        paths.push(format!("{}.spec.{}", base, ext));
        paths.push(format!("__tests__/{}.test.{}", base, ext));
        paths.push(format!("tests/{}.test.{}", base, ext));
    }
    paths
}

/// Get candidate test paths for Python files.
fn candidate_python_test_paths(base: &str) -> Vec<String> {
    vec![
        format!("test_{}.py", base),
        format!("tests/test_{}.py", base),
        format!("{}_test.py", base),
    ]
}
