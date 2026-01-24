# Checkpoint 8E: Performance - Tests Correlation

**Root Feature:** `quench-03b8`

## Overview

Optimize the tests correlation check based on bottlenecks identified in checkpoint 8D benchmarks. While all performance targets are currently met, this checkpoint implements proactive optimizations to improve scalability for large codebases (1000+ files) and reduce latency for common operations.

**Key optimizations:**
1. Hash-based test lookup (O(1) vs O(n) linear search)
2. Cached GlobSet compilation (avoid rebuilding per-call)
3. Early termination in correlation analysis
4. Parallel file classification using rayon

## Project Structure

```
quench/
├── crates/cli/
│   └── src/checks/tests/
│       ├── correlation.rs        # MODIFY: Add optimizations
│       └── correlation_tests.rs  # MODIFY: Add optimization tests
├── crates/cli/
│   └── benches/
│       └── tests.rs              # MODIFY: Add comparison benchmarks
└── reports/
    └── checkpoint-8e-perf.md     # NEW: Performance improvement analysis
```

## Dependencies

No new dependencies required. Existing dependencies:
- `globset` - Already used for pattern matching
- `rayon` - Already available in workspace (used by `ignore` crate)
- `criterion` - Already configured for benchmarks

## Implementation Phases

### Phase 1: Hash-Based Test Lookup

Replace linear search in `has_correlated_test()` with HashSet lookups.

**Current bottleneck (from 8D benchmarks):**
- Linear search: 74µs for misses with 100 test files
- Scales poorly as test file count grows

**File:** `crates/cli/src/checks/tests/correlation.rs`

```rust
use std::collections::HashSet;

/// Pre-computed test correlation index for O(1) lookups.
pub struct TestIndex {
    /// Test files by base name (e.g., "parser" -> ["tests/parser_tests.rs"])
    by_base_name: HashMap<String, Vec<PathBuf>>,
    /// All test file paths for direct matching
    all_paths: HashSet<PathBuf>,
    /// Normalized base names (stripped of _test/_tests suffixes)
    base_names: HashSet<String>,
}

impl TestIndex {
    pub fn new(test_changes: &[PathBuf]) -> Self {
        let mut by_base_name: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut base_names = HashSet::new();

        for path in test_changes {
            if let Some(base) = extract_base_name(path) {
                by_base_name.entry(base.clone()).or_default().push(path.clone());
                base_names.insert(base);
            }
        }

        Self {
            by_base_name,
            all_paths: test_changes.iter().cloned().collect(),
            base_names,
        }
    }

    /// O(1) check for correlated test.
    pub fn has_test_for(&self, source_path: &Path) -> bool {
        let base_name = match source_path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => return false,
        };

        // Check direct base name match
        if self.base_names.contains(base_name) {
            return true;
        }

        // Check with common suffixes/prefixes
        self.base_names.contains(&format!("{}_test", base_name))
            || self.base_names.contains(&format!("{}_tests", base_name))
            || self.base_names.contains(&format!("test_{}", base_name))
    }
}
```

**Verification:**
- Run `cargo test --lib -- correlation` - all tests pass
- Run `cargo bench --bench tests -- has_correlated_test` - should show improvement

### Phase 2: GlobSet Caching

Cache compiled GlobSets to avoid rebuilding on every `analyze_correlation()` call.

**Current issue:** `build_glob_set()` is called 3 times per `analyze_correlation()` invocation, compiling the same patterns repeatedly.

**File:** `crates/cli/src/checks/tests/correlation.rs`

```rust
use std::sync::OnceLock;

/// Cached GlobSets for common pattern configurations.
struct CompiledPatterns {
    test_patterns: GlobSet,
    source_patterns: GlobSet,
    exclude_patterns: GlobSet,
}

impl CompiledPatterns {
    fn from_config(config: &CorrelationConfig) -> Result<Self, String> {
        Ok(Self {
            test_patterns: build_glob_set(&config.test_patterns)?,
            source_patterns: build_glob_set(&config.source_patterns)?,
            exclude_patterns: build_glob_set(&config.exclude_patterns)?,
        })
    }
}

/// Get cached patterns for the default configuration.
fn default_patterns() -> &'static CompiledPatterns {
    static PATTERNS: OnceLock<CompiledPatterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        CompiledPatterns::from_config(&CorrelationConfig::default())
            .expect("default patterns should compile")
    })
}
```

Update `analyze_correlation()` to use cached patterns when config matches default:

```rust
pub fn analyze_correlation(
    changes: &[FileChange],
    config: &CorrelationConfig,
    root: &Path,
) -> CorrelationResult {
    // Use cached patterns for default config, otherwise compile
    let patterns = if config == &CorrelationConfig::default() {
        Cow::Borrowed(default_patterns())
    } else {
        Cow::Owned(CompiledPatterns::from_config(config)
            .unwrap_or_else(|_| CompiledPatterns::empty()))
    };

    // ... rest of function uses patterns.test_patterns, etc.
}
```

**Verification:**
- Run `cargo test --lib -- correlation` - all tests pass
- Run `cargo bench --bench tests -- analyze_correlation` - reduced variance

### Phase 3: Early Termination

Add early termination paths for common scenarios to skip unnecessary work.

**Optimizations:**

1. **Empty changes fast path:**
```rust
pub fn analyze_correlation(...) -> CorrelationResult {
    if changes.is_empty() {
        return CorrelationResult::empty();
    }
    // ...
}
```

2. **Source-only or test-only fast paths:**
```rust
// After classification, if no source changes, skip test correlation
if source_changes.is_empty() {
    return CorrelationResult {
        with_tests: vec![],
        without_tests: vec![],
        test_only: test_changes,
    };
}
```

3. **Single file optimization:**
```rust
// For single source file, inline the lookup instead of building full index
if source_changes.len() == 1 {
    return analyze_single_source(&source_changes[0], &test_changes, ...);
}
```

**Verification:**
- Add unit tests for early termination paths
- Run benchmarks with empty/small inputs to verify fast paths are hit

### Phase 4: Parallel File Classification

Use rayon for parallel classification when processing large change sets.

**File:** `crates/cli/src/checks/tests/correlation.rs`

```rust
use rayon::prelude::*;

const PARALLEL_THRESHOLD: usize = 50;

pub fn analyze_correlation(...) -> CorrelationResult {
    // ... pattern setup ...

    // Classify changes (parallel for large sets)
    let (source_changes, test_changes): (Vec<_>, Vec<_>) = if changes.len() >= PARALLEL_THRESHOLD {
        changes.par_iter()
            .filter(|c| c.change_type != ChangeType::Deleted)
            .partition_map(|change| {
                let rel_path = change.path.strip_prefix(root).unwrap_or(&change.path);
                if patterns.test_patterns.is_match(rel_path) {
                    Either::Right(rel_path.to_path_buf())
                } else if patterns.source_patterns.is_match(rel_path)
                    && !patterns.exclude_patterns.is_match(rel_path)
                {
                    Either::Left(change)
                } else {
                    // Skip non-matching files (filtered out)
                }
            })
    } else {
        // Sequential for small sets (avoid rayon overhead)
        classify_sequential(changes, &patterns, root)
    };

    // ...
}
```

**Verification:**
- Run `cargo bench --bench tests -- detect/large` - should show improvement
- Verify no regressions on small fixtures

### Phase 5: Benchmark Comparison

Run comparative benchmarks and document improvements.

**File:** `crates/cli/benches/tests.rs`

Add comparison group:

```rust
fn bench_optimization_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("tests-optimization");

    // Test index creation vs linear scan
    let test_files: Vec<PathBuf> = (0..100)
        .map(|i| PathBuf::from(format!("tests/module{}_tests.rs", i)))
        .collect();

    group.bench_function("index_creation", |b| {
        b.iter(|| TestIndex::new(&test_files))
    });

    group.bench_function("index_lookup_hit", |b| {
        let index = TestIndex::new(&test_files);
        let source = Path::new("src/module50.rs");
        b.iter(|| index.has_test_for(source))
    });

    group.bench_function("index_lookup_miss", |b| {
        let index = TestIndex::new(&test_files);
        let source = Path::new("src/nonexistent.rs");
        b.iter(|| index.has_test_for(source))
    });

    group.finish();
}
```

**Verification:**
- Run full benchmark suite: `cargo bench --bench tests`
- Compare results against 8D baseline

### Phase 6: Documentation and Report

Create performance improvement report and update documentation.

**File:** `reports/checkpoint-8e-perf.md`

```markdown
# Performance Improvements: Tests Correlation

## Summary

| Optimization | Before | After | Improvement |
|--------------|--------|-------|-------------|
| has_correlated_test (miss, 100 files) | 74 µs | ~100 ns | 740x |
| GlobSet compilation (per call) | ~50 µs | 0 (cached) | ∞ |
| analyze_correlation (empty) | ~1 µs | ~10 ns | 100x |

## Detailed Analysis

### Hash-Based Test Lookup
...

### GlobSet Caching
...
```

**Verification:**
- Report includes before/after metrics
- All benchmarks still pass performance targets

## Key Implementation Details

### TestIndex Design

The `TestIndex` struct pre-computes test file relationships:
- Build once per `analyze_correlation()` call
- O(n) construction, O(1) lookups
- Memory: O(n) where n = number of test files

### GlobSet Caching Strategy

Uses `OnceLock` for thread-safe lazy initialization:
- Default patterns cached statically
- Custom patterns compiled per-call (rare case)
- No cache invalidation needed (patterns are immutable)

### Parallel Processing Threshold

The `PARALLEL_THRESHOLD` of 50 files balances:
- Rayon thread pool overhead (~1-10µs)
- Per-file classification cost (~100ns)
- Memory locality benefits of sequential iteration

### Backward Compatibility

All optimizations are internal implementation changes:
- Public API unchanged
- Same behavior, lower latency
- No configuration changes required

## Verification Plan

1. **Phase 1:** Run `cargo test --lib -- correlation` - all tests pass
2. **Phase 2:** Run `cargo bench --bench tests -- analyze_correlation` - reduced variance
3. **Phase 3:** Add tests for early termination, verify with `cargo test`
4. **Phase 4:** Run `cargo bench --bench tests -- detect/large` - improved throughput
5. **Phase 5:** Full benchmark comparison: `cargo bench --bench tests`
6. **Phase 6:** Review `reports/checkpoint-8e-perf.md` for completeness

**Final verification:**
```bash
make check                    # All checks pass
cargo bench --bench tests     # All benchmarks complete
# Compare against 8D baseline - improvements documented
```
