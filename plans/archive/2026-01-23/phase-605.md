# Phase 605: Docs Check - TOC Validation

**Plan:** `phase-605`
**Root Feature:** `quench-docs`
**Blocked By:** Phase 601 (Docs Check - Specs)

## Overview

Implement TOC (Table of Contents) validation for the `docs` check. This validates that directory tree structures in markdown files reference files that actually exist on disk. The implementation parses fenced code blocks containing directory trees (both box-drawing and indentation formats), extracts file paths, resolves them against the filesystem, and generates `broken_toc` violations for missing files.

Reference: `docs/specs/checks/docs.md#fast-mode-toc-validation`

## Project Structure

Files to create/modify:

```
crates/cli/src/
├── checks/
│   ├── mod.rs              # MODIFY: Replace docs stub with real check
│   └── docs/
│       ├── mod.rs          # NEW: Module root, DocsCheck struct
│       ├── toc.rs          # NEW: TOC parsing and validation
│       └── toc_tests.rs    # NEW: Unit tests for tree parsing
└── config/
    └── mod.rs              # MODIFY: Add DocsConfig struct
```

## Dependencies

- No new external crates required
- Uses existing `globset` for pattern matching (already in deps)
- Uses existing `Check` trait infrastructure
- Uses existing `WalkedFile` from file walker

## Implementation Phases

### Phase 1: Add DocsConfig to Configuration

Add configuration support for the docs check TOC validation.

**File to modify:** `crates/cli/src/config/mod.rs`

Add the DocsConfig struct after existing check configs:

```rust
/// Configuration for docs check.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DocsConfig {
    /// Check level: "error" | "warn" | "off"
    pub check: Option<String>,

    /// TOC validation settings.
    pub toc: TocConfig,
}

/// Configuration for TOC validation.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TocConfig {
    /// Check level: "error" | "warn" | "off"
    pub check: Option<String>,

    /// Include patterns for markdown files.
    #[serde(default = "TocConfig::default_include")]
    pub include: Vec<String>,

    /// Exclude patterns (plans, etc.).
    #[serde(default = "TocConfig::default_exclude")]
    pub exclude: Vec<String>,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            check: None,
            include: Self::default_include(),
            exclude: Self::default_exclude(),
        }
    }
}

impl TocConfig {
    fn default_include() -> Vec<String> {
        vec!["**/*.md".to_string(), "**/*.mdc".to_string()]
    }

    fn default_exclude() -> Vec<String> {
        vec![
            "plans/**".to_string(),
            "plan.md".to_string(),
            "*_plan.md".to_string(),
            "plan_*".to_string(),
        ]
    }
}
```

Add `docs: DocsConfig` to the `CheckConfig` struct.

**Verification:**
- [ ] `cargo build` compiles with new config types
- [ ] Config parsing test with `[check.docs.toc]` section

### Phase 2: Create Docs Check Module Structure

Create the docs check module with the `DocsCheck` struct implementing the `Check` trait.

**File to create:** `crates/cli/src/checks/docs/mod.rs`

```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Documentation validation check.
//!
//! Validates:
//! - TOC entries reference existing files
//! - (Future) Markdown links point to existing files
//! - (Future) Specs have required sections

mod toc;

use crate::check::{Check, CheckContext, CheckResult, Violation};

pub struct DocsCheck;

impl Check for DocsCheck {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn description(&self) -> &'static str {
        "Documentation validation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let mut violations = Vec::new();

        // Check if docs check is disabled
        let check_level = ctx.config.check.docs.check.as_deref().unwrap_or("error");
        if check_level == "off" {
            return CheckResult::passed("docs");
        }

        // Run TOC validation
        toc::validate_toc(ctx, &mut violations);

        // Respect violation limit
        if let Some(limit) = ctx.limit {
            violations.truncate(limit);
        }

        if violations.is_empty() {
            CheckResult::passed("docs")
        } else {
            CheckResult::failed("docs", violations)
        }
    }

    fn default_enabled(&self) -> bool {
        true
    }
}
```

**File to modify:** `crates/cli/src/checks/mod.rs`

Replace the stub with the real implementation:

```rust
pub mod docs;

// In all_checks():
Arc::new(docs::DocsCheck),
```

**Verification:**
- [ ] `cargo build` compiles with new module
- [ ] `cargo run -- --docs` runs without panic

### Phase 3: Implement Fenced Code Block Detection

Parse markdown files to extract fenced code blocks. A fenced code block starts with ``` (optionally with a language tag) and ends with ```.

**File to create:** `crates/cli/src/checks/docs/toc.rs`

```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! TOC (directory tree) validation.

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::check::{CheckContext, Violation};

/// A fenced code block extracted from markdown.
#[derive(Debug)]
struct FencedBlock {
    /// Line number where the block starts (1-indexed, after the opening ```).
    start_line: u32,
    /// Content lines within the block.
    lines: Vec<String>,
}

/// Extract all fenced code blocks from markdown content.
fn extract_fenced_blocks(content: &str) -> Vec<FencedBlock> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_lines = Vec::new();
    let mut start_line = 0u32;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx as u32 + 1;
        let trimmed = line.trim();

        if !in_block && trimmed.starts_with("```") {
            // Start of fenced block
            in_block = true;
            start_line = line_num + 1; // Content starts on next line
            current_lines.clear();
        } else if in_block && trimmed == "```" {
            // End of fenced block
            in_block = false;
            blocks.push(FencedBlock {
                start_line,
                lines: std::mem::take(&mut current_lines),
            });
        } else if in_block {
            current_lines.push(line.to_string());
        }
    }

    blocks
}
```

**Verification:**
- [ ] Unit test: Extract single fenced block
- [ ] Unit test: Extract multiple fenced blocks
- [ ] Unit test: Handle unclosed block (no panic)
- [ ] Unit test: Handle empty block

### Phase 4: Implement Directory Tree Parsing

Parse the content of fenced blocks to detect directory tree structures. Support both box-drawing format (├──, └──, │) and indentation format (spaces/tabs).

**Add to:** `crates/cli/src/checks/docs/toc.rs`

```rust
/// A parsed entry from a directory tree.
#[derive(Debug, PartialEq)]
struct TreeEntry {
    /// Relative line offset within the block (0-indexed).
    line_offset: u32,
    /// The extracted path (may be file or directory).
    path: String,
    /// True if this appears to be a directory (ends with /).
    is_dir: bool,
}

/// Check if a line looks like a directory tree entry.
fn is_tree_line(line: &str) -> bool {
    let trimmed = line.trim();
    // Must contain a path-like segment
    if trimmed.is_empty() {
        return false;
    }
    // Box-drawing characters or indentation followed by path
    trimmed.contains("├") || trimmed.contains("└") || trimmed.contains("│")
        || (trimmed.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '.' || c == '_'))
}

/// Parse a directory tree block into entries.
fn parse_tree_block(block: &FencedBlock) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    let mut current_path_stack: Vec<String> = Vec::new();

    for (offset, line) in block.lines.iter().enumerate() {
        if let Some(entry) = parse_tree_line(line, offset as u32, &mut current_path_stack) {
            entries.push(entry);
        }
    }

    entries
}

/// Parse a single line of a directory tree.
///
/// Returns Some(TreeEntry) if the line contains a path entry.
fn parse_tree_line(
    line: &str,
    line_offset: u32,
    path_stack: &mut Vec<String>,
) -> Option<TreeEntry> {
    // Strip box-drawing characters and measure indent
    let (indent_level, name) = extract_indent_and_name(line)?;

    // Strip trailing comment (after #)
    let name = strip_comment(name);
    if name.is_empty() {
        return None;
    }

    let is_dir = name.ends_with('/');
    let name = name.trim_end_matches('/');

    // Adjust path stack to current indent level
    path_stack.truncate(indent_level);

    // Build full path
    let full_path = if path_stack.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", path_stack.join("/"), name)
    };

    // If directory, push to stack for children
    if is_dir {
        path_stack.push(name.to_string());
    }

    Some(TreeEntry {
        line_offset,
        path: full_path,
        is_dir,
    })
}

/// Extract indent level and name from a tree line.
///
/// Handles both box-drawing (├── name) and indentation (  name) formats.
fn extract_indent_and_name(line: &str) -> Option<(usize, &str)> {
    // Count leading box-drawing characters and spaces
    let mut indent = 0usize;
    let mut chars = line.chars().peekable();
    let mut last_was_branch = false;

    while let Some(&c) = chars.peek() {
        match c {
            '│' | ' ' | '\t' => {
                if c == '│' || c == '\t' || (c == ' ' && !last_was_branch) {
                    if c == '\t' {
                        indent += 1;
                    } else if c == ' ' {
                        // Count 2-4 spaces as one indent level
                    }
                }
                chars.next();
                last_was_branch = false;
            }
            '├' | '└' => {
                chars.next();
                last_was_branch = true;
                // Skip the "── " that follows
                while let Some(&c) = chars.peek() {
                    if c == '─' || c == ' ' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Calculate indent from position in line
                let consumed = line.len() - chars.as_str().len();
                indent = consumed / 4; // Rough estimate: 4 chars per level
            }
            _ => break,
        }
    }

    let remaining: String = chars.collect();
    let name = remaining.trim();

    if name.is_empty() {
        None
    } else {
        Some((indent, name))
    }
}

/// Strip comment suffix (everything after #).
fn strip_comment(name: &str) -> &str {
    if let Some(pos) = name.find('#') {
        name[..pos].trim()
    } else {
        name.trim()
    }
}
```

**Verification:**
- [ ] Unit test: Parse box-drawing format tree
- [ ] Unit test: Parse indentation format tree
- [ ] Unit test: Handle nested directories
- [ ] Unit test: Strip comments after #
- [ ] Unit test: Handle mixed format

### Phase 5: Implement Path Resolution and Validation

Resolve extracted paths against the filesystem using the specified resolution order:
1. Relative to the markdown file's directory
2. Relative to `docs/` directory
3. Relative to project root

**Add to:** `crates/cli/src/checks/docs/toc.rs`

```rust
/// Resolution strategy for TOC paths.
#[derive(Debug, Clone, Copy)]
enum Resolution {
    /// Relative to the markdown file's directory
    FileDir,
    /// Relative to docs/ directory
    DocsDir,
    /// Relative to project root
    Root,
}

/// Resolve a path from a TOC entry.
///
/// Returns the resolved absolute path if found, None otherwise.
fn resolve_path(
    root: &Path,
    md_file: &Path,
    entry_path: &str,
) -> Option<std::path::PathBuf> {
    // Resolution order per spec
    let strategies = [Resolution::FileDir, Resolution::DocsDir, Resolution::Root];

    for strategy in strategies {
        let candidate = match strategy {
            Resolution::FileDir => {
                md_file.parent()?.join(entry_path)
            }
            Resolution::DocsDir => {
                root.join("docs").join(entry_path)
            }
            Resolution::Root => {
                root.join(entry_path)
            }
        };

        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Check if a path should be validated (not excluded).
fn should_validate(path: &str, exclude_set: &GlobSet) -> bool {
    !exclude_set.is_match(path)
}

/// Build a GlobSet from exclude patterns.
fn build_exclude_set(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}
```

**Verification:**
- [ ] Unit test: Resolve path relative to markdown file
- [ ] Unit test: Fall back to docs/ directory
- [ ] Unit test: Fall back to project root
- [ ] Unit test: Return None for missing file

### Phase 6: Implement TOC Validation Entry Point

Wire everything together in the main validation function that processes all markdown files and generates violations.

**Add to:** `crates/cli/src/checks/docs/toc.rs`

```rust
/// Validate TOC entries in all markdown files.
pub fn validate_toc(ctx: &CheckContext, violations: &mut Vec<Violation>) {
    let config = &ctx.config.check.docs.toc;

    // Check if TOC validation is disabled
    let check_level = config.check.as_deref()
        .or(ctx.config.check.docs.check.as_deref())
        .unwrap_or("error");
    if check_level == "off" {
        return;
    }

    // Build include/exclude matchers
    let include_set = build_glob_set(&config.include);
    let exclude_set = build_exclude_set(&config.exclude);

    // Process each markdown file
    for walked in ctx.files {
        let path_str = walked.path.to_string_lossy();

        // Check include patterns
        if !include_set.is_match(&*path_str) {
            continue;
        }

        // Check exclude patterns
        if exclude_set.is_match(&*path_str) {
            continue;
        }

        // Read file content
        let abs_path = ctx.root.join(&walked.path);
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Extract and validate TOC blocks
        validate_file_toc(ctx, &walked.path, &content, violations);
    }
}

/// Validate TOC entries in a single file.
fn validate_file_toc(
    ctx: &CheckContext,
    file_path: &Path,
    content: &str,
    violations: &mut Vec<Violation>,
) {
    let blocks = extract_fenced_blocks(content);

    for block in blocks {
        // Skip blocks that don't look like directory trees
        if !looks_like_tree(&block) {
            continue;
        }

        let entries = parse_tree_block(&block);
        let abs_file = ctx.root.join(file_path);

        for entry in entries {
            // Skip directories (only validate files)
            if entry.is_dir {
                continue;
            }

            // Try to resolve the path
            if resolve_path(ctx.root, &abs_file, &entry.path).is_none() {
                let line = block.start_line + entry.line_offset;
                violations.push(Violation::file(
                    file_path,
                    line,
                    "broken_toc",
                    "File does not exist. Update the tree or create the file.",
                ).with_pattern(entry.path.clone()));
            }
        }
    }
}

/// Check if a fenced block looks like a directory tree.
fn looks_like_tree(block: &FencedBlock) -> bool {
    // Must have at least one line
    if block.lines.is_empty() {
        return false;
    }

    // Check if any line has tree-like characteristics
    block.lines.iter().any(|line| {
        let trimmed = line.trim();
        // Box-drawing characters
        trimmed.contains('├') || trimmed.contains('└') || trimmed.contains('│')
        // Or looks like a path (ends with / or has extension)
        || trimmed.ends_with('/')
        || (trimmed.contains('.') && !trimmed.starts_with('.'))
    })
}

fn build_glob_set(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}
```

**Verification:**
- [ ] `cargo test` passes for new unit tests
- [ ] Behavioral spec `toc_tree_entries_validated_against_filesystem` passes
- [ ] Behavioral spec `broken_toc_path_generates_violation` passes

### Phase 7: Unit Tests for Tree Parsing

Create comprehensive unit tests for the tree parsing logic.

**File to create:** `crates/cli/src/checks/docs/toc_tests.rs`

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn extract_single_fenced_block() {
    let content = r#"# Header

```
foo/
  bar.rs
```

More text.
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].start_line, 4);
    assert_eq!(blocks[0].lines, vec!["foo/", "  bar.rs"]);
}

#[test]
fn extract_multiple_fenced_blocks() {
    let content = r#"
```
block1
```

```
block2
```
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn parse_box_drawing_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "├── lib.rs".to_string(),
            "└── main.rs".to_string(),
        ],
    };
    let entries = parse_tree_block(&block);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1].path, "src/lib.rs");
    assert_eq!(entries[2].path, "src/main.rs");
}

#[test]
fn parse_indentation_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "  lib.rs".to_string(),
            "  main.rs".to_string(),
        ],
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "src/lib.rs"));
    assert!(entries.iter().any(|e| e.path == "src/main.rs"));
}

#[test]
fn strip_comments_from_entries() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "├── lib.rs  # Main library".to_string(),
        ],
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "src/lib.rs"));
    assert!(!entries.iter().any(|e| e.path.contains('#')));
}

#[test]
fn nested_directories() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "docs/".to_string(),
            "├── specs/".to_string(),
            "│   ├── overview.md".to_string(),
            "│   └── config.md".to_string(),
            "└── README.md".to_string(),
        ],
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "docs/specs/overview.md"));
    assert!(entries.iter().any(|e| e.path == "docs/specs/config.md"));
    assert!(entries.iter().any(|e| e.path == "docs/README.md"));
}

#[test]
fn looks_like_tree_detects_box_drawing() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["├── file.rs".to_string()],
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn looks_like_tree_detects_paths() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["src/".to_string(), "  lib.rs".to_string()],
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn looks_like_tree_rejects_code() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["fn main() {".to_string(), "    println!(\"hi\");".to_string()],
    };
    assert!(!looks_like_tree(&block));
}
```

**Verification:**
- [ ] `cargo test checks::docs::toc` passes all unit tests

## Key Implementation Details

### Tree Detection Heuristics

Not every fenced code block is a directory tree. The `looks_like_tree` function uses heuristics:
1. Contains box-drawing characters (├, └, │)
2. Contains paths ending with `/` (directories)
3. Contains entries with file extensions

This avoids false positives from code blocks.

### Path Stack for Nested Directories

The parser maintains a `path_stack` to track the current directory context. When parsing:
- Directory entries (ending with `/`) push to the stack
- File entries build their path from the stack
- Indent level changes adjust the stack depth

### Violation Format

Violations follow the established pattern:

```
docs: FAIL
  CLAUDE.md:72: toc path not found: checks/coverage.md
    File does not exist. Update the tree or create the file.
```

The `broken_toc` violation type includes:
- `file`: The markdown file containing the TOC
- `line`: Line number of the missing entry
- `pattern`: The path that wasn't found (stored in pattern field)
- `advice`: Actionable guidance

### Exclude Patterns

Default exclude patterns prevent validation of:
- `plans/**` - Implementation plans often reference future files
- `plan.md`, `*_plan.md`, `plan_*` - Individual plan files

These can be customized via `[check.docs.toc.exclude]`.

## Verification Plan

### Unit Tests

```bash
cargo test checks::docs::toc -- --nocapture
```

### Behavioral Specs

```bash
# Run docs check specs (remove #[ignore] annotations first)
cargo test --test specs docs::toc
```

Expected passing specs:
- `toc_tree_entries_validated_against_filesystem`
- `broken_toc_path_generates_violation`
- `toc_box_drawing_format_supported`
- `toc_indentation_format_supported`
- `toc_path_resolution_order`

### Manual Testing

```bash
# Test on valid TOC fixture
cargo run -- --docs tests/fixtures/docs/toc-ok
# Expected: passes

# Test on broken TOC fixture
cargo run -- --docs tests/fixtures/docs/toc-broken
# Expected: fails with "toc path not found: missing.md"

# Test on this project (self-test)
cargo run -- --docs .
# Expected: passes (CLAUDE.md TOC is valid)
```

### Full Check Suite

```bash
make check
```

All existing tests should continue to pass.
