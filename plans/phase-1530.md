# Phase 1530: Language Section Output

**Root Feature:** `quench-init`

## Overview

Generate language-specific configuration sections for auto-detected languages during `quench init`. When a language is detected (e.g., Rust from `Cargo.toml`), the output includes a `[lang]` section with dotted-key settings for `cloc.check`, `suppress.check`, and `policy.check`. The sections are appended after the `# Supported Languages:` comment block in the generated `quench.toml`.

**Note:** This phase is largely implemented. The primary work is verification and enabling the behavioral spec.

## Project Structure

Files involved:

```
crates/cli/src/
├── cli.rs           # Contains *_detected_section() functions (already implemented)
├── cli_tests.rs     # Unit tests for section generators (already passing)
├── init.rs          # Detection functions (Phase 1520/1525)
└── main.rs          # run_init() wires sections to output (already implemented)

tests/specs/cli/
└── init.rs          # Enable Phase 1530 spec (init_detected_language_uses_dotted_keys)
```

Reference files:

```
docs/specs/commands/quench-init.md#language-detection
docs/specs/langs/rust.md
docs/specs/langs/golang.md
docs/specs/langs/javascript.md
docs/specs/langs/shell.md
```

## Dependencies

No new dependencies. Uses existing infrastructure from Phase 1520 (language detection) and Phase 1525 (agent detection).

## Implementation Phases

### Phase 1: Verify Existing Implementation

The section generators already exist in `crates/cli/src/cli.rs`:

```rust
pub fn rust_detected_section() -> &'static str {
    r#"[rust]
rust.cloc.check = "error"
rust.policy.check = "error"
rust.suppress.check = "comment"
"#
}

pub fn golang_detected_section() -> &'static str {
    r#"[golang]
golang.cloc.check = "error"
golang.policy.check = "error"
golang.suppress.check = "comment"
"#
}

pub fn javascript_detected_section() -> &'static str {
    r#"[javascript]
javascript.cloc.check = "error"
javascript.policy.check = "error"
javascript.suppress.check = "comment"
"#
}

pub fn shell_detected_section() -> &'static str {
    r#"[shell]
shell.cloc.check = "error"
shell.policy.check = "error"
shell.suppress.check = "forbid"
"#
}
```

Verify each section follows the spec:

| Language | cloc.check | suppress.check | policy.check |
|----------|------------|----------------|--------------|
| rust | `"error"` | `"comment"` | `"error"` |
| golang | `"error"` | `"comment"` | `"error"` |
| javascript | `"error"` | `"comment"` | `"error"` |
| shell | `"error"` | **`"forbid"`** | `"error"` |

**Note:** Shell uses `"forbid"` for suppress.check per `docs/specs/langs/shell.md#suppress` (ShellCheck violations should be fixed, not suppressed).

### Phase 2: Verify Integration in run_init

The `run_init()` function in `crates/cli/src/main.rs` already appends language sections:

```rust
// No --with: run auto-detection
let detected_langs = detect_languages(&cwd);

let mut cfg = default_template().to_string();

// Add language sections
for lang in &detected_langs {
    cfg.push('\n');
    match lang {
        DetectedLanguage::Rust => cfg.push_str(rust_detected_section()),
        DetectedLanguage::Golang => cfg.push_str(golang_detected_section()),
        DetectedLanguage::JavaScript => cfg.push_str(javascript_detected_section()),
        DetectedLanguage::Shell => cfg.push_str(shell_detected_section()),
    }
}
```

The `default_template()` ends with:
```toml
# Supported Languages:
# [rust], [golang], [javascript], [shell]
```

Language sections are appended immediately after this comment block.

### Phase 3: Enable Behavioral Spec

Remove `#[ignore]` from the Phase 1530 spec in `tests/specs/cli/init.rs`:

```rust
/// Spec: docs/specs/commands/quench-init.md#language-detection
///
/// > Detected language appends [lang] section with dotted keys
#[test]
fn init_detected_language_uses_dotted_keys() {
    let temp = Project::empty();
    temp.file("Cargo.toml", "[package]\nname = \"test\"\n");

    quench_cmd()
        .args(["init"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();

    assert!(config.contains("[rust]"));
    assert!(config.contains("rust.cloc.check"));
    assert!(config.contains("rust.policy.check"));
    assert!(config.contains("rust.suppress.check"));
}
```

### Phase 4: Run Full Check

```bash
make check
```

This verifies:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --all`
- `cargo audit`
- `cargo deny check`

## Key Implementation Details

### Output Format

When `quench init` detects Rust (via `Cargo.toml`), the generated `quench.toml` includes:

```toml
# Quench configuration
# https://github.com/alfredjeanlab/quench
version = 1

[check.cloc]
check = "error"

[check.escapes]
check = "error"

[check.agents]
check = "error"

[check.docs]
check = "error"

[check.tests]
check = "off"  # stub in quench v0.3.0

[check.license]
check = "off"  # stub in quench v0.3.0

[git.commit]
check = "off"  # stub in quench v0.3.0

# Supported Languages:
# [rust], [golang], [javascript], [shell]

[rust]
rust.cloc.check = "error"
rust.policy.check = "error"
rust.suppress.check = "comment"
```

### Dotted Key Format

The language sections use dotted keys under the section header:

```toml
[rust]
rust.cloc.check = "error"      # Not [rust.cloc] with check = "error"
rust.policy.check = "error"
rust.suppress.check = "comment"
```

This format is intentional per the spec example in `docs/specs/commands/quench-init.md`.

### Shell Special Case

Shell uses `"forbid"` for suppress.check because:
- ShellCheck violations should be fixed, not suppressed
- This is the default for shell scripts (see `docs/specs/langs/shell.md`)
- Test files still allow suppression via `[shell.suppress.test].check = "allow"`

## Verification Plan

### 1. Unit Tests

The existing unit tests in `crates/cli/src/cli_tests.rs` should already pass:

```bash
cargo test cli::tests
```

### 2. Behavioral Spec

After removing `#[ignore]`:

```bash
cargo test --test specs init_detected_language_uses_dotted_keys
```

Expected: Test passes, verifying dotted keys in output.

### 3. Manual Verification

```bash
# Create temp directory and test
cd /tmp && mkdir lang-test && cd lang-test

# Test Rust detection
echo '[package]
name = "test"' > Cargo.toml
quench init
cat quench.toml | grep -A4 '\[rust\]'
# Expected:
# [rust]
# rust.cloc.check = "error"
# rust.policy.check = "error"
# rust.suppress.check = "comment"

rm quench.toml

# Test Shell detection (forbid)
mkdir scripts
echo '#!/bin/bash' > scripts/test.sh
rm Cargo.toml
quench init
cat quench.toml | grep -A4 '\[shell\]'
# Expected:
# [shell]
# shell.cloc.check = "error"
# shell.policy.check = "error"
# shell.suppress.check = "forbid"

# Cleanup
cd .. && rm -rf lang-test
```

### 4. Full Check

```bash
make check
```

### 5. Spec Coverage

| Roadmap Item | Status | Verification |
|-------------|--------|--------------|
| Generate `[lang]` section header | Already implemented | `rust_detected_section()` etc. |
| Generate `lang.cloc.check = "error"` | Already implemented | Line in each section |
| Generate `lang.suppress.check` | Already implemented | `"comment"` or `"forbid"` |
| Generate `lang.policy.check = "error"` | Already implemented | Line in each section |
| Append after `# Supported Languages:` | Already implemented | `run_init()` appends to template |
| Spec enabled | **TODO** | Remove `#[ignore]` from spec |
