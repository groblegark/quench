// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Build script that generates env var name constants for `env.rs`.

// Build scripts should panic on failure — there is no meaningful recovery.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("env_names.rs");

    let contents = r#"
/// Environment variable: disables color output (any value).
pub const NO_COLOR: &str = "NO_COLOR";
/// Environment variable: forces color output (any value).
pub const COLOR: &str = "COLOR";
/// Environment variable: indicates Claude Code agent environment.
pub const CLAUDE_CODE: &str = "CLAUDE_CODE";
/// Environment variable: indicates Codex agent environment.
pub const CODEX: &str = "CODEX";
/// Environment variable: indicates Cursor agent environment.
pub const CURSOR: &str = "CURSOR";
/// Environment variable: indicates CI environment.
pub const CI: &str = "CI";
/// Environment variable: enables debug file listing.
pub const QUENCH_DEBUG_FILES: &str = "QUENCH_DEBUG_FILES";
/// Environment variable: enables debug/verbose output.
pub const QUENCH_DEBUG: &str = "QUENCH_DEBUG";
/// Environment variable: configures tracing log filter.
pub const QUENCH_LOG: &str = "QUENCH_LOG";
/// Environment variable: user home directory.
pub const HOME: &str = "HOME";
/// Environment variable: XDG data home directory.
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
/// Environment variable: XDG config home directory.
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
"#;

    fs::write(dest, contents).expect("failed to write env_names.rs");
}
