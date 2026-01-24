# Phase 1510: Rename --profile to --with

**Root Feature:** `quench-5065`

## Overview

Rename the `--profile` CLI flag to `--with` for the `quench init` command. This aligns the implementation with the spec in `docs/specs/commands/quench-init.md` which documents `--with` as the flag for explicit profile selection.

## Project Structure

Files to modify:

```
crates/cli/src/
├── cli.rs          # InitArgs struct: rename field, update clap attribute
└── main.rs         # cmd_init function: update field references

tests/specs/cli/
└── init.rs         # Update existing --profile tests to use --with
```

## Dependencies

No new dependencies required. This is a pure refactoring task.

## Implementation Phases

### Phase 1: Update CLI Argument Definition

Modify `crates/cli/src/cli.rs`:

1. Rename `profile` field to `with_profiles` in `InitArgs`
2. Update clap attribute from `--profile` to `--with`
3. Update help text to match spec

```rust
// Before
#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long)]
    pub force: bool,

    /// Configuration profile(s) to use (e.g., rust, claude)
    #[arg(long, short, value_delimiter = ',')]
    pub profile: Vec<String>,
}

// After
#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long)]
    pub force: bool,

    /// Profile(s) to include (e.g., rust, shell, claude)
    #[arg(long = "with", value_delimiter = ',')]
    pub with_profiles: Vec<String>,
}
```

**Note:** Remove `-p` short flag as `--with` is the canonical form per spec.

### Phase 2: Update Field References in main.rs

Update `crates/cli/src/main.rs` (`cmd_init` function):

| Line | Before | After |
|------|--------|-------|
| 516 | `for profile in &args.profile` | `for profile in &args.with_profiles` |
| 537 | `if args.profile.is_empty()` | `if args.with_profiles.is_empty()` |
| 542 | `args.profile.join(", ")` | `args.with_profiles.join(", ")` |

### Phase 3: Update Behavioral Specs

Update `tests/specs/cli/init.rs`:

1. Replace `--profile` with `--with` in all test invocations
2. Remove `#[ignore]` from `init_with_accepts_comma_separated_profiles`

Tests to update:

| Test Function | Change |
|---------------|--------|
| `init_shell_profile_generates_config` | `["init", "--profile", "shell"]` → `["init", "--with", "shell"]` |
| `init_shell_profile_includes_escape_patterns` | `["init", "--profile", "shell"]` → `["init", "--with", "shell"]` |
| `init_combined_profiles_generates_both` | `["init", "--profile", "rust,shell"]` → `["init", "--with", "rust,shell"]` |
| `init_shell_profile_message` | `["init", "--profile", "shell"]` → `["init", "--with", "shell"]` |
| `init_with_accepts_comma_separated_profiles` | Remove `#[ignore]` (already uses `--with`) |

## Key Implementation Details

### Short Flag Removal

The `--profile` flag had `-p` as a short form. The `--with` flag does not have a short form per the spec. This is intentional for clarity.

### Backward Compatibility

No backward compatibility is needed. The `--profile` flag was an internal implementation detail that never matched the spec. The specs written in Phase 1505 already use `--with`.

### Field Naming Convention

The field is named `with_profiles` (not just `with`) because:
- `with` is a Rust keyword and cannot be used as an identifier
- `with_profiles` is descriptive and matches the semantics

## Verification Plan

### 1. Unit Test Check

```bash
cargo test --package quench cli_tests
```

### 2. Behavioral Spec Check

Run init-related specs:

```bash
cargo test --test specs init
```

Expected: 4 tests pass (existing profile tests now using `--with`), plus 1 newly enabled test.

### 3. Verify --with Flag Works

```bash
cargo build --release
./target/release/quench init --with rust 2>&1 | head -5
```

Expected output: `Created quench.toml with profile(s): rust`

### 4. Verify Old Flag Is Gone

```bash
./target/release/quench init --profile rust 2>&1
```

Expected: Error about unknown flag `--profile`

### 5. Full Check

```bash
make check
```

### 6. Spec Coverage

| Spec Requirement | Test Function | Status |
|-----------------|---------------|--------|
| --with accepts comma-separated | `init_with_accepts_comma_separated_profiles` | ✓ Enabled |
| --with rust creates rust config | `init_shell_profile_generates_config` (adapted) | ✓ Pass |
| --with rust,shell creates both | `init_combined_profiles_generates_both` | ✓ Pass |
