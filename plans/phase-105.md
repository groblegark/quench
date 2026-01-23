# Phase 105: CLOC Check - Basic Implementation

**Root Feature:** `quench-1532`

## Overview

Implement the core CLOC (Count Lines of Code) check to count non-blank lines, separate source from test files using configurable patterns, calculate source-to-test ratio, and output metrics in JSON format. This phase focuses on accurate line counting and pattern-based file classification.

## Project Structure

```
crates/cli/src/
├── checks/
│   ├── cloc.rs          # Main implementation (modify)
│   └── cloc_tests.rs    # Unit tests (expand)
├── config.rs            # Config parsing (modify for patterns + exclude)
tests/
├── fixtures/cloc/       # Test fixtures (already exist)
└── specs/cloc.rs        # Behavioral specs (remove #[ignore])
```

## Dependencies

No new external dependencies. Uses existing:
- `ignore` crate (via walker) - gitignore-style pattern matching
- `globset` crate - for source/test pattern matching
- `serde_json` - metrics output

## Implementation Phases

### Phase 1: Non-Blank Line Counting

**Goal**: Count only lines containing non-whitespace characters.

**Current State**: `count_lines()` counts all lines including blank lines.

**Changes**:

1. Modify `count_lines()` in `crates/cli/src/checks/cloc.rs`:

```rust
/// Count non-blank lines in a file.
/// A line is counted if it contains at least one non-whitespace character.
fn count_nonblank_lines(path: &Path) -> std::io::Result<usize> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let count = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .count();
    Ok(count)
}
```

2. Add unit tests for edge cases:
   - Empty file → 0 lines
   - File with only whitespace → 0 lines
   - File with mixed content and blank lines
   - File with trailing newline
   - File without trailing newline

**Verification**: Unit tests pass in `cloc_tests.rs`.

### Phase 2: Source/Test Pattern Matching

**Goal**: Use glob patterns from config to classify files as source or test.

**Current State**: `is_test_file()` uses filename-only heuristics, missing directory patterns like `**/tests/**`.

**Changes**:

1. Add pattern fields to `ClocConfig` in `crates/cli/src/config.rs`:

```rust
pub struct ClocConfig {
    pub max_lines: usize,
    pub max_lines_test: usize,
    pub check: CheckLevel,

    /// Source file patterns (default: language-detected)
    #[serde(default)]
    pub source_patterns: Vec<String>,

    /// Test file patterns
    #[serde(default = "ClocConfig::default_test_patterns")]
    pub test_patterns: Vec<String>,

    /// Patterns to exclude from size limit checks
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl ClocConfig {
    fn default_test_patterns() -> Vec<String> {
        vec![
            "**/tests/**".to_string(),
            "**/test/**".to_string(),
            "**/*_test.*".to_string(),
            "**/*_tests.*".to_string(),
            "**/*.test.*".to_string(),
            "**/*.spec.*".to_string(),
            "**/test_*.*".to_string(),
        ]
    }
}
```

2. Create a `PatternMatcher` in `cloc.rs`:

```rust
use globset::{Glob, GlobSet, GlobSetBuilder};

struct PatternMatcher {
    test_patterns: GlobSet,
    exclude_patterns: GlobSet,
}

impl PatternMatcher {
    fn new(test_patterns: &[String], exclude_patterns: &[String]) -> Self {
        // Build GlobSets from patterns
    }

    fn is_test_file(&self, path: &Path, root: &Path) -> bool {
        let relative = path.strip_prefix(root).unwrap_or(path);
        self.test_patterns.is_match(relative)
    }

    fn is_excluded(&self, path: &Path, root: &Path) -> bool {
        let relative = path.strip_prefix(root).unwrap_or(path);
        self.exclude_patterns.is_match(relative)
    }
}
```

3. Update `ClocCheck::run()` to use pattern matcher instead of `is_test_file()`.

**Verification**: `cloc_separates_source_and_test_by_pattern` spec passes.

### Phase 3: Accurate Metrics Collection

**Goal**: Track source_lines, test_lines, source_files, test_files, and ratio.

**Current State**: Tracks line totals but missing file counts.

**Changes**:

1. Update `ClocCheck::run()` to track all metrics:

```rust
fn run(&self, ctx: &CheckContext) -> CheckResult {
    let matcher = PatternMatcher::new(
        &ctx.config.check.cloc.test_patterns,
        &ctx.config.check.cloc.exclude,
    );

    let mut source_lines: usize = 0;
    let mut source_files: usize = 0;
    let mut test_lines: usize = 0;
    let mut test_files: usize = 0;
    let mut violations = Vec::new();

    for file in ctx.files {
        if !is_text_file(&file.path) {
            continue;
        }

        let is_excluded = matcher.is_excluded(&file.path, ctx.root);
        let is_test = matcher.is_test_file(&file.path, ctx.root);

        match count_nonblank_lines(&file.path) {
            Ok(line_count) => {
                if is_test {
                    test_lines += line_count;
                    test_files += 1;
                } else {
                    source_lines += line_count;
                    source_files += 1;
                }

                // Size limit check (skip excluded files)
                if !is_excluded {
                    let max = if is_test {
                        ctx.config.check.cloc.max_lines_test
                    } else {
                        ctx.config.check.cloc.max_lines
                    };

                    if line_count > max {
                        // ... create violation
                    }
                }
            }
            Err(e) => { /* warn and skip */ }
        }
    }

    let ratio = if source_lines > 0 {
        test_lines as f64 / source_lines as f64
    } else {
        0.0
    };

    let metrics = json!({
        "source_lines": source_lines,
        "source_files": source_files,
        "test_lines": test_lines,
        "test_files": test_files,
        "ratio": (ratio * 100.0).round() / 100.0,
    });

    // ...
}
```

**Verification**:
- `cloc_counts_nonblank_lines_as_loc` passes
- `cloc_json_includes_required_metrics` passes
- `cloc_calculates_source_to_test_ratio` passes

### Phase 4: Exclude Pattern Support

**Goal**: Files matching exclude patterns should not generate violations.

**Current State**: `exclude` field exists in config but not implemented.

**Changes**:

1. Parse `exclude` patterns in config (already stubbed in Phase 2).

2. In violation generation, check exclusion:

```rust
if !is_excluded && line_count > max {
    // Generate violation
}
```

Note: Excluded files still contribute to metrics, they just don't fail size checks.

**Verification**: `cloc_excluded_patterns_dont_generate_violations` passes.

### Phase 5: Package Support (Optional)

**Goal**: Output per-package metrics when packages are configured.

**Current State**: `by_package` is stubbed but not implemented.

**Changes**:

1. Add workspace/package detection to config:

```rust
pub struct WorkspaceConfig {
    pub packages: Vec<String>,
}
```

2. In `ClocCheck::run()`, track per-package metrics:

```rust
let mut package_metrics: HashMap<String, PackageMetrics> = HashMap::new();

// Determine which package a file belongs to
fn file_package(path: &Path, packages: &[PackageConfig]) -> Option<String> {
    for pkg in packages {
        if path.starts_with(&pkg.path) {
            return Some(pkg.name.clone());
        }
    }
    None
}

// Accumulate per-package metrics
if let Some(pkg_name) = file_package(&file.path, &packages) {
    let pkg = package_metrics.entry(pkg_name).or_default();
    if is_test {
        pkg.test_lines += line_count;
        pkg.test_files += 1;
    } else {
        pkg.source_lines += line_count;
        pkg.source_files += 1;
    }
}
```

3. Include `by_package` in result when packages configured:

```rust
let result = if !package_metrics.is_empty() {
    result.with_by_package(package_metrics.into_iter().map(|(k, v)| {
        (k, json!({
            "source_lines": v.source_lines,
            "source_files": v.source_files,
            "test_lines": v.test_lines,
            "test_files": v.test_files,
            "ratio": v.ratio(),
        }))
    }).collect())
} else {
    result
};
```

**Verification**:
- `cloc_omits_by_package_when_not_configured` passes
- `cloc_includes_by_package_when_configured` passes

### Phase 6: Edge Cases & Polish

**Goal**: Handle edge cases robustly.

**Changes**:

1. **Binary file detection**: Improve `is_text_file()` to handle edge cases:
   - Files with no extension
   - Common binary extensions (images, compiled files)

2. **Encoding issues**: Handle UTF-8 errors gracefully:

```rust
fn count_nonblank_lines(path: &Path) -> std::io::Result<usize> {
    let content = std::fs::read(path)?;
    // Try UTF-8, fall back to lossy conversion
    let text = String::from_utf8(content)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());

    Ok(text.lines().filter(|l| !l.trim().is_empty()).count())
}
```

3. **Empty file**: Ensure 0 lines counted, no errors.

4. **Large files**: Ensure streaming (BufReader) handles large files without loading into memory.

**Verification**: Unit tests for edge cases pass.

## Key Implementation Details

### Line Counting Algorithm

```rust
// A line is counted if it contains non-whitespace
line.chars().any(|c| !c.is_whitespace())
// Or equivalently:
!line.trim().is_empty()
```

### Pattern Matching Strategy

1. Test patterns take precedence (a file in `tests/` with `_test.rs` suffix is counted once as test)
2. Relative paths used for matching (strip project root)
3. Case-insensitive matching on Windows

### Ratio Calculation

```rust
ratio = test_lines / source_lines
// Handle division by zero:
if source_lines == 0 { 0.0 } else { test_lines as f64 / source_lines as f64 }
// Round to 2 decimal places for JSON output:
(ratio * 100.0).round() / 100.0
```

### Performance

Target: <100ms for 50k LOC (per spec).

- Use streaming I/O (BufReader)
- Pattern compilation happens once per check run
- Parallel file walking (handled by walker)

## Verification Plan

### Unit Tests (crates/cli/src/checks/cloc_tests.rs)

Add tests for:

1. `count_nonblank_lines_empty_file` - returns 0
2. `count_nonblank_lines_whitespace_only` - returns 0
3. `count_nonblank_lines_mixed_content` - counts correctly
4. `pattern_matcher_identifies_test_directories` - `tests/foo.rs` is test
5. `pattern_matcher_identifies_test_suffixes` - `foo_test.rs` is test
6. `pattern_matcher_excludes_patterns` - `generated/foo.rs` excluded

### Behavioral Specs (tests/specs/cloc.rs)

Remove `#[ignore]` from these Phase 105 specs:

1. `cloc_counts_nonblank_lines_as_loc`
2. `cloc_does_not_count_blank_lines`
3. `cloc_separates_source_and_test_by_pattern`
4. `cloc_calculates_source_to_test_ratio`
5. `cloc_json_includes_required_metrics`
6. `cloc_json_omits_violations_when_none`
7. `cloc_violation_type_is_file_too_large`
8. `cloc_fails_on_source_file_over_max_lines`
9. `cloc_fails_on_test_file_over_max_lines_test`
10. `cloc_excluded_patterns_dont_generate_violations`
11. `cloc_omits_by_package_when_not_configured`
12. `cloc_includes_by_package_when_configured`

### Manual Verification

```bash
# Run unit tests
cargo test -p quench-cli cloc

# Run behavioral specs
cargo test --test specs cloc

# Test on real project
cargo run -- check --cloc -o json | jq '.checks[] | select(.name == "cloc")'
```

### Checklist Before Commit

- [ ] Unit tests in `cloc_tests.rs`
- [ ] `make check` passes (fmt, clippy, test, build, bootstrap, audit, deny)
- [ ] All Phase 105 specs pass (no `#[ignore]`)
- [ ] Commit message lists passing specs
