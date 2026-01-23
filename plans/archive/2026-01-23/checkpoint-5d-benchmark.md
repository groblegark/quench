# Checkpoint 5D: Benchmark - Shell Adapter

**Root Feature:** `quench-5c1a`

## Overview

Performance benchmark checkpoint to measure the Shell language adapter's overhead and efficiency. Following the refactoring in checkpoint-5c, this checkpoint validates that the Shell adapter meets performance targets and compares its overhead against the Rust adapter baseline from checkpoint-4d.

The Shell adapter is architecturally simpler than the Rust adapter:

| Aspect | Shell Adapter | Rust Adapter |
|--------|---------------|--------------|
| Total LOC | 231 | 489 |
| Line-level parsing | None | `#[cfg(test)]` detection |
| Workspace detection | Not applicable | Cargo.toml parsing |
| Suppress parsing | String split (shellcheck codes) | Regex (#[allow]/[expect]) |

Expected outcome: Shell adapter should perform comparably to or better than Rust adapter due to reduced complexity.

Performance targets (from `docs/specs/20-performance.md`):

| Mode | Target | Acceptable |
|------|--------|------------|
| Cold | < 500ms | < 1s |
| Warm | < 100ms | < 200ms |

**Key question:** Does the Shell adapter meet performance targets, and how does its overhead compare to the Rust adapter baseline?

## Project Structure

Key files involved:

```
quench/
├── crates/cli/
│   ├── benches/
│   │   ├── adapter.rs              # Extend with Shell benchmarks
│   │   └── check.rs                # End-to-end benchmarks (existing)
│   └── src/adapter/shell/
│       ├── mod.rs                  # Core adapter (116 LOC)
│       ├── suppress.rs             # Shellcheck suppress parser (83 LOC)
│       └── policy.rs               # Lint policy (32 LOC, uses common)
├── tests/fixtures/
│   ├── shell/                      # Existing shell violations
│   ├── shell-scripts/              # Existing shell project
│   └── bench-shell/                # Shell benchmark fixture (new)
├── reports/
│   └── checkpoint-5d-benchmarks.md # Benchmark results (output)
└── scripts/
    └── gen-shell-fixture           # Fixture generation (new)
```

## Dependencies

**Existing:**
- `criterion` - Benchmarking framework (already configured)
- `hyperfine` - CLI benchmarking tool
- `globset` - Pattern matching (adapter dependency)

**No new dependencies required** - all benchmarking infrastructure from checkpoint-4d is reusable.

## Implementation Phases

### Phase 1: Create Shell Benchmark Fixture

**Goal:** Create `bench-shell` fixture with realistic shell project patterns.

Unlike `bench-rust` (Cargo workspace with `#[cfg(test)]` blocks), `bench-shell` should:
- Be a realistic shell project with `bin/` and `scripts/` directories
- Include shellcheck suppress directives with varying patterns
- Have a mix of source and test files (`.sh` and `.bats`)
- Target ~500 files to match bench-rust for fair comparison

**Create:** `scripts/gen-shell-fixture`

```bash
#!/usr/bin/env bash
set -euo pipefail

FIXTURE_DIR="${1:-tests/fixtures/bench-shell}"
SCRIPT_COUNT="${2:-100}"
UTIL_COUNT="${3:-400}"

mkdir -p "$FIXTURE_DIR"/{bin,scripts,lib,tests}

# Generate bin/ scripts (entry points)
for i in $(seq 1 "$SCRIPT_COUNT"); do
    cat > "$FIXTURE_DIR/bin/cmd_$i.sh" << 'SHELL'
#!/usr/bin/env bash
# Auto-generated script for Shell adapter benchmarking.
set -euo pipefail

# shellcheck source=../lib/utils.sh
source "$(dirname "$0")/../lib/utils.sh"

main() {
    local input="${1:-}"
    if [[ -z "$input" ]]; then
        echo "Usage: $0 <input>" >&2
        exit 1
    fi
    process_input "$input"
}

main "$@"
SHELL
    # Pad to ~50 lines
    for _ in $(seq 1 40); do
        echo "# padding line for benchmark" >> "$FIXTURE_DIR/bin/cmd_$i.sh"
    done
done

# Generate lib/ utilities with shellcheck suppresses
for i in $(seq 1 "$UTIL_COUNT"); do
    cat > "$FIXTURE_DIR/lib/util_$i.sh" << SHELL
#!/usr/bin/env bash
# Utility module $i for benchmarking.

# shellcheck disable=SC2034  # OK: intentionally unused
UTIL_VERSION="1.0.$i"

process_input() {
    local input="\$1"
    echo "Processing: \$input"
}

# More utility functions
format_output() {
    # shellcheck disable=SC2086  # OK: word splitting intentional
    printf "%s\n" \$1
}
SHELL
    # Pad to ~50 lines
    for _ in $(seq 1 35); do
        echo "# padding line for benchmark" >> "$FIXTURE_DIR/lib/util_$i.sh"
    done
done

# Generate test files (.bats)
for i in $(seq 1 50); do
    cat > "$FIXTURE_DIR/tests/test_util_$i.bats" << 'BATS'
#!/usr/bin/env bats
# Test suite for utility module.

setup() {
    load '../lib/utils'
}

@test "process_input handles empty string" {
    run process_input ""
    [ "$status" -eq 0 ]
}

@test "process_input handles special chars" {
    run process_input "hello world"
    [ "$status" -eq 0 ]
}
BATS
    # Pad to ~30 lines
    for _ in $(seq 1 20); do
        echo "# padding line for benchmark" >> "$FIXTURE_DIR/tests/test_util_$i.bats"
    done
done

# Create quench.toml
cat > "$FIXTURE_DIR/quench.toml" << 'TOML'
[check.cloc]
check = "error"
max_lines = 200
max_lines_test = 500

[check.escapes]
check = "warn"

[shell]
lint_changes = "standalone"
TOML

# Create .shellcheckrc
cat > "$FIXTURE_DIR/.shellcheckrc" << 'RC'
# Shellcheck configuration for benchmark fixture
disable=SC1091
RC

echo "Generated Shell benchmark fixture at $FIXTURE_DIR"
echo "Bin scripts: $SCRIPT_COUNT"
echo "Lib utilities: $UTIL_COUNT"
echo "Test files: 50"
echo "Total files: $((SCRIPT_COUNT + UTIL_COUNT + 50))"
```

**Fixture targets:**
- 100 bin scripts + 400 lib utilities + 50 test files = 550 files
- ~50 LOC per file = ~27.5K LOC total
- Shellcheck suppresses in ~80% of lib files (2 per file)

**Verification:**
```bash
chmod +x scripts/gen-shell-fixture
./scripts/gen-shell-fixture tests/fixtures/bench-shell 100 400
find tests/fixtures/bench-shell -name '*.sh' -o -name '*.bats' | wc -l  # ~550
```

**Milestone:** `tests/fixtures/bench-shell` exists with realistic shell project structure.

**Status:** [ ] Pending

---

### Phase 2: Add Shell Adapter Benchmarks

**Goal:** Extend `crates/cli/benches/adapter.rs` with Shell adapter benchmarks.

**Key operations to benchmark:**

1. **Adapter creation** - GlobSet compilation (2 source + 4 test patterns)
2. **File classification** - Pattern matching for `.sh`, `.bash`, `.bats`
3. **Shellcheck suppress parsing** - `parse_shellcheck_suppresses()`
4. **Escape pattern matching** - `set +e` and `eval` detection

**Add to `crates/cli/benches/adapter.rs`:**

```rust
use quench::adapter::shell::{ShellAdapter, parse_shellcheck_suppresses};

/// Benchmark Shell adapter creation.
fn bench_shell_adapter_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shell_adapter_creation");

    group.bench_function("ShellAdapter::new", |b| {
        b.iter(|| black_box(ShellAdapter::new()))
    });

    group.finish();
}

/// Benchmark Shell file classification.
fn bench_shell_classify(c: &mut Criterion) {
    let shell_adapter = ShellAdapter::new();

    // Generate shell paths
    let source_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("scripts/script_{}.sh", i)))
        .collect();
    let bash_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("lib/util_{}.bash", i)))
        .collect();
    let test_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("tests/test_{}.bats", i)))
        .collect();
    let bin_paths: Vec<PathBuf> = (0..1000)
        .map(|i| PathBuf::from(format!("bin/cmd_{}.sh", i)))
        .collect();

    let mut group = c.benchmark_group("shell_classify");

    group.bench_function("1k_source_scripts", |b| {
        b.iter(|| {
            for path in &source_paths {
                black_box(shell_adapter.classify(path));
            }
        })
    });

    group.bench_function("1k_bash_libs", |b| {
        b.iter(|| {
            for path in &bash_paths {
                black_box(shell_adapter.classify(path));
            }
        })
    });

    group.bench_function("1k_bats_tests", |b| {
        b.iter(|| {
            for path in &test_paths {
                black_box(shell_adapter.classify(path));
            }
        })
    });

    group.bench_function("1k_bin_scripts", |b| {
        b.iter(|| {
            for path in &bin_paths {
                black_box(shell_adapter.classify(path));
            }
        })
    });

    group.finish();
}

/// Benchmark shellcheck suppress parsing.
fn bench_shellcheck_suppress_parse(c: &mut Criterion) {
    // Content with shellcheck suppresses (~10% of lines)
    let content_with_suppresses: String = (0..100)
        .map(|i| {
            if i % 10 == 0 {
                "# shellcheck disable=SC2034  # OK: intentional\nUNUSED_VAR=1\n".to_string()
            } else if i % 15 == 0 {
                "# shellcheck disable=SC2086,SC2046\n# OK: word splitting needed\necho $var\n"
                    .to_string()
            } else {
                format!("echo \"line {}\"\n", i)
            }
        })
        .collect();

    // Content without suppresses
    let content_without: String = (0..100)
        .map(|i| format!("echo \"line {}\"\n", i))
        .collect();

    let mut group = c.benchmark_group("shellcheck_suppress_parse");

    group.bench_function("with_suppresses_100_lines", |b| {
        b.iter(|| black_box(parse_shellcheck_suppresses(&content_with_suppresses, None)))
    });

    group.bench_function("without_suppresses_100_lines", |b| {
        b.iter(|| black_box(parse_shellcheck_suppresses(&content_without, None)))
    });

    // With comment pattern requirement
    group.bench_function("with_suppresses_100_lines_pattern", |b| {
        b.iter(|| {
            black_box(parse_shellcheck_suppresses(
                &content_with_suppresses,
                Some("# OK:"),
            ))
        })
    });

    // Larger file (~1000 lines)
    let large_with_suppresses: String = content_with_suppresses.repeat(10);
    group.bench_function("with_suppresses_1000_lines", |b| {
        b.iter(|| black_box(parse_shellcheck_suppresses(&large_with_suppresses, None)))
    });

    group.finish();
}
```

**Update criterion_group:**
```rust
criterion_group!(
    benches,
    // Existing Rust benchmarks
    bench_adapter_creation,
    bench_classify,
    bench_cfg_test_parse,
    bench_classify_lines,
    bench_workspace_detection,
    bench_suppress_parse,
    // New Shell benchmarks
    bench_shell_adapter_creation,
    bench_shell_classify,
    bench_shellcheck_suppress_parse,
);
```

**Verification:**
```bash
cargo bench --bench adapter -- shell
```

**Milestone:** Shell adapter benchmarks run and produce baseline numbers.

**Status:** [ ] Pending

---

### Phase 3: Run End-to-End Benchmarks

**Goal:** Measure full check pipeline performance on Shell fixtures and compare with Rust adapter.

**Hyperfine comparisons:**

```bash
cargo build --release

# Shell fixtures
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/shell-scripts' \
    './target/release/quench check tests/fixtures/shell' \
    --export-markdown reports/shell-fixtures.md

# Compare bench-shell vs bench-rust (similar sizes)
hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/bench-shell' \
    './target/release/quench check tests/fixtures/bench-rust' \
    --export-markdown reports/shell-vs-rust.md

# Cold vs warm on bench-shell
rm -rf tests/fixtures/bench-shell/.quench
hyperfine --warmup 0 --runs 5 \
    './target/release/quench check tests/fixtures/bench-shell' \
    --export-json reports/bench-shell-cold.json

hyperfine --warmup 2 --runs 10 \
    './target/release/quench check tests/fixtures/bench-shell' \
    --export-json reports/bench-shell-warm.json
```

**Expected results:**

| Fixture | Mode | Target | Expected |
|---------|------|--------|----------|
| bench-shell | Cold | < 500ms | ~80-100ms |
| bench-shell | Warm | < 100ms | ~15-25ms |
| shell-scripts | Cold | < 100ms | ~15-20ms |

**Overhead analysis:** Shell adapter should add ≤ 0% overhead vs Rust adapter (simpler operations).

**Milestone:** End-to-end benchmark numbers recorded for Shell fixtures.

**Status:** [ ] Pending

---

### Phase 4: Profile and Analyze

**Goal:** Identify any unexpected hotspots and document performance breakdown.

**Analysis checklist:**

- [ ] What % of time in `ShellAdapter::new()` (GlobSet compilation)?
- [ ] What % of time in `classify()` per file?
- [ ] What % of time in `parse_shellcheck_suppresses()`?
- [ ] Is the Shell adapter overhead negligible (< 5%)?

**Expected breakdown for Shell project check:**

| Phase | Expected % | Notes |
|-------|------------|-------|
| File discovery | 35-45% | Using ignore crate |
| Adapter creation | < 0.5% | One-time cost, fewer patterns than Rust |
| File classification | 3-8% | Per file, simpler GlobSet |
| Suppress parsing | 5-10% | Only files with shellcheck directives |
| Check execution | 35-45% | CLOC, escapes, etc. |
| Output | < 5% | JSON/text generation |

**Comparison with Rust adapter (from 4D):**

| Operation | Rust | Shell (Expected) |
|-----------|------|------------------|
| Adapter creation | 62.3µs | ~40-50µs |
| classify() per 1K files | 0.11µs/file | ~0.08-0.12µs/file |
| Suppress parsing per 100 lines | 5.7µs | ~3-5µs |
| Line classification | 14.3µs | N/A (not needed) |

**Milestone:** Performance breakdown documented, no unexpected hotspots.

**Status:** [ ] Pending

---

### Phase 5: Document Results

**Goal:** Create comprehensive benchmark report at `reports/checkpoint-5d-benchmarks.md`.

**Report template:**

```markdown
# Checkpoint 5D: Benchmark Report - Shell Adapter

Generated: YYYY-MM-DD
Hardware: [CPU, RAM, OS]

## Summary

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| bench-shell cold | < 500ms | XXms | pass/fail |
| bench-shell warm | < 100ms | XXms | pass/fail |
| Shell vs Rust overhead | ≤ 0% | X% | pass/fail |
| classify() per 1K files | < 10ms | Xms | pass/fail |
| parse_shellcheck_suppresses() | < 0.1ms | Xms | pass/fail |

## Detailed Results

### 1. End-to-End Benchmarks

**bench-shell (550 files, ~27.5K LOC):**

| Run | Mean | Std Dev | Min | Max |
|-----|------|---------|-----|-----|
| Cold | | | | |
| Warm | | | | |

**Comparison with Rust adapter (bench-rust):**

| Fixture | Adapter | Files | Cold | Warm |
|---------|---------|-------|------|------|
| bench-shell | ShellAdapter | 550 | | |
| bench-rust | RustAdapter | 510 | | |
| Difference | | | X% | X% |

### 2. Adapter Micro-Benchmarks

**Adapter creation:**

| Adapter | Time | Notes |
|---------|------|-------|
| ShellAdapter::new() | | 2 source + 4 test patterns |
| RustAdapter::new() | 62.3µs | (4D baseline) |

**File classification (1K files):**

| Operation | Time | Per-file |
|-----------|------|----------|
| shell_classify scripts | | |
| shell_classify bats tests | | |
| rust_classify (baseline) | 109.8µs | 0.11µs |

**Shellcheck suppress parsing:**

| Content | Time |
|---------|------|
| 100 lines with suppresses | |
| 100 lines without | |
| 1000 lines with suppresses | |

### 3. Per-Module Breakdown

| Module | LOC | Complexity | Performance Notes |
|--------|-----|------------|-------------------|
| mod.rs | 116 | Low | GlobSet match, 6 patterns |
| suppress.rs | 83 | Low | String split, O(lines) |
| policy.rs | 32 | Low | Uses common utility |

## Conclusions

[Summary comparing Shell vs Rust adapter performance]

## Recommendations

[Any findings or confirmations]
```

**Milestone:** Report complete at `reports/checkpoint-5d-benchmarks.md`.

**Status:** [ ] Pending

## Key Implementation Details

### Shell Adapter Performance Model

The Shell adapter is simpler than Rust, with reduced overhead sources:

```
Total Overhead = Startup + Per-File
```

| Phase | Cost | When |
|-------|------|------|
| Startup | O(patterns) | Once per run (6 patterns vs 9 for Rust) |
| Per-file classify | O(patterns) | Every .sh/.bash/.bats file |
| Suppress parsing | O(lines) | Only when extracting suppresses |

**No line-level parsing** - Unlike Rust's `#[cfg(test)]` detection, Shell has no equivalent. All lines in a source file are counted as source; test files are identified purely by path patterns.

### GlobSet Pattern Comparison

| Adapter | Source Patterns | Test Patterns | Total |
|---------|-----------------|---------------|-------|
| Shell | 2 (`**/*.sh`, `**/*.bash`) | 4 (`tests/**/*.bats`, etc.) | 6 |
| Rust | 4 (`**/*.rs`, workspace paths) | 4 + ignore | 9 |

Fewer patterns = faster GlobSet compilation and matching.

### Shellcheck Suppress Parsing

The `parse_shellcheck_suppresses()` function:
- Time: O(lines) - single pass through content
- Memory: O(suppresses) - stores only found directives
- Simpler than Rust: String split on `,` vs regex for attribute parsing

```rust
// Shell: simple string operations
"SC2034,SC2086".split(',').map(|s| s.trim())

// Rust: regex matching for attributes
#[allow(dead_code, unused_variables)]
```

### Comparison Points

| Aspect | Shell Adapter | Rust Adapter |
|--------|---------------|--------------|
| Creation | ~40-50µs (6 patterns) | 62.3µs (9 patterns) |
| classify() | Same complexity | Same complexity |
| Line-level | Not supported | CfgTestInfo parsing |
| Suppress parsing | String split | Regex matching |
| Default escapes | 2 patterns | 2 patterns |
| Workspace | Not applicable | Cargo.toml parsing |

## Verification Plan

1. **Fixture generation:**
   ```bash
   ./scripts/gen-shell-fixture tests/fixtures/bench-shell 100 400
   find tests/fixtures/bench-shell -name '*.sh' -o -name '*.bats' | wc -l  # ~550
   ```

2. **Micro-benchmarks:**
   ```bash
   cargo bench --bench adapter -- shell
   ```

3. **End-to-end benchmarks:**
   ```bash
   hyperfine './target/release/quench check tests/fixtures/bench-shell'
   ```

4. **Report generation:**
   ```bash
   ls reports/checkpoint-5d-benchmarks.md
   ```

5. **Quality gates:**
   ```bash
   make check
   ```

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Create Shell benchmark fixture | [ ] Pending |
| 2 | Add Shell adapter benchmarks | [ ] Pending |
| 3 | Run end-to-end benchmarks | [ ] Pending |
| 4 | Profile and analyze | [ ] Pending |
| 5 | Document results in report | [ ] Pending |

## Notes

- The `bench-shell` fixture should be added to `.gitignore` to avoid repository bloat
- Shell adapter overhead is expected to be negligible (< 5%) given its simplicity
- This benchmark validates that the refactoring in 5c maintained performance
- Results should be compared against 4D Rust baseline for cross-adapter insights
