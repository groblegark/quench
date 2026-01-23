# Checkpoint 2E: Performance Fixes - CLOC Works

**Root Feature:** `quench-2de9`

## Overview

Performance optimization checkpoint to address bottlenecks identified in checkpoint-2d. The benchmark report shows all targets are exceeded (15x faster than cold target, 8x faster than warm target), but reveals one clear optimization opportunity: the CLOC check reads each file twice (once for line counting, once for token counting).

**Current state (checkpoint-2d):**

| Metric | Target | Actual | Margin |
|--------|--------|--------|--------|
| Cold (bench-medium) | < 500ms | 33ms | 15x faster |
| Warm (bench-medium) | < 100ms | 12ms | 8x faster |
| Cache speedup | > 5x | 7.4x | Exceeded |

**Bottleneck identified:** File I/O dominates (76% of cold runtime). The `run()` method calls both `count_nonblank_lines()` and `count_tokens()` for each file, each reading the file independently.

## Project Structure

```
quench/
├── crates/cli/
│   ├── benches/
│   │   └── check.rs              # End-to-end check benchmarks
│   └── src/
│       └── checks/
│           ├── cloc.rs           # CLOC implementation (optimization target)
│           └── cloc_tests.rs     # Unit tests
├── reports/
│   └── checkpoint-2-benchmarks.md  # Benchmark results from 2d
└── scripts/
    └── gen-bench-fixture         # Fixture generation script
```

## Dependencies

No new dependencies required. Uses existing:
- `criterion` - Benchmarking framework
- `hyperfine` - CLI benchmarking (installed)

## Implementation Phases

### Phase 1: Consolidate File Reading

Eliminate double file reads by combining `count_nonblank_lines` and `count_tokens` into a single function that computes both metrics from one read.

**Current code** (`crates/cli/src/checks/cloc.rs:56-60`):
```rust
match count_nonblank_lines(&file.path) {
    Ok(line_count) => {
        let is_test = matcher.is_test_file(&file.path, ctx.root);
        let is_excluded = matcher.is_excluded(&file.path, ctx.root);
        let token_count = count_tokens(&file.path).unwrap_or(0);
```

**Optimized code:**
```rust
/// Metrics computed from a single file read.
struct FileMetrics {
    nonblank_lines: usize,
    tokens: usize,
}

/// Count non-blank lines and tokens from a single file read.
fn count_file_metrics(path: &Path) -> std::io::Result<FileMetrics> {
    let content = std::fs::read(path)?;
    let text = String::from_utf8(content)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());

    let nonblank_lines = text.lines().filter(|l| !l.trim().is_empty()).count();
    let tokens = text.chars().count() / 4;

    Ok(FileMetrics { nonblank_lines, tokens })
}
```

**Update call site:**
```rust
match count_file_metrics(&file.path) {
    Ok(metrics) => {
        let line_count = metrics.nonblank_lines;
        let token_count = metrics.tokens;
        // ... rest unchanged
```

**Milestone:** Single file read per file in CLOC check.

**Verification:**
```bash
cargo test -p quench -- cloc
cargo bench --bench check -- "bench-medium"
```

---

### Phase 2: Re-run Benchmarks

After the optimization, re-run the same benchmark suite from checkpoint-2d to measure the improvement.

**Benchmark commands:**
```bash
# Build release
cargo build --release

# Clear cache for cold run
rm -rf tests/fixtures/bench-medium/.quench

# Cold run timing
hyperfine --warmup 0 --runs 10 \
    './target/release/quench check tests/fixtures/bench-medium' \
    --export-json /tmp/cold-2e.json

# Warm run timing
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/bench-medium' \
    --export-json /tmp/warm-2e.json

# All fixtures with criterion
cargo bench --bench check
```

**Expected improvement:**
- Cold runs: ~20-30% faster (eliminating ~25ms of redundant I/O)
- Warm runs: Minimal change (cache hit skips file reading)

**Milestone:** New benchmark numbers captured.

---

### Phase 3: Update Benchmark Report

Update `reports/checkpoint-2-benchmarks.md` with Phase 2 results, comparing before/after optimization.

**Add section:**
```markdown
## Checkpoint 2E: Post-Optimization Results

| Metric | Pre-2E | Post-2E | Improvement |
|--------|--------|---------|-------------|
| Cold (bench-medium) | 33ms | XXms | XX% |
| Warm (bench-medium) | 12ms | XXms | XX% |
```

**Milestone:** Report updated with comparative results.

---

### Phase 4: Verify All Targets Still Met

Run the complete verification suite to ensure the optimization didn't break anything.

```bash
# Full test suite
make check

# Benchmark all fixtures
cargo bench --bench check
cargo bench --bench file_walking
```

**Verification checklist:**
- [ ] All unit tests pass
- [ ] All spec tests pass
- [ ] Cold < 500ms (target)
- [ ] Warm < 100ms (target)
- [ ] Clippy clean
- [ ] No new warnings

**Milestone:** All quality gates pass.

## Key Implementation Details

### Why This Matters

From `docs/specs/20-performance.md`:

> **Key insight:** File discovery and reading are often the bottleneck, not pattern matching.

The checkpoint-2d profiling confirmed this:
- File reading: 76% of cold runtime (~25ms)
- Pattern matching: 6% (~2ms)
- Aggregation: 9% (~3ms)

Reading each file twice doubles the dominant cost. Consolidating reads is the highest-impact single change.

### Memory Considerations

The optimization does not increase memory usage:
- Before: Two separate `String` allocations per file (sequential, not concurrent)
- After: One `String` allocation per file

### Caching Interaction

The cache stores violations per file, not file contents. The optimization:
- Does NOT invalidate existing caches
- Does NOT change the cache key (still mtime+size)
- Only affects cold runs (cache misses)

Warm runs see minimal improvement since they skip file reading entirely on cache hits.

### Alternative Considered: memmap

Memory-mapped I/O (`memmap2`) was considered per the performance spec but not implemented because:
1. Current benchmarks are 15x under target
2. Complexity not justified by marginal gains
3. The spec recommends memmap for files >64KB; most source files are smaller

This remains in the P2 backlog if future profiling shows benefit.

## Verification Plan

1. **Unit tests:**
   ```bash
   cargo test -p quench -- cloc
   ```

2. **Spec tests:**
   ```bash
   cargo test --test '*' -- cloc
   ```

3. **Benchmarks (before/after):**
   ```bash
   # Before (record baseline)
   cargo bench --bench check -- "bench-medium" --save-baseline before-2e

   # Apply optimization...

   # After (compare)
   cargo bench --bench check -- "bench-medium" --baseline before-2e
   ```

4. **Quality gates:**
   ```bash
   make check
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Consolidate file reading | [ ] Pending |
| 2 | Re-run benchmarks | [ ] Pending |
| 3 | Update benchmark report | [ ] Pending |
| 4 | Verify all targets | [ ] Pending |

## Notes

- The optimization targets **cold runs only**; warm runs are cache hits that skip file I/O
- Current performance already exceeds targets by 8-15x; this optimization provides further margin
- No architectural changes required; this is a localized refactor in `cloc.rs`
