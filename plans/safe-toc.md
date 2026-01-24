# Plan: Safe TOC Detection

**Plan:** `safe-toc`
**Root Feature:** `quench-docs`
**Related:** Phase 605 (TOC Validation)

## Overview

Improve the TOC (Table of Contents) validator to avoid false positives when fenced code blocks contain example output with file paths. The current `is_tree_line()` heuristic incorrectly identifies error output like `scripts/deploy.sh:23: shellcheck_missing_comment:` as file path entries.

This plan implements three defensive measures:
1. **Language marker support**: Skip TOC validation for blocks with explicit language tags (```text, ```bash, etc.)
2. **Error output pattern rejection**: Detect and reject compiler/linter error output patterns
3. **Improved heuristics**: Strengthen the "looks like a tree" detection to reduce false positives

## Project Structure

Files to modify:

```
crates/cli/src/checks/docs/
├── toc.rs          # MODIFY: Add language detection, improve heuristics
└── toc_tests.rs    # MODIFY: Add tests for new patterns
```

## Dependencies

No new external crates required.

## Implementation Phases

### Phase 1: Add Language Tag Detection to FencedBlock

Track the language tag (if any) when extracting fenced blocks. Blocks with language markers like ```rust, ```bash, ```text are unlikely to be directory trees.

**File:** `crates/cli/src/checks/docs/toc.rs`

Modify `FencedBlock` struct:

```rust
#[derive(Debug)]
struct FencedBlock {
    /// Line number where the block starts (1-indexed, after the opening ```).
    start_line: u32,
    /// Content lines within the block.
    lines: Vec<String>,
    /// Language tag from the opening fence (e.g., "rust", "bash", "text").
    /// None if no tag was specified.
    language: Option<String>,
}
```

Modify `extract_fenced_blocks()` to extract the language:

```rust
if !in_block && trimmed.starts_with("```") {
    in_block = true;
    start_line = line_num + 1;
    current_lines.clear();
    // Extract language tag after ```
    let after_fence = trimmed.strip_prefix("```").unwrap_or("").trim();
    current_language = if after_fence.is_empty() {
        None
    } else {
        // Take first word as language (handles ```rust,linenos)
        Some(after_fence.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .next()
            .unwrap_or("")
            .to_lowercase())
    };
}
```

**Verification:**
- [ ] Unit test: Extract language tag from ```rust block
- [ ] Unit test: Handle no language tag
- [ ] Unit test: Handle language with attributes (```rust,linenos)

### Phase 2: Skip Validation for Language-Tagged Blocks

Update `looks_like_tree()` to skip blocks with certain language tags that are definitively not directory trees.

**File:** `crates/cli/src/checks/docs/toc.rs`

```rust
/// Language tags that indicate the block is NOT a directory tree.
const NON_TREE_LANGUAGES: &[&str] = &[
    // Code languages
    "rust", "rs", "go", "python", "py", "javascript", "js", "typescript", "ts",
    "java", "c", "cpp", "csharp", "cs", "ruby", "rb", "php", "swift", "kotlin",
    "scala", "perl", "lua", "r", "julia", "haskell", "hs", "ocaml", "ml",
    "elixir", "ex", "erlang", "clojure", "clj", "lisp", "scheme", "racket",
    "zig", "nim", "d", "v", "odin", "jai", "carbon",
    // Shell and scripting
    "bash", "sh", "zsh", "fish", "powershell", "pwsh", "bat", "cmd",
    // Config and data (could be tree-like but explicit tag means user knows)
    "toml", "yaml", "yml", "json", "xml", "ini", "cfg",
    // Output and plain text
    "text", "txt", "output", "console", "terminal", "log",
    // Markup
    "html", "css", "scss", "sass", "less",
    // Other
    "sql", "graphql", "gql", "dockerfile", "makefile", "cmake",
];

fn looks_like_tree(block: &FencedBlock) -> bool {
    // Blocks with known non-tree language tags are skipped
    if let Some(ref lang) = block.language {
        if NON_TREE_LANGUAGES.contains(&lang.as_str()) {
            return false;
        }
    }

    // ... existing heuristics
}
```

**Verification:**
- [ ] Unit test: ```bash block is not treated as tree
- [ ] Unit test: ```text block is not treated as tree
- [ ] Unit test: Unlabeled block with tree content still detected
- [ ] Integration: Example output in docs/specs/langs/shell.md no longer triggers violations

### Phase 3: Detect and Reject Error Output Patterns

Add pattern detection for compiler/linter error output that looks like file paths but isn't.

**File:** `crates/cli/src/checks/docs/toc.rs`

Update `is_tree_line()` to reject error output patterns:

```rust
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
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return false;
    }
    if trimmed.contains(" = ") {
        return false;
    }
    if trimmed.starts_with("[[") {
        return false;
    }

    // NEW: Reject error output patterns
    // Pattern: file.ext:line: or file.ext:line:col:
    // Examples: "foo.rs:23:", "src/main.go:45:12:", "script.sh:10: error"
    if looks_like_error_output(trimmed) {
        return false;
    }

    // Directory paths ending with /
    if trimmed.ends_with('/') && !trimmed.contains(' ') && !trimmed.contains('=') {
        return true;
    }

    // ... rest of existing logic
}

/// Check if a line looks like compiler/linter error output.
///
/// Matches patterns like:
/// - `file.ext:123:` (file:line:)
/// - `file.ext:123:45:` (file:line:col:)
/// - `file.ext:123: message` (file:line: message)
fn looks_like_error_output(line: &str) -> bool {
    // Look for pattern: something.ext:digits:
    // Must have: extension with dot, colon, digits, colon
    let Some(colon_pos) = line.find(':') else {
        return false;
    };

    let before_colon = &line[..colon_pos];

    // Must look like a file path (contains dot for extension)
    if !before_colon.contains('.') {
        return false;
    }

    // Check if what follows the colon starts with digits
    let after_colon = &line[colon_pos + 1..];
    let first_after = after_colon.chars().next();

    match first_after {
        Some(c) if c.is_ascii_digit() => {
            // Looks like file.ext:123...
            // Check if followed by another colon (file:line: or file:line:col:)
            if let Some(next_colon) = after_colon.find(':') {
                let between = &after_colon[..next_colon];
                // All digits between first and second colon
                if between.chars().all(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
        _ => {}
    }

    false
}
```

**Verification:**
- [ ] Unit test: `scripts/deploy.sh:23:` detected as error output
- [ ] Unit test: `src/main.rs:45:12: error` detected as error output
- [ ] Unit test: `foo.rs` NOT detected as error output (valid tree entry)
- [ ] Unit test: `Cargo.toml:` NOT detected as error (no line number)

### Phase 4: Strengthen Tree Heuristics

Improve the overall `looks_like_tree()` function to require stronger signals before treating a block as a directory tree.

**File:** `crates/cli/src/checks/docs/toc.rs`

```rust
fn looks_like_tree(block: &FencedBlock) -> bool {
    // Skip blocks with known non-tree language tags
    if let Some(ref lang) = block.language {
        if NON_TREE_LANGUAGES.contains(&lang.as_str()) {
            return false;
        }
    }

    // Must have at least one line
    if block.lines.is_empty() {
        return false;
    }

    // Count different types of tree signals
    let box_drawing_lines = block.lines.iter()
        .filter(|line| {
            let t = line.trim();
            t.contains('├') || t.contains('└') || t.contains('│')
        })
        .count();

    let directory_lines = block.lines.iter()
        .filter(|line| {
            let t = line.trim();
            t.ends_with('/') && !t.contains(' ') && !t.contains('=')
        })
        .count();

    let file_like_lines = block.lines.iter()
        .filter(|line| is_tree_line(line))
        .count();

    // Strong signal: any box-drawing characters
    if box_drawing_lines >= 1 {
        return true;
    }

    // Strong signal: directory lines (ending with /)
    if directory_lines >= 1 && file_like_lines >= 2 {
        return true;
    }

    // Weak signal: multiple file-like lines
    // Require MORE evidence (3+ lines instead of 2)
    // AND no indication this is error output
    if file_like_lines >= 3 {
        // Check that NO lines look like error output
        let has_error_output = block.lines.iter()
            .any(|line| looks_like_error_output(line.trim()));
        if !has_error_output {
            return true;
        }
    }

    false
}
```

**Verification:**
- [ ] Unit test: Block with error output lines not treated as tree
- [ ] Unit test: Block with box-drawing still detected
- [ ] Unit test: Block with directory + files still detected
- [ ] Regression: Existing tree detection still works

### Phase 5: Add Helpful Violation Advice

When a block IS detected as a tree but validation fails, provide guidance on how to mark non-tree blocks.

**File:** `crates/cli/src/checks/docs/toc.rs`

Update the violation message to include guidance:

```rust
fn validate_file_toc(
    ctx: &CheckContext,
    relative_path: &Path,
    content: &str,
    violations: &mut Vec<Violation>,
) {
    let blocks = extract_fenced_blocks(content);

    for block in blocks {
        if !looks_like_tree(&block) {
            continue;
        }

        let entries = parse_tree_block(&block);
        let abs_file = ctx.root.join(relative_path);
        let mut block_violations = Vec::new();

        for entry in entries {
            if entry.is_dir {
                continue;
            }

            if !resolve_path(ctx.root, &abs_file, &entry.path) {
                let line = block.start_line + entry.line_offset;
                block_violations.push((line, entry.path.clone()));
            }
        }

        // If most entries in the block fail, suggest marking as non-tree
        if !block_violations.is_empty() {
            let advice = if block_violations.len() > 2 {
                "File does not exist. If this is example output (not a directory tree), \
                 add a language tag like ```text or ```bash to skip TOC validation."
            } else {
                "File does not exist. Update the tree or create the file."
            };

            for (line, path) in block_violations {
                violations.push(
                    Violation::file(relative_path, line, "broken_toc", advice)
                        .with_pattern(path),
                );
            }
        }
    }
}
```

**Verification:**
- [ ] Unit test: Single missing file shows standard message
- [ ] Unit test: Multiple missing files suggests adding language tag
- [ ] Manual: Verify advice is actionable

### Phase 6: Add Comprehensive Tests

Add unit tests covering the new functionality and regression tests for existing behavior.

**File:** `crates/cli/src/checks/docs/toc_tests.rs`

```rust
// === Phase 1: Language tag extraction ===

#[test]
fn extract_language_tag_rust() {
    let content = "```rust\nfn main() {}\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
}

#[test]
fn extract_language_tag_with_attributes() {
    let content = "```rust,linenos\ncode\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
}

#[test]
fn extract_no_language_tag() {
    let content = "```\nplain block\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, None);
}

// === Phase 2: Language-tagged blocks skipped ===

#[test]
fn bash_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["scripts/deploy.sh:23: error".to_string()],
        language: Some("bash".to_string()),
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn text_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["foo.rs".to_string(), "bar.rs".to_string()],
        language: Some("text".to_string()),
    };
    assert!(!looks_like_tree(&block));
}

// === Phase 3: Error output detection ===

#[test]
fn error_output_file_line() {
    assert!(looks_like_error_output("scripts/deploy.sh:23:"));
    assert!(looks_like_error_output("src/main.rs:45:12:"));
    assert!(looks_like_error_output("foo.go:100: undefined"));
}

#[test]
fn not_error_output() {
    assert!(!looks_like_error_output("foo.rs")); // no line number
    assert!(!looks_like_error_output("src/")); // directory
    assert!(!looks_like_error_output("Cargo.toml")); // no colon
    assert!(!looks_like_error_output("README")); // no extension
}

#[test]
fn error_output_in_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "scripts/deploy.sh:23: shellcheck_missing_comment:".to_string(),
            "  Lint suppression requires justification.".to_string(),
            "scripts/build.sh:45: shellcheck_missing_comment:".to_string(),
        ],
        language: None,
    };
    assert!(!looks_like_tree(&block));
}

// === Regression tests ===

#[test]
fn unlabeled_tree_still_detected() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "├── lib.rs".to_string(),
            "└── main.rs".to_string(),
        ],
        language: None,
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn indentation_tree_still_detected() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "docs/".to_string(),
            "  README.md".to_string(),
            "  overview.md".to_string(),
        ],
        language: None,
    };
    assert!(looks_like_tree(&block));
}
```

**Verification:**
- [ ] `cargo test checks::docs::toc` passes all tests
- [ ] No regressions in existing functionality

## Key Implementation Details

### Language Tag Extraction

The language tag is extracted from the opening fence line after the triple backticks. We normalize to lowercase and handle common variations:
- `rust` → `rust`
- `Rust` → `rust`
- `rust,linenos` → `rust`

### Error Output Pattern

Error output typically follows the pattern `file.ext:line:` or `file.ext:line:col:`. Key distinguishing features:
- Contains a file extension (has `.`)
- Followed by colon and digits (line number)
- Followed by another colon

This is distinct from tree entries which are just `file.ext` or `dir/file.ext` without trailing colons and line numbers.

### Threshold Adjustment

The original code required only 2 tree-like lines to consider a block a tree. This was too aggressive. The new implementation:
- 1+ box-drawing lines → definitely a tree
- 1+ directory lines AND 2+ file lines → likely a tree
- 3+ file-like lines WITHOUT error patterns → possibly a tree

### Advice Enhancement

When multiple entries fail validation in a single block, this is a strong signal that the block might not be a tree at all. The enhanced message guides users to add a language tag.

## Verification Plan

### Unit Tests

```bash
cargo test checks::docs::toc -- --nocapture
```

Expected new tests:
- `extract_language_tag_rust`
- `extract_language_tag_with_attributes`
- `extract_no_language_tag`
- `bash_block_not_tree`
- `text_block_not_tree`
- `error_output_file_line`
- `not_error_output`
- `error_output_in_block_not_tree`
- `unlabeled_tree_still_detected`
- `indentation_tree_still_detected`

### Manual Testing

```bash
# Create a test file with example output
cat > /tmp/test-doc.md << 'EOF'
# Test

Example error output:

```
scripts/deploy.sh:23: shellcheck_missing_comment:
  Lint suppression requires justification.
scripts/build.sh:45: shellcheck_missing_comment:
  Another message.
```
EOF

# Run TOC validation (should not flag these as broken TOC)
cargo run -- --check docs.toc /tmp/test-doc.md
# Expected: no violations

# Test with language tag
cat > /tmp/test-doc2.md << 'EOF'
```text
scripts/deploy.sh:23: error
scripts/build.sh:45: error
```
EOF

cargo run -- --check docs.toc /tmp/test-doc2.md
# Expected: no violations (language tag skips validation)
```

### Full Check Suite

```bash
make check
```

All existing tests should continue to pass.
