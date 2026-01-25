// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Timing data structures for --timing flag.

use std::collections::HashMap;

use serde::Serialize;

/// Phase timing breakdown.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PhaseTiming {
    /// File discovery time.
    pub discovery_ms: u64,
    /// Check execution time.
    pub checking_ms: u64,
    /// Output formatting time.
    pub output_ms: u64,
    /// Total elapsed time.
    pub total_ms: u64,
}

/// Complete timing information.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TimingInfo {
    /// Phase breakdown.
    #[serde(flatten)]
    pub phases: PhaseTiming,
    /// Number of files scanned.
    pub files: usize,
    /// Cache hits.
    pub cache_hits: usize,
    /// Per-check timing (check name -> milliseconds).
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub checks: HashMap<String, u64>,
}

impl PhaseTiming {
    /// Format as text output lines for stderr.
    pub fn format_text(&self) -> String {
        format!(
            "discovery: {}ms\nchecking: {}ms\noutput: {}ms\ntotal: {}ms",
            self.discovery_ms, self.checking_ms, self.output_ms, self.total_ms
        )
    }
}

impl TimingInfo {
    /// Format cache statistics line.
    pub fn format_cache(&self, misses: usize) -> String {
        let total = self.cache_hits + misses;
        if total == 0 {
            "cache: 0/0".to_string()
        } else {
            format!("cache: {}/{}", self.cache_hits, total)
        }
    }
}

#[cfg(test)]
#[path = "timing_tests.rs"]
mod tests;
