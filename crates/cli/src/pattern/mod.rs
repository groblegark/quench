// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Pattern matching for escape hatch detection.
//!
//! Implements the pattern matching hierarchy from docs/specs/20-performance.md:
//! - Single literal: memchr::memmem
//! - Multiple literals: aho-corasick
//! - Complex regex: regex crate

pub mod matcher;

pub(crate) use matcher::{CompiledPattern, PatternError};
#[cfg(test)]
pub(crate) use matcher::{LineMatch, byte_offset_to_line};

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
