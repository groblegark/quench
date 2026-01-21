# Lines of Code Specification

The `loc` check counts lines of code, separating source from test code.

## Purpose

- Track codebase size and growth
- Ensure healthy source-to-test ratio
- Per-subproject breakdown for focused metrics

## Counting Rules

### What Counts as a Line

A line is counted if it contains at least one non-whitespace character.

**Counted**:
```rust
fn main() {           // counted
    println!("hi");   // counted
}                     // counted
```

**Not counted**:
```rust
                      // blank - not counted

fn main() {
                      // blank - not counted
}
```

### Comments

Comments are counted as lines (they're part of the code).

Future consideration: separate comment LOC metric.

## Source vs Test Separation

### Pattern-Based (Language Agnostic)

Default test patterns:
- `**/tests/**` - test directories
- `**/test/**` - test directories
- `**/*_test.*` - test file suffix
- `**/*_tests.*` - test file suffix
- `**/*.test.*` - test file infix
- `**/*.spec.*` - spec file infix
- `**/test_*.*` - test file prefix

Files matching any test pattern are counted as test code.
All other files matching source patterns are counted as source code.

### Language-Specific (Rust)

Rust adapter can parse `#[cfg(test)]` blocks:

```rust
// Source file with inline tests
pub fn add(a: i32, b: i32) -> i32 {  // source LOC: 3
    a + b
}

#[cfg(test)]                          // test LOC: 6
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
```

When `adapters.rust.parse_cfg_test = true` (default), lines inside `#[cfg(test)]`
blocks are counted as test LOC even in source files.

## Output

### Pass (silent by default)

With `--summary` or `-v`:
```
loc: PASS
  source: 12,453 lines (47 files)
  test: 8,921 lines (32 files)
  ratio: 0.72x
```

### Fail

```
loc: FAIL
  source: 12,453 lines (47 files)
  test: 1,245 lines (8 files)
  ratio: 0.10x (min: 0.50x)
    Test coverage appears low. Add tests for uncovered functionality.
```

### JSON Output

```json
{
  "name": "loc",
  "passed": true,
  "metrics": {
    "source_lines": 12453,
    "source_files": 47,
    "test_lines": 8921,
    "test_files": 32,
    "ratio": 0.72
  },
  "subprojects": {
    "cli": {
      "source_lines": 3421,
      "source_files": 15,
      "test_lines": 2890,
      "test_files": 12,
      "ratio": 0.84
    },
    "core": {
      "source_lines": 9032,
      "source_files": 32,
      "test_lines": 6031,
      "test_files": 20,
      "ratio": 0.67
    }
  }
}
```

## Configuration

```toml
[checks.loc]
enabled = true

# Test ratio thresholds (test LOC / source LOC)
test_ratio_min = 0.5   # Warn if tests are < 50% of source
test_ratio_max = 4.0   # Warn if tests are > 400% of source (over-testing?)

# Override default patterns
source_patterns = ["src/**/*.rs", "lib/**/*.rs"]
test_patterns = ["tests/**/*.rs", "**/*_tests.rs"]
```

## Performance

LOC counting must be fast (target: <100ms for 50k LOC).

Implementation:
1. Parallel file walking with `ignore` crate
2. Memory-mapped file reading
3. Simple newline counting (no parsing)
4. Early termination not applicable (must count all)

For Rust `#[cfg(test)]` parsing:
- Use tree-sitter or simple regex matching
- Cache parse results if same file needed by multiple checks
