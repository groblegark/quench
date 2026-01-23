# Checkpoint 4B: Rust Adapter Complete - Validation

**Root Feature:** `quench-rust-adapter`

## Overview

Validation checkpoint that verifies the Rust language adapter produces correct, useful output on standard test fixtures. This creates documented evidence that auto-detection, workspace support, Rust-specific escape patterns, and `#[cfg(test)]` line-level counting all work correctly in practice.

**Checkpoint Criteria:**
- [ ] `quench check` on `fixtures/rust-simple` with no config produces useful output
- [ ] `quench check` on `fixtures/rust-workspace` detects all packages
- [ ] Rust-specific escapes detected in `fixtures/violations`
- [ ] `#[cfg(test)]` LOC counted separately

**Output:** `reports/checkpoint-4-rust-adapter.md`

## Project Structure

Key files involved:

```
quench/
├── tests/fixtures/
│   ├── rust-simple/               # Minimal passing Rust project
│   │   ├── Cargo.toml             # Auto-detection trigger
│   │   ├── quench.toml            # Version 1 config
│   │   ├── src/lib.rs             # Source file
│   │   └── src/lib_tests.rs       # Test file (sibling convention)
│   ├── rust-workspace/            # Multi-package workspace
│   │   ├── Cargo.toml             # [workspace] with members
│   │   ├── crates/cli/            # CLI package
│   │   ├── crates/core/           # Core library package
│   │   └── tests/integration.rs   # Workspace-level tests
│   ├── violations/                # Intentional violations
│   │   └── src/escapes.rs         # .unwrap(), .expect(), unsafe
│   └── rust/
│       └── cfg-test/              # #[cfg(test)] fixture
├── crates/cli/src/adapter/
│   ├── rust.rs                    # Rust adapter (566 lines)
│   └── rust_tests.rs              # Unit tests
├── tests/specs/adapters/rust.rs   # 19 behavioral specs
└── reports/
    └── checkpoint-4-rust-adapter.md  # Validation report (to create)
```

## Dependencies

No new dependencies required. Uses existing:
- `assert_cmd` - CLI execution
- `serde_json` - JSON parsing
- Release binary from `cargo build --release`

## Implementation Phases

### Phase 1: Verify rust-simple Produces Useful Output

Run `quench check` on the minimal Rust project without explicit config to verify auto-detection and default patterns work.

**Expected Behavior:**
- Rust adapter auto-detected (Cargo.toml present)
- Source pattern `**/*.rs` matches `src/lib.rs`
- Test pattern `*_tests.rs` matches `src/lib_tests.rs`
- cloc check produces source/test LOC breakdown
- escapes check applies Rust-specific patterns

**Commands:**
```bash
./target/release/quench check tests/fixtures/rust-simple -o json
./target/release/quench check tests/fixtures/rust-simple
```

**Verification Checklist:**
- [ ] Output shows detected language: "rust"
- [ ] cloc metrics separate source vs test LOC
- [ ] escapes check runs with Rust patterns (no violations expected)
- [ ] Human-readable output is useful (shows file metrics)

**Milestone:** rust-simple produces useful output with auto-detected Rust adapter.

**Status:** [ ] Pending

---

### Phase 2: Verify rust-workspace Detects All Packages

Run `quench check` on the workspace fixture to verify package detection and per-package metrics.

**Expected Packages:**
| Package | Path | Source Files |
|---------|------|--------------|
| cli | `crates/cli/` | `src/main.rs` |
| core | `crates/core/` | `src/lib.rs` |
| (root) | `tests/` | `integration.rs` |

**Commands:**
```bash
./target/release/quench check tests/fixtures/rust-workspace -o json
./target/release/quench check tests/fixtures/rust-workspace
```

**Verification Checklist:**
- [ ] JSON output includes `packages` array or per-package breakdown
- [ ] Both `cli` and `core` packages detected
- [ ] Metrics include per-package LOC (source and test)
- [ ] Integration tests at workspace root counted correctly

**Milestone:** All workspace packages detected with correct metrics.

**Status:** [ ] Pending

---

### Phase 3: Verify Rust-Specific Escapes in violations

Run escapes check on the violations fixture to verify Rust-specific escape patterns are detected.

**Expected Violations in `src/escapes.rs`:**

| Line | Pattern | Action | Expected Result |
|------|---------|--------|-----------------|
| 5 | `.unwrap()` | forbid | `forbidden` violation |
| 10 | `.expect(` | forbid | `forbidden` violation |
| 15 | `unsafe { }` | comment | `missing_comment` violation |
| 21 | `unsafe { }` with SAFETY | comment | Pass (comment found) |
| 25 | `#[allow(dead_code)]` | comment | `missing_comment` violation (if configured) |
| 30 | `#[allow(dead_code)]` with JUSTIFIED | comment | Pass |

**Commands:**
```bash
./target/release/quench check tests/fixtures/violations --escapes -o json
./target/release/quench check tests/fixtures/violations --escapes
```

**Verification Checklist:**
- [ ] `.unwrap()` at line 5 reported as forbidden
- [ ] `.expect(` at line 10 reported as forbidden
- [ ] `unsafe` at line 15 reported as missing_comment
- [ ] `unsafe` at line 21 passes (has SAFETY comment)
- [ ] Violation advice mentions Rust-specific guidance
- [ ] Total violations ≥ 3 from escapes.rs

**Milestone:** All Rust-specific escape patterns correctly detected.

**Status:** [ ] Pending

---

### Phase 4: Verify #[cfg(test)] LOC Counted Separately

Run cloc check on the cfg-test fixture to verify line-level splitting for `#[cfg(test)]` blocks.

**Fixture Structure (`tests/fixtures/rust/cfg-test/src/lib.rs`):**
```rust
// Source code (not in cfg(test))
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
```

**Commands:**
```bash
./target/release/quench check tests/fixtures/rust/cfg-test --cloc -o json
./target/release/quench check tests/fixtures/rust/cfg-test --cloc
```

**Verification Checklist:**
- [ ] JSON metrics show separate `source_loc` and `test_loc` for the same file
- [ ] Lines inside `#[cfg(test)]` counted as test LOC
- [ ] Lines outside `#[cfg(test)]` counted as source LOC
- [ ] Per-file breakdown shows the split

**Milestone:** `#[cfg(test)]` blocks correctly counted as test LOC.

**Status:** [ ] Pending

---

### Phase 5: Create Validation Report

Generate `reports/checkpoint-4-rust-adapter.md` documenting all checkpoint criteria with evidence.

**Report Template:**
```markdown
# Checkpoint 4B: Rust Adapter Complete - Validation Report

Generated: YYYY-MM-DD

## Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| rust-simple useful output | ✓/✗ | ... |
| rust-workspace package detection | ✓/✗ | ... |
| Rust-specific escapes detected | ✓/✗ | ... |
| #[cfg(test)] LOC separation | ✓/✗ | ... |

**Overall Status: PASS/FAIL**

## Detailed Results

### 1. rust-simple Output
[Command outputs and verification]

### 2. Workspace Package Detection
[JSON showing packages, per-package metrics]

### 3. Rust-Specific Escape Detection
[Violation list from violations fixture]

### 4. #[cfg(test)] Line Splitting
[Metrics showing source vs test LOC split]

## Conclusion
[Summary of Rust adapter validation]
```

**Milestone:** Report created with all checkpoint criteria documented.

**Status:** [ ] Pending

---

### Phase 6: Run Full Test Suite

Execute `make check` to ensure all quality gates pass.

```bash
make check
```

**Checklist:**
- [ ] `cargo fmt --all -- --check` - no formatting issues
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - no warnings
- [ ] `cargo test --all` - all tests pass
- [ ] `cargo test rust_adapter` - all 19 Rust adapter specs pass
- [ ] `cargo build --all` - builds successfully
- [ ] `./scripts/bootstrap` - conventions pass
- [ ] `cargo audit` - no critical vulnerabilities
- [ ] `cargo deny check` - licenses/bans OK

**Milestone:** All quality gates pass.

**Status:** [ ] Pending

## Key Implementation Details

### Rust Adapter Capabilities Being Validated

1. **Auto-Detection**
   - Triggered by `Cargo.toml` in project root
   - Sets language to "rust"
   - Applies Rust-specific defaults

2. **Default Patterns**
   ```rust
   source_patterns: ["**/*.rs"]
   test_patterns: ["tests/**", "*_test.rs", "*_tests.rs"]
   ignore_patterns: ["target/**"]
   ```

3. **Workspace Detection** (via `CargoWorkspace`)
   - Parses `[workspace]` section from Cargo.toml
   - Expands member globs (e.g., `crates/*`)
   - Provides per-package metric breakdown

4. **Default Escape Patterns**
   | Pattern | Action | Comment Required |
   |---------|--------|------------------|
   | `unsafe { }` | comment | `// SAFETY:` |
   | `.unwrap()` | forbid | (allowed in tests) |
   | `.expect(` | forbid | (allowed in tests) |
   | `mem::transmute` | comment | `// SAFETY:` |

5. **#[cfg(test)] Splitting** (via `CfgTestInfo`)
   - Parses brace-balanced `#[cfg(test)]` blocks
   - Returns line ranges for test code
   - Configurable via `split_cfg_test = true|false`

### Output Format Examples

**rust-simple JSON Output (expected):**
```json
{
  "project": {
    "language": "rust",
    "detected": true
  },
  "checks": {
    "cloc": {
      "passed": true,
      "metrics": {
        "source_loc": 10,
        "test_loc": 5,
        "files": {
          "src/lib.rs": { "source": 10 },
          "src/lib_tests.rs": { "test": 5 }
        }
      }
    }
  }
}
```

**rust-workspace Packages (expected):**
```json
{
  "packages": [
    { "name": "cli", "path": "crates/cli" },
    { "name": "core", "path": "crates/core" }
  ],
  "metrics": {
    "by_package": {
      "cli": { "source_loc": 15, "test_loc": 8 },
      "core": { "source_loc": 20, "test_loc": 10 }
    }
  }
}
```

### Escape Violation Format

**Text Output:**
```
escapes: FAIL
  src/escapes.rs:5: forbidden (.unwrap() in production code)
    Remove this escape hatch from production code.
  src/escapes.rs:10: forbidden (.expect() in production code)
    Remove this escape hatch from production code.
  src/escapes.rs:15: missing_comment (unsafe without // SAFETY:)
    Add a // SAFETY: comment explaining why this unsafe block is sound.
```

## Verification Plan

### Quick Verification

```bash
# Build release binary
cargo build --release

# Test each criterion
./target/release/quench check tests/fixtures/rust-simple -o json | jq '.project.language'
./target/release/quench check tests/fixtures/rust-workspace -o json | jq '.packages'
./target/release/quench check tests/fixtures/violations --escapes -o json | jq '.violations | length'
./target/release/quench check tests/fixtures/rust/cfg-test --cloc -o json | jq '.metrics'
```

### Full Verification

```bash
# Run all behavioral specs for Rust adapter
cargo test --test specs rust_adapter

# Run quality gates
make check
```

### Checkpoint Criteria Mapping

| Criterion | Phase | Verification |
|-----------|-------|--------------|
| rust-simple useful output | Phase 1 | JSON shows language, metrics |
| rust-workspace package detection | Phase 2 | JSON shows packages array |
| Rust-specific escapes | Phase 3 | Violations detected in escapes.rs |
| #[cfg(test)] separation | Phase 4 | Metrics show source/test split |

## Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | Verify rust-simple useful output | [ ] Pending |
| 2 | Verify rust-workspace package detection | [ ] Pending |
| 3 | Verify Rust-specific escapes in violations | [ ] Pending |
| 4 | Verify #[cfg(test)] LOC separation | [ ] Pending |
| 5 | Create validation report | [ ] Pending |
| 6 | Run full test suite | [ ] Pending |
