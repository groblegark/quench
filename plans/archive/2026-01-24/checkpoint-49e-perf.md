# Checkpoint 49E: JavaScript Adapter Performance Optimizations

**Root Feature:** `quench-68fa`

## Overview

Performance optimization checkpoint targeting the JavaScript adapter based on findings from checkpoint-49d-benchmark. While all performance targets are currently met with 8x margin (12ms vs 100ms target), this checkpoint applies small to medium optimizations to improve efficiency and establish patterns for future adapters.

Focus areas:
1. **Pattern matching efficiency** - Reduce GlobSet overhead through fast-path checks
2. **File walking optimization** - Faster node_modules exclusion during traversal

These optimizations provide:
- Faster classification for common paths (node_modules, common extensions)
- Reduced GlobSet matching overhead for typical file patterns
- Foundation for scaling to larger codebases

## Project Structure

```
quench/
├── crates/cli/
│   ├── benches/
│   │   └── javascript.rs            # Existing JS benchmarks (add new cases)
│   └── src/
│       ├── adapter/
│       │   ├── javascript/
│       │   │   └── mod.rs           # Adapter with pattern matching (modify)
│       │   └── glob.rs              # Glob utilities (extend)
│       └── walker.rs                # File walker (modify)
├── tests/specs/adapters/
│   └── javascript.rs                # Adapter tests (verify unchanged behavior)
└── reports/
    └── checkpoint-49-javascript-adapter.md  # Update with new benchmarks
```

## Dependencies

**Existing crates (no additions needed):**
- `globset` - GlobSet pattern matching
- `ignore` - File walking with gitignore support
- `criterion` - Benchmarking

## Implementation Phases

### Phase 1: Fast-Path Extension Check for Source Files

**Goal:** Skip GlobSet matching for common source file extensions.

**Rationale:** The 6 source patterns (`**/*.js`, `**/*.jsx`, etc.) can be replaced with a simple extension check for classification. GlobSet is still needed for complex test patterns, but source files are the majority of matches.

**Implementation:**

In `crates/cli/src/adapter/javascript/mod.rs`:

```rust
impl JavaScriptAdapter {
    /// Fast extension check for source files.
    /// Returns Some(true) for JS/TS extensions, None if GlobSet needed.
    #[inline]
    fn is_js_extension(path: &Path) -> Option<bool> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "mts"))
    }
}

impl Adapter for JavaScriptAdapter {
    fn classify(&self, path: &Path) -> FileKind {
        // Fast path: check ignore prefix first (node_modules, dist, etc.)
        if self.should_ignore(path) {
            return FileKind::Other;
        }

        // Test patterns must be checked before source (more specific)
        if self.test_patterns.is_match(path) {
            return FileKind::Test;
        }

        // Fast path: extension check instead of GlobSet for source
        if Self::is_js_extension(path).unwrap_or(false) {
            return FileKind::Source;
        }

        FileKind::Other
    }
}
```

**Benchmark target:** classify() for source files should improve from ~0.16µs to ~0.05µs per file.

**Verification:**
```bash
cargo test --test specs javascript
cargo bench --bench javascript -- "js_1k_source"
```

**Milestone:** Extension-based fast path reduces source file classification time.

**Status:** [ ] Pending

---

### Phase 2: Fast-Path Ignore Prefix Check

**Goal:** Check common ignore prefixes (node_modules, dist) before GlobSet matching.

**Rationale:** The `should_ignore()` currently uses GlobSet for 5 patterns. A prefix check for the most common case (node_modules) is faster than GlobSet matching.

**Implementation:**

In `crates/cli/src/adapter/javascript/mod.rs`:

```rust
/// Common ignore prefixes to check before GlobSet.
/// Order: most common first for early exit.
const IGNORE_PREFIXES: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    ".next",
    "coverage",
];

impl JavaScriptAdapter {
    /// Check if path should be ignored using fast prefix check first.
    pub fn should_ignore(&self, path: &Path) -> bool {
        // Fast path: check common prefixes
        if let Some(first_component) = path.components().next() {
            if let std::path::Component::Normal(name) = first_component {
                if let Some(name_str) = name.to_str() {
                    for prefix in IGNORE_PREFIXES {
                        if name_str == *prefix {
                            return true;
                        }
                    }
                }
            }
        }

        // Fallback: GlobSet for edge cases (patterns in subdirectories)
        self.ignore_patterns.is_match(path)
    }
}
```

**Benchmark target:** node_modules classification should improve from ~0.03µs to ~0.01µs per file.

**Verification:**
```bash
cargo test --test specs javascript
cargo bench --bench javascript -- "js_1k_node_modules"
```

**Milestone:** Prefix-based fast path reduces ignore check time for common directories.

**Status:** [ ] Pending

---

### Phase 3: Walker-Level Node_modules Filtering

**Goal:** Filter node_modules at the walker level to avoid sending paths to adapter.

**Rationale:** The walker already uses the `ignore` crate's override system, but adding explicit skip logic for common directories during traversal is more efficient than filtering after discovery.

**Current flow:**
1. Walker discovers file in node_modules
2. Walker sends to channel
3. Adapter classifies as Other (ignored)

**Optimized flow:**
1. Walker checks if entering node_modules directory
2. Walker skips entire subtree

**Implementation:**

In `crates/cli/src/walker.rs`, add custom filter function:

```rust
/// Directories to skip entirely during walking.
/// These are filtered during traversal, not after discovery.
pub const SKIP_DIRECTORIES: &[&str] = &[
    "node_modules",
    ".git",
];

impl FileWalker {
    /// Check if a directory entry should be skipped entirely.
    #[inline]
    fn should_skip_dir(entry: &ignore::DirEntry) -> bool {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            return false;
        }

        entry.file_name()
            .to_str()
            .map(|name| SKIP_DIRECTORIES.contains(&name))
            .unwrap_or(false)
    }
}
```

Then in `walk_parallel` and `walk_sequential`, add skip logic:

```rust
// In parallel walker closure:
Box::new(move |entry| match entry {
    Ok(entry) => {
        // Skip configured directories entirely
        if Self::should_skip_dir(&entry) {
            return WalkState::Skip;  // Skip entire subtree
        }
        // ... rest of logic
    }
    // ...
})
```

**Note:** This requires modifying the walker to accept a filter function or configuration for skip directories.

**Benchmark target:** End-to-end time on projects with node_modules should show improvement.

**Verification:**
```bash
cargo test --test specs
cargo build --release
hyperfine './target/release/quench check tests/fixtures/js-simple'
```

**Milestone:** Walker skips ignored directories at traversal time, not after discovery.

**Status:** [ ] Pending

---

### Phase 4: Benchmark and Document Improvements

**Goal:** Run comparative benchmarks and document optimization impact.

**Benchmarks to run:**

```bash
# Build optimized binary
cargo build --release

# Micro-benchmarks (before/after comparison)
cargo bench --bench javascript -- --save-baseline pre-opt
# Apply optimizations
cargo bench --bench javascript -- --baseline pre-opt

# End-to-end benchmarks
hyperfine --warmup 2 --runs 20 \
    './target/release/quench check tests/fixtures/js-simple' \
    './target/release/quench check tests/fixtures/js-monorepo' \
    --export-markdown reports/bench-49e-e2e.md
```

**Add benchmark cases to `crates/cli/benches/javascript.rs`:**

```rust
/// Benchmark fast-path extension checking.
fn bench_js_extension_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_fast_path");

    let paths: Vec<PathBuf> = ["tsx", "ts", "js", "jsx", "mjs", "mts", "css", "json", "md"]
        .iter()
        .cycle()
        .take(1000)
        .enumerate()
        .map(|(i, ext)| PathBuf::from(format!("src/file_{}.{}", i, ext)))
        .collect();

    group.bench_function("extension_check_1k_mixed", |b| {
        b.iter(|| {
            for path in &paths {
                black_box(JavaScriptAdapter::is_js_extension(path));
            }
        })
    });

    group.finish();
}

/// Benchmark ignore prefix check.
fn bench_js_ignore_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_ignore_fast_path");

    let paths: Vec<PathBuf> = [
        "node_modules/pkg/index.js",
        "dist/bundle.js",
        "src/app.ts",
        "build/output.js",
        ".next/cache/data.json",
        "coverage/lcov.info",
    ].iter()
        .cycle()
        .take(1000)
        .map(PathBuf::from)
        .collect();

    group.bench_function("prefix_check_1k_mixed", |b| {
        b.iter(|| {
            for path in &paths {
                // Test the fast path function directly
                black_box(has_ignore_prefix(path));
            }
        })
    });

    group.finish();
}
```

**Update report at `reports/checkpoint-49-javascript-adapter.md`:**

Add section:
```markdown
## Optimization Results (Checkpoint 49E)

### Summary

| Optimization | Before | After | Improvement |
|--------------|--------|-------|-------------|
| Source file classify | 0.16µs | TBDµs | TBD% |
| node_modules classify | 0.03µs | TBDµs | TBD% |
| End-to-end js-simple | 12.3ms | TBDms | TBD% |

### Phase Details

[Document each optimization's impact]
```

**Verification:**
```bash
grep "Optimization Results" reports/checkpoint-49-javascript-adapter.md
```

**Milestone:** Performance improvements documented with before/after comparisons.

**Status:** [ ] Pending

## Key Implementation Details

### Fast-Path vs GlobSet Trade-offs

| Approach | Time | Flexibility | Use Case |
|----------|------|-------------|----------|
| Extension check | ~5ns | Limited | Source files with known extensions |
| Prefix check | ~10ns | Limited | First-component ignore patterns |
| GlobSet match | ~100ns | Full | Complex patterns, wildcards |

The optimization strategy is to use fast paths for common cases while falling back to GlobSet for edge cases.

### Walker Skip vs Adapter Filter

| Level | Mechanism | Effect |
|-------|-----------|--------|
| Walker skip | `WalkState::Skip` | Entire subtree skipped, no I/O |
| Walker override | `!node_modules` | Pattern-based filtering during walk |
| Adapter filter | `should_ignore()` | Post-discovery classification |

The walker-level skip is most efficient because it prevents I/O operations on ignored subtrees.

### Backward Compatibility

All optimizations maintain identical behavior:
- Same files classified as Source/Test/Other
- Same ignore patterns applied
- Same workspace detection
- Tests must pass unchanged

### Memory Impact

These optimizations:
- Add ~200 bytes for static prefix arrays
- Remove potential for allocations in hot path
- No new heap allocations per file

## Verification Plan

1. **Behavioral tests unchanged:**
   ```bash
   cargo test --test specs javascript
   ```

2. **Micro-benchmarks show improvement:**
   ```bash
   cargo bench --bench javascript
   ```

3. **End-to-end performance:**
   ```bash
   cargo build --release
   hyperfine './target/release/quench check tests/fixtures/js-simple'
   hyperfine './target/release/quench check tests/fixtures/js-monorepo'
   ```

4. **Quality gates pass:**
   ```bash
   make check
   ```

5. **Report updated:**
   ```bash
   grep "49E" reports/checkpoint-49-javascript-adapter.md
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Fast-path extension check for source files | [ ] Pending |
| 2 | Fast-path ignore prefix check | [ ] Pending |
| 3 | Walker-level node_modules filtering | [ ] Pending |
| 4 | Benchmark and document improvements | [ ] Pending |

## Notes

- Current performance (8x margin) means these optimizations are refinements, not critical fixes
- The patterns established here apply to future language adapters
- Walker-level filtering benefits all adapters, not just JavaScript
- Consider making fast-path patterns configurable per-adapter in future
