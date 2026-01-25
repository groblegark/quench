# Checkpoint: Go Adapter Basic Complete - Validation

**Plan:** `checkpoint-go-1b-validate`
**Root Feature:** `quench-0b9f`

## Overview

Dogfooding validation for the Go adapter. This plan verifies that the Go adapter produces useful output on test fixtures, correctly detects Go-specific escape patterns, and documents any unexpected behaviors for follow-up fixes.

## Project Structure

Relevant files for validation:

```
tests/fixtures/
├── go-simple/              # Minimal Go project (passes all checks)
├── go-multi/               # Multi-package Go project
├── golang/                 # Comprehensive Go-specific test fixtures
│   ├── auto-detect/
│   ├── unsafe-pointer-fail/ok/
│   ├── linkname-fail/ok/
│   ├── noescape-fail/ok/
│   ├── nolint-*/
│   ├── module-packages/
│   └── vendor-ignore/
└── violations/
    └── go/                 # Go escape violations
        ├── unsafe.go
        ├── linkname.go
        ├── noescape.go
        └── nolint.go

reports/
└── checkpoint-go-1-basic.md  # Validation report (to create)

tests/specs/adapters/
└── golang.rs               # 24 behavioral specs for Go
```

## Dependencies

- Built quench binary (`cargo build --release`)
- Existing test fixtures (already present)
- jq for JSON parsing in validation scripts (optional)

## Implementation Phases

### Phase 1: Run Manual Validation on go-simple

Run `quench check` on the go-simple fixture and verify output.

```bash
cd tests/fixtures/go-simple
cargo run -p quench -- check --format json 2>&1 | jq .
cargo run -p quench -- check 2>&1
```

**Expected Output:**
- `cloc` check passes with source/test file counts
- `escapes` check runs (no violations expected)
- Human output shows "PASS: cloc, escapes"
- JSON output includes metrics with `source_files`, `source_lines`, `test_files`, `test_lines`

**Verification:**
- [ ] Source files counted correctly (5 .go files)
- [ ] Test files counted correctly (1 *_test.go file)
- [ ] No violations reported
- [ ] Output is readable and useful

### Phase 2: Run Manual Validation on go-multi

Run `quench check` on the go-multi fixture and verify package detection.

```bash
cd tests/fixtures/go-multi
cargo run -p quench -- check --format json 2>&1 | jq .
```

**Expected Output:**
- Multiple packages detected (cmd/server, cmd/cli, pkg/api, pkg/storage, internal/core)
- Source files from all packages counted
- Test files (api_test.go, storage_test.go, core_test.go) counted

**Verification:**
- [ ] All packages detected (at least 5 directories with .go files)
- [ ] Source files counted from all packages
- [ ] Test files counted correctly
- [ ] No violations reported

### Phase 3: Verify Go Escapes in violations Fixture

Run `quench check` on the violations fixture and verify Go escape patterns are detected.

```bash
cd tests/fixtures/violations
cargo run -p quench -- check escapes --format json 2>&1 | jq .
```

**Expected Go Violations:**

| File | Pattern | Expected Violation |
|------|---------|-------------------|
| `go/unsafe.go:8` | `unsafe.Pointer` | missing `// SAFETY:` comment |
| `go/linkname.go` | `//go:linkname` | missing `// LINKNAME:` comment |
| `go/noescape.go` | `//go:noescape` | missing `// NOESCAPE:` comment |
| `go/nolint.go` | `//nolint` | missing justification comment |

**Note:** The violations fixture is detected as a Rust project (has `src/` with `.rs` files). Go escape patterns are configured explicitly in `quench.toml` with `source = ["**/*.go"]` filters, so they should still work.

**Verification:**
- [ ] `unsafe.Pointer` violation detected with correct advice
- [ ] `//go:linkname` violation detected (if pattern configured)
- [ ] `//go:noescape` violation detected (if pattern configured)
- [ ] `//nolint` violation detected (if golang.suppress configured)

### Phase 4: Run Automated Test Suite

Run the full Go adapter test suite to verify all specs pass.

```bash
cargo test golang
cargo test go -- --include-ignored
```

**Expected Results:**
- 24 behavioral specs in `tests/specs/adapters/golang.rs`
- All unit tests in `crates/cli/src/adapter/go/*_tests.rs`
- No failures, some may be ignored (for future phases)

**Verification:**
- [ ] All non-ignored specs pass
- [ ] Unit tests pass (go_tests.rs, policy_tests.rs, suppress_tests.rs)
- [ ] Document any ignored tests and why

### Phase 5: Create/Update Exact output tests

Add Exact output tests for Go adapter output following the pattern in `tests/specs/checks/agents.rs`.

Add to `tests/specs/adapters/golang.rs`:

```rust
use insta::assert_snapshot;

/// Snapshot: Go adapter output on simple project
#[test]
fn snapshot_go_simple_json() {
    let result = cli().on("go-simple").json().passes();
    let json = result.raw_json();
    let redacted = regex::Regex::new(r#""timestamp": "[^"]+""#)
        .expect("valid regex")
        .replace(&json, r#""timestamp": "[REDACTED]""#);
    assert_snapshot!(redacted);
}

/// Snapshot: Go escape violation output
#[test]
fn snapshot_unsafe_pointer_fail_text() {
    let result = check("escapes").on("golang/unsafe-pointer-fail").fails();
    assert_snapshot!(result.stdout());
}
```

**Verification:**
- [ ] Exact output tests compile and run
- [ ] Snapshots capture expected output format
- [ ] Snapshots stored in `tests/specs/snapshots/`

### Phase 6: Document Results

Create `reports/checkpoint-go-1-basic.md` following the format of `reports/checkpoint-5-shell-adapter.md`.

**Report Structure:**
```markdown
# Checkpoint Go-1: Go Adapter Basic Complete - Validation Report

Generated: [DATE]

## Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| go-simple useful output | ? | |
| go-multi package detection | ? | |
| Go escapes detected | ? | |
| Exact output tests | ? | |

**Overall Status: ?**

## Detailed Results

### 1. go-simple Output
[JSON and human output]

### 2. go-multi Package Detection
[Package list and metrics]

### 3. Go Escape Detection
[Violation output for each pattern]

### 4. Test Suite Results
[cargo test output summary]

### 5. Unexpected Behaviors
[Any issues found, with links to follow-up tasks]

## Conclusion
[Summary of Go adapter status]
```

## Key Implementation Details

### Violations Fixture Configuration

The `violations/quench.toml` already configures Go escape patterns:

```toml
[[check.escapes.patterns]]
name = "go_unsafe_pointer"
pattern = "unsafe\\.Pointer"
action = "comment"
comment = "// SAFETY:"
source = ["**/*.go"]

[[check.escapes.patterns]]
name = "go_linkname"
pattern = "//go:linkname"
action = "comment"
comment = "// LINKNAME:"
source = ["**/*.go"]

[[check.escapes.patterns]]
name = "go_noescape"
pattern = "//go:noescape"
action = "comment"
comment = "// NOESCAPE:"
source = ["**/*.go"]

[golang.suppress]
check = "comment"
```

### Snapshot Test Dependencies

Add `insta` to dev-dependencies if not present:

```toml
[dev-dependencies]
insta = "1.34"
```

### Expected Test Counts

Based on the existing specs:
- `golang.rs`: 24 behavioral specs
- `go_tests.rs`: ~15 unit tests
- `policy_tests.rs`: ~9 unit tests
- `suppress_tests.rs`: ~15 unit tests
- Total: ~63 Go-related tests

## Verification Plan

1. **Phase 1-3**: Manual verification with quench CLI
   - Run commands, capture output
   - Compare against expected behavior
   - Document any discrepancies

2. **Phase 4**: Automated test verification
   - Run `cargo test golang` - all should pass
   - Run `cargo test go` - all should pass
   - Count passed/failed/ignored

3. **Phase 5**: Snapshot test creation
   - Add Exact output tests to golang.rs
   - Run `cargo insta test` to generate
   - Review and accept snapshots

4. **Phase 6**: Documentation
   - Create report with all findings
   - Note any unexpected behaviors
   - Link to follow-up issues if needed

## Checkpoint Criteria Verification

| Criterion | How Verified |
|-----------|--------------|
| `quench check` on go-simple produces useful output | Phase 1 manual run |
| `quench check` on go-multi detects all packages | Phase 2 manual run |
| Go-specific escapes detected in violations | Phase 3 manual run |
| Exact output tests for Go adapter output | Phase 5 implementation |
