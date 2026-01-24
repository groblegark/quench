# Phase 470: Go Adapter - Policy

**Root Feature:** `quench-70fe`

## Overview

Enable the behavioral spec for Go lint configuration policy enforcement. The implementation is already complete - this phase updates the behavioral spec from an `#[ignore]` fixture-based test to a working `temp_project()` + git-based test that properly simulates changed files.

**Current Status:**
- ✅ Config: `GoPolicyConfig` in `config/go.rs`
- ✅ Policy logic: `go/policy.rs` (delegates to common)
- ✅ Unit tests: `go/policy_tests.rs` (5 tests passing)
- ✅ Integration: `check_go_lint_policy()` in `checks/escapes/lint_policy.rs`
- ✅ Fixture: `tests/fixtures/golang/lint-policy-fail/`
- ⏳ Behavioral spec: marked `#[ignore]` - needs git context pattern

**Reference Implementation:** Phase 325 (Rust adapter policy) - see `tests/specs/adapters/rust.rs:576-632`

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── adapter/go/
│   │   ├── policy.rs          # EXISTS: policy wrapper
│   │   └── policy_tests.rs    # EXISTS: 5 unit tests
│   ├── config/go.rs           # EXISTS: GoPolicyConfig
│   └── checks/escapes/
│       └── lint_policy.rs     # EXISTS: check_go_lint_policy()
├── tests/
│   ├── specs/adapters/
│   │   └── golang.rs          # UPDATE: remove #[ignore], add git pattern
│   └── fixtures/golang/
│       └── lint-policy-fail/  # EXISTS: can delete (not needed)
└── plans/
    └── phase-470.md
```

## Dependencies

No new dependencies. Uses existing:
- `temp_project()` helper from spec prelude
- `std::process::Command` for git operations
- `--base HEAD` flag for diff detection

## Implementation Phases

### Phase 1: Update Behavioral Spec

Replace the ignored fixture-based test with a working `temp_project()` + git pattern.

**Update `tests/specs/adapters/golang.rs`:**

```rust
/// Spec: docs/specs/langs/golang.md#policy
///
/// > `lint_changes = "standalone"` requires lint config in separate PRs.
#[test]
fn lint_config_changes_with_source_fails_standalone_policy() {
    let dir = temp_project();

    // Setup quench.toml with standalone policy
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[golang.policy]
lint_changes = "standalone"
lint_config = [".golangci.yml"]
"#,
    )
    .unwrap();

    // Setup go.mod
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/test\n\ngo 1.21\n",
    )
    .unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Create initial commit with source
    std::fs::write(
        dir.path().join("main.go"),
        "package main\n\nfunc main() {}\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Add both lint config and source changes
    std::fs::write(
        dir.path().join(".golangci.yml"),
        "linters:\n  enable:\n    - errcheck\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("main.go"),
        "package main\n\nfunc main() {}\nfunc helper() {}\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Check with --base HEAD should detect mixed changes
    check("escapes")
        .pwd(dir.path())
        .args(&["--base", "HEAD"])
        .fails()
        .stdout_has("lint config changes must be standalone");
}
```

**Milestone:** Spec runs without `#[ignore]` and fails correctly on mixed changes.

**Verification:**
```bash
cargo test --test specs lint_config_changes_with_source_fails_standalone_policy
```

---

### Phase 2: Add Standalone-Passes Test

Add a complementary test that verifies lint-only changes pass.

**Add to `tests/specs/adapters/golang.rs`:**

```rust
/// Spec: docs/specs/langs/golang.md#policy
///
/// > Lint config changes only (no source) passes standalone policy.
#[test]
fn lint_config_standalone_passes() {
    let dir = temp_project();

    // Setup quench.toml with standalone policy
    std::fs::write(
        dir.path().join("quench.toml"),
        r#"
version = 1
[golang.policy]
lint_changes = "standalone"
lint_config = [".golangci.yml"]
"#,
    )
    .unwrap();

    // Setup go.mod
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/test\n\ngo 1.21\n",
    )
    .unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Create initial commit
    std::fs::write(
        dir.path().join("main.go"),
        "package main\n\nfunc main() {}\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Add ONLY lint config change (no source changes)
    std::fs::write(
        dir.path().join(".golangci.yml"),
        "linters:\n  enable:\n    - errcheck\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Should pass - only lint config changed
    check("escapes")
        .pwd(dir.path())
        .args(&["--base", "HEAD"])
        .passes();
}
```

**Milestone:** Both policy specs pass - mixed changes fail, lint-only passes.

**Verification:**
```bash
cargo test --test specs golang.*lint_config
```

---

### Phase 3: Cleanup Old Fixture

The `tests/fixtures/golang/lint-policy-fail/` fixture is no longer needed since we use `temp_project()`. Remove it to keep the fixture directory clean.

**Delete:**
- `tests/fixtures/golang/lint-policy-fail/.golangci.yml`
- `tests/fixtures/golang/lint-policy-fail/go.mod`
- `tests/fixtures/golang/lint-policy-fail/main.go`
- `tests/fixtures/golang/lint-policy-fail/quench.toml`
- `tests/fixtures/golang/lint-policy-fail/` (directory)

**Milestone:** Fixture directory cleaned up.

**Verification:**
```bash
make check
```

---

## Key Implementation Details

### Why temp_project() Pattern

The `lint_changes = "standalone"` policy requires comparing changed files against a base revision. This needs:

1. A git repository with history
2. A base commit to compare against
3. Uncommitted/staged changes representing the "PR"

Static fixtures can't provide this because:
- No git history in test fixtures
- `--base` flag requires a valid git ref
- `ctx.changed_files` is `None` without `--base`

The `temp_project()` pattern creates ephemeral repos with proper git history.

### Test Flow

```
1. Create temp directory
2. Write quench.toml with policy
3. Write go.mod (language detection)
4. git init + config
5. Create initial source file
6. git add + commit (base state)
7. Add lint config + modify source (mixed changes)
8. git add (staged changes)
9. Run quench --escapes --base HEAD
10. Assert failure with expected message
```

### Error Message Format

From `checks/escapes/lint_policy.rs:92-95`:

```
escapes: FAIL
  lint config changes must be standalone
    Changed lint config: .golangci.yml
    Also changed source: main.go
  Submit lint config changes in a separate PR.
```

### Lint Config Files Detected

Default from `config/go.rs:116-121`:
- `.golangci.yml`
- `.golangci.yaml`
- `.golangci.toml`

## Verification Plan

### After Each Phase

```bash
# Run specific spec
cargo test --test specs lint_config_changes

# Run all golang specs
cargo test --test specs golang

# Check for compile errors
cargo build --all
```

### Final Verification

```bash
# Full test suite
cargo test --all

# Quality gates
make check
```

### Test Matrix

| Test Case | Changed Files | Expected |
|-----------|--------------|----------|
| Mixed changes | .golangci.yml + main.go | FAIL |
| Lint config only | .golangci.yml | PASS |
| Source only | main.go | PASS |
| Policy disabled | lint_changes = "none" | PASS |
| No --base flag | No git comparison | PASS |

## Summary

| Phase | Task | Files | Status |
|-------|------|-------|--------|
| 1 | Update behavioral spec | `tests/specs/adapters/golang.rs` | [ ] Pending |
| 2 | Add standalone-passes test | `tests/specs/adapters/golang.rs` | [ ] Pending |
| 3 | Remove old fixture | `tests/fixtures/golang/lint-policy-fail/` | [ ] Pending |

**Estimated Complexity:** Low - pattern exists in Rust tests, just adapt for Go.
