// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Source/test file correlation logic.

mod analysis;
mod checking;
mod diff;

// Re-export core types and analysis functions
pub use analysis::{
    CommitAnalysis, CorrelationConfig, CorrelationResult, TestIndex, analyze_commit,
    analyze_correlation, candidate_js_test_paths, candidate_test_paths, find_test_locations,
    has_correlated_test,
};

// Re-export diff analysis
pub use diff::{DiffRange, changes_in_cfg_test, has_inline_test_changes};

// Re-export checking functions
pub use checking::{check_branch_scope, check_commit_scope, missing_tests_advice};
