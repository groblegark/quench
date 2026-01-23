# Checkpoint 4D: Benchmark - Rust Adapter

**Root Feature:** `quench-4d-benchmark`

## Overview

Performance benchmark checkpoint to measure the Rust language adapter's overhead and efficiency. Following the refactoring in checkpoint-4c, this checkpoint validates that the modular adapter structure meets performance targets and identifies any bottlenecks introduced by Rust-specific features:

- GlobSet pattern compilation and matching
- `#[cfg(test)]` block parsing
- Cargo workspace detection
- Suppress attribute parsing
- Line classification for mixed source/test files

Performance targets (from `docs/specs/20-performance.md`):

| Mode | Target | Acceptable |
|------|--------|------------|
| Cold | < 500ms | < 1s |
| Warm | < 100ms | < 200ms |

**Key question:** Does the Rust adapter add measurable overhead compared to the generic adapter, and is that overhead acceptable given the enhanced functionality?

## Project Structure

Key files involved:

```
quench/
├── crates/cli/
│   ├── benches/
│   │   ├── check.rs              # End-to-end check benchmarks (existing)
│   │   └── adapter.rs            # Adapter-specific benchmarks (new)
│   └── src/adapter/rust/
│       ├── mod.rs                # Core adapter (~150 LOC)
│       ├── cfg_test.rs           # #[cfg(test)] parser (~90 LOC)
│       ├── workspace.rs          # Cargo workspace parser
│       ├── suppress.rs           # Suppress attribute parser
│       └── policy.rs             # Lint policy checker
├── tests/fixtures/
│   ├── rust-simple/              # Minimal Rust project (existing)
│   ├── rust-workspace/           # Multi-package workspace (existing)
│   ├── bench-medium/             # 500 files, 50K LOC (reuse from 2D)
│   └── bench-rust/               # Rust-specific benchmark fixture (new)
├── reports/
│   └── checkpoint-4d-benchmarks.md  # Benchmark results (output)
└── scripts/
    └── gen-bench-fixture         # Fixture generation (existing from 2D)
```

## Dependencies

**Existing:**
- `criterion` - Benchmarking framework (already configured)
- `flamegraph` - Flame graph profiling
- `hyperfine` - CLI benchmarking tool
- `globset` - Pattern matching (adapter dependency)

**Install if needed:**
```bash
cargo install flamegraph
brew install hyperfine  # or apt-get install hyperfine
```

## Implementation Phases

### Phase 1: Create Rust-Specific Benchmark Fixture

**Goal:** Create `bench-rust` fixture with Rust-specific patterns that exercise all adapter features.

Unlike `bench-medium` (generic files for CLOC/escapes benchmarks), `bench-rust` should:
- Be a valid Cargo workspace with multiple packages
- Include `#[cfg(test)]` blocks for inline test detection
- Include `#[allow(...)]` and `#[expect(...)]` attributes
- Have a realistic distribution of source vs test files

**Script update:** Extend `scripts/gen-bench-fixture` or create `scripts/gen-rust-fixture`:

```bash
#!/usr/bin/env bash
set -euo pipefail

FIXTURE_DIR="${1:-tests/fixtures/bench-rust}"
PACKAGE_COUNT="${2:-5}"
FILES_PER_PACKAGE="${3:-100}"
TARGET_LOC="${4:-50000}"

mkdir -p "$FIXTURE_DIR"

# Create workspace Cargo.toml
cat > "$FIXTURE_DIR/Cargo.toml" << 'TOML'
[workspace]
members = ["crates/*"]
resolver = "2"
TOML

mkdir -p "$FIXTURE_DIR/crates"

# Generate packages
for pkg in $(seq 1 "$PACKAGE_COUNT"); do
    PKG_DIR="$FIXTURE_DIR/crates/pkg_$pkg"
    mkdir -p "$PKG_DIR/src" "$PKG_DIR/tests"

    # Package Cargo.toml
    cat > "$PKG_DIR/Cargo.toml" << TOML
[package]
name = "pkg_$pkg"
version = "0.1.0"
edition = "2021"
TOML

    # Generate source files with #[cfg(test)] blocks
    for i in $(seq 1 "$FILES_PER_PACKAGE"); do
        cat > "$PKG_DIR/src/module_$i.rs" << 'RUST'
//! Auto-generated module for Rust adapter benchmarking.

#[allow(dead_code)]
pub fn process_item(item: &str) -> String {
    item.to_uppercase()
}

#[allow(unused_variables)]
pub fn transform_data(data: &[i32]) -> Vec<i32> {
    data.iter().map(|x| x * 2).collect()
}

// Source code with inline tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_item() {
        assert_eq!(process_item("hello"), "HELLO");
    }

    #[test]
    fn test_transform_data() {
        let result = transform_data(&[1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
    }
}
RUST
        # Pad to ~100 lines per file
        for _ in $(seq 1 80); do
            echo "// padding line for benchmark" >> "$PKG_DIR/src/module_$i.rs"
        done
    done

    # Create lib.rs that imports modules
    echo "//! Package $pkg" > "$PKG_DIR/src/lib.rs"
    for i in $(seq 1 "$FILES_PER_PACKAGE"); do
        echo "pub mod module_$i;" >> "$PKG_DIR/src/lib.rs"
    done

    # Generate dedicated test files
    cat > "$PKG_DIR/tests/integration.rs" << 'RUST'
#![allow(clippy::unwrap_used)]

#[test]
fn integration_test() {
    assert!(true);
}
RUST
done

# Create quench.toml
cat > "$FIXTURE_DIR/quench.toml" << 'TOML'
[check.cloc]
check = "error"
max_lines = 200
max_lines_test = 500

[check.escapes]
check = "warn"

[rust]
lint_changes = "standalone"
TOML

echo "Generated Rust benchmark fixture at $FIXTURE_DIR"
echo "Packages: $PACKAGE_COUNT"
echo "Files per package: $FILES_PER_PACKAGE"
echo "Total files: $((PACKAGE_COUNT * FILES_PER_PACKAGE))"
```

**Fixture targets:**
- 5 packages × 100 files = 500 source files
- ~100 LOC per file = ~50K LOC total
- Each source file has `#[cfg(test)]` block (~20 lines)
- Each package has dedicated test file

**Verification:**
```bash
chmod +x scripts/gen-rust-fixture
./scripts/gen-rust-fixture tests/fixtures/bench-rust 5 100
find tests/fixtures/bench-rust -name '*.rs' | wc -l  # ~505
wc -l tests/fixtures/bench-rust/crates/*/src/*.rs | tail -1  # ~50000
```

**Milestone:** `tests/fixtures/bench-rust` exists with workspace structure and `#[cfg(test)]` blocks.

**Status:** [ ] Pending

---

### Phase 2: Add Adapter Micro-Benchmarks

**Goal:** Create `crates/cli/benches/adapter.rs` with focused benchmarks for adapter operations.

**Key operations to benchmark:**

1. **Adapter creation** - GlobSet compilation overhead
2. **File classification** - `classify()` method cost per file
3. **Line classification** - `classify_lines()` with cfg_test parsing
4. **Workspace detection** - `CargoWorkspace::from_root()`
5. **Suppress parsing** - `parse_suppress_attrs()` on file content

**New benchmark file:** `crates/cli/benches/adapter.rs`

```rust
//! Adapter-specific benchmarks.
//!
//! Measures overhead of language adapter operations.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, black_box};
use std::path::{Path, PathBuf};

// Import adapter types (adjust based on actual module structure)
use quench::adapter::rust::{CfgTestInfo, CargoWorkspace, RustAdapter, parse_suppress_attrs};
use quench::adapter::{Adapter, GenericAdapter};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures")
        .join(name)
}

/// Benchmark adapter creation (GlobSet compilation).
fn bench_adapter_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("adapter_creation");

    group.bench_function("RustAdapter::new", |b| {
        b.iter(|| black_box(RustAdapter::new()))
    });

    group.bench_function("GenericAdapter::new", |b| {
        b.iter(|| black_box(GenericAdapter::new(&[], &[])))
    });

    group.finish();
}

/// Benchmark file classification.
fn bench_classify(c: &mut Criterion) {
    let rust_adapter = RustAdapter::new();
    let generic_adapter = GenericAdapter::new(&[], &["tests/**".to_string()]);

    // Generate test paths
    let source_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("src/module_{}.rs", i)))
        .collect();
    let test_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("tests/test_{}.rs", i)))
        .collect();

    let mut group = c.benchmark_group("classify");

    group.bench_function("rust_1k_source", |b| {
        b.iter(|| {
            for path in &source_paths {
                black_box(rust_adapter.classify(path));
            }
        })
    });

    group.bench_function("rust_1k_test", |b| {
        b.iter(|| {
            for path in &test_paths {
                black_box(rust_adapter.classify(path));
            }
        })
    });

    group.bench_function("generic_1k_source", |b| {
        b.iter(|| {
            for path in &source_paths {
                black_box(generic_adapter.classify(path));
            }
        })
    });

    group.finish();
}

/// Benchmark #[cfg(test)] parsing.
fn bench_cfg_test_parse(c: &mut Criterion) {
    // Content with #[cfg(test)] block
    let content_with_cfg: String = (0..100)
        .map(|i| {
            if i == 50 {
                "#[cfg(test)]\nmod tests {\n    #[test]\n    fn test() {}\n}\n".to_string()
            } else {
                format!("pub fn func_{}() {{}}\n", i)
            }
        })
        .collect();

    // Content without #[cfg(test)]
    let content_without_cfg: String = (0..100)
        .map(|i| format!("pub fn func_{}() {{}}\n", i))
        .collect();

    let mut group = c.benchmark_group("cfg_test_parse");

    group.bench_function("with_cfg_test_100_lines", |b| {
        b.iter(|| black_box(CfgTestInfo::parse(&content_with_cfg)))
    });

    group.bench_function("without_cfg_test_100_lines", |b| {
        b.iter(|| black_box(CfgTestInfo::parse(&content_without_cfg)))
    });

    // Larger file
    let large_content: String = content_with_cfg.repeat(10);  // ~1000 lines
    group.bench_function("with_cfg_test_1000_lines", |b| {
        b.iter(|| black_box(CfgTestInfo::parse(&large_content)))
    });

    group.finish();
}

/// Benchmark classify_lines (full line classification with cfg_test).
fn bench_classify_lines(c: &mut Criterion) {
    let adapter = RustAdapter::new();
    let path = Path::new("src/lib.rs");

    // Mixed source/test content
    let content: String = (0..100)
        .map(|i| {
            if i >= 60 && i < 80 {
                if i == 60 { "#[cfg(test)]\nmod tests {\n".to_string() }
                else if i == 79 { "}\n".to_string() }
                else { "    #[test]\n    fn test() {}\n".to_string() }
            } else {
                format!("pub fn func_{}() {{}}\n", i)
            }
        })
        .collect();

    let mut group = c.benchmark_group("classify_lines");

    group.bench_function("100_lines_mixed", |b| {
        b.iter(|| black_box(adapter.classify_lines(path, &content)))
    });

    // Pure test file
    let test_path = Path::new("tests/integration.rs");
    group.bench_function("100_lines_test_file", |b| {
        b.iter(|| black_box(adapter.classify_lines(test_path, &content)))
    });

    group.finish();
}

/// Benchmark workspace detection.
fn bench_workspace_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("workspace_detection");

    let rust_simple = fixture_path("rust-simple");
    let rust_workspace = fixture_path("rust-workspace");

    if rust_simple.exists() {
        group.bench_with_input(
            BenchmarkId::new("from_root", "rust-simple"),
            &rust_simple,
            |b, path| b.iter(|| black_box(CargoWorkspace::from_root(path)))
        );
    }

    if rust_workspace.exists() {
        group.bench_with_input(
            BenchmarkId::new("from_root", "rust-workspace"),
            &rust_workspace,
            |b, path| b.iter(|| black_box(CargoWorkspace::from_root(path)))
        );
    }

    group.finish();
}

/// Benchmark suppress attribute parsing.
fn bench_suppress_parse(c: &mut Criterion) {
    // Content with various suppress attributes
    let content_with_suppresses: String = (0..100)
        .map(|i| {
            if i % 10 == 0 {
                "#[allow(dead_code)]\npub fn func() {}\n".to_string()
            } else if i % 15 == 0 {
                "#[expect(unused_variables)] // reason: test\npub fn func() {}\n".to_string()
            } else {
                format!("pub fn func_{}() {{}}\n", i)
            }
        })
        .collect();

    let content_without: String = (0..100)
        .map(|i| format!("pub fn func_{}() {{}}\n", i))
        .collect();

    let mut group = c.benchmark_group("suppress_parse");

    group.bench_function("with_attrs_100_lines", |b| {
        b.iter(|| black_box(parse_suppress_attrs(&content_with_suppresses, None)))
    });

    group.bench_function("without_attrs_100_lines", |b| {
        b.iter(|| black_box(parse_suppress_attrs(&content_without, None)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_adapter_creation,
    bench_classify,
    bench_cfg_test_parse,
    bench_classify_lines,
    bench_workspace_detection,
    bench_suppress_parse,
);
criterion_main!(benches);
```

**Update `Cargo.toml` benches section:**
```toml
[[bench]]
name = "adapter"
harness = false
```

**Verification:**
```bash
cargo bench --bench adapter
```

**Milestone:** Adapter micro-benchmarks run and produce baseline numbers.

**Status:** [ ] Pending

---

### Phase 3: Run End-to-End Benchmarks on Rust Fixtures

**Goal:** Measure full check pipeline performance on Rust projects and compare with generic adapter.

**Hyperfine comparisons:**

```bash
cargo build --release

# Compare rust-simple vs rust-workspace
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/rust-simple' \
    './target/release/quench check tests/fixtures/rust-workspace' \
    --export-markdown reports/rust-fixtures.md

# Compare bench-rust (Rust adapter) vs bench-medium (generic patterns)
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/bench-rust' \
    './target/release/quench check tests/fixtures/bench-medium' \
    --export-markdown reports/rust-vs-generic.md

# Cold vs warm on bench-rust
rm -rf tests/fixtures/bench-rust/.quench
hyperfine --warmup 0 --runs 5 \
    './target/release/quench check tests/fixtures/bench-rust' \
    --export-json reports/bench-rust-cold.json

hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/bench-rust' \
    --export-json reports/bench-rust-warm.json
```

**Criterion benchmarks:**
```bash
cargo bench --bench check -- "bench-rust"
```

**Expected results:**

| Fixture | Mode | Target | Notes |
|---------|------|--------|-------|
| bench-rust | Cold | < 500ms | Full workspace parsing |
| bench-rust | Warm | < 100ms | Cached results |
| rust-simple | Cold | < 100ms | Minimal project |
| rust-workspace | Cold | < 200ms | Multi-package |

**Overhead analysis:** Rust adapter should add < 10% overhead vs generic adapter on equivalent workloads.

**Milestone:** End-to-end benchmark numbers recorded for Rust fixtures.

**Status:** [ ] Pending

---

### Phase 4: Profile Adapter Hot Paths

**Goal:** Identify performance hotspots using flamegraph and instruments.

**Profiling commands:**

```bash
# Flamegraph on bench-rust
cargo flamegraph --bench adapter -- "classify"
cargo flamegraph -- check tests/fixtures/bench-rust

# macOS Instruments
xcrun xctrace record --template 'Time Profiler' \
    --launch ./target/release/quench -- check tests/fixtures/bench-rust
```

**Analysis checklist:**

- [ ] What % of time in `RustAdapter::new()` (GlobSet compilation)?
- [ ] What % of time in `classify()` per file?
- [ ] What % of time in `CfgTestInfo::parse()` for line classification?
- [ ] What % of time in `CargoWorkspace::from_root()` at startup?
- [ ] Is the adapter overhead < 10% of total check time?

**Expected breakdown for Rust project check:**

| Phase | Expected % | Notes |
|-------|------------|-------|
| File discovery | 30-40% | Using ignore crate |
| Adapter creation | < 1% | One-time cost |
| File classification | 5-10% | Per file, GlobSet match |
| Line classification | 10-20% | Only for source files with #[cfg(test)] |
| Check execution | 30-40% | CLOC, escapes, etc. |
| Output | < 5% | JSON/text generation |

**Milestone:** Flame graph generated, hotspots documented.

**Status:** [ ] Pending

---

### Phase 5: Document Results in Report

**Goal:** Create comprehensive benchmark report at `reports/checkpoint-4d-benchmarks.md`.

**Report template:**

```markdown
# Checkpoint 4D: Benchmark Report - Rust Adapter

Generated: YYYY-MM-DD
Hardware: [CPU, RAM, OS]

## Summary

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| bench-rust cold | < 500ms | XXXms | pass/fail |
| bench-rust warm | < 100ms | XXms | pass/fail |
| Adapter overhead vs generic | < 10% | X% | pass/fail |
| classify() per 1K files | < 10ms | Xms | pass/fail |
| CfgTestInfo::parse() per file | < 0.1ms | Xms | pass/fail |

## Detailed Results

### 1. End-to-End Benchmarks

**bench-rust (500 files, 50K LOC, Rust workspace):**

| Run | Mean | Std Dev | Min | Max |
|-----|------|---------|-----|-----|
| Cold | | | | |
| Warm | | | | |

**Comparison with generic adapter (bench-medium):**

| Fixture | Adapter | Cold | Warm |
|---------|---------|------|------|
| bench-rust | RustAdapter | | |
| bench-medium | GenericAdapter | | |
| Overhead | | X% | X% |

### 2. Adapter Micro-Benchmarks

**Adapter creation:**

| Adapter | Time | Notes |
|---------|------|-------|
| RustAdapter::new() | | GlobSet compilation |
| GenericAdapter::new() | | |

**File classification (1K files):**

| Operation | Time | Per-file |
|-----------|------|----------|
| classify() source files | | |
| classify() test files | | |

**#[cfg(test)] parsing:**

| Content | Time | Per-line |
|---------|------|----------|
| 100 lines with cfg | | |
| 100 lines without cfg | | |
| 1000 lines with cfg | | |

**Workspace detection:**

| Fixture | Time |
|---------|------|
| rust-simple | |
| rust-workspace | |

**Suppress parsing:**

| Content | Time |
|---------|------|
| 100 lines with attrs | |
| 100 lines without | |

### 3. Profiling Analysis

**Flame graph:** [link to SVG]

| Function | % of Total | Notes |
|----------|------------|-------|
| file walking | | ignore crate |
| RustAdapter::classify | | |
| CfgTestInfo::parse | | |
| check execution | | |

### 4. Per-Module Breakdown

| Module | LOC | Complexity | Performance Notes |
|--------|-----|------------|-------------------|
| mod.rs | 150 | Low | GlobSet match is O(patterns) |
| cfg_test.rs | 90 | Medium | Line-by-line parsing, O(lines) |
| workspace.rs | ~100 | Medium | File I/O + TOML parsing |
| suppress.rs | ~130 | Medium | Regex matching per line |
| policy.rs | ~60 | Low | Simple file categorization |

## Conclusions

[Summary of findings]

## Recommendations

[Any optimizations needed, or confirmation that performance is acceptable]

### Potential Optimizations (if needed)

1. **Lazy adapter creation:** Create adapter only when Rust files detected
2. **GlobSet caching:** Share compiled GlobSet across adapter instances
3. **Streaming cfg_test parsing:** Stop parsing once block found (early exit)
4. **Parallel line classification:** Use rayon for large files
```

**Milestone:** Report complete at `reports/checkpoint-4d-benchmarks.md`.

**Status:** [ ] Pending

## Key Implementation Details

### Adapter Performance Model

The Rust adapter adds overhead at several points:

```
Total Overhead = Startup + Per-File + Per-Line
```

| Phase | Cost | When |
|-------|------|------|
| Startup | O(patterns) | Once per run |
| Per-file classify | O(patterns) | Every .rs file |
| Per-line classify | O(lines) | Source files with potential #[cfg(test)] |

### GlobSet Performance

The `globset` crate compiles patterns into an efficient matcher:
- Compilation: O(patterns × pattern_length)
- Matching: O(path_components × patterns) with optimizations

For Rust adapter (4 source patterns, 4 test patterns, 1 ignore pattern):
- Compilation: ~microseconds
- Matching: ~nanoseconds per path

### CfgTestInfo Parsing Complexity

The `CfgTestInfo::parse()` function:
- Time: O(lines × line_length)
- Memory: O(cfg_test_blocks) for storing ranges

Optimizations already applied:
- String literal detection (skip braces inside strings)
- Early exit when block closes (brace_depth == 0)
- No regex, pure character iteration

### Comparison Points

| Aspect | Generic Adapter | Rust Adapter |
|--------|-----------------|--------------|
| Creation | Minimal | GlobSet compile |
| classify() | Simple pattern match | Multi-pattern GlobSet |
| Line-level | Not supported | CfgTestInfo parsing |
| Default escapes | Empty | 4 patterns |
| Workspace | Not supported | Cargo.toml parsing |

## Verification Plan

1. **Fixture generation:**
   ```bash
   ./scripts/gen-rust-fixture tests/fixtures/bench-rust 5 100
   find tests/fixtures/bench-rust -name '*.rs' | wc -l  # ~505
   ```

2. **Micro-benchmarks:**
   ```bash
   cargo bench --bench adapter
   ```

3. **End-to-end benchmarks:**
   ```bash
   hyperfine './target/release/quench check tests/fixtures/bench-rust'
   ```

4. **Profiling:**
   ```bash
   cargo flamegraph -- check tests/fixtures/bench-rust
   open flamegraph.svg
   ```

5. **Report generation:**
   ```bash
   ls reports/checkpoint-4d-benchmarks.md
   ```

6. **Quality gates:**
   ```bash
   make check
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Create Rust-specific benchmark fixture | [ ] Pending |
| 2 | Add adapter micro-benchmarks | [ ] Pending |
| 3 | Run end-to-end benchmarks on Rust fixtures | [ ] Pending |
| 4 | Profile adapter hot paths | [ ] Pending |
| 5 | Document results in report | [ ] Pending |

## Notes

- The `bench-rust` fixture should be added to `.gitignore` to avoid repository bloat
- Benchmark results vary by hardware; document machine specs in the report
- The Rust adapter overhead should be minimal (< 10%) given the enhanced functionality
- If overhead exceeds targets, consider:
  - Lazy initialization (defer GlobSet until first .rs file)
  - Shared pattern cache across adapter instances
  - Conditional parsing (skip cfg_test for files without `#[cfg(test)]` substring)
