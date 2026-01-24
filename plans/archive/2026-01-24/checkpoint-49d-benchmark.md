# Checkpoint 49D: Benchmark - JavaScript Adapter

**Root Feature:** `quench-68fa`

## Overview

Performance benchmark checkpoint to measure the JavaScript/TypeScript adapter's overhead and efficiency. Following the validation in checkpoint-49b and refactoring check in checkpoint-49c, this checkpoint profiles the JavaScript adapter on existing fixtures and identifies any bottlenecks.

Key areas to benchmark:
- GlobSet pattern compilation (22 patterns: 6 source, 11 test, 5 ignore)
- File classification with `node_modules/` filtering
- npm/pnpm workspace detection (`JsWorkspace::from_root()`)
- ESLint/Biome suppress directive parsing
- End-to-end `quench check` on `js-simple` and `js-monorepo` fixtures

Performance targets (from `docs/specs/20-performance.md`):

| Mode | Target | Acceptable |
|------|--------|------------|
| Cold | < 500ms | < 1s |
| Warm | < 100ms | < 200ms |

**Key question:** Does the JavaScript adapter meet performance targets on typical JS/TS projects, and are there any bottlenecks in pattern matching, file detection, or workspace enumeration?

## Project Structure

Key files involved:

```
quench/
├── crates/cli/
│   ├── benches/
│   │   └── adapter.rs            # Adapter benchmarks (extend for JS)
│   └── src/adapter/javascript/
│       ├── mod.rs                # Core adapter (~161 LOC)
│       ├── workspace.rs          # npm/pnpm workspace parser (~162 LOC)
│       ├── suppress.rs           # ESLint/Biome directive parser (~396 LOC)
│       └── policy.rs             # Lint policy checker
├── tests/fixtures/
│   ├── js-simple/                # Minimal JS project (existing)
│   └── js-monorepo/              # Multi-package pnpm workspace (existing)
├── reports/
│   └── checkpoint-49-javascript-adapter.md  # Benchmark results (output)
└── scripts/
    └── gen-bench-fixture         # Fixture generation (existing)
```

## Dependencies

**Existing:**
- `criterion` - Benchmarking framework (already configured)
- `hyperfine` - CLI benchmarking tool
- `globset` - Pattern matching (adapter dependency)
- `serde_yaml` - pnpm-workspace.yaml parsing

**Install if needed:**
```bash
brew install hyperfine  # or apt-get install hyperfine
```

## Implementation Phases

### Phase 1: Add JavaScript Adapter Micro-Benchmarks

**Goal:** Extend `crates/cli/benches/adapter.rs` with JavaScript-specific benchmarks.

**Key operations to benchmark:**

1. **JavaScriptAdapter::new()** - GlobSet compilation for 22 patterns
2. **classify()** - File classification with ignore pattern checking
3. **JsWorkspace::from_root()** - pnpm/npm workspace detection
4. **parse_javascript_suppresses()** - ESLint + Biome directive parsing

**Add to `crates/cli/benches/adapter.rs`:**

```rust
use quench::adapter::javascript::{
    JavaScriptAdapter, JsWorkspace, parse_javascript_suppresses,
};

/// Benchmark JavaScript adapter creation.
fn bench_js_adapter_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_adapter_creation");

    group.bench_function("JavaScriptAdapter::new", |b| {
        b.iter(|| black_box(JavaScriptAdapter::new()))
    });

    group.finish();
}

/// Benchmark JavaScript file classification.
fn bench_js_classify(c: &mut Criterion) {
    let js_adapter = JavaScriptAdapter::new();

    // Generate test paths
    let source_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("src/components/component_{}.tsx", i)))
        .collect();
    let test_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("src/components/component_{}.test.tsx", i)))
        .collect();
    let node_modules_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("node_modules/pkg_{}/index.js", i)))
        .collect();

    let mut group = c.benchmark_group("js_classify");

    group.bench_function("js_1k_source", |b| {
        b.iter(|| {
            for path in &source_paths {
                black_box(js_adapter.classify(path));
            }
        })
    });

    group.bench_function("js_1k_test", |b| {
        b.iter(|| {
            for path in &test_paths {
                black_box(js_adapter.classify(path));
            }
        })
    });

    group.bench_function("js_1k_node_modules_ignored", |b| {
        b.iter(|| {
            for path in &node_modules_paths {
                black_box(js_adapter.classify(path));
            }
        })
    });

    group.finish();
}

/// Benchmark JavaScript workspace detection.
fn bench_js_workspace_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_workspace_detection");

    let js_simple = fixture_path("js-simple");
    let js_monorepo = fixture_path("js-monorepo");

    if js_simple.exists() {
        group.bench_with_input(
            BenchmarkId::new("from_root", "js-simple"),
            &js_simple,
            |b, path| b.iter(|| black_box(JsWorkspace::from_root(path))),
        );
    }

    if js_monorepo.exists() {
        group.bench_with_input(
            BenchmarkId::new("from_root", "js-monorepo"),
            &js_monorepo,
            |b, path| b.iter(|| black_box(JsWorkspace::from_root(path))),
        );
    }

    group.finish();
}

/// Benchmark ESLint/Biome suppress parsing.
fn bench_js_suppress_parse(c: &mut Criterion) {
    // Content with various ESLint suppresses
    let content_with_eslint: String = (0..100)
        .map(|i| {
            if i % 10 == 0 {
                "// eslint-disable-next-line no-unused-vars -- OK: intentional\nconst x = 1;\n".to_string()
            } else if i % 15 == 0 {
                "/* eslint-disable @typescript-eslint/no-explicit-any */\nfunction legacy(): any {}\n/* eslint-enable */\n".to_string()
            } else {
                format!("const value_{} = {};\n", i, i)
            }
        })
        .collect();

    // Content with Biome suppresses
    let content_with_biome: String = (0..100)
        .map(|i| {
            if i % 10 == 0 {
                "// biome-ignore lint/suspicious/noExplicitAny: legacy code\nfunction legacy(): any {}\n".to_string()
            } else {
                format!("const value_{} = {};\n", i, i)
            }
        })
        .collect();

    let content_without: String = (0..100)
        .map(|i| format!("const value_{} = {};\n", i, i))
        .collect();

    let mut group = c.benchmark_group("js_suppress_parse");

    group.bench_function("eslint_100_lines", |b| {
        b.iter(|| black_box(parse_javascript_suppresses(&content_with_eslint, None)))
    });

    group.bench_function("biome_100_lines", |b| {
        b.iter(|| black_box(parse_javascript_suppresses(&content_with_biome, None)))
    });

    group.bench_function("none_100_lines", |b| {
        b.iter(|| black_box(parse_javascript_suppresses(&content_without, None)))
    });

    // Larger file
    let large_eslint: String = content_with_eslint.repeat(10);
    group.bench_function("eslint_1000_lines", |b| {
        b.iter(|| black_box(parse_javascript_suppresses(&large_eslint, None)))
    });

    group.finish();
}
```

**Update criterion_group! to include new benchmarks:**
```rust
criterion_group!(
    benches,
    // ... existing benchmarks ...
    bench_js_adapter_creation,
    bench_js_classify,
    bench_js_workspace_detection,
    bench_js_suppress_parse,
);
```

**Verification:**
```bash
cargo bench --bench adapter -- "js_"
```

**Milestone:** JavaScript adapter micro-benchmarks run and produce baseline numbers.

**Status:** [ ] Pending

---

### Phase 2: Run End-to-End Benchmarks on JavaScript Fixtures

**Goal:** Measure full check pipeline performance on `js-simple` and `js-monorepo`.

**Hyperfine comparisons:**

```bash
cargo build --release

# Benchmark js-simple (minimal project)
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/js-simple' \
    --export-json reports/bench-js-simple.json

# Benchmark js-monorepo (pnpm workspace)
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/js-monorepo' \
    --export-json reports/bench-js-monorepo.json

# Cold runs (no warmup, simulate first-time run)
rm -rf tests/fixtures/js-simple/.quench tests/fixtures/js-monorepo/.quench
hyperfine --warmup 0 --runs 5 \
    './target/release/quench check tests/fixtures/js-simple' \
    './target/release/quench check tests/fixtures/js-monorepo' \
    --export-markdown reports/js-cold-runs.md

# Compare with Rust fixtures (reference)
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/js-simple' \
    './target/release/quench check tests/fixtures/rust-simple' \
    --export-markdown reports/js-vs-rust.md
```

**Expected results:**

| Fixture | Files | Mode | Target | Notes |
|---------|-------|------|--------|-------|
| js-simple | ~5 | Cold | < 100ms | Minimal project |
| js-simple | ~5 | Warm | < 50ms | Cached results |
| js-monorepo | ~10 | Cold | < 150ms | pnpm workspace |
| js-monorepo | ~10 | Warm | < 75ms | Cached results |

**Verification:**
```bash
ls reports/bench-js-*.json
```

**Milestone:** End-to-end benchmark numbers recorded for JavaScript fixtures.

**Status:** [ ] Pending

---

### Phase 3: Profile Pattern Matching Performance

**Goal:** Analyze GlobSet pattern matching efficiency for 22 patterns.

**Profile GlobSet overhead:**

The JavaScript adapter compiles 22 patterns into 3 GlobSets:
- **Source patterns (6):** `**/*.{js,jsx,ts,tsx,mjs,mts}`
- **Test patterns (11):** `**/*.{test,spec}.*`, `__tests__/**`, `test/**`, `tests/**`
- **Ignore patterns (5):** `node_modules/**`, `dist/**`, `build/**`, `.next/**`, `coverage/**`

**Key questions:**
1. How does 22-pattern GlobSet compare to Rust's 9-pattern GlobSet?
2. Is `should_ignore()` check adding measurable overhead?
3. Does the 3-step classify (ignore → test → source) affect performance?

**Benchmark comparison:**

```rust
/// Compare pattern counts across adapters.
fn bench_pattern_count_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_count_comparison");

    // JavaScript: 22 patterns (6 + 11 + 5)
    group.bench_function("js_adapter_22_patterns", |b| {
        b.iter(|| black_box(JavaScriptAdapter::new()))
    });

    // Rust: 9 patterns (1 + 4 + 1 + ignore set)
    group.bench_function("rust_adapter_9_patterns", |b| {
        b.iter(|| black_box(RustAdapter::new()))
    });

    // Generic: variable patterns
    group.bench_function("generic_adapter_6_patterns", |b| {
        b.iter(|| black_box(GenericAdapter::with_defaults()))
    });

    group.finish();
}
```

**Analysis checklist:**

- [ ] GlobSet compilation time for 22 vs 9 patterns
- [ ] Per-file classify() cost with ignore pattern check
- [ ] Pattern matching order optimization (ignore first is correct)
- [ ] Memory overhead of 3 separate GlobSets

**Milestone:** Pattern matching overhead quantified and documented.

**Status:** [ ] Pending

---

### Phase 4: Profile Workspace Detection

**Goal:** Measure workspace enumeration overhead for different configurations.

**Key operations to analyze:**

1. **pnpm-workspace.yaml parsing** - YAML deserialization + pattern expansion
2. **package.json workspaces** - JSON parsing + array/object form handling
3. **Pattern expansion** - Directory scanning for `packages/*` patterns

**Benchmark different workspace sizes:**

```rust
/// Benchmark workspace detection with different package counts.
fn bench_workspace_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_workspace_scaling");

    // Note: Would need generated fixtures for meaningful scaling tests
    // For now, use existing fixtures

    let js_simple = fixture_path("js-simple");
    let js_monorepo = fixture_path("js-monorepo");

    // Non-workspace (should be fast, no parsing)
    if js_simple.exists() {
        group.bench_with_input(
            BenchmarkId::new("non_workspace", "js-simple"),
            &js_simple,
            |b, path| b.iter(|| black_box(JsWorkspace::from_root(path))),
        );
    }

    // pnpm workspace (2 packages)
    if js_monorepo.exists() {
        group.bench_with_input(
            BenchmarkId::new("pnpm_workspace", "js-monorepo"),
            &js_monorepo,
            |b, path| b.iter(|| black_box(JsWorkspace::from_root(path))),
        );
    }

    group.finish();
}
```

**Expected breakdown:**

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Non-workspace check | < 50µs | File existence check only |
| pnpm-workspace.yaml | < 200µs | YAML parse + 1 glob |
| package.json workspaces | < 150µs | JSON parse |
| Pattern expansion | O(packages) | ~50µs per package |

**Milestone:** Workspace detection timing documented.

**Status:** [ ] Pending

---

### Phase 5: Document Results in Report

**Goal:** Create comprehensive benchmark report at `reports/checkpoint-49-javascript-adapter.md`.

**Note:** This file already exists from checkpoint-49b validation. Append benchmark results section.

**Report structure to add:**

```markdown
## Benchmark Results

Generated: YYYY-MM-DD
Hardware: [CPU, RAM, OS]

### Summary

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| js-simple cold | < 100ms | XXms | pass/fail |
| js-simple warm | < 50ms | XXms | pass/fail |
| js-monorepo cold | < 150ms | XXms | pass/fail |
| js-monorepo warm | < 75ms | XXms | pass/fail |
| JavaScriptAdapter::new() | < 100µs | XXµs | pass/fail |
| classify() per 1K files | < 1ms | XXµs | pass/fail |
| JsWorkspace::from_root() | < 500µs | XXµs | pass/fail |

### Detailed Results

#### 1. End-to-End Benchmarks

**js-simple:**

| Run | Mean | Std Dev | Min | Max |
|-----|------|---------|-----|-----|
| Cold | | | | |
| Warm | | | | |

**js-monorepo:**

| Run | Mean | Std Dev | Min | Max |
|-----|------|---------|-----|-----|
| Cold | | | | |
| Warm | | | | |

#### 2. Adapter Micro-Benchmarks

**Adapter creation:**

| Adapter | Time | Patterns | Notes |
|---------|------|----------|-------|
| JavaScriptAdapter::new() | | 22 | 3 GlobSets |
| RustAdapter::new() | | 9 | Reference |

**File classification (1K files):**

| Operation | Time | Per-file |
|-----------|------|----------|
| classify() source files | | |
| classify() test files | | |
| classify() node_modules (ignored) | | |

**Workspace detection:**

| Fixture | Time | Type |
|---------|------|------|
| js-simple | | Non-workspace |
| js-monorepo | | pnpm |

**Suppress parsing:**

| Content | Time |
|---------|------|
| ESLint 100 lines | |
| Biome 100 lines | |
| No suppresses 100 lines | |

### Conclusions

[Summary of findings]

### Recommendations

[Any optimizations needed, or confirmation that performance is acceptable]

### Potential Optimizations (if needed)

1. **Lazy GlobSet compilation:** Defer until first JS file detected
2. **Pattern consolidation:** Combine source patterns into fewer globs
3. **Ignore pattern early-exit:** Check node_modules prefix before GlobSet
4. **Workspace caching:** Cache parsed workspace info per directory
```

**Milestone:** Report complete at `reports/checkpoint-49-javascript-adapter.md`.

**Status:** [ ] Pending

## Key Implementation Details

### JavaScript Adapter Pattern Structure

The JavaScript adapter uses 22 total patterns across 3 GlobSets:

| GlobSet | Patterns | Purpose |
|---------|----------|---------|
| source_patterns | 6 | Match JS/TS source files |
| test_patterns | 11 | Match test files/directories |
| ignore_patterns | 5 | Exclude build artifacts |

**Classification order (optimized):**
1. Check ignore patterns first (fail fast for node_modules)
2. Check test patterns (more specific)
3. Check source patterns (broad catch-all)

### GlobSet Performance Model

```
Compilation cost = O(patterns × pattern_complexity)
Matching cost = O(path_components × patterns)
```

For JavaScript adapter (22 patterns):
- Compilation: ~100µs expected (one-time cost)
- Matching: ~0.1-0.2µs per file expected

### Workspace Detection Complexity

```
pnpm-workspace.yaml: O(packages in config) + O(packages on disk)
package.json:        O(packages in config) + O(packages on disk)
```

The pattern expansion (`packages/*`) requires directory listing which is I/O bound.

### Comparison with Rust Adapter

| Aspect | Rust Adapter | JavaScript Adapter |
|--------|--------------|-------------------|
| Patterns | 9 total | 22 total |
| GlobSets | 3 | 3 |
| Ignore check | target/** | node_modules/** + 4 more |
| Line-level | #[cfg(test)] parsing | None |
| Workspace | Cargo.toml | package.json/pnpm-workspace.yaml |
| Suppress parsing | #[allow], #[expect] | eslint-disable, biome-ignore |

The JavaScript adapter has more patterns but no line-level classification, so overall performance should be comparable.

## Verification Plan

1. **Micro-benchmarks:**
   ```bash
   cargo bench --bench adapter -- "js_"
   ```

2. **End-to-end benchmarks:**
   ```bash
   cargo build --release
   hyperfine './target/release/quench check tests/fixtures/js-simple'
   hyperfine './target/release/quench check tests/fixtures/js-monorepo'
   ```

3. **Verify fixtures exist:**
   ```bash
   ls tests/fixtures/js-simple tests/fixtures/js-monorepo
   ```

4. **Quality gates:**
   ```bash
   make check
   ```

5. **Report generated:**
   ```bash
   grep "Benchmark Results" reports/checkpoint-49-javascript-adapter.md
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Add JavaScript adapter micro-benchmarks | [ ] Pending |
| 2 | Run end-to-end benchmarks on JS fixtures | [ ] Pending |
| 3 | Profile pattern matching performance | [ ] Pending |
| 4 | Profile workspace detection | [ ] Pending |
| 5 | Document results in report | [ ] Pending |

## Notes

- The existing fixtures (`js-simple`, `js-monorepo`) are sufficient for initial benchmarking
- If performance targets are not met, consider creating a larger `bench-js` fixture (similar to `bench-rust`)
- The JavaScript adapter has no line-level classification (unlike Rust's #[cfg(test)]), which simplifies the performance model
- Pattern count (22) is higher than Rust (9), but GlobSet compilation is still expected to be sub-millisecond
- Workspace detection uses file I/O and may show more variance than pure CPU operations
