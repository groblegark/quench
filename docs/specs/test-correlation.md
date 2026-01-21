# Test Correlation Specification

The `test-correlation` check ensures code changes are accompanied by test changes.

## Purpose

Verify that modifications to source code have corresponding test coverage:
- New source files should have new test files
- Changes to source files should include test updates
- Prevents shipping untested code

## Scope

Correlation can be checked at different scopes:

### Branch Scope (Default)

Checks all changes on the branch together:
- All source and test changes across all commits count
- Order doesn't matter (tests first or code first both work)
- Ideal for PR checks

```bash
quench --compare-branch main
```

### Commit Scope

Checks individual commits with **asymmetric rules**:
- Tests without code = **OK** (TDD recognized)
- Code without tests = **FAIL**
- Supports "tests first" workflows naturally

```bash
quench --staged          # Pre-commit
quench --since HEAD~5    # Recent commits
```

```toml
[checks.test-correlation]
scope = "branch"  # or "commit"
```

## Modes

### Require Mode (Default)

Source changes require corresponding test changes:
- New source files → require new test file (or test additions)
- Modified source files → require test changes
- Deletions → no test requirement

### Strict Mode

Explicit requirements:
- Every source file must have a corresponding test file
- Any source file change requires test file change
- No exceptions

### Advisory Mode

Warn but don't fail:
- Report when tests appear missing
- Exit code 0 regardless
- Useful during adoption

## Change Detection

### Git Integration

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

### Inline Test Changes (Rust)

For Rust, changes to `#[cfg(test)]` blocks in the same file **satisfy the test requirement**:
- Adding `#[test]` functions counts as test changes
- Modifying existing test code counts
- No separate test file required if inline tests updated

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

## Placeholder Tests (Future)

Support for placeholder tests that indicate planned implementation:

**Rust:**
```rust
#[test]
#[ignore = "TODO: implement parser"]
fn test_parser() { todo!() }
```

**JavaScript/TypeScript:**
```javascript
test.todo('parser should handle edge case');
test.fixme('parser broken on empty input');
```

When placeholder tests exist for a source file, correlation is satisfied even without implementation—the test intent is recorded.

```toml
[checks.test-correlation]
# Recognize placeholder patterns as valid correlation
allow_placeholders = true  # default: true
```

## Output

### Pass (silent)

No output when correlation is satisfied.

### Fail (require mode)

```
test-correlation: FAIL
  src/parser.rs: modified, no corresponding test changes
    Add tests in tests/parser_tests.rs or update inline #[cfg(test)] block
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
  src/parser.rs: changes without test updates
    Consider adding tests for the modified functionality.
```

### JSON Output

```json
{
  "name": "test-correlation",
  "passed": false,
  "mode": "require",
  "scope": "branch",
  "violations": [
    {
      "source_file": "src/parser.rs",
      "change_type": "modified",
      "lines_added": 67,
      "lines_removed": 12,
      "test_file": null,
      "inline_tests": false,
      "test_changes": false,
      "advice": "Add tests in tests/parser_tests.rs or update inline #[cfg(test)] block"
    },
    {
      "source_file": "src/lexer.rs",
      "change_type": "added",
      "lines_added": 234,
      "lines_removed": 0,
      "test_file": null,
      "inline_tests": false,
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

# Mode: require | strict | advisory
mode = "require"

# Scope: branch | commit
# branch = all changes on branch count together (order doesn't matter)
# commit = per-commit checking with asymmetric rules (tests-first OK)
scope = "branch"

# Strict mode settings
require_mirror = false         # Require 1:1 source→test mapping

# Placeholder tests
allow_placeholders = true      # #[ignore], test.todo(), test.fixme() count

# Test file patterns (extend defaults)
test_patterns = [
  "tests/**/*",          # Everything in tests/ directory
  "test/**/*",           # Everything in test/ directory
  "**/*_test.rs",        # Suffix pattern (outside test dirs)
  "**/*_tests.rs",
  "**/*.spec.rs",
  "**/*.spec.ts",
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
