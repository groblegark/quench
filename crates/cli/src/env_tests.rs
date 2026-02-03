// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn names_no_color_is_correct() {
    assert_eq!(names::NO_COLOR, "NO_COLOR");
}

#[test]
fn names_color_is_correct() {
    assert_eq!(names::COLOR, "COLOR");
}

#[test]
fn names_claude_code_is_correct() {
    assert_eq!(names::CLAUDE_CODE, "CLAUDE_CODE");
}

#[test]
fn names_codex_is_correct() {
    assert_eq!(names::CODEX, "CODEX");
}

#[test]
fn names_cursor_is_correct() {
    assert_eq!(names::CURSOR, "CURSOR");
}

#[test]
fn names_ci_is_correct() {
    assert_eq!(names::CI, "CI");
}

#[test]
fn names_quench_debug_files_is_correct() {
    assert_eq!(names::QUENCH_DEBUG_FILES, "QUENCH_DEBUG_FILES");
}

#[test]
fn names_quench_debug_is_correct() {
    assert_eq!(names::QUENCH_DEBUG, "QUENCH_DEBUG");
}

#[test]
fn names_quench_log_is_correct() {
    assert_eq!(names::QUENCH_LOG, "QUENCH_LOG");
}

#[test]
fn names_home_is_correct() {
    assert_eq!(names::HOME, "HOME");
}

#[test]
fn names_xdg_data_home_is_correct() {
    assert_eq!(names::XDG_DATA_HOME, "XDG_DATA_HOME");
}

#[test]
fn names_xdg_config_home_is_correct() {
    assert_eq!(names::XDG_CONFIG_HOME, "XDG_CONFIG_HOME");
}

#[test]
fn quench_log_var_returns_correct_name() {
    assert_eq!(quench_log_var(), "QUENCH_LOG");
}
