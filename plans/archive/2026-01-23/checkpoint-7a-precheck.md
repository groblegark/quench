# Checkpoint 7A: Pre-Checkpoint Fix - Docs Check Complete

**Plan:** `checkpoint-7a-precheck`
**Root Feature:** `quench-docs`
**Depends On:** Phase 635 (Docs Check - Area Mapping)

## Overview

Complete the docs check feature by un-ignoring all Phase 602 behavioral specs and ensuring they pass. The implementation already exists and has unit tests; this checkpoint focuses on validating behavioral specs against the existing implementation and fixing any gaps.

**Current State:**
- Core implementation: ✅ Complete (`crates/cli/src/checks/docs/`)
- Unit tests: ✅ Passing
- Fixtures: ✅ Exist
- Behavioral specs: ⚠️ 16 specs still marked `#[ignore = "TODO: Phase 602"]`

**Goal:** Remove all Phase 602 ignores and have all specs pass.

## Project Structure

```
tests/specs/checks/docs/
├── toc.rs          # 5 ignored specs → UNIGNORE
├── sections.rs     # 3 ignored specs → UNIGNORE
├── index.rs        # 1 ignored spec → UNIGNORE
├── output.rs       # 6 ignored specs → UNIGNORE
├── links.rs        # ✅ All passing
├── commit.rs       # ✅ All passing
└── content.rs      # ✅ All passing

tests/fixtures/docs/
├── toc-ok/         # TOC validation passes
├── toc-broken/     # TOC validation fails
├── section-required/   # Required section missing
├── section-forbidden/  # Forbidden section present
├── index-auto/     # Auto-detect index file
├── link-broken/    # Broken link fixture
└── ...
```

## Dependencies

No new dependencies. Uses existing:
- `globset` for pattern matching
- `serde_json` for JSON output

## Implementation Phases

### Phase 1: TOC Spec Validation

**Goal:** Un-ignore 5 TOC specs and verify they pass.

**Ignored specs in `tests/specs/checks/docs/toc.rs`:**
1. `toc_tree_entries_validated_against_filesystem` - fixture: `docs/toc-ok`
2. `broken_toc_path_generates_violation` - fixture: `docs/toc-broken`
3. `toc_box_drawing_format_supported` - temp project
4. `toc_indentation_format_supported` - temp project
5. `toc_path_resolution_order` - temp project

**Actions:**
1. Remove `#[ignore = "TODO: Phase 602 - Docs Check Implementation"]` from each
2. Run `cargo test --test specs toc`
3. If failures, analyze output and fix:
   - Adjust expected output strings if format changed
   - Update fixtures if file structure expectations differ
   - Fix implementation if behavior is incorrect

**Verification:**
```bash
cargo test --test specs toc -- --nocapture
```

---

### Phase 2: Section Validation Specs

**Goal:** Un-ignore 3 section validation specs and verify they pass.

**Ignored specs in `tests/specs/checks/docs/sections.rs`:**
1. `missing_required_section_in_spec_generates_violation` - fixture: `docs/section-required`
2. `forbidden_section_in_spec_generates_violation` - fixture: `docs/section-forbidden`
3. `section_matching_is_case_insensitive` - temp project

**Fixture analysis:**

`section-required` config:
```toml
[check.docs]
path = "docs/specs"
sections.required = ["Purpose"]
```

`incomplete.md` intentionally lacks "Purpose" section.

**Actions:**
1. Remove ignores
2. Run tests, analyze failures
3. Fix output expectations or implementation

**Known considerations:**
- Verify advice message format matches spec expectations
- Ensure violation type is exactly `missing_section` / `forbidden_section`

**Verification:**
```bash
cargo test --test specs sections -- --nocapture
```

---

### Phase 3: Index Detection Specs

**Goal:** Un-ignore 1 index detection spec.

**Ignored spec in `tests/specs/checks/docs/index.rs`:**
1. `specs_directory_index_file_detected` - fixture: `docs/index-auto`

**Actions:**
1. Remove ignore
2. Verify JSON output includes `metrics.index_file`
3. Fix metrics structure if needed

**Implementation check:**

Look in `crates/cli/src/checks/docs/mod.rs` or `specs.rs` for where metrics are populated. Ensure `index_file` is included in JSON output.

**Verification:**
```bash
cargo test --test specs index -- --nocapture
```

---

### Phase 4: JSON Output Format Specs

**Goal:** Un-ignore 6 JSON output format specs.

**Ignored specs in `tests/specs/checks/docs/output.rs`:**
1. `docs_violation_type_is_one_of_expected_values`
2. `broken_toc_violation_structure`
3. `broken_link_violation_structure`
4. `missing_section_violation_structure`
5. `docs_json_metrics_structure`
6. `forbidden_section_violation_structure`

**Expected violation structure:**
```json
{
  "type": "broken_toc",
  "file": "CLAUDE.md",
  "line": 5,
  "path": "missing.md",
  "advice": "..."
}
```

**Actions:**
1. Remove ignores
2. Run tests
3. Ensure violation JSON includes required fields:
   - `file`, `line`, `type`, `advice` for all
   - `path` for `broken_toc`
   - `target` for `broken_link`
   - `section` for `missing_section` / `forbidden_section`

**Implementation check:**

Verify in `crates/cli/src/check.rs` that `Violation` struct has all fields:
```rust
pub struct Violation {
    pub file: String,
    pub line: usize,
    #[serde(rename = "type")]
    pub violation_type: String,
    pub advice: String,
    // Context fields:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,      // for broken_toc
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,    // for broken_link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,   // for section violations
}
```

**Verification:**
```bash
cargo test --test specs output -- --nocapture
```

---

### Phase 5: Integration Testing

**Goal:** Run full test suite and verify no regressions.

**Actions:**
1. Run all docs specs:
   ```bash
   cargo test --test specs docs
   ```
2. Run full make check:
   ```bash
   make check
   ```
3. Verify cache version bump not needed (no logic changes, only test changes)

**Verification checklist:**
- [ ] All 16 previously-ignored specs pass
- [ ] No regressions in existing specs
- [ ] `make check` completes successfully

---

### Phase 6: Cleanup and Documentation

**Goal:** Finalize and commit.

**Actions:**
1. Remove any TODO comments referencing Phase 602
2. Verify no remaining `#[ignore = "TODO: Phase 602"]` in codebase
3. Run `grep -r "Phase 602" tests/` to confirm

**Commit message template:**
```
feat(docs): complete docs check behavioral specs

Un-ignore and validate all Phase 602 docs check specs:
- TOC validation: 5 specs
- Section validation: 3 specs
- Index detection: 1 spec
- JSON output format: 6 specs

Passing specs:
- toc_tree_entries_validated_against_filesystem
- broken_toc_path_generates_violation
- toc_box_drawing_format_supported
- toc_indentation_format_supported
- toc_path_resolution_order
- missing_required_section_in_spec_generates_violation
- forbidden_section_in_spec_generates_violation
- section_matching_is_case_insensitive
- specs_directory_index_file_detected
- docs_violation_type_is_one_of_expected_values
- broken_toc_violation_structure
- broken_link_violation_structure
- missing_section_violation_structure
- docs_json_metrics_structure
- forbidden_section_violation_structure
```

## Key Implementation Details

### Spec-First Approach

These specs were written before implementation (TDD style). The implementation may have diverged slightly. When fixing failures:

1. **Prefer fixing the implementation** if the spec matches documented behavior
2. **Prefer fixing the spec** only if the implementation represents better behavior

### JSON Output Fields

Ensure violations serialize correctly:

```rust
// In check.rs or docs/mod.rs
impl Violation {
    pub fn broken_toc(file: &str, line: usize, path: &str) -> Self {
        Violation {
            file: file.to_string(),
            line,
            violation_type: "broken_toc".to_string(),
            path: Some(path.to_string()),
            ..Default::default()
        }
    }
}
```

### Fixture Expectations

| Fixture | Expected Result | Violation Type |
|---------|-----------------|----------------|
| `docs/toc-ok` | PASS | - |
| `docs/toc-broken` | FAIL | `broken_toc` |
| `docs/section-required` | FAIL | `missing_section` |
| `docs/section-forbidden` | FAIL | `forbidden_section` |
| `docs/index-auto` | PASS | - |
| `docs/link-broken` | FAIL | `broken_link` |

## Verification Plan

### Unit Tests
```bash
cargo test -p quench docs
```

### Behavioral Specs
```bash
# All docs specs
cargo test --test specs docs

# Specific groups
cargo test --test specs toc
cargo test --test specs sections
cargo test --test specs index
cargo test --test specs output
```

### Full Suite
```bash
make check
```

### Manual Check
```bash
# Verify no remaining Phase 602 ignores
grep -r "Phase 602" tests/specs/
# Should return empty

# Count passing docs specs
cargo test --test specs docs 2>&1 | grep -c "test result: ok"
```
