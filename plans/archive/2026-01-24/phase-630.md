# Phase 630: Docs Check - Commit Checking (CI)

**Root Feature:** `quench-85b3`

## Overview

Add commit checking to the docs check for CI mode. This validates that feature commits (feat:, breaking:, etc.) have corresponding documentation updates. The check examines commits on the current branch compared to a base branch and reports when feature commits lack doc changes.

**Key Behaviors:**
- Disabled by default (must explicitly enable via `[check.docs.commit]`)
- Requires `--ci` flag to run (CI mode only)
- Identifies feature commits by conventional commit type prefixes
- Supports area mapping to require specific docs for scoped commits
- Detects doc changes in `docs/` directory (or mapped paths)

## Project Structure

```
crates/cli/src/
├── cli.rs                    # Add --ci flag
├── check.rs                  # Add ci_mode to CheckContext
├── main.rs                   # Pass ci_mode to context
├── config/
│   ├── mod.rs                # Add DocsAreaConfig struct
│   └── checks.rs             # Add DocsCommitConfig, extend DocsConfig
├── checks/docs/
│   ├── mod.rs                # Integrate commit checking
│   ├── commit.rs             # NEW: Commit checking logic
│   └── commit_tests.rs       # NEW: Unit tests
tests/
├── specs/checks/docs/
│   └── commit.rs             # Existing behavioral tests (remove #[ignore])
└── fixtures/docs/
    └── area-mapping/         # Existing fixture
```

## Dependencies

No new external crates. Uses existing:
- `std::process::Command` for git operations
- `glob` for pattern matching (already in deps)

## Implementation Phases

### Phase 1: CLI and Context Extension

Add `--ci` flag to the CLI and pass CI mode to check context.

**crates/cli/src/cli.rs:**

```rust
#[derive(clap::Args)]
pub struct CheckArgs {
    // ... existing fields ...

    /// CI mode: run slow checks, auto-detect base branch
    #[arg(long)]
    pub ci: bool,
}
```

**crates/cli/src/check.rs:**

```rust
pub struct CheckContext<'a> {
    // ... existing fields ...

    /// Whether running in CI mode (enables slow checks like commit validation).
    pub ci_mode: bool,
}
```

**crates/cli/src/main.rs:**

Update `CheckContext` construction to include `ci_mode: args.ci`.

When `--ci` is set without `--base`, auto-detect base branch:
1. Try `main`
2. Fall back to `master`
3. Skip commit checking if neither exists

**Verification:**
- `quench check --ci --help` shows the flag
- Unit test for base branch auto-detection

---

### Phase 2: Configuration Schema

Add commit checking configuration to `DocsConfig`.

**crates/cli/src/config/checks.rs:**

```rust
/// Documentation check configuration.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocsConfig {
    // ... existing fields ...

    /// Commit checking configuration (CI mode).
    #[serde(default)]
    pub commit: DocsCommitConfig,

    /// Area mappings for scoped commit requirements.
    #[serde(default)]
    pub area: HashMap<String, DocsAreaConfig>,
}

/// Configuration for commit checking in CI mode.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocsCommitConfig {
    /// Check level: "error" | "warn" | "off" (default: "off")
    #[serde(default = "DocsCommitConfig::default_check")]
    pub check: String,

    /// Commit types that require documentation.
    /// Default: ["feat", "feature", "story", "breaking"]
    #[serde(default = "DocsCommitConfig::default_types")]
    pub types: Vec<String>,
}

impl Default for DocsCommitConfig {
    fn default() -> Self {
        Self {
            check: Self::default_check(),
            types: Self::default_types(),
        }
    }
}

impl DocsCommitConfig {
    fn default_check() -> String {
        "off".to_string()
    }

    fn default_types() -> Vec<String> {
        vec![
            "feat".to_string(),
            "feature".to_string(),
            "story".to_string(),
            "breaking".to_string(),
        ]
    }
}

/// Area mapping for scoped commits.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocsAreaConfig {
    /// Required docs pattern (glob).
    pub docs: String,

    /// Source files that trigger this area (optional glob).
    #[serde(default)]
    pub source: Option<String>,
}
```

**Verification:**
- Unit tests in `config/checks_tests.rs` for TOML parsing
- Test default values, custom types, area mappings

---

### Phase 3: Git Operations

Create helpers for git operations needed by commit checking.

**crates/cli/src/checks/docs/commit.rs:**

```rust
use std::path::Path;
use std::process::Command;

/// A parsed conventional commit.
#[derive(Debug)]
pub struct ConventionalCommit {
    pub hash: String,
    pub commit_type: String,
    pub scope: Option<String>,
    pub message: String,
}

/// Get commits on current branch not in base branch.
pub fn get_branch_commits(root: &Path, base: &str) -> Result<Vec<ConventionalCommit>, String> {
    // git log --format="%H %s" base..HEAD
    let output = Command::new("git")
        .args(["log", "--format=%H %s", &format!("{}..HEAD", base)])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines()
        .filter_map(|line| parse_commit_line(line))
        .collect())
}

/// Parse a commit line into conventional commit parts.
fn parse_commit_line(line: &str) -> Option<ConventionalCommit> {
    let (hash, message) = line.split_once(' ')?;

    // Parse conventional commit: type(scope): message or type: message
    let re = regex::Regex::new(r"^(\w+)(?:\(([^)]+)\))?:\s*(.+)$").ok()?;
    let caps = re.captures(message)?;

    Some(ConventionalCommit {
        hash: hash[..7].to_string(),  // Short hash
        commit_type: caps.get(1)?.as_str().to_lowercase(),
        scope: caps.get(2).map(|m| m.as_str().to_string()),
        message: message.to_string(),
    })
}

/// Get files changed on current branch vs base.
pub fn get_changed_files(root: &Path, base: &str) -> Result<Vec<String>, String> {
    // git diff --name-only base..HEAD
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{}..HEAD", base)])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(String::from).collect())
}

/// Check if any changed files match a glob pattern.
pub fn has_changes_matching(changed_files: &[String], pattern: &str) -> bool {
    let matcher = glob::Pattern::new(pattern).ok();
    matcher.map_or(false, |m| {
        changed_files.iter().any(|f| m.matches(f))
    })
}
```

**Verification:**
- Unit tests with mock git output
- Test conventional commit parsing edge cases

---

### Phase 4: Commit Validation Logic

Implement the main commit checking algorithm.

**crates/cli/src/checks/docs/commit.rs** (continued):

```rust
use crate::check::{CheckContext, Violation};
use crate::config::checks::{DocsCommitConfig, DocsAreaConfig};
use std::collections::HashMap;

/// Result of commit validation.
pub struct CommitValidation {
    pub violations: Vec<Violation>,
    pub feature_commits: usize,
    pub with_docs: usize,
}

/// Validate that feature commits have documentation.
pub fn validate_commits(
    ctx: &CheckContext,
    config: &DocsCommitConfig,
    areas: &HashMap<String, DocsAreaConfig>,
) -> CommitValidation {
    let mut result = CommitValidation {
        violations: Vec::new(),
        feature_commits: 0,
        with_docs: 0,
    };

    // Determine base branch
    let base = match detect_base_branch(ctx.root) {
        Some(b) => b,
        None => return result, // No base branch, skip check
    };

    // Get branch commits
    let commits = match get_branch_commits(ctx.root, &base) {
        Ok(c) => c,
        Err(_) => return result, // Git error, skip check
    };

    // Filter to feature commits
    let feature_commits: Vec<_> = commits
        .into_iter()
        .filter(|c| config.types.contains(&c.commit_type))
        .collect();

    result.feature_commits = feature_commits.len();

    if feature_commits.is_empty() {
        return result;
    }

    // Get changed files
    let changed_files = match get_changed_files(ctx.root, &base) {
        Ok(f) => f,
        Err(_) => return result,
    };

    // Check each feature commit
    for commit in &feature_commits {
        let has_docs = check_commit_has_docs(commit, &changed_files, areas);
        if has_docs {
            result.with_docs += 1;
        } else {
            result.violations.push(create_violation(commit, areas));
        }
    }

    result
}

fn check_commit_has_docs(
    commit: &ConventionalCommit,
    changed_files: &[String],
    areas: &HashMap<String, DocsAreaConfig>,
) -> bool {
    // If commit has scope, check for matching area
    if let Some(scope) = &commit.scope {
        if let Some(area) = areas.get(scope) {
            return has_changes_matching(changed_files, &area.docs);
        }
    }

    // Default: any change in docs/ satisfies requirement
    has_changes_matching(changed_files, "docs/**")
}

fn create_violation(
    commit: &ConventionalCommit,
    areas: &HashMap<String, DocsAreaConfig>,
) -> Violation {
    let advice = if let Some(scope) = &commit.scope {
        if let Some(area) = areas.get(scope) {
            format!("Update {} with the new functionality.", area.docs)
        } else {
            "Update docs/ with the new functionality.".to_string()
        }
    } else {
        "Update docs/ with the new functionality.".to_string()
    };

    Violation {
        file: None,
        line: None,
        violation_type: "missing_docs".to_string(),
        advice,
        value: None,
        threshold: None,
        pattern: None,
        lines: None,
        nonblank: None,
        other_file: None,
        section: None,
    }
    .with_commit(&commit.hash, &commit.message)
}

fn detect_base_branch(root: &Path) -> Option<String> {
    // Check if main exists
    if branch_exists(root, "main") {
        return Some("main".to_string());
    }
    // Fall back to master
    if branch_exists(root, "master") {
        return Some("master".to_string());
    }
    None
}

fn branch_exists(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

**Note:** Need to extend `Violation` with commit-specific fields:

```rust
// In crates/cli/src/check.rs
pub struct Violation {
    // ... existing fields ...

    /// Commit hash for commit-level violations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// Full commit message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Expected docs pattern (for area-specific violations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_docs: Option<String>,
}

impl Violation {
    pub fn with_commit(mut self, hash: &str, message: &str) -> Self {
        self.commit = Some(hash.to_string());
        self.message = Some(message.to_string());
        self
    }
}
```

**Verification:**
- Unit tests for validation logic
- Test area mapping matching
- Test default docs/ detection

---

### Phase 5: Integration and Output

Integrate commit checking into the docs check and format output.

**crates/cli/src/checks/docs/mod.rs:**

```rust
mod commit;

impl Check for DocsCheck {
    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let mut violations = Vec::new();

        // ... existing checks (toc, links, specs) ...

        // Commit checking (CI mode only)
        if ctx.ci_mode {
            commit::validate_commit_docs(ctx, &mut violations);
        }

        // ... rest of implementation ...
    }
}
```

**crates/cli/src/checks/docs/commit.rs:**

```rust
/// Entry point for commit validation from docs check.
pub fn validate_commit_docs(ctx: &CheckContext, violations: &mut Vec<Violation>) {
    let commit_config = &ctx.config.check.docs.commit;

    // Skip if disabled
    if commit_config.check == "off" {
        return;
    }

    let areas = &ctx.config.check.docs.area;
    let result = validate_commits(ctx, commit_config, areas);

    // Collect violations
    for v in result.violations {
        if ctx.limit.is_some_and(|l| violations.len() >= l) {
            break;
        }
        violations.push(v);
    }
}
```

**Text Output Format:**

```
docs: FAIL
  Branch has feature commits without documentation:
    abc123: feat(api): add export endpoint
    def456: feat: new user settings
  Update docs/ or add area mapping in quench.toml.
```

With area mapping:

```
docs: FAIL
  feat(api) commits require changes in docs/api/**
    No changes found in docs/api/
    Update docs/api/ with the new API functionality.
```

**Verification:**
- Behavioral tests pass (remove `#[ignore]` from `tests/specs/checks/docs/commit.rs`)
- Output format matches spec

---

### Phase 6: Polish and Documentation

1. **Update metrics** in `CheckResult`:
   ```rust
   "metrics": {
       "feature_commits": 2,
       "with_docs": 0
   }
   ```

2. **Bump `CACHE_VERSION`** in `crates/cli/src/cache.rs`

3. **Run full test suite**:
   ```bash
   make check
   ```

4. **Remove `#[ignore]` from behavioral tests** in `tests/specs/checks/docs/commit.rs`

## Key Implementation Details

### Conventional Commit Parsing

Parse commits using regex: `^(\w+)(?:\(([^)]+)\))?:\s*(.+)$`

| Input | Type | Scope | Match |
|-------|------|-------|-------|
| `feat: add feature` | feat | - | Yes |
| `feat(api): add endpoint` | feat | api | Yes |
| `fix: bug` | fix | - | No (not in types) |
| `Add feature` | - | - | No (not conventional) |

### Area Mapping Resolution

1. If commit has scope (`feat(api):`), look up area by name
2. If area exists, require changes matching `area.docs` pattern
3. If no area or no scope, require any change in `docs/**`

### Base Branch Detection

1. If `--base` provided, use that
2. If `--ci` without `--base`:
   - Check if `main` branch exists → use `main`
   - Check if `master` branch exists → use `master`
   - Neither exists → skip commit checking silently

### Git Commands Used

| Operation | Command |
|-----------|---------|
| List commits | `git log --format="%H %s" base..HEAD` |
| List changed files | `git diff --name-only base..HEAD` |
| Check branch exists | `git rev-parse --verify branch` |

## Verification Plan

### Unit Tests

- `crates/cli/src/config/checks_tests.rs` - Config parsing
- `crates/cli/src/checks/docs/commit_tests.rs`:
  - Conventional commit parsing
  - Area matching logic
  - Base branch detection
  - Commit filtering by type

### Behavioral Tests

Remove `#[ignore]` from `tests/specs/checks/docs/commit.rs`:
- `feature_commit_without_doc_change_generates_violation_ci_mode`
- `area_mapping_restricts_doc_requirement_to_specific_paths`
- `commit_checking_disabled_by_default`

### Integration

```bash
# Full test suite
make check

# Specific tests
cargo test --test specs commit
cargo test -p quench commit

# Manual testing
cargo run -- check --ci --docs
```

### Manual Verification

1. Create a test repo with feature commits but no docs
2. Run `quench check --ci` → should fail
3. Add doc changes → should pass
4. Test area mapping with scoped commits
