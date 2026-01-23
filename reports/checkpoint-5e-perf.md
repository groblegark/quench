# Checkpoint 5E: Performance Report - Shell Adapter Optimization

Generated: 2026-01-23
Hardware: Apple M3 Max, 36 GB RAM, macOS 26.2

## Investigation Summary

This checkpoint investigated why ShellAdapter::new() (78.3µs with 6 patterns) was 26% slower than RustAdapter::new() (57.9µs with 6 patterns) despite having equivalent pattern counts.

### Root Cause Analysis

**Finding 1: Test pattern compilation dominates creation time**

| Pattern Set | Shell | Rust | Difference |
|-------------|-------|------|------------|
| Test patterns | 77.9µs | 48.2µs | +62% slower |
| Source patterns | 0.83µs | 0.39µs | +113% slower |
| Ignore patterns | N/A | 9.5µs | N/A |

The test patterns account for ~99% of Shell adapter creation time (77.9µs of 78.6µs).

**Finding 2: Extension complexity affects pattern compilation**

| Pattern | Time |
|---------|------|
| `**/*.rs` | 387ns |
| `**/*.sh` | 391ns |
| `**/*.bash` | 438ns (+13%) |
| `**/*.bats` | 438ns (+13%) |

Longer extensions (`.bash`, `.bats`) compile ~13% slower than shorter ones (`.rs`, `.sh`).

**Finding 3: Rust uses simpler test patterns**

- Rust test patterns: `tests/**` (directory prefix only)
- Shell test patterns: `tests/**/*.bats` (directory prefix + extension filter)

The extension filter in Shell patterns adds significant compilation overhead.

**Finding 4: Redundant pattern identified**

The pattern `*_test.sh` was redundant with `**/*_test.sh` because `**/` matches zero or more path components, including zero (root level).

### Optimization Applied

Removed redundant `*_test.sh` pattern from ShellAdapter test patterns.

**Before:** 4 test patterns (`tests/**/*.bats`, `test/**/*.bats`, `*_test.sh`, `**/*_test.sh`)
**After:** 3 test patterns (`tests/**/*.bats`, `test/**/*.bats`, `**/*_test.sh`)

## Results

| Metric | Before (5D) | After (5E) | Change |
|--------|-------------|------------|--------|
| ShellAdapter::new() | 78.6µs | 77.7µs | -1.1% |
| shell_test_patterns | 77.9µs | 76.8µs | -1.4% |
| classify() per 1K files | 43µs | 34-41µs | -5 to -21% |
| bench-shell cold | 14.1ms | ~14ms | No change |

The optimization provides a modest 1.1% improvement in adapter creation time.

## Detailed Benchmark Results

### GlobSet Pattern Compilation

| Benchmark | Time |
|-----------|------|
| shell_source_patterns (2 patterns) | 826ns |
| shell_test_patterns (3 patterns, optimized) | 76.8µs |
| rust_source_pattern (1 pattern) | 386ns |
| rust_test_patterns (4 patterns) | 48.2µs |
| rust_ignore_pattern (1 pattern) | 9.5µs |
| shell_combined_single_build (5 patterns) | 77.3µs |
| rust_combined_single_build (6 patterns) | 50.6µs |

### Single Pattern Analysis

| Pattern | Time |
|---------|------|
| `**/*.sh` | 391ns |
| `**/*.rs` | 387ns |
| `**/*.bash` | 438ns |
| `**/*.bats` | 438ns |

### Classification Performance

| Benchmark | Time | Per-file |
|-----------|------|----------|
| shell_1k_source_scripts | 34.3µs | 0.034µs |
| shell_1k_bash_libs | 41.0µs | 0.041µs |
| shell_1k_bats_tests | 41.3µs | 0.041µs |
| shell_1k_bin_scripts | 40.7µs | 0.041µs |

Classification performance improved slightly (34-41µs vs 43µs baseline), likely due to reduced pattern count.

## Conclusions

1. **Root cause understood**: Shell adapter creation is slower due to:
   - Test patterns with extension filters (`tests/**/*.bats`) being more expensive than simple directory patterns (`tests/**`)
   - Longer extensions (`.bash`, `.bats`) adding ~13% overhead per pattern

2. **Optimization applied**: Removed redundant `*_test.sh` pattern for 1.1% improvement

3. **No further optimization warranted**:
   - The 78µs creation time is 0.6% of total check time (~12ms)
   - It's a one-time cost per check invocation
   - More aggressive optimization would add complexity for diminishing returns

4. **Design guidance for future adapters**:
   - Prefer simple directory patterns (`tests/**`) over extension-filtered patterns (`tests/**/*.ext`)
   - Shorter extensions compile slightly faster
   - The `**/` prefix matches zero or more components, so `**/*_test.sh` covers root-level files

## Verification

All checks pass:
- 52 Shell adapter unit tests
- 177 total project tests
- `make check` quality gate

## Files Changed

- `crates/cli/src/adapter/shell/mod.rs` - Removed redundant pattern
- `crates/cli/src/adapter/glob_tests.rs` - Added test verifying `**/` root-level matching
- `crates/cli/benches/adapter.rs` - Added GlobSet pattern benchmarks
