# TOC validator flags example output as broken TOC entries

## Description

The docs TOC validator (`broken_toc` check) incorrectly flags example command output in fenced code blocks as broken file references when the output contains file paths.

### Current Behavior

When documentation includes example error output with file paths, the TOC validator treats the code block as a directory tree and tries to validate each line as a file reference:

```markdown
**Example outputs:**

```
scripts/deploy.sh:23: shellcheck_missing_comment: # shellcheck disable=SC2086
  Lint suppression requires justification.
  Is unquoted expansion intentional here?
  Add a comment above the directive.

scripts/build.sh:45: shellcheck_missing_comment: # shellcheck disable=SC2154
  Lint suppression requires justification.
  Is this variable defined externally?
  If so, add one of:
    # OK: ...
    # INTENTIONAL: ...
```
```

This triggers violations:
```
docs/specs/langs/shell.md:112: broken_toc: scripts/deploy.sh:23: shellcheck_missing_comment:
    File does not exist. Update the tree or create the file.
docs/specs/langs/shell.md:113: broken_toc: Lint suppression requires justification.
    File does not exist. Update the tree or create the file.
...
```

### Root Cause

The `is_tree_line()` function in `crates/cli/src/checks/docs/toc.rs` detects lines containing `.` as potential file paths:

```rust
// Line 312
if trimmed.contains('.') && !trimmed.starts_with('.') {
    // Reject if it looks like code or config
    let code_patterns = ['(', ')', '=', ';', '{', '}', '"', '\'', '[', ']'];
    ...
    if !has_code_pattern && !has_code_keyword && !trimmed.contains("//") {
        ...
        return true;  // Treats as file path
    }
}
```

The heuristic misidentifies error output like `scripts/deploy.sh:23:` as file paths because:
- It contains `.` (in `.sh`)
- It doesn't contain code patterns like `(`, `=`, etc.
- It passes the file-like pattern check

The `looks_like_tree()` check requires at least 2 tree-like lines, which is met when there are multiple example outputs.

### Impact

- Documentation cannot include realistic example output with file paths

### Proposed Solutions

1. **Language marker support**: Skip TOC validation for fenced blocks with language markers:
   ```rust
   fn extract_fenced_blocks(content: &str) -> Vec<FencedBlock> {
       // Detect ```text, ```bash, etc. and skip validation
   }
   ```

2. **Colon-based rejection**: Reject lines with `:` followed by line numbers as they're error output:
   ```rust
   // Reject error output patterns like "file.rs:123:"
   if trimmed.contains('.') && trimmed.contains("::digit::") {
       return false;
   }
   ```

3. **Ensure failure provides advice for non-TOC**: "How should I mark this correctly"

### Files Involved

- `crates/cli/src/checks/docs/toc.rs` - TOC validation logic
  - `extract_fenced_blocks()` - Line 195
  - `is_tree_line()` - Line 278
  - `looks_like_tree()` - Line 261

### Affected Documentation

Currently blocking:
- `docs/specs/langs/shell.md` - Lines 112-120
- Potentially other files with similar examples

### Workaround Applied

Temporarily reduced shell.md to single example to avoid 2-line tree detection threshold.
