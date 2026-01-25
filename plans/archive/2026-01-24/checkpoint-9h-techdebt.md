# Checkpoint 9H: Tech Debt - Git Check

**Root Feature:** `quench-971h`
**Follows:** checkpoint-9g-bugfix (deleted file handling fixes)

## Overview

Address tech debt in the git check module that accumulated during the rapid git2 migration phases. The primary issues are duplicate constants, missing merge commit handling, and gaps in integration test coverage.

**Goals:**
- Consolidate duplicate `DEFAULT_TYPES`/`COMMIT_TYPES` constants into a single source of truth
- Add merge commit skip option to avoid false positives from git-generated commit messages
- Add behavioral specs for the git check's `run()` method with real git repositories
- Minor improvements to code organization and documentation

## Project Structure

```
quench/
├── crates/cli/
│   └── src/
│       ├── checks/git/
│       │   ├── mod.rs              # MODIFY: add merge commit detection
│       │   ├── mod_tests.rs        # MODIFY: add merge commit tests
│       │   ├── parse.rs            # MODIFY: export DEFAULT_TYPES, add is_merge_commit
│       │   ├── parse_tests.rs      # MODIFY: add merge detection tests
│       │   ├── docs.rs             # MODIFY: use shared DEFAULT_TYPES
│       │   └── docs_tests.rs       # (no changes expected)
│       └── config/
│           └── mod.rs              # MODIFY: add skip_merge option
├── tests/
│   └── specs/checks/
│       └── git.rs                  # MODIFY: add run() integration tests
├── docs/specs/checks/
│   └── git.md                      # MODIFY: document skip_merge option
└── plans/
    └── checkpoint-9h-techdebt.md   # THIS FILE
```

## Dependencies

No new dependencies required. Uses existing:
- `git2 = "0.19"`
- `regex`

## Implementation Phases

### Phase 1: Consolidate Duplicate Constants

The `DEFAULT_TYPES` constant is duplicated between `parse.rs` and `docs.rs`. Consolidate to `parse.rs` (the canonical source) and re-export.

**File:** `crates/cli/src/checks/git/parse.rs`

Already exports `DEFAULT_TYPES`:
```rust
pub const DEFAULT_TYPES: &[&str] = &[
    "feat", "fix", "chore", "docs", "test", "refactor", "perf", "ci", "build", "style",
];
```

**File:** `crates/cli/src/checks/git/docs.rs`

**Current (duplicated):**
```rust
/// Default commit types to search for.
const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "chore", "docs", "test", "refactor", "perf", "ci", "build", "style",
];
```

**Fixed (use shared constant):**
```rust
use super::parse::DEFAULT_TYPES;

// Remove COMMIT_TYPES constant entirely
```

**Update regex initialization:**
```rust
// Before
let types_pattern = COMMIT_TYPES.join("|");

// After
let types_pattern = DEFAULT_TYPES.join("|");
```

**Verification:**
- `cargo test -p quench -- docs_tests` passes
- `cargo test -p quench -- parse_tests` passes

---

### Phase 2: Add Merge Commit Detection

Add helper to detect merge commits and optionally skip them during validation.

**File:** `crates/cli/src/checks/git/parse.rs`

Add detection function:
```rust
/// Check if a commit message is a merge commit.
///
/// Detects patterns like:
/// - "Merge branch 'feature' into main"
/// - "Merge pull request #123 from user/branch"
/// - "Merge remote-tracking branch 'origin/main'"
pub fn is_merge_commit(message: &str) -> bool {
    message.starts_with("Merge ")
}
```

**File:** `crates/cli/src/checks/git/parse_tests.rs`

Add tests:
```rust
// =============================================================================
// MERGE COMMIT DETECTION TESTS
// =============================================================================

#[test]
fn detects_merge_branch() {
    assert!(is_merge_commit("Merge branch 'feature' into main"));
}

#[test]
fn detects_merge_pull_request() {
    assert!(is_merge_commit("Merge pull request #123 from user/branch"));
}

#[test]
fn detects_merge_remote_tracking() {
    assert!(is_merge_commit("Merge remote-tracking branch 'origin/main'"));
}

#[test]
fn does_not_detect_conventional_commit() {
    assert!(!is_merge_commit("feat: add merge functionality"));
}

#[test]
fn does_not_detect_message_containing_merge() {
    assert!(!is_merge_commit("fix: merge conflict in parser"));
}
```

**Verification:**
- `cargo test -p quench -- is_merge_commit` passes

---

### Phase 3: Add skip_merge Configuration Option

Add config option to skip merge commits (enabled by default).

**File:** `crates/cli/src/config/mod.rs`

Update `GitCommitConfig`:
```rust
/// Git commit validation settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GitCommitConfig {
    /// Check level: "error" | "warn" | "off"
    pub check: Option<String>,

    /// Commit format: "conventional" | "none" (default: "conventional")
    pub format: Option<String>,

    /// Allowed commit types (None = use defaults, Some([]) = any type)
    pub types: Option<Vec<String>>,

    /// Allowed scopes (None = any scope allowed)
    pub scopes: Option<Vec<String>>,

    /// Check that commit format is documented in agent files (default: true)
    pub agents: bool,

    /// Create .gitmessage template with --fix (default: true)
    pub template: bool,

    /// Skip merge commits (e.g., "Merge branch 'x'") (default: true)
    #[serde(default = "default_true")]
    pub skip_merge: bool,
}

impl Default for GitCommitConfig {
    fn default() -> Self {
        Self {
            check: None,
            format: None,
            types: None,
            scopes: None,
            agents: true,
            template: true,
            skip_merge: true,  // Add this
        }
    }
}
```

**File:** `crates/cli/src/config/mod_tests.rs`

Add test:
```rust
#[test]
fn git_skip_merge_defaults_to_true() {
    let config = GitCommitConfig::default();
    assert!(config.skip_merge);
}

#[test]
fn git_skip_merge_can_be_disabled() {
    let toml = r#"
version = 1
[git.commit]
skip_merge = false
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(!config.git.commit.skip_merge);
}
```

**Verification:**
- `cargo test -p quench -- git_skip_merge` passes

---

### Phase 4: Integrate Merge Skip in Check Logic

Update the git check to skip merge commits when configured.

**File:** `crates/cli/src/checks/git/mod.rs`

Update imports:
```rust
pub use parse::{DEFAULT_TYPES, ParseResult, ParsedCommit, parse_conventional_commit, is_merge_commit};
```

Update `validate_commit` function signature and logic:
```rust
/// Validate a single commit and add violations if invalid.
///
/// Returns `true` if the commit was validated, `false` if skipped.
pub fn validate_commit(
    commit: &Commit,
    config: &GitCommitConfig,
    violations: &mut Vec<Violation>,
) -> bool {
    // Skip merge commits if configured
    if config.skip_merge && is_merge_commit(&commit.message) {
        return false; // Skipped
    }

    match parse_conventional_commit(&commit.message) {
        // ... existing logic unchanged
    }

    true // Validated
}
```

Update the loop in `run()` to track validated count:
```rust
// Validate each commit (if any)
let mut validated_count = 0;
for commit in &commits {
    if validate_commit(commit, config, &mut violations) {
        validated_count += 1;
    }
}

// Build metrics (only if commits were validated)
let metrics = if validated_count > 0 {
    let commits_with_violations = violations
        .iter()
        .filter_map(|v| v.commit.as_ref())
        .collect::<std::collections::HashSet<_>>()
        .len();

    Some(serde_json::json!({
        "commits_checked": validated_count,
        "commits_valid": validated_count - commits_with_violations,
        "commits_skipped": commits.len() - validated_count,
    }))
} else {
    None
};
```

**File:** `crates/cli/src/checks/git/mod_tests.rs`

Update existing test and add new tests:
```rust
// Update existing test to expect skip by default
#[test]
fn skips_merge_commit_by_default() {
    let commit = Commit {
        hash: "abc1234".to_string(),
        message: "Merge branch 'feature' into main".to_string(),
    };
    let config = GitCommitConfig::default();
    let mut violations = Vec::new();

    let validated = validate_commit(&commit, &config, &mut violations);

    assert!(!validated, "merge commit should be skipped");
    assert!(violations.is_empty());
}

#[test]
fn validates_merge_commit_when_skip_disabled() {
    let commit = Commit {
        hash: "abc1234".to_string(),
        message: "Merge branch 'feature' into main".to_string(),
    };
    let mut config = GitCommitConfig::default();
    config.skip_merge = false;
    let mut violations = Vec::new();

    let validated = validate_commit(&commit, &config, &mut violations);

    assert!(validated, "merge commit should be validated");
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].violation_type, "invalid_format");
}
```

**Verification:**
- `cargo test -p quench -- skips_merge` passes
- `cargo test -p quench -- validates_merge` passes

---

### Phase 5: Add Behavioral Specs

Add integration tests that exercise the full `run()` method with real git repositories.

**File:** `tests/specs/checks/git.rs`

Add integration tests:
```rust
/// Spec: docs/specs/checks/git.md#scope
///
/// > Merge commits should be skipped by default
#[test]
fn git_check_skips_merge_commits() {
    let temp = Project::empty();
    temp.config(
        r#"[git.commit]
check = "error"
agents = false
"#,
    );
    temp.file("CLAUDE.md", "# Project\n");

    git_init(&temp);
    git_initial_commit(&temp);

    // Create feature branch with valid commit
    git_branch(&temp, "feature");
    temp.file("feature.txt", "content");
    git_add_all(&temp);
    git_commit(&temp, "feat: add feature file");

    // Merge back to main (creates merge commit)
    git_checkout(&temp, "main");
    // Use --no-ff to force merge commit
    std::process::Command::new("git")
        .args(["merge", "--no-ff", "feature", "-m", "Merge branch 'feature'"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Should pass - merge commit is skipped, feat commit is valid
    check("git").pwd(temp.path()).args(&["--base", "HEAD~2"]).passes();
}

/// Spec: docs/specs/checks/git.md#scope
///
/// > Merge commits can be validated if skip_merge = false
#[test]
fn git_check_validates_merge_commits_when_configured() {
    let temp = Project::empty();
    temp.config(
        r#"[git.commit]
check = "error"
agents = false
skip_merge = false
"#,
    );
    temp.file("CLAUDE.md", "# Project\n");

    git_init(&temp);
    git_initial_commit(&temp);

    // Create merge commit
    git_branch(&temp, "feature");
    temp.file("feature.txt", "content");
    git_add_all(&temp);
    git_commit(&temp, "feat: add feature");

    git_checkout(&temp, "main");
    std::process::Command::new("git")
        .args(["merge", "--no-ff", "feature", "-m", "Merge branch 'feature'"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Should fail - merge commit violates conventional format
    check("git")
        .pwd(temp.path())
        .args(&["--base", "HEAD~2"])
        .fails()
        .stdout_has("invalid_format");
}

/// Spec: docs/specs/checks/git.md#metrics
///
/// > Metrics should include skipped count
#[test]
fn git_check_reports_skipped_in_metrics() {
    let temp = Project::empty();
    temp.config(
        r#"[git.commit]
check = "error"
agents = false
"#,
    );
    temp.file("CLAUDE.md", "# Project\n");

    git_init(&temp);
    git_initial_commit(&temp);

    git_branch(&temp, "feature");
    temp.file("a.txt", "a");
    git_add_all(&temp);
    git_commit(&temp, "feat: first");

    temp.file("b.txt", "b");
    git_add_all(&temp);
    git_commit(&temp, "Merge branch 'other'"); // Will be skipped

    temp.file("c.txt", "c");
    git_add_all(&temp);
    git_commit(&temp, "fix: third");

    // JSON output should show skipped count
    check("git")
        .pwd(temp.path())
        .args(&["--base", "main", "-o", "json"])
        .passes()
        .stdout_has("commits_skipped");
}
```

**Add helper functions if needed:**
```rust
fn git_checkout(temp: &Project, branch: &str) {
    std::process::Command::new("git")
        .args(["checkout", branch])
        .current_dir(temp.path())
        .output()
        .expect("git checkout failed");
}
```

**Verification:**
- `cargo test --test specs -- git_check_skips` passes
- `cargo test --test specs -- git_check_validates_merge` passes
- `cargo test --test specs -- git_check_reports_skipped` passes

---

### Phase 6: Update Documentation and Final Cleanup

Update spec documentation and run full test suite.

**File:** `docs/specs/checks/git.md`

Add documentation for `skip_merge` option:

In Configuration section:
```toml
[git.commit]
check = "error"                    # error | warn | off
# format = "conventional"          # conventional | none (default: conventional)
# skip_merge = true                # Skip merge commits (default: true)
```

Add section after Scope Validation:

```markdown
### Merge Commits

By default, merge commits are skipped:
- `Merge branch 'feature' into main`
- `Merge pull request #123 from user/branch`
- `Merge remote-tracking branch 'origin/main'`

| Setting | Behavior |
|---------|----------|
| `skip_merge = true` (default) | Skip merge commits silently |
| `skip_merge = false` | Validate merge commits against format |

This avoids false positives from git-generated commit messages.
```

Update JSON output example to include `commits_skipped`:
```json
{
  "name": "git",
  "passed": true,
  "metrics": {
    "commits_checked": 5,
    "commits_valid": 5,
    "commits_skipped": 1
  }
}
```

**Verification:**
- `make check` passes (all lint, test, build steps)
- `cargo doc --no-deps -p quench` builds cleanly

---

## Key Implementation Details

### Merge Commit Detection

Simple prefix matching is sufficient for merge commit detection:

```rust
pub fn is_merge_commit(message: &str) -> bool {
    message.starts_with("Merge ")
}
```

This covers all common git merge message formats:
- `Merge branch 'x' into y`
- `Merge pull request #N from x`
- `Merge remote-tracking branch 'origin/x'`
- `Merge commit 'abc123'`

### Backward Compatibility

The `skip_merge = true` default ensures existing configurations continue to work without changes. Projects that previously had failing git checks due to merge commits will now pass.

### Metrics Change

The metrics now include three fields instead of two:
- `commits_checked`: Number of commits actually validated
- `commits_valid`: Commits that passed validation
- `commits_skipped`: Merge commits that were skipped

This is a non-breaking change to the metrics schema.

### validate_commit Return Value

Changed from `()` to `bool` to indicate whether the commit was validated or skipped. This allows the caller to track counts accurately.

## Verification Plan

1. **Phase 1:** `cargo test -p quench -- docs_tests`
2. **Phase 2:** `cargo test -p quench -- is_merge_commit`
3. **Phase 3:** `cargo test -p quench -- git_skip_merge`
4. **Phase 4:** `cargo test -p quench -- skips_merge`
5. **Phase 5:** `cargo test --test specs -- git_check_skips`
6. **Phase 6:** `make check`

**Final verification:**
```bash
make check                           # All tests pass
quench check --base main             # Works on this repo
quench check --ci                    # Metrics include skipped count
```

## Checklist

- [ ] Remove duplicate `COMMIT_TYPES` from `docs.rs`, use `DEFAULT_TYPES` from `parse.rs`
- [ ] Add `is_merge_commit()` function to `parse.rs`
- [ ] Add `skip_merge` config option (default: true)
- [ ] Update `validate_commit()` to skip merge commits when configured
- [ ] Update metrics to include `commits_skipped`
- [ ] Add behavioral specs for merge commit handling
- [ ] Update `docs/specs/checks/git.md` with `skip_merge` documentation
- [ ] Bump `CACHE_VERSION` (check behavior changed)
- [ ] Run `make check`
