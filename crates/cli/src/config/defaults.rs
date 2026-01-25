// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Centralized default values for configuration.
//!
//! All default values are documented here for easy reference.
//! Individual config structs delegate to these constants via their `default_*` methods.

/// Default file size limits.
pub mod size {
    /// Default max lines for source files (750).
    pub const MAX_LINES: usize = 750;

    /// Default max lines for test files (1100).
    pub const MAX_LINES_TEST: usize = 1100;

    /// Default max tokens (~5k words, suitable for LLM context).
    pub const MAX_TOKENS: usize = 20000;

    /// Default max lines for spec files (1000).
    pub const MAX_LINES_SPEC: usize = 1000;
}

/// Default advice messages.
pub mod advice {
    /// Default advice for source file cloc violations.
    pub const CLOC_SOURCE: &str = "\
Can the code be made more concise?

Look for repetitive patterns that could be extracted into helper functions
or consider refactoring to be more unit testable.

If not, split large source files into sibling modules or submodules in a folder,

Avoid picking and removing individual lines to satisfy the linter,
prefer properly refactoring out testable code blocks.";

    /// Default advice for test file cloc violations.
    pub const CLOC_TEST: &str = "\
Can tests be parameterized or use shared fixtures to be more concise?
Look for repetitive patterns that could be extracted into helper functions.
If not, split large test files into a folder.";
}

/// Default glob patterns for test file detection.
pub mod test_patterns {
    /// Generic test patterns that work across languages.
    pub fn generic() -> Vec<String> {
        vec![
            "**/tests/**".to_string(),
            "**/test/**".to_string(),
            "**/*_test.*".to_string(),
            "**/*_tests.*".to_string(),
            "**/*.test.*".to_string(),
            "**/*.spec.*".to_string(),
            "**/test_*.*".to_string(),
        ]
    }
}
