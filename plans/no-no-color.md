# Plan: Remove --[no-]color Flags, Use NO_COLOR/COLOR Env Vars

## Overview

Replace the `--color` and `--no-color` CLI flags with support for the Unix-standard `NO_COLOR` and `COLOR` environment variables. This follows the [no-color.org](https://no-color.org/) convention adopted by many CLI tools.

**Current behavior:**
```
if --no-color:
    no color
elif --color:
    use color
else (default):
    if not stdout.is_tty():
        no color
    elif env.CLAUDE_CODE or env.CODEX or env.CI:
        no color
    else:
        use color
```

**Target behavior:**
```
if env.NO_COLOR:
    no color
elif env.COLOR:
    use color
else (default):
    if not stdout.is_tty():
        no color
    elif env.CLAUDE_CODE or env.CODEX or env.CI or env.CURSOR:
        no color
    else:
        use color
```

## Project Structure

Files to modify:
```
crates/cli/src/
├── cli.rs           # Remove --color and --no-color from CheckArgs
├── color.rs         # Update resolve_color() to use env vars
├── color_tests.rs   # Update unit tests for env var behavior
├── cmd_check.rs     # Simplify resolve_color() call
└── help.rs          # Remove color flag consolidation (no longer needed)

docs/specs/
└── 03-output.md     # Update colorization detection spec

tests/specs/
├── output/format.rs # Update colorization tests
├── cli/help.rs      # Remove --[no-]color help test
└── config/env.rs    # Update env var tests
```

## Dependencies

None - uses existing `termcolor` crate and `std::env`.

## Implementation Phases

### Phase 1: Update Color Resolution Logic

**File:** `crates/cli/src/color.rs`

1. Remove `is_no_color_env()` function (QUENCH_NO_COLOR deprecated)
2. Update `resolve_color()` to check env vars instead of CLI flags:

```rust
/// Resolve color choice from environment variables.
///
/// Priority: NO_COLOR > COLOR > auto-detect
pub fn resolve_color() -> ColorChoice {
    // NO_COLOR spec: any value (including empty) disables color
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorChoice::Never;
    }
    // COLOR=1 forces color (non-standard but common)
    if std::env::var_os("COLOR").is_some() {
        return ColorChoice::Always;
    }
    // Auto-detect
    if !std::io::stdout().is_terminal() {
        return ColorChoice::Never;
    }
    if is_agent_environment() {
        return ColorChoice::Never;
    }
    ColorChoice::Auto
}
```

3. Keep `is_agent_environment()` unchanged

**File:** `crates/cli/src/color_tests.rs`

Update tests:
- Remove tests for `--color` and `--no-color` flags
- Add tests for `NO_COLOR` and `COLOR` env vars
- Test priority: `NO_COLOR` > `COLOR` > auto-detect

### Phase 2: Remove CLI Flags

**File:** `crates/cli/src/cli.rs`

Remove from `CheckArgs`:
```rust
// DELETE these lines (44-50):
/// Force color output
#[arg(long)]
pub color: bool,

/// Disable color output
#[arg(long)]
pub no_color: bool,
```

**File:** `crates/cli/src/cmd_check.rs`

Update the call at line ~460:
```rust
// FROM:
let color_choice = resolve_color(args.color, args.no_color || is_no_color_env());

// TO:
let color_choice = resolve_color();
```

### Phase 3: Update Help Formatting

**File:** `crates/cli/src/help.rs`

The consolidation logic for `--[no-]color` is no longer needed. The help.rs module can remain as-is since it only consolidates flag pairs that exist - with `--color`/`--no-color` removed, it will simply not find a pair to consolidate.

No changes required.

### Phase 4: Update Spec Documentation

**File:** `docs/specs/03-output.md`

Update the colorization detection logic section (around line 269):

```markdown
## Colorization

### Detection Logic

```
if env.NO_COLOR:
    no color
elif env.COLOR:
    use color
else (default):
    if not stdout.is_tty():
        no color
    elif env.CLAUDE_CODE or env.CODEX or env.CI or env.CURSOR:
        no color
    else:
        use color
```

The `NO_COLOR` and `COLOR` environment variables follow the [no-color.org](https://no-color.org/) convention.

- `NO_COLOR`: Any value (including empty string) disables color
- `COLOR`: Any value forces color output (overrides TTY/agent detection)
```

### Phase 5: Update Behavioral Tests

**File:** `tests/specs/output/format.rs`

Update `no_color_flag_disables_color` test:
```rust
/// Spec: docs/specs/03-output.md#colorization
///
/// > Color disabled when NO_COLOR env var is set
#[test]
fn no_color_env_disables_color() {
    // No ANSI escape codes in output
    cli()
        .on("output-test")
        .env("NO_COLOR", "1")
        .exits(1)
        .stdout_lacks("\x1b[");
}
```

Add test for `COLOR` forcing color:
```rust
/// Spec: docs/specs/03-output.md#colorization
///
/// > Color enabled when COLOR env var is set (even without TTY)
#[test]
fn color_env_forces_color() {
    // Note: This test runs in a non-TTY environment (piped stdout)
    // COLOR should force color output regardless
    cli()
        .on("output-test")
        .env("COLOR", "1")
        .exits(1)
        .stdout_has("\x1b[");
}
```

**File:** `tests/specs/cli/help.rs`

Remove or update `check_help_shows_consolidated_color_flag` test since `--[no-]color` will no longer exist.

**File:** `tests/specs/config/env.rs`

Update `env_no_color_disables_color` test:
```rust
/// Spec: docs/specs/03-output.md#colorization
///
/// > NO_COLOR=1 disables color output
#[test]
fn env_no_color_disables_color() {
    let temp = Project::empty();
    temp.config(MINIMAL_CONFIG);
    temp.file("test.rs", "fn main() {}\n");

    let output = quench_cmd()
        .arg("check")
        .current_dir(temp.path())
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // ANSI escape codes start with \x1b[
    assert!(
        !stdout.contains("\x1b["),
        "output should not contain ANSI codes"
    );
}
```

### Phase 6: Cleanup & Verification

1. Remove deprecated `QUENCH_NO_COLOR` handling
2. Run `make check` to verify all tests pass
3. Bump `CACHE_VERSION` if any check logic changed (not needed here - color is output-only)

## Key Implementation Details

### NO_COLOR Specification

Per [no-color.org](https://no-color.org/):
- `NO_COLOR` when set to **any value** (including empty string) disables color
- Check with `std::env::var_os("NO_COLOR").is_some()`, NOT by checking the value

### COLOR Convention

While not part of the official no-color spec, `COLOR` or `CLICOLOR_FORCE` is commonly used to force color:
- Similar behavior: any value forces color
- Takes precedence over TTY detection and agent environment
- Does NOT take precedence over `NO_COLOR` (explicit disable wins)

### Backward Compatibility

- `QUENCH_NO_COLOR` is being removed in favor of standard `NO_COLOR`
- Users should update scripts to use `NO_COLOR=1` instead
- No deprecation warning needed - the migration is straightforward

## Verification Plan

1. **Unit tests** (`cargo test`):
   - `NO_COLOR=1` returns `ColorChoice::Never`
   - `COLOR=1` returns `ColorChoice::Always`
   - `NO_COLOR` takes priority over `COLOR`
   - Auto-detect still works (TTY check, agent environment)

2. **Behavioral specs** (`cargo test --test specs`):
   - `env_no_color_disables_color` passes
   - `color_env_forces_color` passes
   - `color_disabled_when_claude_code_env_set` passes
   - `color_disabled_when_not_tty` passes
   - Help output no longer shows `--[no-]color`

3. **Manual verification**:
   ```bash
   # Verify NO_COLOR works
   NO_COLOR=1 cargo run -- check

   # Verify COLOR forces color (even when piped)
   COLOR=1 cargo run -- check | cat

   # Verify agent detection still works
   CLAUDE_CODE=1 cargo run -- check
   ```

4. **Run full check**:
   ```bash
   make check
   ```
