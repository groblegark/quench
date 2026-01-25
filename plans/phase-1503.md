# Phase 1503: DRY Refactoring - Production and Test Code

**Root Feature:** `quench-code-quality`
**Depends On:** Phase 1502 (Close Review Gaps)

## Overview

Address code duplication, verbosity, and DRY violations identified in the comprehensive code review. This phase covers 15 high and medium priority issues across production code (8 issues) and unit tests (7 issues).

**Estimated Impact:** ~330 lines reducible, improved maintainability

## Project Structure

```
quench/crates/cli/src/
├── file_size.rs              # UPDATE: Consolidate human_size functions
├── walker.rs                 # UPDATE: Extract build_walked_file helper
├── git.rs                    # UPDATE: Extract commit iteration helper
├── cmd_init.rs               # UPDATE: Add enum helper methods
├── cmd_check.rs              # UPDATE: Extract workspace detection pattern
├── runner.rs                 # UPDATE: Extract CheckContext builder
├── report/
│   ├── mod.rs                # UPDATE: Extract create_formatter helper
│   ├── text.rs               # UPDATE: Use shared metrics iterator
│   ├── html.rs               # UPDATE: Use shared metrics iterator
│   └── markdown.rs           # UPDATE: Use shared metrics iterator
├── checks/git/
│   └── mod_tests.rs          # UPDATE: Add Commit::test helper
├── test_helpers.rs           # CREATE: Shared test utilities
├── file_size_tests.rs        # UPDATE: Parameterize tests
├── git_tests.rs              # UPDATE: Use test helpers
├── walker_tests.rs           # UPDATE: Use config helper
└── report/
    ├── mod_tests.rs          # UPDATE: Use parameterized tests
    ├── text_tests.rs         # UPDATE: Use assertion helpers
    ├── html_tests.rs         # UPDATE: Use assertion helpers
    └── markdown_tests.rs     # UPDATE: Use assertion helpers
```

## Implementation Phases

---

### Phase 1: Production Code - HIGH Priority

#### 1.1 Consolidate Human Size Formatting

**Problem:** Two nearly identical functions with different spacing.

**Files:**
- `file_size.rs:54-65` - `human_size()` outputs `"1.5MB"`
- `report/mod.rs:232-242` - `human_bytes()` outputs `"1.5 MB"`

**Solution:** Keep one canonical function, add optional spacing parameter.

**File:** `crates/cli/src/file_size.rs`

```rust
/// Format bytes as human-readable string.
///
/// With `spaced: false`: "1.5MB", "256KB", "100B"
/// With `spaced: true`:  "1.5 MB", "256 KB", "100 B"
pub fn human_size(bytes: u64, spaced: bool) -> String {
    let space = if spaced { " " } else { "" };
    if bytes >= 1_000_000 {
        format!("{:.1}{space}MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}{space}KB", bytes as f64 / 1_000.0)
    } else {
        format!("{}{space}B", bytes)
    }
}
```

**File:** `crates/cli/src/report/mod.rs`

```rust
// Remove human_bytes() function, use:
use crate::file_size::human_size;

// Replace calls: human_bytes(size) → human_size(size, true)
```

**Verification:**
```bash
cargo test -p quench -- human_size
cargo test -p quench -- report
```

---

#### 1.2 Extract Walker File Builder

**Problem:** Identical WalkedFile construction in parallel and sequential paths (~36 lines duplicated).

**File:** `crates/cli/src/walker.rs`

**Before (lines 307-326 and 398-417):**
```rust
let (mtime_secs, mtime_nanos) = meta.as_ref().ok()
    .and_then(|m| m.modified().ok())
    .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default())
    .map(|d| (d.as_secs(), d.subsec_nanos()))
    .unwrap_or((0, 0));
let depth = entry.depth();
let size_class = FileSizeClass::from_size(size);
let walked = WalkedFile {
    path: entry.into_path(),
    size,
    mtime_secs,
    mtime_nanos,
    depth,
    size_class,
};
```

**After:** Add helper function:

```rust
/// Build WalkedFile from directory entry and metadata.
fn build_walked_file(entry: ignore::DirEntry, size: u64, meta: &std::io::Result<fs::Metadata>) -> WalkedFile {
    let (mtime_secs, mtime_nanos) = meta.as_ref().ok()
        .and_then(|m| m.modified().ok())
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default())
        .map(|d| (d.as_secs(), d.subsec_nanos()))
        .unwrap_or((0, 0));

    WalkedFile {
        path: entry.into_path(),
        size,
        mtime_secs,
        mtime_nanos,
        depth: entry.depth(),
        size_class: FileSizeClass::from_size(size),
    }
}
```

**Usage in both paths:**
```rust
let walked = build_walked_file(entry, size, &meta);
```

**Verification:**
```bash
cargo test -p quench -- walker
```

---

### Phase 2: Production Code - MEDIUM Priority

#### 2.1 Extract Commit Iteration Helper

**Problem:** Identical commit extraction logic in two functions (~10 lines duplicated).

**File:** `crates/cli/src/git.rs`

**Before (lines 91-102 and 124-131):**
```rust
for oid in revwalk {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;
    let hash = oid.to_string()[..7].to_string();
    let message = commit.summary().unwrap_or("").to_string();
    commits.push(Commit { hash, message });
}
```

**After:** Add helper function:

```rust
/// Collect commits from a revwalk into a Vec.
fn collect_commits(repo: &Repository, revwalk: git2::Revwalk) -> Result<Vec<Commit>> {
    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(Commit {
            hash: oid.to_string()[..7].to_string(),
            message: commit.summary().unwrap_or("").to_string(),
        });
    }
    Ok(commits)
}
```

**Verification:**
```bash
cargo test -p quench -- git
```

---

#### 2.2 Add Enum Helper Methods

**Problem:** Repeated match arms over DetectedLanguage and DetectedAgent enums.

**File:** `crates/cli/src/cmd_init.rs`

**After:** Add methods to enums:

```rust
impl DetectedLanguage {
    /// Short name for display/logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Golang => "golang",
            Self::JavaScript => "javascript",
            Self::Shell => "shell",
        }
    }

    /// Config section for this language.
    pub fn config_section(&self) -> &'static str {
        match self {
            Self::Rust => rust_detected_section(),
            Self::Golang => golang_detected_section(),
            Self::JavaScript => javascript_detected_section(),
            Self::Shell => shell_detected_section(),
        }
    }
}

impl DetectedAgent {
    /// Short name for display/logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Cursor(_) => "cursor",
        }
    }
}
```

**Usage:**
```rust
// Before:
for lang in &detected_langs {
    detected_names.push(match lang {
        DetectedLanguage::Rust => "rust",
        // ...
    });
}

// After:
for lang in &detected_langs {
    detected_names.push(lang.name());
}
```

**Verification:**
```bash
cargo test -p quench -- init
```

---

#### 2.3 Extract Workspace Detection Pattern

**Problem:** Similar workspace auto-detection for Rust and JavaScript (~80 lines structural duplication).

**File:** `crates/cli/src/cmd_check.rs`

**After:** Add generic helper:

```rust
/// Auto-detect workspace packages if not configured.
fn auto_detect_packages<W: Workspace>(
    root: &Path,
    packages: &mut Vec<String>,
    package_names: &mut HashMap<String, String>,
    workspace_fn: impl FnOnce(&Path) -> W,
) {
    if packages.is_empty() {
        let workspace = workspace_fn(root);
        if workspace.is_workspace() {
            *packages = workspace.package_paths();
            *package_names = workspace.package_names();
            debug!(
                "Auto-detected {} workspace with {} packages",
                W::LANGUAGE,
                packages.len()
            );
        }
    }
}

trait Workspace {
    const LANGUAGE: &'static str;
    fn is_workspace(&self) -> bool;
    fn package_paths(&self) -> Vec<String>;
    fn package_names(&self) -> HashMap<String, String>;
}
```

**Note:** This is a larger refactor. Consider if the ~80 lines saved justifies the abstraction complexity. May defer if workspace logic is unlikely to expand.

**Verification:**
```bash
cargo test -p quench -- check
```

---

#### 2.4 Extract Formatter Factory

**Problem:** Formatter dispatch duplicated in two functions (6 lines exact duplication).

**File:** `crates/cli/src/report/mod.rs`

**Before (lines 195-200 and 218-223):**
```rust
let formatter: Box<dyn ReportFormatter> = match format {
    OutputFormat::Text => Box::new(TextFormatter),
    OutputFormat::Json => Box::new(JsonFormatter::new(compact)),
    OutputFormat::Html => Box::new(HtmlFormatter),
    OutputFormat::Markdown => Box::new(MarkdownFormatter),
};
```

**After:** Add helper:

```rust
/// Create formatter for the given output format.
fn create_formatter(format: OutputFormat, compact: bool) -> Box<dyn ReportFormatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter),
        OutputFormat::Json => Box::new(JsonFormatter::new(compact)),
        OutputFormat::Html => Box::new(HtmlFormatter),
        OutputFormat::Markdown => Box::new(MarkdownFormatter),
    }
}
```

**Verification:**
```bash
cargo test -p quench -- report
```

---

#### 2.5 Extract CheckContext Builder

**Problem:** Nearly identical CheckContext construction in two paths.

**File:** `crates/cli/src/runner.rs`

**After:** Add builder method on RunnerConfig:

```rust
impl RunnerConfig {
    /// Create CheckContext with the given files reference.
    fn build_context<'a>(
        &'a self,
        root: &'a Path,
        files: &'a [WalkedFile],
        config: &'a Config,
        violation_count: &'a AtomicUsize,
    ) -> CheckContext<'a> {
        CheckContext {
            root,
            files,
            config,
            limit: self.limit,
            violation_count,
            changed_files: self.changed_files.as_deref(),
            fix: self.fix,
            dry_run: self.dry_run,
            ci_mode: self.ci_mode,
            base_branch: self.base_branch.as_deref(),
            staged: self.staged,
        }
    }
}
```

**Verification:**
```bash
cargo test -p quench -- runner
```

---

#### 2.6 Simplify Report Formatter Macros

**Problem:** Three separate macros with duplicated metric iteration logic.

**Files:** `report/text.rs`, `report/html.rs`, `report/markdown.rs`

**Approach:** Create a metrics visitor pattern or iterator that formatters consume.

**File:** `crates/cli/src/report/metrics_iter.rs` (new)

```rust
/// Iterate over all metrics in a baseline with their formatted values.
pub struct MetricsIter<'a> {
    baseline: &'a Baseline,
    filter: &'a dyn CheckFilter,
    index: usize,
}

pub struct MetricEntry {
    pub category: &'static str,
    pub name: &'static str,
    pub value: String,
    pub unit: Option<&'static str>,
}

impl<'a> Iterator for MetricsIter<'a> {
    type Item = MetricEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through: coverage, escapes, build_time, binary_size, test_time
        // Return formatted MetricEntry for each
    }
}
```

**Note:** This is a moderate refactor. Evaluate if the 3 macros are complex enough to warrant the abstraction.

**Verification:**
```bash
cargo test -p quench -- report
```

---

### Phase 3: Unit Tests - HIGH Priority

#### 3.1 Add Commit Test Helper

**Problem:** `Commit { hash: "...".to_string(), message: "...".to_string() }` repeated 30+ times.

**File:** `crates/cli/src/git.rs` (add to impl block)

```rust
#[cfg(test)]
impl Commit {
    /// Create commit for testing.
    pub fn test(hash: &str, message: &str) -> Self {
        Self {
            hash: hash.to_string(),
            message: message.to_string(),
        }
    }
}
```

**Usage in tests:**
```rust
// Before:
Commit { hash: "abc1234".to_string(), message: "feat: add feature".to_string() }

// After:
Commit::test("abc1234", "feat: add feature")
```

**Files to update:**
- `git_tests.rs`
- `checks/git/mod_tests.rs`
- `checks/git/docs_tests.rs`

**Verification:**
```bash
cargo test -p quench -- git
cargo test -p quench -- commit
```

---

#### 3.2 Add Formatter Test Helpers

**Problem:** Identical test setup pattern across 3 formatter test files.

**File:** `crates/cli/src/report/test_support.rs` (extend existing)

```rust
/// Assert formatter output contains all expected strings.
pub fn assert_format_contains<F: Formatter>(
    formatter: &F,
    baseline: &Baseline,
    filter: &dyn CheckFilter,
    expected: &[&str],
) {
    let output = formatter.format(baseline, filter).unwrap();
    for expected_str in expected {
        assert!(
            output.contains(expected_str),
            "Expected output to contain '{}', got:\n{}",
            expected_str,
            output
        );
    }
}

/// Assert buffered and streamed output match.
pub fn assert_buffered_matches_streamed<F: Formatter>(
    formatter: &F,
    baseline: &Baseline,
    filter: &dyn CheckFilter,
) {
    let buffered = formatter.format(baseline, filter).unwrap();
    let mut streamed = Vec::new();
    formatter.format_to(&mut streamed, baseline, filter).unwrap();
    let streamed_str = String::from_utf8(streamed).unwrap();
    assert_eq!(buffered, streamed_str);
}
```

**Files to update:**
- `report/text_tests.rs`
- `report/html_tests.rs`
- `report/markdown_tests.rs`

**Verification:**
```bash
cargo test -p quench -- report
```

---

#### 3.3 Parameterize File Size Tests

**Problem:** 12 near-identical test functions for file size classification.

**File:** `crates/cli/src/file_size_tests.rs`

**Before (~99 lines):**
```rust
#[test]
fn classify_small_file_1kb() {
    assert_eq!(FileSizeClass::from_size(1024), FileSizeClass::Small);
}

#[test]
fn classify_small_file_63kb() {
    assert_eq!(FileSizeClass::from_size(63 * 1024), FileSizeClass::Small);
}
// ... 10 more similar tests
```

**After (~25 lines):**
```rust
#[test]
fn file_size_classification() {
    let cases = [
        // (size, expected_class, description)
        (1024, FileSizeClass::Small, "1KB"),
        (63 * 1024, FileSizeClass::Small, "63KB"),
        (MMAP_THRESHOLD - 1, FileSizeClass::Small, "just under threshold"),
        (MMAP_THRESHOLD, FileSizeClass::Normal, "at threshold"),
        (100 * 1024, FileSizeClass::Normal, "100KB"),
        (500 * 1024, FileSizeClass::Normal, "500KB"),
        (1024 * 1024, FileSizeClass::Oversized, "1MB"),
        (5 * 1024 * 1024, FileSizeClass::Oversized, "5MB"),
        (10 * 1024 * 1024, FileSizeClass::TooLarge, "10MB"),
        (100 * 1024 * 1024, FileSizeClass::TooLarge, "100MB"),
    ];

    for (size, expected, desc) in cases {
        assert_eq!(
            FileSizeClass::from_size(size),
            expected,
            "Failed for {} ({} bytes)",
            desc,
            size
        );
    }
}

#[test]
fn human_size_formatting() {
    let cases = [
        (500, false, "500B"),
        (500, true, "500 B"),
        (1500, false, "1.5KB"),
        (1500, true, "1.5 KB"),
        (1_500_000, false, "1.5MB"),
        (1_500_000, true, "1.5 MB"),
    ];

    for (bytes, spaced, expected) in cases {
        assert_eq!(human_size(bytes, spaced), expected);
    }
}
```

**Verification:**
```bash
cargo test -p quench -- file_size
```

---

### Phase 4: Unit Tests - MEDIUM Priority

#### 4.1 Add Walker Config Helper

**Problem:** Similar WalkerConfig setup repeated 6+ times.

**File:** `crates/cli/src/walker_tests.rs`

```rust
/// Create WalkerConfig for testing with common defaults.
fn test_walker_config() -> WalkerConfig {
    WalkerConfig {
        git_ignore: false,
        hidden: false,
        ..Default::default()
    }
}

/// Create WalkerConfig with custom overrides.
fn test_walker_config_with(f: impl FnOnce(&mut WalkerConfig)) -> WalkerConfig {
    let mut config = test_walker_config();
    f(&mut config);
    config
}
```

**Usage:**
```rust
// Before:
let walker = FileWalker::new(WalkerConfig {
    git_ignore: false,
    hidden: false,
    max_depth: Some(2),
    ..Default::default()
});

// After:
let walker = FileWalker::new(test_walker_config_with(|c| c.max_depth = Some(2)));
```

**Verification:**
```bash
cargo test -p quench -- walker
```

---

#### 4.2 Add Git Test File Helper

**Problem:** Repeated `fs::write()` + `git_add()` pattern (~15 instances).

**File:** `crates/cli/src/git_tests.rs`

```rust
/// Create file and stage it in git.
fn create_and_stage(temp: &TempDir, filename: &str, content: &str) {
    std::fs::write(temp.path().join(filename), content).unwrap();
    git_add(temp, filename);
}

/// Create file, stage, and commit.
fn create_stage_commit(temp: &TempDir, filename: &str, content: &str, message: &str) {
    create_and_stage(temp, filename, content);
    git_commit(temp, message);
}
```

**Usage:**
```rust
// Before:
std::fs::write(temp.path().join("file.txt"), "content").unwrap();
git_add(&temp, "file.txt");
git_commit(&temp, "feat: add file");

// After:
create_stage_commit(&temp, "file.txt", "content", "feat: add file");
```

**Verification:**
```bash
cargo test -p quench -- git
```

---

#### 4.3 Add Baseline Filter Test Helper

**Problem:** Repeated baseline creation and filter assertion pattern.

**File:** `crates/cli/src/report/mod_tests.rs`

```rust
/// Assert that filtering baseline excludes expected metrics.
fn assert_filter_excludes(
    baseline: &Baseline,
    excluded: &[&str],
    should_be_none: &[&str],
    should_be_some: &[&str],
) {
    let filter = ExcludeChecks(excluded.iter().map(|s| s.to_string()).collect());
    let filtered = FilteredMetrics::new(baseline, &filter);

    for metric in should_be_none {
        assert!(
            get_metric(&filtered, metric).is_none(),
            "{} should be None when excluding {:?}",
            metric,
            excluded
        );
    }

    for metric in should_be_some {
        assert!(
            get_metric(&filtered, metric).is_some(),
            "{} should be Some when excluding {:?}",
            metric,
            excluded
        );
    }
}
```

**Verification:**
```bash
cargo test -p quench -- filter
```

---

### Phase 5: Final Verification

**Goal:** Ensure all refactoring passes tests and maintains behavior.

```bash
# Unit tests
cargo test -p quench

# Behavioral specs
cargo test --test specs

# Full CI check
make check

# Verify no performance regression
cargo bench -p quench
```

---

## Summary Table

| Phase | Category | Issues | Est. Lines Saved |
|-------|----------|--------|------------------|
| 1 | Production HIGH | 2 | ~50 |
| 2 | Production MEDIUM | 6 | ~100 |
| 3 | Tests HIGH | 3 | ~130 |
| 4 | Tests MEDIUM | 3 | ~50 |
| **Total** | | **14** | **~330** |

## Exit Criteria

**Production Code:**
- [ ] `human_size()` consolidated with spacing parameter
- [ ] `build_walked_file()` helper extracted in walker.rs
- [ ] `collect_commits()` helper extracted in git.rs
- [ ] Enum helper methods added to DetectedLanguage/DetectedAgent
- [ ] `create_formatter()` helper extracted in report/mod.rs
- [ ] `build_context()` method added to RunnerConfig
- [ ] Workspace detection pattern extracted (or deferred with justification)
- [ ] Report formatter macros simplified (or deferred with justification)

**Unit Tests:**
- [ ] `Commit::test()` helper added and used
- [ ] Formatter test helpers added to test_support.rs
- [ ] file_size_tests.rs parameterized (12 tests → 2-3)
- [ ] Walker config helper added
- [ ] Git file creation helpers added
- [ ] Baseline filter test helper added

**Quality:**
- [ ] `make check` passes
- [ ] No test count reduction (same coverage, less code)
- [ ] No performance regression

## Notes

### Deferral Candidates

Two items may be deferred if abstraction complexity outweighs benefit:

1. **Workspace detection pattern (2.3):** The trait-based abstraction adds complexity. If only Rust and JavaScript workspaces exist and no more are planned, the duplication may be acceptable.

2. **Report formatter macros (2.6):** The MetricsIter abstraction is elegant but adds indirection. If the macros are rarely modified, the duplication may be acceptable.

Document reasoning if deferring.
