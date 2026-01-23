# Phase 045: Performance - Caching (P0)

## Overview

Implement file-level caching to achieve 10x speedup on iterative runs. Caching is P0 per the performance spec because agents iterate repeatedly on the same codebase—most runs are re-runs where few files changed.

**Target:** Warm run <100ms (down from ~500ms cold run)

## Project Structure

```
crates/cli/src/
├── cache.rs           # NEW: Cache types and logic
├── cache_tests.rs     # NEW: Unit tests
├── cli.rs             # MODIFY: Add --no-cache flag
├── runner.rs          # MODIFY: Integrate cache lookup
├── main.rs            # MODIFY: Initialize/persist cache
└── error.rs           # MODIFY: Add cache error variants
.quench/
└── cache.bin          # Persistent cache file (bincode format)
```

## Dependencies

Already available in workspace:
- `serde` / `serde_json` - serialization
- `thiserror` - error types

New dependencies:
- `bincode` - compact binary serialization for cache file
- `dashmap` - concurrent hashmap for thread-safe cache access

## Implementation Phases

### Phase 1: Core Cache Types

Define the cache data structures and basic operations.

**Files:** `crates/cli/src/cache.rs`, `crates/cli/src/cache_tests.rs`

**Types:**

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Cache version for invalidation on format changes
const CACHE_VERSION: u32 = 1;

/// Metadata used as cache key for a single file
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileCacheKey {
    pub mtime_secs: i64,
    pub mtime_nanos: u32,
    pub size: u64,
}

/// Cached result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileResult {
    pub key: FileCacheKey,
    pub violations: Vec<CachedViolation>,
}

/// Minimal violation data for cache storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedViolation {
    pub check: String,
    pub line: Option<u32>,
    pub violation_type: String,
    pub advice: String,
}

/// Persistent cache structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentCache {
    pub version: u32,
    pub quench_version: String,
    pub config_hash: u64,
    pub files: HashMap<PathBuf, CachedFileResult>,
}

/// Runtime cache wrapper
pub struct FileCache {
    inner: DashMap<PathBuf, CachedFileResult>,
    config_hash: u64,
    quench_version: String,
}
```

**Key Methods:**

```rust
impl FileCache {
    pub fn new(config_hash: u64) -> Self;
    pub fn from_persistent(path: &Path, config_hash: u64) -> Result<Self>;
    pub fn lookup(&self, path: &Path, key: &FileCacheKey) -> Option<Vec<CachedViolation>>;
    pub fn insert(&self, path: PathBuf, key: FileCacheKey, violations: Vec<CachedViolation>);
    pub fn persist(&self, path: &Path) -> Result<()>;
    pub fn stats(&self) -> CacheStats;
}

pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub entries: usize,
}
```

**Milestone:** Cache types compile, unit tests for serialization pass.

---

### Phase 2: Cache Lookup Integration

Integrate cache lookup into the check runner.

**Files:** `crates/cli/src/runner.rs`, `crates/cli/src/check.rs`

**Changes to `runner.rs`:**

```rust
pub struct CheckRunner {
    checks: Vec<Arc<dyn Check>>,
    config: RunnerConfig,
    cache: Option<Arc<FileCache>>,  // NEW
}

impl CheckRunner {
    pub fn with_cache(mut self, cache: Arc<FileCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    fn run_check_for_file(&self, check: &dyn Check, file: &WalkedFile, ctx: &CheckContext) -> Vec<Violation> {
        // Check cache first
        if let Some(cache) = &self.cache {
            let key = FileCacheKey::from_metadata(&file.metadata);
            if let Some(cached) = cache.lookup(&file.path, &key) {
                return cached.into_iter()
                    .filter(|v| v.check == check.name())
                    .map(Violation::from)
                    .collect();
            }
        }

        // Cache miss: run check
        let violations = check.run(ctx);

        // Populate cache
        if let Some(cache) = &self.cache {
            let key = FileCacheKey::from_metadata(&file.metadata);
            cache.insert(file.path.clone(), key, violations.iter().map(CachedViolation::from).collect());
        }

        violations
    }
}
```

**FileCacheKey extraction:**

```rust
impl FileCacheKey {
    pub fn from_metadata(meta: &std::fs::Metadata) -> Self {
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let duration = mtime.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        Self {
            mtime_secs: duration.as_secs() as i64,
            mtime_nanos: duration.subsec_nanos(),
            size: meta.len(),
        }
    }
}
```

**Milestone:** Cache lookup integrated, `--no-cache` behavior verifiable.

---

### Phase 3: Cache Population

Populate cache after processing and track statistics.

**Files:** `crates/cli/src/runner.rs`, `crates/cli/src/cache.rs`

**Atomic counters for stats:**

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct FileCache {
    inner: DashMap<PathBuf, CachedFileResult>,
    config_hash: u64,
    quench_version: String,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl FileCache {
    pub fn lookup(&self, path: &Path, key: &FileCacheKey) -> Option<Vec<CachedViolation>> {
        if let Some(entry) = self.inner.get(path) {
            if entry.key == *key {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.violations.clone());
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: self.inner.len(),
        }
    }
}
```

**Milestone:** Cache populates on first run, subsequent lookups return hits.

---

### Phase 4: Persistent Cache

Save cache to disk between sessions.

**Files:** `crates/cli/src/cache.rs`, `crates/cli/src/main.rs`

**Cache file location:** `.quench/cache.bin` relative to project root.

**Serialization:**

```rust
impl FileCache {
    pub fn persist(&self, path: &Path) -> Result<(), CacheError> {
        let cache = PersistentCache {
            version: CACHE_VERSION,
            quench_version: self.quench_version.clone(),
            config_hash: self.config_hash,
            files: self.inner.iter().map(|e| (e.key().clone(), e.value().clone())).collect(),
        };

        // Write atomically via temp file
        let temp_path = path.with_extension("tmp");
        let file = File::create(&temp_path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, &cache)?;
        std::fs::rename(&temp_path, path)?;
        Ok(())
    }

    pub fn from_persistent(path: &Path, config_hash: u64) -> Result<Self, CacheError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cache: PersistentCache = bincode::deserialize_from(reader)?;

        // Validate version
        if cache.version != CACHE_VERSION {
            return Err(CacheError::VersionMismatch);
        }
        if cache.quench_version != env!("CARGO_PKG_VERSION") {
            return Err(CacheError::QuenchVersionMismatch);
        }
        if cache.config_hash != config_hash {
            return Err(CacheError::ConfigChanged);
        }

        Ok(Self {
            inner: cache.files.into_iter().collect(),
            config_hash,
            quench_version: cache.quench_version,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        })
    }
}
```

**Main integration:**

```rust
// In main.rs or run_check()
fn run_check(args: &CheckArgs) -> Result<ExitCode> {
    let config = load_config()?;
    let config_hash = hash_config(&config);

    // Load or create cache
    let cache = if args.no_cache {
        None
    } else {
        let cache_path = root.join(".quench/cache.bin");
        match FileCache::from_persistent(&cache_path, config_hash) {
            Ok(cache) => Some(Arc::new(cache)),
            Err(_) => Some(Arc::new(FileCache::new(config_hash))),
        }
    };

    // ... run checks ...

    // Persist cache
    if let Some(cache) = &cache {
        let cache_path = root.join(".quench/cache.bin");
        std::fs::create_dir_all(cache_path.parent().unwrap())?;
        let _ = cache.persist(&cache_path); // Best effort
    }
}
```

**Milestone:** Cache persists across sessions, warm runs use disk cache.

---

### Phase 5: Cache Invalidation

Handle config changes, version changes, and CLI flag.

**Files:** `crates/cli/src/cli.rs`, `crates/cli/src/cache.rs`, `crates/cli/src/config.rs`

**CLI flag:**

```rust
// In cli.rs CheckArgs
#[derive(Parser)]
pub struct CheckArgs {
    // ... existing args ...

    /// Bypass the cache (force fresh check)
    #[arg(long)]
    pub no_cache: bool,
}
```

**Config hashing:**

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn hash_config(config: &Config) -> u64 {
    let mut hasher = DefaultHasher::new();
    // Hash relevant config fields that affect check results
    config.check.hash(&mut hasher);
    hasher.finish()
}
```

**Invalidation scenarios:**

| Trigger | Action |
|---------|--------|
| File mtime changed | Re-check that file |
| File size changed | Re-check that file |
| Config changed (hash mismatch) | Invalidate entire cache |
| Quench version changed | Invalidate entire cache |
| `--no-cache` flag | Skip cache entirely |

**Milestone:** Cache correctly invalidates on config/version changes.

---

### Phase 6: Benchmarking & Verification

Verify performance targets are met.

**Files:** `tests/specs/`, benchmark scripts

**Benchmark setup:**

Create `tests/fixtures/bench-medium/` with ~500 files, ~50K LOC to match spec targets.

**Benchmark script:**

```bash
#!/bin/bash
# scripts/bench-cache.sh

set -e
PROJECT="tests/fixtures/bench-medium"

echo "=== Cold run (no cache) ==="
rm -rf "$PROJECT/.quench/cache.bin"
time cargo run --release -- check "$PROJECT" --no-cache

echo "=== Warm run (with cache) ==="
time cargo run --release -- check "$PROJECT"

echo "=== Second warm run ==="
time cargo run --release -- check "$PROJECT"
```

**Performance targets:**

| Run | Target | Acceptable |
|-----|--------|------------|
| Cold | <500ms | <1s |
| Warm | <100ms | <200ms |

**Spec test:**

```rust
// tests/specs/cache_spec.rs
#[test]
fn warm_run_faster_than_cold() {
    let project = TestProject::bench_medium();

    // Cold run
    project.remove_cache();
    let cold = project.timed_check();

    // Warm run
    let warm = project.timed_check();

    // Warm should be significantly faster
    assert!(warm < cold / 2, "Warm run should be at least 2x faster");
}

#[test]
fn no_cache_flag_bypasses_cache() {
    let project = TestProject::bench_medium();

    // Populate cache
    project.run_check();

    // Touch a file
    project.touch_file("src/main.rs");

    // With cache: uses cached results (wrong)
    // Without cache: re-checks (correct)
    let with_cache = project.run_check();
    let without_cache = project.run_check_with(&["--no-cache"]);

    // Results should differ if file change introduced violation
}
```

**Milestone:** Warm run <100ms on bench-medium, all cache specs pass.

---

## Key Implementation Details

### Thread Safety

The cache must be thread-safe since checks run in parallel via rayon:

- Use `DashMap` for concurrent read/write access
- Atomic counters for hit/miss statistics
- No locks on the hot path (lookup is read-only)

### Cache Key Design

```
CacheKey = (path, mtime_secs, mtime_nanos, size)
```

Why not content hash?
- Reading file to hash defeats caching benefit
- mtime+size is sufficient for iterative development
- Content hash useful for CI (different concern)

### Binary Format

Use `bincode` instead of JSON for cache file:
- 10x smaller file size
- 10x faster serialization
- Cache file is internal, not user-facing

### Error Handling

Cache errors are non-fatal:
- Failed to load cache → start fresh
- Failed to persist cache → log warning, continue
- Corrupted cache → delete and start fresh

### Memory Budget

Per performance spec, target <100MB memory for fast checks:
- ~50K files × 200 bytes per entry = ~10MB cache overhead
- Well within budget

---

## Verification Plan

### Unit Tests (`cache_tests.rs`)

1. **Serialization roundtrip** - PersistentCache survives bincode serialize/deserialize
2. **Cache hit detection** - Same mtime+size returns cached result
3. **Cache miss on mtime change** - Modified file triggers re-check
4. **Cache miss on size change** - Resized file triggers re-check
5. **Config hash invalidation** - Changed config clears cache
6. **Version invalidation** - Changed quench version clears cache

### Integration Tests (`tests/specs/`)

1. **Warm run performance** - <100ms on bench-medium fixture
2. **--no-cache bypasses cache** - Fresh check every time
3. **Cache persists across runs** - .quench/cache.bin created and reused
4. **File modification detected** - Touched file re-checked
5. **Config change invalidates** - Modified quench.toml clears cache

### Manual Verification

```bash
# 1. Build release binary
cargo build --release

# 2. Run cold check
rm -rf .quench/cache.bin
time ./target/release/quench check tests/fixtures/bench-medium

# 3. Run warm check (should be <100ms)
time ./target/release/quench check tests/fixtures/bench-medium

# 4. Verify cache file exists
ls -la tests/fixtures/bench-medium/.quench/cache.bin

# 5. Test --no-cache flag
time ./target/release/quench check tests/fixtures/bench-medium --no-cache
```

---

## Summary

| Phase | Deliverable | Verification |
|-------|-------------|--------------|
| 1 | Cache types | Unit tests pass |
| 2 | Cache lookup | Integration compiles |
| 3 | Cache population | Hits observed in stats |
| 4 | Persistent cache | cache.bin created |
| 5 | Invalidation | Config/version changes clear cache |
| 6 | Benchmarks | Warm run <100ms |

**Expected outcome:** 10x speedup on iterative runs (500ms → 50ms), directly serving the primary agent use case.
