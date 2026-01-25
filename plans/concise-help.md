# Plan: Concise Help Formatting

## Overview

Consolidate `--flag` / `--no-flag` pairs into single `--[no-]flag` help entries.
Currently, clap displays each flag on a separate line, making help output verbose.
This plan implements custom help formatting to display negatable flags more concisely.

## Project Structure

```
crates/cli/src/
├── cli.rs           # Existing: Flag definitions (modify)
├── cli_tests.rs     # Existing: Parsing tests (extend)
├── help.rs          # New: Custom help formatting
└── help_tests.rs    # New: Help completeness tests

tests/specs/
└── help_spec.rs     # New: Help output behavioral tests
```

## Dependencies

No new dependencies required. Clap 4's derive macros and `clap::Command` API
provide sufficient customization hooks.

## Implementation Phases

### Phase 1: Research and Baseline

**Goal:** Capture current help output and understand clap's customization points.

1. Add snapshot test capturing current `--help` output for all commands
2. Document all negatable flag pairs in the codebase
3. Verify clap doesn't have built-in `--[no-]` formatting (it doesn't as of v4)

**Negatable flag pairs identified:**
- `--color` / `--no-color`
- `--limit` / `--no-limit`
- `--cache` (only `--no-cache` exists)
- 9 check toggles: `--cloc`, `--escapes`, `--agents`, `--docs`, `--tests`,
  `--git`, `--build`, `--license`, `--placeholders` (each with `--no-` variant)

**Verification:** Baseline help snapshot tests pass.

---

### Phase 2: Implement Help Formatter

**Goal:** Create custom help renderer that combines negatable flag pairs.

Create `crates/cli/src/help.rs` with:

```rust
use clap::Command;

/// Formats help text with consolidated --[no-] flags
pub fn format_help(cmd: &Command) -> String {
    let mut help = Vec::new();
    cmd.write_help(&mut help).unwrap();
    let raw_help = String::from_utf8(help).unwrap();
    consolidate_negatable_flags(&raw_help)
}

/// Identifies --flag/--no-flag pairs and merges them
fn consolidate_negatable_flags(help: &str) -> String {
    // Parse help into sections (Options, Arguments, Commands)
    // For each --no-X line, find matching --X line
    // Combine into --[no-]X with merged description
    // Remove the --no-X line
}
```

**Approach:**
1. Parse raw help text line by line
2. Identify lines starting with `--no-` pattern
3. Find corresponding positive flag line
4. Merge into `--[no-]flag` with positive flag's description
5. Remove redundant `--no-` line

**Edge cases:**
- `--no-cache` has no positive counterpart → keep as-is
- `--no-limit` pairs with `--limit <N>` → format as `--[no-]limit [N]`
- Preserve alignment and formatting

**Verification:** Unit tests for consolidation logic.

---

### Phase 3: Integrate Custom Help

**Goal:** Replace default clap help with custom formatter.

Modify `crates/cli/src/main.rs`:

```rust
use crate::help::format_help;

fn main() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) if e.kind() == clap::error::ErrorKind::DisplayHelp => {
            print!("{}", format_help(&Cli::command()));
            return Ok(());
        }
        Err(e) if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            print!("{}", format_help(&Cli::command()));
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // Handle subcommand help
    match &cli.command {
        Some(Command::Help) => {
            print!("{}", format_help(&Cli::command()));
            return Ok(());
        }
        // ... existing logic
    }
}
```

Also handle `quench check --help` by formatting subcommand help.

**Verification:** Running `quench --help` shows consolidated flags.

---

### Phase 4: Help Completeness Tests

**Goal:** Ensure custom formatting doesn't lose any content.

Create `crates/cli/src/help_tests.rs`:

```rust
#[test]
fn help_contains_all_flags() {
    let help = format_help(&Cli::command());

    // Every defined flag should appear in help
    assert!(help.contains("--config") || help.contains("-C"));
    assert!(help.contains("--output") || help.contains("-o"));
    assert!(help.contains("--[no-]color") || help.contains("--color"));
    assert!(help.contains("--[no-]limit"));
    assert!(help.contains("--no-cache")); // No positive variant

    // Check toggles
    for check in ["cloc", "escapes", "agents", "docs", "tests",
                  "git", "build", "license", "placeholders"] {
        assert!(help.contains(&format!("--[no-]{check}")),
                "Missing --[no-]{check}");
    }
}

#[test]
fn help_contains_all_subcommands() {
    let help = format_help(&Cli::command());
    assert!(help.contains("check"));
    assert!(help.contains("report"));
    assert!(help.contains("init"));
}

#[test]
fn subcommand_help_is_complete() {
    let cmd = Cli::command();
    let check_cmd = cmd.find_subcommand("check").unwrap();
    let help = format_help(check_cmd);

    // Verify check-specific flags
    assert!(help.contains("--[no-]cloc"));
    assert!(help.contains("--baseline") || help.contains("-b"));
}
```

Create `tests/specs/help_spec.rs` for behavioral tests:

```rust
#[test]
fn help_shows_consolidated_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_quench"))
        .args(["check", "--help"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show consolidated format
    assert!(stdout.contains("--[no-]color"));

    // Should NOT show separate --no-color line
    let no_color_count = stdout.matches("--no-color").count();
    assert_eq!(no_color_count, 0, "Found separate --no-color line");
}
```

**Verification:** All completeness tests pass.

---

### Phase 5: Edge Cases and Polish

**Goal:** Handle all edge cases cleanly.

1. **`--no-cache` without `--cache`:** Keep as standalone `--no-cache`
2. **`--limit <N>` with `--no-limit`:** Format as `--[no-]limit [N]`
3. **Help for `quench` vs `quench check`:** Both should use custom formatter
4. **`quench help check`:** Should also use custom formatter
5. **Error messages:** Clap error messages mentioning flags should still work

Add tests for each edge case:

```rust
#[test]
fn standalone_no_flag_preserved() {
    let help = format_help(&Cli::command());
    // --no-cache has no --cache counterpart
    assert!(help.contains("--no-cache"));
    assert!(!help.contains("--[no-]cache"));
}

#[test]
fn flag_with_value_formats_correctly() {
    let cmd = Cli::command();
    let check_cmd = cmd.find_subcommand("check").unwrap();
    let help = format_help(check_cmd);

    // --limit takes a value, --no-limit doesn't
    // Should show as: --[no-]limit [N]
    assert!(help.contains("--[no-]limit"));
}
```

**Verification:** All edge case tests pass, `make check` succeeds.

---

### Phase 6: Documentation Update

**Goal:** Update spec to reflect implementation.

1. Update `docs/specs/01-cli.md` if help format is specified there
2. Ensure examples in docs match actual output

**Verification:** Docs match reality, full `make check` passes.

## Key Implementation Details

### Help Text Parsing Strategy

Parse clap's help output using these patterns:

```rust
// Identify option lines (start with whitespace + --)
let option_line_re = Regex::new(r"^\s+(--\S+)").unwrap();

// Identify --no-X flags
let no_flag_re = Regex::new(r"--no-(\w+)").unwrap();

// Match value placeholders
let value_re = Regex::new(r"--(\w+)\s+<([^>]+)>").unwrap();
```

### Consolidation Algorithm

```
1. Parse help into lines
2. Build map: flag_name -> (line_index, has_value, value_name)
3. For each --no-X flag:
   a. If X exists in map:
      - Modify X's line to use --[no-]X format
      - If X has value, make it optional: --[no-]X [VALUE]
      - Mark --no-X line for removal
   b. If X doesn't exist:
      - Keep --no-X line unchanged
4. Remove marked lines
5. Reconstruct help text
```

### Alignment Preservation

Clap aligns descriptions. After consolidation, re-align:

```rust
fn realign_options(lines: &mut [String]) {
    // Find maximum flag width
    let max_width = lines.iter()
        .filter(|l| l.trim_start().starts_with('-'))
        .map(|l| flag_width(l))
        .max()
        .unwrap_or(0);

    // Pad all flags to match
    for line in lines {
        if line.trim_start().starts_with('-') {
            *line = pad_flag_to(line, max_width);
        }
    }
}
```

## Verification Plan

### Unit Tests (`help_tests.rs`)

| Test | Description |
|------|-------------|
| `consolidates_simple_pair` | `--color`/`--no-color` → `--[no-]color` |
| `preserves_standalone_no_flag` | `--no-cache` stays as-is |
| `handles_flag_with_value` | `--limit <N>`/`--no-limit` → `--[no-]limit [N]` |
| `preserves_descriptions` | Description text not lost |
| `maintains_alignment` | All descriptions align |
| `handles_empty_help` | No crash on edge cases |

### Completeness Tests (`help_tests.rs`)

| Test | Description |
|------|-------------|
| `help_contains_all_flags` | Every CLI flag appears in help |
| `help_contains_all_subcommands` | All subcommands listed |
| `subcommand_help_is_complete` | Subcommand flags present |
| `no_duplicate_flag_entries` | Each flag appears exactly once |

### Behavioral Tests (`tests/specs/help_spec.rs`)

| Test | Description |
|------|-------------|
| `main_help_uses_consolidated_format` | `quench --help` shows `--[no-]` |
| `check_help_uses_consolidated_format` | `quench check --help` shows `--[no-]` |
| `report_help_uses_consolidated_format` | `quench report --help` shows `--[no-]` |
| `help_subcommand_works` | `quench help check` works |

### Integration Verification

```bash
# Full test suite
make check

# Manual verification
quench --help
quench check --help
quench report --help
quench help check
```

## Expected Help Output (After)

```
quench check --help

Run quality checks

Usage: quench check [OPTIONS] [PATH]...

Arguments:
  [PATH]...  Files or directories to check

Options:
  -C, --config <CONFIG>    Use specific config file [env: QUENCH_CONFIG=]
  -o, --output <FORMAT>    Output format [default: text] [possible values: text, json, html, markdown]
      --[no-]color         Force/disable color output
      --[no-]limit [N]     Violations to display (default: 15); --no-limit shows all
      --no-cache           Bypass file cache
      --[no-]cloc          Run only / skip the cloc check
      --[no-]escapes       Run only / skip the escapes check
      ...
  -h, --help               Print help
```
