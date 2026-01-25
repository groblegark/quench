#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

// Mock language config for testing
struct MockLang;

impl LanguageDefaults for MockLang {
    fn default_source() -> Vec<String> {
        vec!["src/**/*.mock".to_string()]
    }

    fn default_tests() -> Vec<String> {
        vec!["tests/**/*.mock".to_string()]
    }

    fn default_ignore() -> Vec<String> {
        vec!["vendor/**".to_string()]
    }
}

// Language with no default ignore
struct NoIgnoreLang;

impl LanguageDefaults for NoIgnoreLang {
    fn default_source() -> Vec<String> {
        vec!["**/*.noi".to_string()]
    }

    fn default_tests() -> Vec<String> {
        vec!["**/*_test.noi".to_string()]
    }
}

#[test]
fn resolve_uses_lang_config_first() {
    let patterns = resolve_patterns::<MockLang>(
        &["custom/**/*.rs".to_string()],
        &["my_tests/**".to_string()],
        &["build/**".to_string()],
        &[],
    );

    assert_eq!(patterns.source, vec!["custom/**/*.rs"]);
    assert_eq!(patterns.test, vec!["my_tests/**"]);
    assert_eq!(patterns.ignore, vec!["build/**"]);
}

#[test]
fn resolve_falls_back_to_project_test_patterns() {
    let patterns = resolve_patterns::<MockLang>(
        &[],
        &[], // No lang tests
        &[],
        &["fallback_tests/**".to_string()], // Project fallback
    );

    // Test uses fallback, source/ignore use defaults
    assert_eq!(patterns.source, vec!["src/**/*.mock"]);
    assert_eq!(patterns.test, vec!["fallback_tests/**"]);
    assert_eq!(patterns.ignore, vec!["vendor/**"]);
}

#[test]
fn resolve_falls_back_to_defaults() {
    let patterns = resolve_patterns::<MockLang>(&[], &[], &[], &[]);

    assert_eq!(patterns.source, MockLang::default_source());
    assert_eq!(patterns.test, MockLang::default_tests());
    assert_eq!(patterns.ignore, MockLang::default_ignore());
}

#[test]
fn resolve_uses_empty_ignore_when_no_default() {
    let patterns = resolve_patterns::<NoIgnoreLang>(&[], &[], &[], &[]);

    assert_eq!(patterns.source, vec!["**/*.noi"]);
    assert_eq!(patterns.test, vec!["**/*_test.noi"]);
    assert!(patterns.ignore.is_empty());
}

#[test]
fn resolve_lang_config_takes_precedence_over_fallback() {
    let patterns = resolve_patterns::<MockLang>(
        &["lang_src/**".to_string()],
        &["lang_tests/**".to_string()],
        &["lang_ignore/**".to_string()],
        &["fallback/**".to_string()], // Should be ignored
    );

    assert_eq!(patterns.source, vec!["lang_src/**"]);
    assert_eq!(patterns.test, vec!["lang_tests/**"]);
    assert_eq!(patterns.ignore, vec!["lang_ignore/**"]);
}
