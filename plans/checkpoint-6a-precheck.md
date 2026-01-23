# Checkpoint 6A: Pre-Checkpoint Fix - Dogfooding Milestone 1

## Overview

Verification checkpoint to ensure the codebase is clean and all behavioral specs pass before proceeding to dogfooding milestone 1. This checkpoint validates:

- Code quality (fmt, clippy clean)
- Agents check behavioral specs (25 tests)
- Dry-run mode behavioral specs (5 tests)

**Status: COMPLETE** - All tasks verified passing.

## Project Structure

Relevant test files:

```
tests/specs/
├── checks/
│   └── agents.rs       # 25 behavioral specs for agents check
└── cli/
    └── dry_run.rs      # 5 behavioral specs for --dry-run mode
```

## Dependencies

No new dependencies. Uses existing test infrastructure:

- `assert_cmd` for CLI testing
- `tempfile` for temp directories
- Spec prelude helpers (`check()`, `cli()`, `temp_project()`)

## Verification Tasks

### Task 1: Code Quality

**Requirement:** cargo fmt and clippy must be clean with no warnings.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

**Status:** PASS

### Task 2: Agents Check Specs (25 tests)

All agents behavioral specs must pass. Organized by implementation phase:

| Phase | Category | Tests |
|-------|----------|-------|
| 1 | File Detection | 3 |
| 2 | Violation Detection | 5 |
| 3 | Content Rules | 6 |
| 4 | JSON Output & Fix | 6 |
| 525 | Text Output Format | 5 |

**Key specs verified:**
- `agents_detects_claude_md_at_project_root`
- `agents_missing_required_file_generates_violation`
- `agents_markdown_table_generates_violation`
- `agents_fix_syncs_files_from_sync_source`
- `agents_missing_file_text_output`

**Status:** PASS (25/25)

### Task 3: Dry-Run Specs (5 tests)

All dry-run behavioral specs must pass:

| Category | Test | Description |
|----------|------|-------------|
| Error Handling | `dry_run_without_fix_is_error` | --dry-run requires --fix |
| Output Format | `dry_run_shows_files_that_would_be_modified` | Shows affected files |
| Output Format | `dry_run_shows_diff_of_changes` | Shows old/new content |
| Exit Code | `dry_run_exits_0_when_fixes_needed` | Preview succeeds |
| File Integrity | `dry_run_does_not_modify_files` | No actual writes |

**Status:** PASS (5/5)

## Key Implementation Details

### Test Infrastructure

Specs use black-box testing via CLI:

```rust
// Single check test
check("agents").on("agents/basic").json().passes();

// CLI test with temp directory
let dir = temp_project();
cli().pwd(dir.path()).args(&["--fix", "--dry-run"]).passes();
```

### Agents Check Features Verified

1. **File detection** - CLAUDE.md, .cursorrules at project root
2. **Violation types** - missing_file, forbidden_file, out_of_sync, missing_section, forbidden_section, forbidden_table, forbidden_diagram, forbidden_mermaid, file_too_large
3. **Content rules** - tables, box_diagrams, mermaid blocks, max_lines, max_tokens
4. **Fix mode** - Syncs files from sync_source
5. **JSON output** - files_found, in_sync metrics, fixed field

### Dry-Run Features Verified

1. **Flag validation** - `--dry-run` requires `--fix`
2. **Preview output** - Shows files that would be modified with diff
3. **Exit behavior** - Returns 0 even when fixes available
4. **Safety** - Never modifies files

## Verification Plan

### Automated Verification

Run the full check suite:

```bash
make check
```

This runs:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all`
4. `cargo build --all`
5. `./scripts/bootstrap`
6. `cargo audit`
7. `cargo deny check`

### Targeted Test Runs

```bash
# Agents specs only
cargo test --test specs agents

# Dry-run specs only
cargo test --test specs dry_run

# All specs
cargo test --test specs
```

### Expected Results

```
test result: ok. 210 passed; 0 failed; 6 ignored; 0 measured
```

The 6 ignored tests are unrelated to agents/dry-run (bench-deep fixture tests).

## Summary

| Task | Status | Count |
|------|--------|-------|
| cargo fmt clean | PASS | - |
| cargo clippy clean | PASS | - |
| Agents specs | PASS | 25/25 |
| Dry-run specs | PASS | 5/5 |
| **Total** | **PASS** | **30/30** |

Checkpoint 6A verified. Ready to proceed to dogfooding milestone 1.
