# Test Correlation Specification

The `test-correlation` check ensures code changes are accompanied by test changes.

## Purpose

Verify that modifications to source code have corresponding test coverage:
- New source files should have new test files
- Changes to source files should include test updates
- Prevents shipping untested code

## Modes

### Smart Mode (Default)

Heuristic-based detection:
- New source files → require new test file (or test additions)
- Significant source changes (>20 lines) → require test changes
- Small refactors (<20 lines) → advisory only
- Deletions → no test requirement

### Strict Mode

Explicit requirements:
- Every source file must have a corresponding test file
- Any source file change requires test file change in same commit/PR
- No exceptions

### Advisory Mode

Warn but don't fail:
- Report when tests appear missing
- Exit code 0 regardless
- Useful during adoption

## Change Detection

### Git Integration

Quench uses git to detect changes:

```bash
# Staged changes (pre-commit)
quench --staged

# Branch changes (PR/CI)
quench --compare-branch main

# Specific commits
quench --since HEAD~5
```

### What Counts as "Changed"

- Added files: new file in diff
- Modified files: existing file with changes
- Deleted files: not checked (no test required)

## Test File Matching

### Pattern-Based (Language Agnostic)

For source file `src/parser.rs`, look for tests in:
1. `tests/parser.rs`
2. `tests/parser_test.rs`
3. `tests/parser_tests.rs`
4. `src/parser_test.rs`
5. `src/parser_tests.rs`
6. `test/parser.rs`
7. Any file matching test patterns containing "parser"

### Directory Mirroring (Optional)

When `require_mirror = true`:
```
src/foo/bar.rs  →  tests/foo/bar_test.rs (required)
```

### Language-Specific (Rust)

For Rust, also check:
- `#[cfg(test)]` block in same file
- `#[test]` functions added

## Inference vs Detection

### Inferred Test Coverage

"Smart" mode infers that tests exist somewhere:
- Source: `src/parser.rs`
- Test pattern match: `tests/parser_tests.rs` contains "parser"
- Inference: tests likely cover this module

### Detected Test Changes

Explicit detection of test changes in the diff:
- Source changed: `src/parser.rs` (+50 lines)
- Test changed: `tests/parser_tests.rs` (+30 lines)
- Detection: tests were updated alongside source

## Output

### Pass (silent)

No output when correlation is satisfied.

### Fail (smart mode)

```
test-correlation: FAIL
  src/parser.rs: 67 lines added, no corresponding test changes
    Add tests for the new parser functionality in tests/parser_tests.rs
  src/lexer.rs: new file, no test file found
    Create tests/lexer_tests.rs with tests for the lexer module
```

### Fail (strict mode)

```
test-correlation: FAIL
  src/utils/helpers.rs: no corresponding test file
    Create src/utils/helpers_tests.rs or tests/utils/helpers_test.rs
```

### Advisory (warn mode)

```
test-correlation: WARN
  src/parser.rs: significant changes without test updates
    Consider adding tests for the modified functionality.
```

### JSON Output

```json
{
  "name": "test-correlation",
  "passed": false,
  "mode": "smart",
  "violations": [
    {
      "source_file": "src/parser.rs",
      "change_type": "modified",
      "lines_added": 67,
      "lines_removed": 12,
      "test_file": null,
      "test_changes": false,
      "advice": "Add tests for the new parser functionality."
    },
    {
      "source_file": "src/lexer.rs",
      "change_type": "added",
      "lines_added": 234,
      "lines_removed": 0,
      "test_file": null,
      "test_changes": false,
      "advice": "Create tests/lexer_tests.rs with tests for the lexer module."
    }
  ],
  "summary": {
    "source_files_changed": 5,
    "with_test_changes": 3,
    "without_test_changes": 2
  }
}
```

## Configuration

```toml
[checks.test-correlation]
enabled = true

# Mode: smart | strict | advisory
mode = "smart"

# Smart mode thresholds
significant_change_lines = 20  # Lines added to trigger test requirement

# Strict mode settings
require_mirror = false         # Require 1:1 source→test mapping

# Test file patterns (extend defaults)
test_patterns = [
  "tests/**/*",          # Everything in tests/ directory
  "test/**/*",           # Everything in test/ directory
  "**/*_test.rs",        # Suffix pattern (outside test dirs)
  "**/*_tests.rs",
  "**/*.spec.rs",
]

# Source patterns to check
source_patterns = ["src/**/*.rs"]

# Exclude patterns (never require tests)
exclude = [
  "**/mod.rs",           # Module declarations
  "**/lib.rs",           # Library roots
  "**/main.rs",          # Binary entry points
  "**/generated/**",     # Generated code
]
```

## Future: Comment Annotations

Future support for explicit annotations:

```rust
// quench:source
pub fn important_function() { ... }

// quench:test-for:important_function
#[test]
fn test_important_function() { ... }
```

Or file-level:
```rust
//! quench:test-file
```

This allows explicit marking when automatic detection is insufficient.
