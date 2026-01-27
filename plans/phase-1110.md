# Phase 1110: License Check - Validation

## Overview

Validates license headers in source files, detecting:
- Missing SPDX/Copyright headers
- Wrong license identifier (vs configured)
- Outdated copyright year (vs current year)
- File filtering by language patterns
- Exclude patterns for generated/vendor files

**Status:** Core validation is already implemented in Phase 1105. This phase adds copyright holder validation and verifies complete integration.

## Project Structure

```
crates/cli/src/
├── checks/
│   ├── license.rs        # License check implementation
│   └── license_tests.rs  # Unit tests
└── config/
    └── mod.rs            # LicenseConfig struct

tests/
├── specs/checks/
│   └── license.rs        # Behavioral specs
└── fixtures/license/
    ├── valid-headers/    # Passes all checks
    ├── missing-header/   # Missing SPDX/Copyright
    ├── wrong-license/    # Apache-2.0 vs MIT
    ├── outdated-year/    # 2025 vs 2026
    ├── mixed-violations/ # Multiple violation types
    └── with-shebang/     # Shell scripts with shebang
```

## Dependencies

Already in place:
- `chrono` - Current year detection
- `globset` - Pattern matching for include/exclude
- `regex` - SPDX/Copyright line parsing

## Implementation Phases

### Phase 1: Add Copyright Holder Validation

The copyright holder is detected but not validated. Add validation:

```rust
// In license.rs, after year validation:
if let Some(expected_holder) = config.copyright.as_deref() {
    let found_holder = copyright.get(2).map(|m| m.as_str()).unwrap_or("");
    if !found_holder.contains(expected_holder) {
        violations.push(
            Violation::file(
                relative_path,
                find_line_number(content, "Copyright"),
                "wrong_copyright",
                format!(
                    "Expected: {}, found: {}. Update copyright holder.",
                    expected_holder, found_holder
                ),
            )
            .with_expected_found(expected_holder, found_holder),
        );
    }
}
```

**Files changed:**
- `crates/cli/src/checks/license.rs` - Add holder validation
- `crates/cli/src/checks/license_tests.rs` - Unit tests

### Phase 2: Add wrong_copyright Fixture and Spec

Create test fixture:

```
tests/fixtures/license/wrong-copyright/
├── quench.toml           # license = "MIT", copyright = "Acme Corp"
├── CLAUDE.md
└── src/lib.rs            # Copyright (c) 2026 Other Corp
```

Add behavioral spec:

```rust
#[test]
fn license_wrong_copyright_generates_violation() {
    let license = check("license")
        .on("license/wrong-copyright")
        .args(&["--ci"])
        .json()
        .fails();
    let violation = license.require_violation("wrong_copyright");
    assert_eq!(violation.get("expected").and_then(|v| v.as_str()), Some("Acme Corp"));
}
```

**Files changed:**
- `tests/fixtures/license/wrong-copyright/` - New fixture
- `tests/specs/checks/license.rs` - New spec

### Phase 3: Update Spec for Violation Types

Update the violation types spec to include `wrong_copyright`:

```rust
#[test]
fn license_violation_types_are_expected_values() {
    // ...
    let valid_types = ["missing_header", "outdated_year", "wrong_license", "wrong_copyright"];
    // ...
}
```

Update spec doc:
- `docs/specs/checks/license-headers.md` - Add `wrong_copyright` to violation types

### Phase 4: Add Metrics for Copyright Violations

Add `files_wrong_copyright` counter to metrics output:

```rust
let mut files_wrong_copyright = 0;

// ... in validation loop ...
files_wrong_copyright += 1;

// ... in metrics json ...
let metrics = json!({
    "files_checked": files_checked,
    "files_with_headers": files_with_headers,
    "files_missing_headers": files_missing_headers,
    "files_outdated_year": files_outdated_year,
    "files_wrong_license": files_wrong_license,
    "files_wrong_copyright": files_wrong_copyright,
});
```

### Phase 5: Update Outline

Mark Phase 1105 and 1110 as complete in `plans/.0-outline.md`.

## Key Implementation Details

### Existing Validation Logic

The license check already validates:

1. **Missing header**: No SPDX or Copyright line in first 10 lines (skipping shebang)
2. **Wrong license**: SPDX identifier differs from configured `license`
3. **Outdated year**: Copyright year doesn't include current year (handles ranges like `2020-2026`)

### Pattern Matching

File selection uses two matchers:

1. **Include patterns** (`[check.license.patterns]`): Language-keyed patterns
   ```toml
   [check.license.patterns]
   rust = ["**/*.rs"]
   shell = ["**/*.sh", "scripts/*"]
   ```

2. **Exclude patterns** (`exclude`): Global exclusions
   ```toml
   exclude = ["**/generated/**", "**/vendor/**"]
   ```

When no patterns configured, uses default extension list:
- `rs`, `ts`, `tsx`, `js`, `jsx`, `go`, `c`, `cpp`, `h`
- `sh`, `bash`, `py`, `rb`, `yaml`, `yml`

### Shebang Handling

Shell scripts often start with `#!/bin/bash`. The `get_header_lines` function skips the shebang when searching for license headers:

```rust
fn get_header_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().take(max_lines + 1).collect();
    let start = if lines.first().map(|l| l.starts_with("#!")).unwrap_or(false) { 1 } else { 0 };
    // ...
}
```

### Copyright Year Range Handling

Supports both single year and range formats:
- `2026` - Matches if equals current year
- `2020-2026` - Matches if current year is within range

## Verification Plan

### Unit Tests

```bash
cargo test --lib license
```

Tests in `license_tests.rs`:
- `year_includes_current_*` - Year range logic
- `is_supported_extension_*` - Default extension matching
- `get_header_lines_*` - Shebang skipping
- `find_line_number_*` - Line number extraction

### Behavioral Specs

```bash
cargo test --test specs license
```

Specs in `tests/specs/checks/license.rs`:
- `license_detects_spdx_header` - Valid headers pass
- `license_missing_header_generates_violation` - Missing header detected
- `license_wrong_license_generates_violation` - Wrong license detected
- `license_outdated_year_generates_violation` - Outdated year detected
- `license_wrong_copyright_generates_violation` - Wrong holder detected (new)
- `license_violation_types_are_expected_values` - All types valid
- `license_json_includes_metrics` - Metrics present
- `license_skipped_without_ci_flag` - CI-only behavior
- `license_disabled_by_default` - Opt-in check

### Integration Test

```bash
cargo build && target/debug/quench check --ci --license
```

Run on quench itself to verify license headers are valid.

### Full Check Suite

```bash
make check
```

## Commit Plan

```
feat(license): add copyright holder validation (Phase 1110)

- Validate copyright holder matches configured value
- Add wrong_copyright violation type
- Add wrong-copyright test fixture
- Add files_wrong_copyright metric

Passing specs:
- license_wrong_copyright_generates_violation
- license_violation_types_are_expected_values (updated)
```
