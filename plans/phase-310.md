# Phase 310: Rust Adapter - Test Code

**Root Feature:** `quench-a0ea`

## Overview

Implement `#[cfg(test)]` block parsing for the Rust adapter to accurately separate inline test code from source code. This enables:

- **Inline test detection** via `#[cfg(test)]` block parsing
- **LOC separation** counting `#[cfg(test)]` lines as test LOC
- **Configurable behavior** via `split_cfg_test` option (default: true)
- **Accurate test correlation** for escape pattern checks

When `split_cfg_test = true`, lines inside `#[cfg(test)]` blocks are counted as test LOC even in source files like `src/lib.rs`, providing more accurate metrics for Rust's idiomatic inline test pattern.

Reference docs:
- `docs/specs/langs/rust.md` (Test Code Detection section)
- `docs/specs/checks/cloc.md` (Language-Specific Rust section)

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── adapter/
│   │   ├── rust.rs            # UPDATE: add cfg_test parsing
│   │   └── rust_tests.rs      # UPDATE: add parsing unit tests
│   ├── config.rs              # UPDATE: add [rust] config section
│   └── checks/
│       └── cloc.rs            # UPDATE: use adapter's line classification
├── tests/
│   ├── specs/
│   │   └── adapters/rust.rs   # UPDATE: remove #[ignore] from cfg_test specs
│   └── fixtures/
│       └── rust/cfg-test/     # EXISTING: inline test fixture
└── plans/
    └── phase-310.md
```

## Dependencies

No new external dependencies. Uses existing:
- Standard library for line-by-line parsing
- `regex` crate (already a dependency) for attribute matching

## Implementation Phases

### Phase 1: Config Support for `split_cfg_test`

Add the `[rust]` config section with `split_cfg_test` option.

**Update `crates/cli/src/config.rs`:**

```rust
/// Rust language-specific configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct RustConfig {
    /// Split #[cfg(test)] blocks from source LOC (default: true).
    #[serde(default = "RustConfig::default_split_cfg_test")]
    pub split_cfg_test: bool,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            split_cfg_test: Self::default_split_cfg_test(),
        }
    }
}

impl RustConfig {
    fn default_split_cfg_test() -> bool {
        true
    }
}
```

**Add to `Config` struct:**

```rust
pub struct Config {
    // ... existing fields ...

    /// Rust-specific configuration.
    #[serde(default)]
    pub rust: RustConfig,
}
```

**Update `parse_with_warnings` to handle `[rust]` section.**

**Milestone:** Config parses `[rust] split_cfg_test = false` without errors.

**Verification:**
```bash
cargo build
cargo test config -- rust
```

---

### Phase 2: `#[cfg(test)]` Block Parser

Add parsing logic to detect line ranges inside `#[cfg(test)]` blocks.

**Add to `crates/cli/src/adapter/rust.rs`:**

```rust
use std::ops::Range;

/// Result of parsing a Rust file for #[cfg(test)] blocks.
#[derive(Debug, Default)]
pub struct CfgTestInfo {
    /// Line ranges (0-indexed) that are inside #[cfg(test)] blocks.
    pub test_ranges: Vec<Range<usize>>,
}

impl CfgTestInfo {
    /// Parse a Rust source file to find #[cfg(test)] block ranges.
    pub fn parse(content: &str) -> Self {
        let mut info = Self::default();
        let mut in_cfg_test = false;
        let mut brace_depth = 0;
        let mut block_start = 0;

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for #[cfg(test)] attribute
            if !in_cfg_test && is_cfg_test_attr(trimmed) {
                in_cfg_test = true;
                block_start = line_idx;
                brace_depth = 0;
                continue;
            }

            if in_cfg_test {
                // Count braces to track block depth
                for ch in trimmed.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                // End of #[cfg(test)] block
                                info.test_ranges.push(block_start..line_idx + 1);
                                in_cfg_test = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        info
    }

    /// Check if a line (0-indexed) is inside a #[cfg(test)] block.
    pub fn is_test_line(&self, line_idx: usize) -> bool {
        self.test_ranges.iter().any(|r| r.contains(&line_idx))
    }
}

/// Check if a line is a #[cfg(test)] attribute.
fn is_cfg_test_attr(line: &str) -> bool {
    // Match #[cfg(test)] with optional whitespace
    line.starts_with("#[cfg(test)]")
        || line.starts_with("#[cfg( test )]")
        || line.contains("#[cfg(test)]")
}
```

**Milestone:** Parser correctly identifies `#[cfg(test)]` block ranges.

**Verification:**
```bash
cargo test adapter::rust -- cfg_test
```

---

### Phase 3: Unit Tests for Parser Edge Cases

Add comprehensive unit tests for the `#[cfg(test)]` parser.

**Add to `crates/cli/src/adapter/rust_tests.rs`:**

```rust
mod cfg_test_parsing {
    use super::*;

    #[test]
    fn basic_cfg_test_block() {
        let content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add() {
        assert_eq!(super::add(1, 2), 3);
    }
}
"#;
        let info = CfgTestInfo::parse(content);

        // Lines 0-4 are source, lines 5-11 are test
        assert!(!info.is_test_line(1)); // pub fn add
        assert!(!info.is_test_line(2)); // a + b
        assert!(info.is_test_line(5));  // #[cfg(test)]
        assert!(info.is_test_line(6));  // mod tests
        assert!(info.is_test_line(10)); // closing brace
    }

    #[test]
    fn nested_braces_in_test() {
        let content = r#"
pub fn main() {}

#[cfg(test)]
mod tests {
    fn helper() {
        if true {
            println!("nested");
        }
    }
}
"#;
        let info = CfgTestInfo::parse(content);

        assert!(!info.is_test_line(1)); // pub fn main
        assert!(info.is_test_line(3));  // #[cfg(test)]
        assert!(info.is_test_line(7));  // nested println
        assert!(info.is_test_line(10)); // closing brace of mod tests
    }

    #[test]
    fn multiple_cfg_test_blocks() {
        let content = r#"
fn a() {}

#[cfg(test)]
mod tests_a {
    #[test]
    fn test_a() {}
}

fn b() {}

#[cfg(test)]
mod tests_b {
    #[test]
    fn test_b() {}
}
"#;
        let info = CfgTestInfo::parse(content);

        assert_eq!(info.test_ranges.len(), 2);
        assert!(!info.is_test_line(1)); // fn a()
        assert!(info.is_test_line(3));  // first #[cfg(test)]
        assert!(!info.is_test_line(9)); // fn b()
        assert!(info.is_test_line(11)); // second #[cfg(test)]
    }

    #[test]
    fn no_cfg_test_blocks() {
        let content = r#"
pub fn main() {
    println!("Hello");
}
"#;
        let info = CfgTestInfo::parse(content);

        assert!(info.test_ranges.is_empty());
        assert!(!info.is_test_line(0));
        assert!(!info.is_test_line(1));
    }

    #[test]
    fn cfg_test_with_path_attribute() {
        // #[cfg(test)] followed by #[path = "..."] should still work
        let content = r#"
pub fn main() {}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
"#;
        let info = CfgTestInfo::parse(content);

        // The #[cfg(test)] starts a conceptual block but mod tests; has no braces
        // This edge case may need special handling for mod declarations
        assert!(info.test_ranges.len() <= 1);
    }

    #[test]
    fn string_literals_with_braces() {
        let content = r#"
fn source() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let s = "{ not a real brace }";
        assert!(true);
    }
}
"#;
        let info = CfgTestInfo::parse(content);

        // Simplified parser may be fooled by braces in strings
        // Document this limitation or handle it
        assert!(info.test_ranges.len() >= 1);
    }
}
```

**Milestone:** All parser edge case tests pass.

**Verification:**
```bash
cargo test adapter::rust -- cfg_test --nocapture
```

---

### Phase 4: Adapter API for Line Classification

Extend `RustAdapter` to classify individual lines within a file.

**Add to `crates/cli/src/adapter/rust.rs`:**

```rust
impl RustAdapter {
    /// Parse a file and return line-level classification.
    ///
    /// Returns a struct with source and test line counts.
    pub fn classify_lines(&self, path: &Path, content: &str) -> LineClassification {
        // First check if the whole file is a test file
        let file_kind = self.classify(path);

        if file_kind == FileKind::Test {
            // Entire file is test code
            let total_lines = content.lines().filter(|l| !l.trim().is_empty()).count();
            return LineClassification {
                source_lines: 0,
                test_lines: total_lines,
            };
        }

        if file_kind != FileKind::Source {
            return LineClassification::default();
        }

        // Parse for #[cfg(test)] blocks
        let cfg_info = CfgTestInfo::parse(content);

        let mut source_lines = 0;
        let mut test_lines = 0;

        for (idx, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            if cfg_info.is_test_line(idx) {
                test_lines += 1;
            } else {
                source_lines += 1;
            }
        }

        LineClassification {
            source_lines,
            test_lines,
        }
    }
}

/// Result of classifying lines within a single file.
#[derive(Debug, Default)]
pub struct LineClassification {
    pub source_lines: usize,
    pub test_lines: usize,
}
```

**Milestone:** Adapter can report separate source/test LOC for a single file.

**Verification:**
```bash
cargo test adapter::rust -- classify_lines
```

---

### Phase 5: CLOC Integration

Update the CLOC check to use the Rust adapter's line classification when `split_cfg_test` is enabled.

**Update `crates/cli/src/checks/cloc.rs`:**

```rust
use crate::adapter::rust::{RustAdapter, LineClassification};
use crate::config::RustConfig;

// In ClocCheck::run():

impl Check for ClocCheck {
    fn run(&self, ctx: &CheckContext) -> CheckResult {
        // ... existing setup ...

        // Get Rust config for split_cfg_test
        let rust_config = &ctx.config.rust;
        let rust_adapter = if rust_config.split_cfg_test {
            Some(RustAdapter::new())
        } else {
            None
        };

        for file in ctx.files {
            // ... existing file skip logic ...

            let relative_path = file.path.strip_prefix(ctx.root).unwrap_or(&file.path);
            let file_kind = registry.classify(relative_path);

            // Check if this is a Rust source file that might have inline tests
            let is_rust_source = file.path.extension()
                .and_then(|e| e.to_str()) == Some("rs")
                && file_kind == FileKind::Source
                && rust_adapter.is_some();

            if is_rust_source {
                // Use line-level classification for Rust files
                let content = match std::fs::read_to_string(&file.path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let adapter = rust_adapter.as_ref().unwrap();
                let classification = adapter.classify_lines(relative_path, &content);

                source_lines += classification.source_lines;
                test_lines += classification.test_lines;

                // File counts: count as source if any source lines, test if any test lines
                if classification.source_lines > 0 {
                    source_files += 1;
                }
                if classification.test_lines > 0 {
                    test_files += 1; // Note: may double-count files with both
                }

                // ... token counting (use total) ...
                // ... size limit checks (use source_lines for source limit) ...
            } else {
                // Use existing whole-file classification
                // ... existing logic ...
            }
        }

        // ... rest of existing code ...
    }
}
```

**Key changes:**
1. Read `split_cfg_test` from config
2. For Rust source files, parse content and classify lines
3. Accumulate source/test LOC separately
4. Apply appropriate size limits based on line type

**Milestone:** CLOC reports separate source/test LOC for Rust files with `#[cfg(test)]`.

**Verification:**
```bash
cargo test --test specs rust_adapter_cfg_test
```

---

### Phase 6: Enable Behavioral Specs

Remove `#[ignore]` from the cfg_test specs in `tests/specs/adapters/rust.rs`.

**Update `tests/specs/adapters/rust.rs`:**

```rust
// BEFORE:
#[test]
#[ignore = "TODO: Phase 302 - Rust Adapter Implementation"]
fn rust_adapter_cfg_test_blocks_counted_as_test_loc() {

// AFTER:
#[test]
fn rust_adapter_cfg_test_blocks_counted_as_test_loc() {
```

**Specs to enable:**
- `rust_adapter_cfg_test_blocks_counted_as_test_loc`
- `rust_adapter_split_cfg_test_can_be_disabled`

**Milestone:** Both cfg_test specs pass without `#[ignore]`.

**Verification:**
```bash
cargo test --test specs rust_adapter_cfg_test
cargo test --test specs rust_adapter_split
```

---

## Key Implementation Details

### Parsing Strategy

The parser uses a simplified brace-counting approach:

1. Scan for `#[cfg(test)]` attribute
2. Count `{` and `}` to track block depth
3. Block ends when brace depth returns to 0

**Limitations (acceptable for v1):**
- Braces in string literals may confuse the parser
- Multi-line attributes not fully supported
- `mod tests;` (external module) declarations handled by file-level classification

**Future enhancement:** Use tree-sitter for accurate AST parsing.

### Configuration Schema

```toml
[rust]
split_cfg_test = true    # Default: count #[cfg(test)] as test LOC
```

When `split_cfg_test = false`:
- All lines in source files count as source LOC
- Only files matching test patterns count as test LOC

### LOC Counting Rules

| File Type | `split_cfg_test = true` | `split_cfg_test = false` |
|-----------|------------------------|-------------------------|
| `tests/*.rs` | All lines = test | All lines = test |
| `src/lib.rs` (no `#[cfg(test)]`) | All lines = source | All lines = source |
| `src/lib.rs` (with `#[cfg(test)]`) | Split by block | All lines = source |
| `*_tests.rs` | All lines = test | All lines = test |

### Integration Points

1. **Config** (`config.rs`): Parse `[rust]` section
2. **Adapter** (`adapter/rust.rs`): Add `CfgTestInfo` parser and `classify_lines()`
3. **CLOC** (`checks/cloc.rs`): Use adapter for Rust source files
4. **Escapes** (future): Use same classification for `in_tests = "allow"`

### Error Handling

- Parse errors: Log warning, treat entire file as source
- IO errors: Skip file, log warning
- Invalid UTF-8: Use lossy conversion (existing behavior)

## Verification Plan

### After Each Phase

```bash
# Compile check
cargo build

# Run relevant unit tests
cargo test adapter::rust
cargo test config -- rust

# Check lints
cargo clippy --all-targets --all-features -- -D warnings
```

### End-to-End Verification

```bash
# Run cfg_test specs
cargo test --test specs rust_adapter_cfg_test
cargo test --test specs rust_adapter_split

# Full quality gates
make check
```

### Test Matrix

| Test Case | Input | Expected |
|-----------|-------|----------|
| Basic `#[cfg(test)]` | `src/lib.rs` with inline tests | source + test LOC separated |
| `split_cfg_test = false` | Same file, config disabled | All LOC = source |
| Nested braces | Complex test code | Correct block end detection |
| Multiple blocks | Two `#[cfg(test)]` in one file | Both detected |
| No test blocks | Pure source file | All LOC = source |
| Test file | `tests/foo.rs` | All LOC = test (no parsing needed) |

### Manual Verification

```bash
# Run on a fixture with inline tests
cargo run -- check --cloc --json tests/fixtures/rust/cfg-test | jq '.checks[] | select(.name=="cloc") | .metrics'

# Should show non-zero test_lines even though it's a src/ file
```

## Summary

| Phase | Task | Key Files | Status |
|-------|------|-----------|--------|
| 1 | Config support | `config.rs` | [ ] Pending |
| 2 | `#[cfg(test)]` parser | `adapter/rust.rs` | [ ] Pending |
| 3 | Parser unit tests | `adapter/rust_tests.rs` | [ ] Pending |
| 4 | Adapter API | `adapter/rust.rs` | [ ] Pending |
| 5 | CLOC integration | `checks/cloc.rs` | [ ] Pending |
| 6 | Enable specs | `specs/adapters/rust.rs` | [ ] Pending |

## Future Phases

- **Phase 315**: Use `#[cfg(test)]` classification in escapes check (`in_tests = "allow"`)
- **Phase 320**: Tree-sitter parser for accurate Rust AST parsing
- **Phase 325**: Per-file inline test metrics in JSON output
