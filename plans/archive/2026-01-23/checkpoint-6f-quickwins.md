# Checkpoint 6F: Quick Wins - Dogfooding Milestone 1

**Root Feature:** `quench-2bcc`

## Overview

This checkpoint delivers high-value, low-risk improvements to enhance the dogfooding experience. With core functionality validated (6B), refactoring complete (6C), and performance infrastructure in place (6E), this checkpoint focuses on polish that makes quench more usable in daily AI agent workflows.

Key goals:
1. **Go profile support** - Add `quench init --profile golang` for Go projects
2. **Improved violation context** - Show surrounding code for better actionability
3. **Config validation UX** - Clearer error messages for misconfiguration
4. **Baseline auto-improvement** - Auto-update baseline when metrics improve (ratchet up)
5. **Documentation sync** - Ensure Go adapter documentation is complete

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── cli.rs                  # UPDATE: Add golang profile option
│   ├── init.rs                 # UPDATE: Add Go profile template
│   ├── config/
│   │   ├── mod.rs              # UPDATE: Add Go profile defaults
│   │   └── go.rs               # EXISTS: Go config (may need profile defaults)
│   ├── output/
│   │   └── text.rs             # UPDATE: Add context lines for violations
│   ├── baseline.rs             # UPDATE: Auto-improve baseline logic
│   └── runner.rs               # UPDATE: Connect baseline improvement
├── docs/specs/
│   ├── 01-cli.md               # UPDATE: Document golang profile
│   └── langs/golang.md         # UPDATE: Add profile defaults section
├── tests/
│   ├── specs/
│   │   ├── cli/
│   │   │   └── init.rs         # UPDATE: Add Go profile tests
│   │   └── output/
│   │       └── formatting.rs   # UPDATE: Add context line tests
│   └── fixtures/
│       └── go-init/            # NEW: Go initialization fixture
└── reports/
    └── quick-wins-6f.md        # NEW: Summary of changes
```

## Dependencies

No new external dependencies. This checkpoint uses existing infrastructure:

- `clap` - CLI argument parsing (exists)
- `toml` - Configuration generation (exists)
- `similar` - Diff generation for context (exists)

## Implementation Phases

### Phase 1: Go Profile for `quench init`

**Goal:** Enable `quench init --profile golang` to generate Go-specific configuration.

The CLI spec already documents profile support for `rust` and `shell`. With the Go adapter complete (checkpoint-go-1e), users should be able to initialize Go projects with appropriate defaults.

**File:** `crates/cli/src/cli.rs`

Update profile validation to include `golang`:

```rust
// Valid profiles: rust, shell, golang, claude, cursor
fn validate_profiles(profiles: &[String]) -> Result<(), String> {
    let valid = ["rust", "shell", "golang", "claude", "cursor"];
    for p in profiles {
        if !valid.contains(&p.as_str()) {
            return Err(format!("unknown profile: {p}"));
        }
    }
    Ok(())
}
```

**File:** `crates/cli/src/init.rs`

Add Go profile template:

```rust
fn golang_profile() -> &'static str {
    r#"
[golang]
binary_size = true
build_time = true

[golang.suppress]
check = "comment"               # Require comment for //nolint

[golang.policy]
lint_changes = "standalone"     # Lint config changes need separate PR
lint_config = [".golangci.yml", ".golangci.yaml", "golangci.toml"]
"#
}
```

**File:** `crates/cli/src/config/mod.rs`

Add Go detection to auto-detection:

```rust
fn detect_languages(root: &Path) -> Vec<&'static str> {
    let mut langs = vec![];
    if root.join("Cargo.toml").exists() {
        langs.push("rust");
    }
    if root.join("go.mod").exists() {
        langs.push("golang");
    }
    if has_shell_files(root) {
        langs.push("shell");
    }
    langs
}
```

**Verification:**
```bash
# Test profile generation
cargo run -- init --profile golang --dry-run
# Should show Go-specific config

# Test auto-detection
cd tests/fixtures/go-simple && cargo run -- init --dry-run
# Should detect go.mod and suggest golang profile
```

### Phase 2: Violation Context Lines

**Goal:** Show surrounding code when reporting violations for better actionability.

Currently violations show file:line with the violation message. Adding 1-2 lines of context helps agents understand and fix issues faster.

**File:** `crates/cli/src/output/text.rs`

Add context line rendering:

```rust
struct ViolationContext {
    before: Vec<String>,  // Lines before violation (0-2)
    line: String,         // The violation line
    after: Vec<String>,   // Lines after violation (0-2)
    line_num: usize,
}

fn format_violation_with_context(
    file: &Path,
    line_num: usize,
    violation: &str,
    context_lines: usize,
) -> String {
    // Read surrounding lines from file
    // Format with line numbers, highlight violation line
    // Keep total context small for token efficiency
}
```

**Output example:**
```
escapes: FAIL
  src/parser.rs:42: unsafe_block
     40 |     let result = compute();
     41 |     // SAFETY: input validated above
  >  42 |     unsafe { ptr.read() }
     43 |     process(result)
    Add a // SAFETY: comment explaining why this is sound.
```

**Configuration:**
```toml
[output]
context_lines = 2              # Lines before/after violations (default: 2)
# context_lines = 0            # Disable context (minimal output)
```

**Verification:**
```bash
cargo test --test specs formatting
# Verify context appears in output
```

### Phase 3: Config Validation UX

**Goal:** Provide clear, actionable error messages for configuration problems.

Common configuration mistakes should produce helpful error messages that guide users to fixes.

**File:** `crates/cli/src/config/mod.rs`

Improve error messages:

```rust
#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error(
        "unknown check '{check}' in quench.toml\n  \
         Valid checks: cloc, escapes, agents, docs, tests, git, build, license\n  \
         Did you mean '{suggestion}'?"
    )]
    UnknownCheck { check: String, suggestion: String },

    #[error(
        "invalid escape action '{action}' for pattern '{pattern}'\n  \
         Valid actions: count, comment, forbid"
    )]
    InvalidEscapeAction { action: String, pattern: String },

    #[error(
        "missing required field '{field}' in [{section}]\n  \
         Example:\n  {example}"
    )]
    MissingField { field: String, section: String, example: String },
}

fn suggest_check(unknown: &str) -> &'static str {
    // Levenshtein distance to suggest closest match
    match unknown {
        "escape" | "escaps" => "escapes",
        "agent" | "claude" => "agents",
        "test" | "testing" => "tests",
        "doc" | "documentation" => "docs",
        "loc" | "lines" => "cloc",
        _ => "cloc",
    }
}
```

**Verification:**
```bash
# Create invalid config and check error message
echo '[check.escaps]' > /tmp/bad.toml
cargo run -- check -C /tmp/bad.toml
# Should suggest "escapes"
```

### Phase 4: Baseline Auto-Improvement

**Goal:** Automatically update baseline when metrics improve (ratchet up).

When coverage increases or escape counts decrease, the baseline should update to lock in the improvement. This implements the "metrics can improve, never regress" philosophy.

**File:** `crates/cli/src/baseline.rs`

Add improvement detection and update:

```rust
/// Check if new metrics are strictly better than baseline.
fn metrics_improved(baseline: &Metrics, current: &Metrics) -> bool {
    // Coverage increased
    let coverage_better = current.coverage >= baseline.coverage;
    // Escape counts decreased or stayed same
    let escapes_better = current.escapes <= baseline.escapes;
    // At least one metric actually improved
    let any_improved = current.coverage > baseline.coverage
        || current.escapes < baseline.escapes
        || current.build_time < baseline.build_time;

    coverage_better && escapes_better && any_improved
}

/// Update baseline file when metrics improve.
fn maybe_update_baseline(
    baseline_path: &Path,
    baseline: &Metrics,
    current: &Metrics,
    dry_run: bool,
) -> Result<bool> {
    if !metrics_improved(baseline, current) {
        return Ok(false);
    }

    if dry_run {
        println!("baseline would improve:");
        print_improvements(baseline, current);
        return Ok(true);
    }

    // Write updated baseline
    let content = toml::to_string_pretty(current)?;
    std::fs::write(baseline_path, content)?;

    println!("baseline improved:");
    print_improvements(baseline, current);

    Ok(true)
}
```

**CLI integration:**
```bash
quench check --ci --save .quench/baseline.json
# If metrics improved, automatically updates baseline
# If metrics regressed, fails with diff
```

**Verification:**
```bash
# Manually lower escape count in test fixture
# Run check, verify baseline updates
cargo test --test specs baseline_improvement
```

### Phase 5: Documentation Sync

**Goal:** Ensure Go adapter documentation is complete and consistent with implementation.

**File:** `docs/specs/01-cli.md`

Add golang to profile table:

```markdown
| Profile | Description |
|---------|-------------|
| `rust` | Cargo workspace, clippy escapes, unsafe/unwrap detection |
| `shell` | Shellcheck integration, set +e/eval escapes |
| `golang` | Go modules, nolint escapes, unsafe.Pointer detection |
| `claude` | CLAUDE.md with required sections, sync setup |
| `cursor` | .cursorrules with required sections, sync setup |
```

**File:** `docs/specs/langs/golang.md`

Add profile defaults section (matching rust.md and shell.md pattern):

```markdown
## Profile Defaults

When initializing with `--profile golang`, the following defaults are applied:

### Default Configuration

```toml
[golang]
binary_size = true              # Track go build binary sizes
build_time = true               # Track build times

[golang.suppress]
check = "comment"               # //nolint requires adjacent comment
# allow = ["errcheck"]          # Codes that don't need comment
# forbid = []                   # Codes never allowed

[golang.policy]
lint_changes = "standalone"     # golangci.yml changes need separate PR
lint_config = [".golangci.yml", ".golangci.yaml", "golangci.toml"]
```

### Escape Pattern Defaults

```toml
[check.escapes.go]
patterns = [
    { pattern = "unsafe\\.Pointer", action = "comment" },
    { pattern = "//go:linkname", action = "forbid" },
    { pattern = "//go:noescape", action = "comment" },
    { pattern = "//go:nosplit", action = "comment" },
]
```

### Integration with quench init

```bash
# Initialize Go project
quench init --profile golang

# Initialize mixed project
quench init --profile golang,rust

# Auto-detect (will detect go.mod)
quench init
```
```

**Verification:**
```bash
# Documentation builds without warnings
# All examples are valid TOML
```

### Phase 6: Final Verification

**Goal:** Ensure all changes work together and pass CI.

**Steps:**
1. Run full test suite
2. Dogfood: run quench on quench
3. Verify Go project initialization works end-to-end
4. Update dogfooding report with quick wins summary

**Verification:**
```bash
# Full CI check
make check

# Dogfooding
cargo run -- check

# Go profile end-to-end
cargo run -- init --profile golang --dry-run

# Verify context lines appear
cargo run -- check -o text 2>&1 | head -50
```

## Key Implementation Details

### Profile System Architecture

Profiles are additive and composable:

```
quench init --profile rust,golang,claude
```

Generates config that combines:
1. Rust-specific settings (`[rust]` section)
2. Go-specific settings (`[golang]` section)
3. Agent-specific settings (`[check.agents]` section)

**Merge strategy:** Later profiles override earlier ones for conflicting keys.

### Context Lines Trade-offs

Context lines improve actionability but increase token usage:

| Lines | Tokens (approx) | Use case |
|-------|-----------------|----------|
| 0 | Minimal | Agent with tight context |
| 2 | +60-100 per violation | Default, good balance |
| 5 | +150-200 per violation | Human review |

Default to 2 lines. Respect `--no-limit` to also expand context.

### Baseline Improvement Safety

Only auto-update baseline when ALL of:
1. No metric regressed
2. At least one metric improved
3. In CI mode (`--ci` flag)

This prevents accidental baseline corruption while enabling gradual improvement.

### Error Message Patterns

Good error messages follow this structure:
```
what went wrong
  where: specific location
  why: explanation
  fix: concrete action
```

Example:
```
unknown check 'escaps' in quench.toml
  at: quench.toml:15
  Valid checks: cloc, escapes, agents, docs, tests, git, build, license
  Did you mean 'escapes'?
```

## Verification Plan

### Phase 1 Verification
```bash
# Go profile generates valid config
cargo run -- init --profile golang --dry-run | toml-lint

# Auto-detection finds go.mod
cd tests/fixtures/go-simple
cargo run -- init --dry-run
# Should show golang in detected languages
```

### Phase 2 Verification
```bash
# Context lines appear in output
cargo run -- check 2>&1 | grep -A3 "FAIL"
# Should show numbered context lines

# Context respects config
echo '[output]
context_lines = 0' > /tmp/no-context.toml
cargo run -- check -C /tmp/no-context.toml 2>&1
# Should show no context
```

### Phase 3 Verification
```bash
# Typo suggestions work
echo '[check.escaps]' > /tmp/bad.toml
cargo run -- check -C /tmp/bad.toml 2>&1 | grep "Did you mean"
# Should suggest 'escapes'
```

### Phase 4 Verification
```bash
# Baseline improvement detection
cargo test --test specs baseline
# All baseline tests pass

# Dry-run shows would-improve
cargo run -- check --ci --dry-run
# Should show "baseline would improve" when metrics better
```

### Phase 5 Verification
```bash
# Documentation examples are valid TOML
grep -A20 '```toml' docs/specs/langs/golang.md | grep -v '```' | toml-lint

# All profile references consistent
grep -r "profile" docs/specs/01-cli.md | grep golang
# Should mention golang
```

### Phase 6 (Final) Verification
```bash
# Full CI
make check

# Dogfooding passes
cargo run -- check

# Go project workflow
mkdir /tmp/go-test && cd /tmp/go-test
go mod init test
cargo run -- init
cargo run -- check
# Should work end-to-end
```

## Exit Criteria

- [ ] `quench init --profile golang` generates valid Go configuration
- [ ] Auto-detection recognizes `go.mod` and suggests golang profile
- [ ] Violations show context lines (configurable, default 2)
- [ ] Config errors show helpful suggestions for typos
- [ ] Baseline auto-updates when metrics improve in CI mode
- [ ] `docs/specs/01-cli.md` documents golang profile
- [ ] `docs/specs/langs/golang.md` has profile defaults section
- [ ] All tests pass: `make check`
- [ ] Dogfooding passes: `quench check` on quench
