#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// Unit tests for escapes check internals
// Behavioral tests are in tests/specs/checks/escapes.rs

use super::*;

mod comment_detection {
    use super::*;

    #[test]
    fn finds_comment_on_same_line() {
        let content = "unsafe { code } // SAFETY: reason";
        assert!(has_justification_comment(content, 1, "// SAFETY:"));
    }

    #[test]
    fn finds_comment_on_preceding_line() {
        let content = "// SAFETY: reason\nunsafe { code }";
        assert!(has_justification_comment(content, 2, "// SAFETY:"));
    }

    #[test]
    fn finds_comment_through_blank_lines() {
        let content = "// SAFETY: reason\n\nunsafe { code }";
        assert!(has_justification_comment(content, 3, "// SAFETY:"));
    }

    #[test]
    fn finds_comment_through_other_comments() {
        let content = "// SAFETY: reason\n// more context\nunsafe { code }";
        assert!(has_justification_comment(content, 3, "// SAFETY:"));
    }

    #[test]
    fn stops_at_code_line() {
        let content = "// SAFETY: old\nfn other() {}\nunsafe { code }";
        assert!(!has_justification_comment(content, 3, "// SAFETY:"));
    }

    #[test]
    fn no_comment_returns_false() {
        let content = "unsafe { code }";
        assert!(!has_justification_comment(content, 1, "// SAFETY:"));
    }
}

mod is_comment_line_tests {
    use super::*;

    #[test]
    fn c_style_single() {
        assert!(is_comment_line("// comment"));
        assert!(is_comment_line("  // indented"));
    }

    #[test]
    fn c_style_block() {
        assert!(is_comment_line("/* block */"));
        assert!(is_comment_line(" * continuation"));
    }

    #[test]
    fn shell_style() {
        assert!(is_comment_line("# comment"));
        assert!(is_comment_line("  # indented"));
    }

    #[test]
    fn code_is_not_comment() {
        assert!(!is_comment_line("fn main() {}"));
        assert!(!is_comment_line("let x = 1;"));
    }
}
