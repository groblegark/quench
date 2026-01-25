# Checkpoint Go-1H: Tech Debt - DRY Up Tests

**Plan:** `checkpoint-go-1h-techdebt`
**Root Feature:** `quench-0d07`

## Overview

Address tech debt in Go adapter test code. Focus on test maintainability through:
- Converting repetitive tests to table-driven format using `yare::parameterized`
- Extracting common test helpers to reduce boilerplate
- Aligning test patterns with existing Rust/Shell adapter conventions
- Ensuring consistent test naming

## Project Structure

Files to modify:

```
crates/cli/src/adapter/
├── go_tests.rs           # 106 lines → ~75 lines (parameterize)
└── go/
    ├── policy_tests.rs   # 91 lines → ~60 lines (extract helper)
    └── suppress_tests.rs # 103 lines → ~55 lines (parameterize)
```

Reference files (already use yare):
```
crates/cli/src/adapter/
├── rust_tests.rs         # Lines 12-51: parameterized examples
├── rust/policy_tests.rs  # Lines 13-23: default_policy() helper
└── shell_tests.rs        # Lines 18-45: parameterized examples
```

## Dependencies

Already present:
- `yare` crate (used in rust_tests.rs, shell_tests.rs)

## Implementation Phases

### Phase 1: Convert `go_tests.rs` to Parameterized Tests

**Goal:** Align with rust_tests.rs and shell_tests.rs patterns.

**File:** `crates/cli/src/adapter/go_tests.rs`

**Change 1a:** Add yare import and convert classification tests (lines 8-44)

**Before:**
```rust
#[test]
fn classifies_go_files_as_source() {
    let adapter = GoAdapter::new();
    assert_eq!(adapter.classify(Path::new("main.go")), FileKind::Source);
    assert_eq!(adapter.classify(Path::new("pkg/lib.go")), FileKind::Source);
    assert_eq!(
        adapter.classify(Path::new("internal/config/config.go")),
        FileKind::Source
    );
}

#[test]
fn classifies_test_files_as_test() {
    let adapter = GoAdapter::new();
    assert_eq!(adapter.classify(Path::new("main_test.go")), FileKind::Test);
    assert_eq!(
        adapter.classify(Path::new("pkg/lib_test.go")),
        FileKind::Test
    );
}

#[test]
fn ignores_vendor_directory() {
    let adapter = GoAdapter::new();
    assert_eq!(
        adapter.classify(Path::new("vendor/dep/dep.go")),
        FileKind::Other
    );
    assert!(adapter.should_ignore(Path::new("vendor/foo/bar.go")));
}

#[test]
fn non_go_files_are_other() {
    let adapter = GoAdapter::new();
    assert_eq!(adapter.classify(Path::new("README.md")), FileKind::Other);
    assert_eq!(adapter.classify(Path::new("Makefile")), FileKind::Other);
}
```

**After:**
```rust
use yare::parameterized;

#[parameterized(
    src_root = { "main.go", FileKind::Source },
    src_pkg = { "pkg/lib.go", FileKind::Source },
    src_nested = { "internal/config/config.go", FileKind::Source },
    test_root = { "main_test.go", FileKind::Test },
    test_pkg = { "pkg/lib_test.go", FileKind::Test },
    vendor = { "vendor/dep/dep.go", FileKind::Other },
    readme = { "README.md", FileKind::Other },
    makefile = { "Makefile", FileKind::Other },
)]
fn classify_path(path: &str, expected: FileKind) {
    let adapter = GoAdapter::new();
    assert_eq!(
        adapter.classify(Path::new(path)),
        expected,
        "path {:?} should be {:?}",
        path,
        expected
    );
}

#[parameterized(
    vendor_root = { "vendor/foo/bar.go", true },
    src_main = { "main.go", false },
    pkg = { "pkg/lib.go", false },
)]
fn should_ignore_path(path: &str, expected: bool) {
    let adapter = GoAdapter::new();
    assert_eq!(
        adapter.should_ignore(Path::new(path)),
        expected,
        "path {:?} should_ignore = {}",
        path,
        expected
    );
}
```

**Change 1b:** Convert escape pattern tests (lines 54-77)

**Before:**
```rust
#[test]
fn provides_default_escape_patterns() {
    use super::Adapter;
    let adapter = GoAdapter::new();
    let escapes = adapter.default_escapes();

    assert_eq!(escapes.len(), 3);

    let unsafe_ptr = escapes.iter().find(|e| e.name == "unsafe_pointer").unwrap();
    assert_eq!(unsafe_ptr.pattern, r"unsafe\.Pointer");
    assert_eq!(unsafe_ptr.comment, Some("// SAFETY:"));

    let linkname = escapes.iter().find(|e| e.name == "go_linkname").unwrap();
    assert_eq!(linkname.pattern, r"//go:linkname");
    assert_eq!(linkname.comment, Some("// LINKNAME:"));

    let noescape = escapes.iter().find(|e| e.name == "go_noescape").unwrap();
    assert_eq!(noescape.pattern, r"//go:noescape");
    assert_eq!(noescape.comment, Some("// NOESCAPE:"));
}
```

**After:**
```rust
#[test]
fn returns_three_default_escape_patterns() {
    let adapter = GoAdapter::new();
    assert_eq!(adapter.default_escapes().len(), 3);
}

#[parameterized(
    unsafe_pointer = { "unsafe_pointer", r"unsafe\.Pointer", Some("// SAFETY:") },
    go_linkname = { "go_linkname", r"//go:linkname", Some("// LINKNAME:") },
    go_noescape = { "go_noescape", r"//go:noescape", Some("// NOESCAPE:") },
)]
fn default_escape_pattern(name: &str, pattern: &str, expected_comment: Option<&str>) {
    let adapter = GoAdapter::new();
    let patterns = adapter.default_escapes();
    let found = patterns
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("pattern {:?} not found", name));
    assert_eq!(found.pattern, pattern, "pattern {:?}", name);
    assert_eq!(found.comment, expected_comment, "comment for {:?}", name);
}
```

**Milestone:** `cargo test go_tests` passes, tests use parameterized format.

---

### Phase 2: Extract Helper and DRY `policy_tests.rs`

**Goal:** Match rust/policy_tests.rs pattern with `default_policy()` helper.

**File:** `crates/cli/src/adapter/go/policy_tests.rs`

**Change 2a:** Extract `default_policy()` helper function

**Before:** Each test creates inline `GoPolicyConfig`:
```rust
#[test]
fn no_policy_allows_mixed_changes() {
    let policy = GoPolicyConfig {
        lint_changes: LintChangesPolicy::None,
        lint_config: vec![".golangci.yml".to_string()],
    };
    // ...
}

#[test]
fn standalone_policy_allows_lint_only() {
    let policy = GoPolicyConfig {
        lint_changes: LintChangesPolicy::Standalone,
        lint_config: vec![".golangci.yml".to_string()],
    };
    // ...
}
```

**After:** Extract helper, use struct update syntax:
```rust
fn default_policy() -> GoPolicyConfig {
    GoPolicyConfig {
        lint_changes: LintChangesPolicy::Standalone,
        lint_config: vec![".golangci.yml".to_string()],
    }
}

#[test]
fn no_policy_allows_mixed_changes() {
    let policy = GoPolicyConfig {
        lint_changes: LintChangesPolicy::None,
        ..default_policy()
    };
    let files: Vec<&Path> = vec![Path::new(".golangci.yml"), Path::new("main.go")];
    let result = check_lint_policy(&files, &policy, go_classifier);
    assert!(!result.standalone_violated);
}

#[test]
fn standalone_policy_allows_lint_only() {
    let policy = default_policy();
    // ...
}
```

**Change 2b:** Simplify file array construction

**Before:**
```rust
let files: Vec<&Path> = vec![Path::new(".golangci.yml"), Path::new("main.go")];
```

**After:** Use array-to-vec pattern from rust/policy_tests.rs:
```rust
let files = [Path::new(".golangci.yml"), Path::new("main.go")];
let file_refs: Vec<&Path> = files.to_vec();
```

This is more consistent with the existing Rust adapter tests and makes it clearer that we're testing with a specific set of files.

**Milestone:** `cargo test policy_tests` passes, helper extracted.

---

### Phase 3: Parameterize `suppress_tests.rs`

**Goal:** Convert 10 repetitive directive parsing tests to table-driven format.

**File:** `crates/cli/src/adapter/go/suppress_tests.rs`

**Change 3:** Add yare import and convert parsing tests

**Before:**
```rust
#[test]
fn parses_nolint_all() {
    let content = "//nolint\nfoo()";
    let directives = parse_nolint_directives(content, None);

    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].line, 0);
    assert!(directives[0].codes.is_empty());
}

#[test]
fn parses_nolint_single_code() {
    let content = "//nolint:errcheck\nfoo()";
    let directives = parse_nolint_directives(content, None);

    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].codes, vec!["errcheck"]);
}

#[test]
fn parses_nolint_multiple_codes() {
    let content = "//nolint:errcheck,gosec,staticcheck\nfoo()";
    let directives = parse_nolint_directives(content, None);

    assert_eq!(directives.len(), 1);
    assert_eq!(
        directives[0].codes,
        vec!["errcheck", "gosec", "staticcheck"]
    );
}
// ... 7 more similar tests
```

**After:**
```rust
use yare::parameterized;

// Tests for single-directive parsing
#[parameterized(
    nolint_all = { "//nolint\nfoo()", 0, &[], false },
    nolint_single = { "//nolint:errcheck\nfoo()", 0, &["errcheck"], false },
    nolint_multiple = { "//nolint:errcheck,gosec,staticcheck\nfoo()", 0, &["errcheck", "gosec", "staticcheck"], false },
    nolint_end_of_line = { "foo() //nolint:errcheck", 0, &["errcheck"], false },
)]
fn parse_nolint_directive(
    content: &str,
    expected_line: usize,
    expected_codes: &[&str],
    expected_has_comment: bool,
) {
    let directives = parse_nolint_directives(content, None);
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].line, expected_line);
    assert_eq!(
        directives[0].codes,
        expected_codes.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(directives[0].has_comment, expected_has_comment);
}

// Tests for justification comment detection
#[parameterized(
    inline_comment = { "//nolint:errcheck // This error is intentionally ignored", None, true, Some("This error is intentionally ignored") },
    previous_line = { "// OK: This is justified\n//nolint:errcheck", None, true, None },
    blank_line_before = { "// Comment\n\n//nolint:errcheck", None, false, None },
    required_pattern_miss = { "// Some comment\n//nolint:errcheck", Some("// OK:"), false, None },
    required_pattern_match = { "// OK: Justified reason\n//nolint:errcheck", Some("// OK:"), true, None },
)]
fn parse_justification_comment(
    content: &str,
    required_pattern: Option<&str>,
    expected_has_comment: bool,
    expected_comment_text: Option<&str>,
) {
    let directives = parse_nolint_directives(content, required_pattern);
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].has_comment, expected_has_comment);
    if let Some(text) = expected_comment_text {
        assert_eq!(directives[0].comment_text.as_deref(), Some(text));
    }
}

// Separate test for multiple directives (not parameterizable)
#[test]
fn multiple_directives_in_file() {
    let content = "//nolint:errcheck\nfoo()\n//nolint:gosec\nbar()";
    let directives = parse_nolint_directives(content, None);

    assert_eq!(directives.len(), 2);
    assert_eq!(directives[0].line, 0);
    assert_eq!(directives[1].line, 2);
}
```

**Milestone:** `cargo test suppress_tests` passes, 10 tests consolidated to ~3 parameterized blocks.

---

### Phase 4: Final Verification

**Goal:** Ensure all changes pass quality gates.

```bash
make check
```

This runs:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --all`
- `./scripts/bootstrap`
- `cargo audit`
- `cargo deny check`

**Milestone:** All quality gates pass.

## Key Implementation Details

### Why Parameterized Tests

The `yare::parameterized` macro provides:
1. **Clearer test structure** - inputs and expected outputs side-by-side
2. **Better error messages** - each case named, easy to identify failures
3. **Less boilerplate** - one test function covers many cases
4. **Consistency** - matches rust_tests.rs and shell_tests.rs patterns

### Patterns to Follow

From `rust_tests.rs`:
```rust
#[parameterized(
    case_name = { input1, input2, expected },
)]
fn test_name(param1: Type1, param2: Type2, expected: Expected) {
    // single assertion with context
    assert_eq!(actual, expected, "context {:?}", param1);
}
```

From `rust/policy_tests.rs`:
```rust
fn default_policy() -> PolicyConfig {
    PolicyConfig { /* defaults */ }
}

#[test]
fn test_with_override() {
    let policy = PolicyConfig {
        specific_field: other_value,
        ..default_policy()
    };
}
```

### What We're NOT Changing

- Production code (no changes to `go.rs`, `go/policy.rs`, `go/suppress.rs`)
- Integration tests in `tests/specs/adapters/golang.rs` (different scope)
- Test behavior (only refactoring, not changing coverage)

## Verification Plan

| Phase | Command | Expected Result |
|-------|---------|-----------------|
| 1 | `cargo test go_tests` | All tests pass, uses parameterized |
| 2 | `cargo test policy_tests -- --test-threads=1` | All tests pass, helper extracted |
| 3 | `cargo test suppress_tests` | All tests pass, parameterized |
| 4 | `make check` | All quality gates pass |

## Summary

| Phase | Task | Lines Before | Lines After |
|-------|------|-------------|-------------|
| 1 | Parameterize go_tests.rs | 106 | ~75 |
| 2 | Extract helper in policy_tests.rs | 91 | ~60 |
| 3 | Parameterize suppress_tests.rs | 103 | ~55 |
| 4 | Verification | — | — |

**Total reduction:** ~300 lines → ~190 lines (~37% reduction)

**Checkpoint Criteria:**
- [ ] DRY up common patterns in Go adapter tests
- [ ] Use yare (table-driven tests) where beneficial
- [ ] Shorten verbose test setups
- [ ] Extract common test fixtures/helpers
- [ ] Ensure test naming is consistent
- [ ] Remove duplicate test coverage (consolidate into parameterized tests)
