# Checkpoint 1D: Benchmark Analysis - CLI Runs

**Root Feature:** `quench-409c`

## Overview

Run criterion benchmarks and profile CLI performance to identify bottlenecks. This checkpoint establishes baseline measurements and documents whether current implementation meets performance targets defined in `docs/specs/20-performance.md`.

## Project Structure

```
quench/
├── crates/cli/benches/        # Existing criterion benchmarks
│   ├── baseline.rs            # CLI startup --help/--version
│   ├── file_walking.rs        # Walker isolation benchmarks
│   └── check.rs               # End-to-end check benchmarks
├── tests/fixtures/            # Generated benchmark fixtures
│   ├── bench-small/           # 50 files, 5K LOC
│   ├── bench-medium/          # 500 files, 50K LOC (primary target)
│   ├── bench-large/           # 5000 files, 500K LOC
│   ├── bench-deep/            # 1000 files, 55+ levels deep
│   └── bench-large-files/     # 100 files, ~10MB total
├── reports/
│   └── checkpoint-1-benchmarks.md  # Output: benchmark results
└── scripts/
    ├── gen-fixtures           # Fixture generator
    └── bench-ci               # CI benchmark runner
```

## Dependencies

Already present in `crates/cli/Cargo.toml`:
- `criterion` with `html_reports` feature - benchmarking framework

Profiling tools (system-level):
- **macOS**: `xcrun xctrace` (Time Profiler) - built into Xcode
- **Linux**: `perf` - system profiler
- **Cross-platform**: `flamegraph` crate - `cargo install flamegraph`

Optional for detailed analysis:
- `hyperfine` - command-line benchmarking (`brew install hyperfine`)

## Implementation Phases

### Phase 1: Generate Fixtures and Run Criterion Benchmarks

**Goal:** Collect baseline benchmark data from criterion suite.

**Tasks:**
1. Generate benchmark fixtures if not present:
   ```bash
   ./scripts/gen-fixtures
   ```

2. Build release binary:
   ```bash
   cargo build --release
   ```

3. Run criterion benchmarks:
   ```bash
   make bench
   # Or directly:
   # cargo bench --bench baseline
   # cargo bench --bench file_walking
   # cargo bench --bench check
   ```

4. Save baseline for future comparison:
   ```bash
   ./scripts/bench-ci --save-baseline
   ```

**Output:** Criterion reports in `target/criterion/` with HTML reports.

**Verification:** Benchmark completes without errors, HTML reports generated.

---

### Phase 2: Profile CLI Startup Time

**Goal:** Measure time from process start to argument parsing complete.

**Target:** < 50ms

**Tasks:**
1. Use `hyperfine` for statistical measurement:
   ```bash
   hyperfine --warmup 3 './target/release/quench --version'
   hyperfine --warmup 3 './target/release/quench --help'
   ```

2. Extract startup time from criterion `baseline` benchmark results.

3. If above target, profile with flamegraph:
   ```bash
   cargo flamegraph --bench baseline -- --bench 'version'
   ```

**Key areas to examine:**
- `clap` argument parsing overhead
- `tracing_subscriber` initialization
- Static initialization (regex compilation, etc.)

**Verification:** Startup time documented with statistical confidence interval.

---

### Phase 3: Profile Config Discovery Time

**Goal:** Measure time to find and parse `quench.toml`.

**Target:** < 10ms typical

**Tasks:**
1. Create isolated config discovery benchmark or use timing instrumentation:
   ```bash
   QUENCH_LOG=quench::config=debug hyperfine --warmup 3 \
     './target/release/quench check --config-only tests/fixtures/minimal'
   ```

2. Measure both scenarios:
   - Config present in target directory
   - Config absent (walk up to filesystem root)

3. If above target, profile config parsing:
   - TOML parsing time
   - Config validation time
   - Pattern compilation time (if patterns in config)

**Verification:** Config discovery time documented for both present/absent cases.

---

### Phase 4: Profile File Walking Time

**Goal:** Measure file discovery performance in isolation.

**Target:** < 200ms on 50K LOC (bench-medium fixture)

**Tasks:**
1. Use existing `file_walking` benchmark on bench-medium:
   ```bash
   cargo bench --bench file_walking -- 'bench-medium'
   ```

2. Compare single-threaded vs parallel walking.

3. Measure component breakdown:
   - Directory traversal time
   - Gitignore matching overhead
   - File type filtering time

4. If above target, identify bottleneck:
   ```bash
   cargo flamegraph --bench file_walking -- --bench 'bench-medium/parallel'
   ```

**Key metrics:**
- Files discovered per second
- Gitignore patterns evaluated per file
- Thread utilization

**Verification:** File walking time on bench-medium documented.

---

### Phase 5: Run End-to-End Benchmarks

**Goal:** Measure full check pipeline performance.

**Tasks:**
1. Run check benchmark on all fixtures:
   ```bash
   cargo bench --bench check
   ```

2. Use `hyperfine` for real-world measurement:
   ```bash
   hyperfine --warmup 3 \
     './target/release/quench check tests/fixtures/bench-medium'
   ```

3. Compare against performance targets from spec:

   | Mode | Target | Acceptable | Unacceptable |
   |------|--------|------------|--------------|
   | Fast checks (cold) | < 500ms | < 1s | > 2s |

4. Profile if exceeding targets:
   ```bash
   cargo flamegraph -- check tests/fixtures/bench-medium
   ```

**Verification:** End-to-end times documented for all fixtures.

---

### Phase 6: Document Results and Bottlenecks

**Goal:** Write comprehensive benchmark report.

**Output file:** `reports/checkpoint-1-benchmarks.md`

**Report structure:**
```markdown
# Checkpoint 1D: Benchmark Analysis

Generated: YYYY-MM-DD

## Summary

| Component | Target | Measured | Status |
|-----------|--------|----------|--------|
| CLI startup | <50ms | Xms ± Y | ✓/✗ |
| Config discovery | <10ms | Xms ± Y | ✓/✗ |
| File walking (50K LOC) | <200ms | Xms ± Y | ✓/✗ |
| Full check (50K LOC) | <500ms | Xms ± Y | ✓/✗ |

## Detailed Results

### Criterion Benchmarks
[Benchmark output summary]

### Profiling Findings
[Flamegraph analysis if bottlenecks found]

### Bottlenecks Identified
[Ordered list of performance issues]

### Recommendations
[P0/P1/P2 optimization priorities if needed]

## Environment

- Platform: [OS version]
- CPU: [model, cores]
- Rust version: [rustc --version]
- Build profile: release (LTO enabled)
```

**Verification:** Report written with all sections complete.

## Key Implementation Details

### Benchmarking Commands Summary

```bash
# Generate fixtures
./scripts/gen-fixtures

# Build release binary
cargo build --release

# Run all criterion benchmarks
make bench

# Individual benchmarks
cargo bench --bench baseline        # Startup
cargo bench --bench file_walking    # Walker isolation
cargo bench --bench check           # End-to-end

# Save baseline for CI
./scripts/bench-ci --save-baseline

# Statistical measurement
hyperfine --warmup 3 './target/release/quench --version'
hyperfine --warmup 3 './target/release/quench check tests/fixtures/bench-medium'

# Profiling (macOS)
xcrun xctrace record --template 'Time Profiler' --launch \
  ./target/release/quench check tests/fixtures/bench-medium

# Profiling (flamegraph)
cargo flamegraph -- check tests/fixtures/bench-medium
```

### Performance Targets Reference

From `docs/specs/20-performance.md`:

| Mode | Target | Acceptable | Unacceptable |
|------|--------|------------|--------------|
| Fast checks (cold) | < 500ms | < 1s | > 2s |
| Fast checks (warm) | < 100ms | < 200ms | > 500ms |
| CI checks | < 5s | < 15s | > 30s |

**Checkpoint-specific targets:**
- CLI startup: < 50ms
- Config discovery: < 10ms
- File walking (50K LOC): < 200ms

### Performance Model

Expected time distribution:

| Phase | % of Time | Dominant Factor |
|-------|-----------|-----------------|
| File discovery | 30-50% | Directory traversal, gitignore |
| File reading | 20-30% | I/O, filesystem latency |
| Pattern matching | 20-40% | CPU, pattern complexity |
| Aggregation/output | <5% | Negligible |

## Verification Plan

1. **Fixtures exist:** `./scripts/gen-fixtures --verify` returns success
2. **Benchmarks run:** `make bench` completes without errors
3. **Report complete:** `reports/checkpoint-1-benchmarks.md` contains all sections
4. **Targets documented:** Each target has measured value with status
5. **Bottlenecks listed:** Any components exceeding targets have analysis
6. **Make check passes:** `make check` validates all tests pass

### Success Criteria

- [ ] All criterion benchmarks execute successfully
- [ ] CLI startup time measured and documented
- [ ] Config discovery time measured and documented
- [ ] File walking time on bench-medium measured and documented
- [ ] Full check time compared against spec targets
- [ ] Bottlenecks identified and prioritized (if any)
- [ ] Report written to `reports/checkpoint-1-benchmarks.md`
