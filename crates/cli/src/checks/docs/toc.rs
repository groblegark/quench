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
    let name = strip_comment(&name);
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
fn extract_indent_and_name(line: &str) -> Option<(usize, String)> {
    // Check if this is a box-drawing tree line
    let has_box_drawing = line.contains('├') || line.contains('└') || line.contains('│');

    if has_box_drawing {
        // Box-drawing format
        // The indent is determined by the column position of ├ or └
        // Each tree level typically takes 4 characters: "│   " or "├── "
        let mut column = 0usize;
        let mut chars = line.chars().peekable();

        loop {
            match chars.peek() {
                Some(&c @ ('│' | '├' | '└')) => {
                    // Found a box-drawing character
                    chars.next();
                    column += 1;

                    if c == '├' || c == '└' {
                        // Branch marker - skip "── " that follows and extract name
                        while let Some(&c) = chars.peek() {
                            if c == '─' || c == ' ' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        break;
                    }
                    // For │, continue to count more prefix
                }
                Some(' ') => {
                    chars.next();
                    column += 1;
                }
                _ => break,
            }
        }

        // Calculate indent level: typically 4 columns per level
        // But the first level starts at column 0 with ├──
        // So column 0-3 = level 1, column 4-7 = level 2, etc.
        let indent = column.div_ceil(4);

        let remaining: String = chars.collect();
        let name = remaining.trim().to_string();

        if name.is_empty() {
            None
        } else {
            Some((indent, name))
        }
    } else {
        // Indentation format (spaces/tabs)
        let mut indent = 0usize;
        let mut chars = line.chars().peekable();

        // Count leading whitespace
        let mut spaces: usize = 0;
        while let Some(&c) = chars.peek() {
            match c {
                ' ' => {
                    spaces += 1;
                    chars.next();
                }
                '\t' => {
                    indent += 1;
                    chars.next();
                }
                _ => break,
            }
        }

        // Convert spaces to indent level (2 spaces = 1 level)
        if spaces > 0 {
            indent += spaces.div_ceil(2);
        }

        let remaining: String = chars.collect();
        let name = remaining.trim().to_string();

        if name.is_empty() {
            None
        } else {
            Some((indent, name))
        }
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

/// Resolve a path from a TOC entry.
///
/// Returns true if the path exists, checking in order:
/// 1. Relative to the markdown file's directory
/// 2. Relative to docs/ directory
/// 3. Relative to project root
fn resolve_path(root: &Path, md_file: &Path, entry_path: &str) -> bool {
    // Resolution order per spec
    // 1. Relative to markdown file's directory
    if let Some(parent) = md_file.parent() {
        let candidate = parent.join(entry_path);
        if candidate.exists() {
            return true;
        }
    }

    // 2. Relative to docs/ directory
    let docs_candidate = root.join("docs").join(entry_path);
    if docs_candidate.exists() {
        return true;
    }

    // 3. Relative to project root
    let root_candidate = root.join(entry_path);
    if root_candidate.exists() {
        return true;
    }

    false
}

/// Build a GlobSet from patterns.
fn build_glob_set(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}

/// Check if a fenced block looks like a directory tree.
fn looks_like_tree(block: &FencedBlock) -> bool {
    // Must have at least one line
    if block.lines.is_empty() {
        return false;
    }

    // Count lines that look like tree entries
    let tree_line_count = block.lines.iter().filter(|line| is_tree_line(line)).count();

    // Require at least 2 tree-like lines to be considered a tree
    // This avoids false positives from single-line file references
    tree_line_count >= 2
}

/// Check if a line looks like a directory tree entry.
fn is_tree_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Empty lines don't count
    if trimmed.is_empty() {
        return false;
    }

    // Box-drawing characters are strong tree indicators
    if trimmed.contains('├') || trimmed.contains('└') || trimmed.contains('│') {
        return true;
    }

    // Reject TOML/config patterns
    // [section] headers
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return false;
    }
    // key = value assignments
    if trimmed.contains(" = ") {
        return false;
    }
    // TOML table arrays [[...]]
    if trimmed.starts_with("[[") {
        return false;
    }

    // Directory paths ending with /
    if trimmed.ends_with('/') && !trimmed.contains(' ') && !trimmed.contains('=') {
        return true;
    }

    // Check for file-like patterns that aren't code or config
    // A file path typically looks like: foo/bar.rs, lib.rs, etc.
    if trimmed.contains('.') && !trimmed.starts_with('.') {
        // Reject if it looks like code or config
        let code_patterns = ['(', ')', '=', ';', '{', '}', '"', '\'', '[', ']'];
        let code_keywords = [
            "let ", "fn ", "use ", "pub ", "mod ", "const ", "static ", "name ", "path ",
        ];

        let has_code_pattern = code_patterns.iter().any(|&c| trimmed.contains(c));
        let has_code_keyword = code_keywords.iter().any(|kw| trimmed.contains(kw));

        if !has_code_pattern && !has_code_keyword && !trimmed.contains("//") {
            // Looks like a file path: no spaces except possibly in comments
            let before_comment = trimmed.split('#').next().unwrap_or(trimmed).trim();
            if !before_comment.contains(' ')
                || before_comment.starts_with("├")
                || before_comment.starts_with("└")
            {
                return true;
            }
        }
    }

    false
}

/// Validate TOC entries in all markdown files.
pub fn validate_toc(ctx: &CheckContext, violations: &mut Vec<Violation>) {
    let config = &ctx.config.check.docs.toc;

    // Check if TOC validation is disabled
    let check_level = config
        .check
        .as_deref()
        .or(ctx.config.check.docs.check.as_deref())
        .unwrap_or("error");
    if check_level == "off" {
        return;
    }

    // Build include/exclude matchers
    let include_set = build_glob_set(&config.include);
    let exclude_set = build_glob_set(&config.exclude);

    // Process each markdown file
    for walked in ctx.files {
        // Get relative path for glob matching
        let relative_path = walked.path.strip_prefix(ctx.root).unwrap_or(&walked.path);
        let path_str = relative_path.to_string_lossy();

        // Check include patterns
        if !include_set.is_match(&*path_str) {
            continue;
        }

        // Check exclude patterns
        if exclude_set.is_match(&*path_str) {
            continue;
        }

        // Read file content (walked.path is already absolute)
        let content = match std::fs::read_to_string(&walked.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Extract and validate TOC blocks
        validate_file_toc(ctx, relative_path, &content, violations);
    }
}

/// Validate TOC entries in a single file.
fn validate_file_toc(
    ctx: &CheckContext,
    relative_path: &Path,
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
        let abs_file = ctx.root.join(relative_path);

        for entry in entries {
            // Skip directories (only validate files)
            if entry.is_dir {
                continue;
            }

            // Try to resolve the path
            if !resolve_path(ctx.root, &abs_file, &entry.path) {
                let line = block.start_line + entry.line_offset;
                violations.push(
                    Violation::file(
                        relative_path,
                        line,
                        "broken_toc",
                        "File does not exist. Update the tree or create the file.",
                    )
                    .with_pattern(entry.path.clone()),
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "toc_tests.rs"]
mod tests;
