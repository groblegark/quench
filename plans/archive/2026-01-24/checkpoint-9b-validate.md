# Checkpoint 9B: Git Check Complete - Validation

**Plan:** `checkpoint-9b-validate`
**Root Feature:** `quench-git`
**Depends On:** Checkpoint 9A (Pre-Checkpoint Fix)

## Overview

Validate the git check feature is complete and working end-to-end. This checkpoint confirms:
- `quench check --git` validates commit messages against conventional commit format
- `quench check --git --fix` creates `.gitmessage` template and configures git
- Output format is correct for both text and JSON modes
- Snapshot tests capture expected output format

**Current State:**
- Core implementation: Complete in `crates/cli/src/checks/git/`
- Unit tests: 69 passing (`*_tests.rs` siblings)
- Behavioral specs: 21 passing in `tests/specs/checks/git.rs`
- Fixtures: `tests/fixtures/git/` (5 fixture directories)
- No output format tests yet (no `stdout_eq` tests for git)

**Goal:** Add snapshot tests for git output and create validation report.

## Project Structure

```
crates/cli/src/checks/git/
├── mod.rs              # GitCheck implementation (263 lines)
├── mod_tests.rs        # Unit tests (327 lines)
├── parse.rs            # Conventional commit parsing (121 lines)
├── parse_tests.rs      # Parser unit tests (221 lines)
├── docs.rs             # Agent documentation checking (103 lines)
├── docs_tests.rs       # Documentation check unit tests (175 lines)
├── template.rs         # .gitmessage template generation (94 lines)
└── template_tests.rs   # Template unit tests (107 lines)

tests/specs/checks/git.rs          # 21 behavioral specs (557 lines)

tests/fixtures/git/
├── conventional-ok/               # Valid conventional commits
├── invalid-format/                # Missing type/scope
├── invalid-type/                  # Disallowed type
├── invalid-scope/                 # Disallowed scope
└── missing-docs/                  # No format documentation

reports/checkpoint-9-git-check.md  # Validation report (to be created)
```

## Dependencies

No external dependencies beyond existing crate dependencies.

## Implementation Phases

### Phase 1: Verify Existing Tests Pass

**Goal:** Confirm all existing git check tests pass.

**Actions:**
1. Run unit tests:
   ```bash
   cargo test -p quench checks::git
   ```
   Expected: 69 tests pass

2. Run behavioral specs:
   ```bash
   cargo test --test specs git
   ```
   Expected: 21 tests pass

3. Check no ignored specs:
   ```bash
   grep -r "#\[ignore" tests/specs/checks/git.rs
   ```
   Expected: No output

---

### Phase 2: Add Snapshot Tests for Text Output

**Goal:** Add exact output comparison tests for git check text output.

**Location:** `tests/specs/checks/git.rs` (new section at end)

**Tests to add:**

```rust
// =============================================================================
// EXACT OUTPUT FORMAT SPECS
// =============================================================================

/// Spec: docs/specs/checks/git.md#output
///
/// > Missing docs shows human-readable violation with file reference.
#[test]
fn exact_missing_docs_text() {
    check("git").on("git/missing-docs").fails().stdout_eq(
        r###"git: FAIL
  CLAUDE.md: missing commit format documentation
    Document your commit format in CLAUDE.md (e.g., "Use feat: or fix: prefixes")
FAIL: git
"###,
    );
}

/// Spec: docs/specs/checks/git.md#output
///
/// > PASS status when no violations.
#[test]
fn exact_git_pass_text() {
    let temp = Project::empty();
    temp.config(r#"[git.commit]
check = "error"
agents = false
"#);
    temp.file(
        "CLAUDE.md",
        "# Project\n\n## Directory Structure\n\nMinimal.\n\n## Landing the Plane\n\n- Done\n",
    );

    check("git").pwd(temp.path()).passes().stdout_eq("PASS: git\n");
}
```

**Key patterns:**
- Use `stdout_eq()` for exact format verification
- Reference spec document in doc comment
- Cover both pass and fail cases

---

### Phase 3: Add Snapshot Tests for Fix Output

**Goal:** Add tests verifying `--fix` output format.

**Tests to add:**

```rust
/// Spec: docs/specs/checks/git.md#fix-output
///
/// > FIXED status shows actions taken.
#[test]
fn exact_fix_creates_template_text() {
    let temp = Project::empty();
    temp.config(r#"[git.commit]
check = "error"
template = true
"#);
    temp.file(
        "CLAUDE.md",
        "# Project\n\n## Commits\n\nfeat: format\n\n## Directory Structure\n\nMinimal.\n\n## Landing the Plane\n\n- Done\n",
    );

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    check("git")
        .pwd(temp.path())
        .args(&["--fix"])
        .passes()
        .stdout_has("FIXED")
        .stdout_has(".gitmessage");
}

/// Spec: docs/specs/checks/git.md#fix-output
///
/// > JSON output includes fixed:true and actions array.
#[test]
fn exact_fix_json_structure() {
    let temp = Project::empty();
    temp.config(r#"[git.commit]
check = "error"
template = true
"#);
    temp.file(
        "CLAUDE.md",
        "# Project\n\n## Commits\n\nfeat: format\n\n## Directory Structure\n\nMinimal.\n\n## Landing the Plane\n\n- Done\n",
    );

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let result = check("git")
        .pwd(temp.path())
        .args(&["--fix"])
        .json()
        .passes();

    assert_eq!(
        result.require("fixed").as_bool(),
        Some(true),
        "should have fixed: true"
    );
}
```

---

### Phase 4: Verify Commit Message Validation

**Goal:** Manual verification of commit validation in CI mode.

**Manual test script:**
```bash
# Create test project
cd /tmp && rm -rf git-test && mkdir git-test && cd git-test
git init

# Add quench config
cat > quench.toml << 'EOF'
version = 1
[git.commit]
check = "error"
types = ["feat", "fix"]
agents = false
EOF

# Add CLAUDE.md
cat > CLAUDE.md << 'EOF'
# Test Project

## Directory Structure

Minimal.

## Landing the Plane

- Done
EOF

# Valid commit should pass
git add . && git commit -m "feat: initial commit"
git checkout -b feature
echo "test" > test.txt && git add . && git commit -m "feat: add test"
quench check --git --ci
# Expected: PASS

# Invalid commit should fail
echo "test2" > test2.txt && git add . && git commit -m "update stuff"
quench check --git --ci
# Expected: FAIL with invalid_format
```

---

### Phase 5: Create Validation Report

**Goal:** Document validation results in checkpoint report.

**Location:** `reports/checkpoint-9-git-check.md`

**Report structure:**
```markdown
# Checkpoint 9: Git Check Complete - Validation Report

**Date**: YYYY-MM-DD
**Status**: PASS/FAIL

## Summary

| Criterion | Status | Details |
|-----------|--------|---------|
| quench check --git validates commits | PASS | ... |
| quench check --git --fix creates template | PASS | ... |
| Snapshot tests for git output | PASS | N new specs |
| All git specs | PASS | 21 + N specs passing |

## Phase 1: ...
```

---

### Phase 6: Final Verification

**Goal:** Run full test suite and make check.

**Actions:**
1. Run all tests:
   ```bash
   make check
   ```

2. Verify spec count:
   ```bash
   cargo test --test specs git -- --list 2>&1 | grep -c ": test$"
   ```
   Expected: 21 + new snapshot tests

3. Commit with appropriate message

## Key Implementation Details

### Git Check Configuration

```toml
[git.commit]
check = "error"           # "error" | "warn" | "off"
format = "conventional"   # "conventional" | "none"
types = ["feat", "fix", "chore", "docs", "test", "refactor"]
scopes = ["api", "cli"]   # Optional - any scope if omitted
agents = true             # Check CLAUDE.md for format docs
template = true           # Generate .gitmessage with --fix
```

### Violation Types

| Type | Description | Fields |
|------|-------------|--------|
| `invalid_format` | Commit message format wrong | `commit`, `advice` |
| `invalid_type` | Type not in allowed list | `commit`, `advice` |
| `invalid_scope` | Scope not in allowed list | `commit`, `advice` |
| `missing_docs` | No format in agent files | `file`, `advice` |

### Text Output Format

**Pass:**
```
PASS: git
```

**Fail (violations):**
```
git: FAIL
  CLAUDE.md: missing commit format documentation
    Document your commit format in CLAUDE.md (e.g., "Use feat: or fix: prefixes")
FAIL: git
```

**Fixed:**
```
git: FIXED
  Created .gitmessage
  Configured git commit.template
PASS: git
```

### JSON Output Structure

```json
{
  "name": "git",
  "passed": false,
  "violations": [
    {
      "type": "missing_docs",
      "file": "CLAUDE.md",
      "advice": "Document your commit format..."
    }
  ],
  "metrics": {}
}
```

## Verification Plan

### Unit Tests
```bash
cargo test -p quench checks::git
# Expected: 69 tests pass
```

### Behavioral Specs
```bash
cargo test --test specs git
# Expected: 21+ tests pass, 0 ignored
```

### Manual Testing
```bash
# Test commit validation
cd /tmp && mkdir git-validation-test && cd git-validation-test
git init
echo 'version = 1
[git.commit]
check = "error"
agents = false' > quench.toml
echo '# Project

## Directory Structure

Minimal.

## Landing the Plane

- Done' > CLAUDE.md
git add . && git commit -m "feat: initial"
git checkout -b feature
echo "test" > test.txt && git add . && git commit -m "bad message"
quench check --git --ci
# Expected: FAIL

# Test fix behavior
quench check --git --fix
ls -la .gitmessage
git config commit.template
# Expected: .gitmessage exists, git config set
```

### Full Suite
```bash
make check
# Expected: All checks pass
```

## Checklist

- [ ] Phase 1: Existing tests pass (69 unit, 21 behavioral)
- [ ] Phase 2: Snapshot tests for text output added
- [ ] Phase 3: Snapshot tests for fix output added
- [ ] Phase 4: Manual commit validation verified
- [ ] Phase 5: Validation report created
- [ ] Phase 6: Full `make check` passes
- [ ] No `#[ignore]` in git specs
- [ ] Report saved to `reports/checkpoint-9-git-check.md`
