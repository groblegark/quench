# Checkpoint 15F: Quick Wins - Ratcheting

## Overview

This checkpoint delivers high-value, low-risk improvements to the ratcheting system. The ratchet infrastructure is complete (baseline I/O, metrics comparison, config), but several polish features will improve usability for both agents and human developers.

Key goals:
1. **Ratchet JSON output** - Include ratchet results in JSON for tooling integration
2. **Per-metric advice** - Context-aware advice messages for each metric type
3. **Baseline age warning** - Alert when baseline is stale and may need refresh
4. **Ratchet warn level** - Report regressions without failing (for gradual adoption)
5. **Baseline diff on regression** - Show exactly what changed when ratchet fails

## Project Structure

```
quench/
├── crates/cli/src/
│   ├── output/
│   │   ├── json.rs              # UPDATE: Add ratchet to JSON output
│   │   └── text.rs              # UPDATE: Per-metric advice messages
│   ├── config/
│   │   └── ratchet.rs           # UPDATE: Add stale_days config
│   ├── ratchet.rs               # UPDATE: Add staleness check, improve advice
│   └── baseline.rs              # UPDATE: Add age calculation
├── docs/specs/
│   ├── 03-output.md             # UPDATE: Document ratchet JSON schema
│   └── 04-ratcheting.md         # UPDATE: Add stale baseline, warn level docs
├── tests/
│   ├── specs/modes/
│   │   └── ratchet.rs           # UPDATE: Add new behavioral tests
│   └── fixtures/
│       └── ratchet/             # Existing fixtures (may add more)
└── reports/
    └── quick-wins-15f.md        # NEW: Summary of changes
```

## Dependencies

No new external dependencies. Uses existing infrastructure:

- `chrono` - Already used for baseline timestamps
- `serde_json` - Already used for JSON output
- `termcolor` - Already used for colored output

## Implementation Phases

### Phase 1: Ratchet JSON Output

**Goal:** Include ratchet comparison results in JSON output for CI tooling and programmatic access.

Currently, ratchet results only appear in text output. Agents and CI pipelines using `--output json` don't see ratchet failures in the structured output.

**File:** `crates/cli/src/output/json.rs`

Add ratchet section to output schema:

```rust
/// Ratchet comparison result for JSON output.
#[derive(Debug, Serialize)]
pub struct RatchetOutput {
    pub passed: bool,
    pub comparisons: Vec<MetricComparisonOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub improvements: Vec<MetricImprovementOutput>,
}

#[derive(Debug, Serialize)]
pub struct MetricComparisonOutput {
    pub name: String,
    pub current: f64,
    pub baseline: f64,
    pub tolerance: f64,
    pub max_allowed: f64,
    pub passed: bool,
    pub improved: bool,
}

#[derive(Debug, Serialize)]
pub struct MetricImprovementOutput {
    pub name: String,
    pub old_value: f64,
    pub new_value: f64,
}
```

**File:** `crates/cli/src/main.rs`

Pass ratchet result to JSON formatter:

```rust
OutputFormat::Json => {
    let mut formatter = JsonFormatter::new(std::io::stdout());
    formatter.write_with_ratchet(&output, ratchet_result.as_ref())?;
}
```

**Expected JSON output:**

```json
{
  "passed": false,
  "checks": [...],
  "ratchet": {
    "passed": false,
    "comparisons": [
      {
        "name": "escapes.unsafe",
        "current": 5,
        "baseline": 3,
        "tolerance": 0,
        "max_allowed": 3,
        "passed": false,
        "improved": false
      }
    ],
    "improvements": []
  }
}
```

**Verification:**
```bash
# JSON output includes ratchet
cargo run -- check --output json 2>/dev/null | jq '.ratchet'
# Should show ratchet object when baseline exists
```

### Phase 2: Per-Metric Advice Messages

**Goal:** Provide contextual, actionable advice specific to each metric type.

Current advice is generic: "Escape hatch count increased. Clean up or update baseline."
Better advice helps agents understand what action to take.

**File:** `crates/cli/src/ratchet.rs`

Add advice method to `MetricComparison`:

```rust
impl MetricComparison {
    /// Get contextual advice for this metric failure.
    pub fn advice(&self) -> &'static str {
        if self.name.starts_with("escapes.") {
            match self.name.as_str() {
                n if n.contains("unsafe") => {
                    "Reduce unsafe blocks or add // SAFETY: comments."
                }
                n if n.contains("unwrap") => {
                    "Replace .unwrap() with proper error handling."
                }
                n if n.contains("todo") || n.contains("fixme") => {
                    "Resolve TODO/FIXME comments before merging."
                }
                _ => "Reduce escape hatch usage or update baseline with --fix."
            }
        } else if self.name.starts_with("binary_size.") {
            "Reduce binary size: strip symbols, remove unused deps, enable LTO."
        } else if self.name.starts_with("build_time.") {
            "Reduce build time: check for new heavy deps or complex generics."
        } else if self.name.starts_with("test_time.") {
            "Reduce test time: parallelize tests or optimize slow tests."
        } else if self.name.starts_with("coverage.") {
            "Increase test coverage for changed code."
        } else {
            "Metric regressed. Clean up or update baseline with --fix."
        }
    }
}
```

**File:** `crates/cli/src/output/text.rs`

Use the advice method:

```rust
for comp in &result.comparisons {
    if !comp.passed {
        writeln!(
            self.stdout,
            "  {}: {} (max: {} from baseline)",
            comp.name, comp.current as i64, comp.baseline as i64
        )?;
        writeln!(self.stdout, "    {}", comp.advice())?;
    }
}
```

**Verification:**
```bash
# Create regression and check advice
cargo test --test specs ratchet_advice
# Different escapes patterns show different advice
```

### Phase 3: Baseline Age Warning

**Goal:** Warn when baseline is stale to prompt refresh.

A stale baseline (e.g., >30 days old) may not reflect current project norms. Warning helps teams maintain accurate baselines.

**File:** `crates/cli/src/config/ratchet.rs`

Add stale threshold config:

```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RatchetConfig {
    // ... existing fields ...

    /// Days before baseline is considered stale (0 to disable).
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
}

fn default_stale_days() -> u32 {
    30 // Default: warn if baseline > 30 days old
}
```

**File:** `crates/cli/src/baseline.rs`

Add age calculation:

```rust
impl Baseline {
    /// Get the age of this baseline in days.
    pub fn age_days(&self) -> i64 {
        let now = Utc::now();
        (now - self.updated).num_days()
    }

    /// Check if baseline is stale (older than threshold).
    pub fn is_stale(&self, threshold_days: u32) -> bool {
        threshold_days > 0 && self.age_days() > threshold_days as i64
    }
}
```

**File:** `crates/cli/src/main.rs`

Add stale warning after loading baseline:

```rust
match Baseline::load(&baseline_path) {
    Ok(Some(baseline)) => {
        // Warn if baseline is stale
        if baseline.is_stale(config.ratchet.stale_days) {
            eprintln!(
                "warning: baseline is {} days old. Consider refreshing with --fix.",
                baseline.age_days()
            );
        }
        // ... continue with comparison
    }
    // ...
}
```

**Config example:**
```toml
[ratchet]
stale_days = 14    # Warn if baseline > 2 weeks old
# stale_days = 0   # Disable stale warning
```

**Verification:**
```bash
# Create old baseline and check warning
cargo test --test specs ratchet_stale_warning
```

### Phase 4: Ratchet Warn Level

**Goal:** Allow reporting regressions without failing the check.

Useful for gradual adoption: see what would fail before enforcing.

**File:** `crates/cli/src/output/text.rs`

Handle warn level:

```rust
pub fn write_ratchet(
    &mut self,
    result: &RatchetResult,
    check_level: CheckLevel
) -> std::io::Result<()> {
    let has_failures = result.comparisons.iter().any(|c| !c.passed);

    if !has_failures && result.improvements.is_empty() {
        return Ok(()); // Nothing to report
    }

    self.stdout.set_color(&scheme::check_name())?;
    write!(self.stdout, "ratchet")?;
    self.stdout.reset()?;
    write!(self.stdout, ": ")?;

    if has_failures {
        if check_level == CheckLevel::Warn {
            self.stdout.set_color(&scheme::warn())?;
            writeln!(self.stdout, "WARN")?;
        } else {
            self.stdout.set_color(&scheme::fail())?;
            writeln!(self.stdout, "FAIL")?;
        }
        self.stdout.reset()?;

        for comp in &result.comparisons {
            if !comp.passed {
                writeln!(
                    self.stdout,
                    "  {}: {} (max: {} from baseline)",
                    comp.name, comp.current as i64, comp.baseline as i64
                )?;
                writeln!(self.stdout, "    {}", comp.advice())?;
            }
        }
    } else {
        // Improvements only
        self.stdout.set_color(&scheme::pass())?;
        writeln!(self.stdout, "PASS")?;
        self.stdout.reset()?;

        for comp in &result.comparisons {
            if comp.improved {
                writeln!(
                    self.stdout,
                    "  {}: {} (baseline: {}) improved",
                    comp.name, comp.current as i64, comp.baseline as i64
                )?;
            }
        }
    }

    Ok(())
}
```

**File:** `crates/cli/src/main.rs`

Update exit code logic:

```rust
let ratchet_failed = ratchet_result.as_ref().is_some_and(|r| {
    !r.passed && config.ratchet.check == CheckLevel::Error
});
```

**Config example:**
```toml
[ratchet]
check = "warn"   # Report regressions but don't fail
```

**Verification:**
```bash
# Warn level shows WARN not FAIL
echo '[ratchet]
check = "warn"' > /tmp/warn.toml
cargo run -- check -C /tmp/warn.toml
# Should show "ratchet: WARN" on regression, exit 0
```

### Phase 5: Documentation and Final Verification

**Goal:** Update specs and ensure all changes work together.

**File:** `docs/specs/03-output.md`

Add ratchet JSON schema section:

```markdown
### Ratchet Output

When ratcheting is enabled and a baseline exists, the output includes a `ratchet` object:

```json
{
  "ratchet": {
    "passed": false,
    "comparisons": [
      {
        "name": "escapes.unsafe",
        "current": 5,
        "baseline": 3,
        "tolerance": 0,
        "max_allowed": 3,
        "passed": false,
        "improved": false
      }
    ],
    "improvements": []
  }
}
```
```

**File:** `docs/specs/04-ratcheting.md`

Add sections for new features:

```markdown
### Stale Baseline Warning

Configure when to warn about old baselines:

```toml
[ratchet]
stale_days = 30    # Warn if baseline > 30 days old (default)
stale_days = 0     # Disable stale warning
```

### Warn Level

Use warn level to see regressions without failing:

```toml
[ratchet]
check = "warn"     # Report regressions, exit 0
```

This is useful for:
- Gradual adoption of ratcheting
- Informational CI runs on feature branches
- Understanding impact before enforcement
```

**Verification:**
```bash
# Full CI check
make check

# Dogfooding with ratchet
cargo run -- check

# JSON output includes ratchet
cargo run -- check --output json 2>/dev/null | jq '.ratchet'

# Stale warning appears
# (need fixture with old baseline)

# Warn level exits 0 on regression
# (need config + regression fixture)
```

## Key Implementation Details

### JSON Output Parity

The JSON output should mirror the text output information:
- When ratchet passes silently (no baseline, no comparison), omit `ratchet` key
- When ratchet has comparisons, include full comparison details
- Always include `passed` boolean at top level

### Advice Message Guidelines

Good advice is:
- Actionable: tells what to do, not just what's wrong
- Specific: different advice for different metrics
- Concise: one line, no fluff

Examples:
- "Reduce unsafe blocks or add // SAFETY: comments." (for escapes.unsafe)
- "Reduce binary size: strip symbols, remove unused deps, enable LTO." (for binary_size)
- "Increase test coverage for changed code." (for coverage)

### Stale Warning Behavior

- Only warn once per run (not per metric)
- Warning goes to stderr (doesn't interfere with JSON stdout)
- Configurable threshold, default 30 days
- Set to 0 to disable completely

### Warn vs Error Level

The check level controls:
- **error** (default): regressions cause exit code 1
- **warn**: regressions shown but exit code 0
- **off**: no ratchet checking at all

Warn level still shows the same output as error, just with WARN status and successful exit.

## Verification Plan

### Phase 1 Verification
```bash
# JSON includes ratchet when baseline exists
cargo run -- check --output json 2>/dev/null | jq '.ratchet.passed'
# Should return true or false

# JSON omits ratchet when no baseline
rm -f .quench/baseline.json
cargo run -- check --output json 2>/dev/null | jq '.ratchet'
# Should return null
```

### Phase 2 Verification
```bash
# Escape advice varies by pattern
cargo test --test specs ratchet_advice_varies
# unsafe pattern shows SAFETY comment advice
# unwrap pattern shows error handling advice
```

### Phase 3 Verification
```bash
# Stale baseline triggers warning
cargo test --test specs ratchet_stale_baseline
# Should show "baseline is X days old" on stderr
```

### Phase 4 Verification
```bash
# Warn level shows WARN and exits 0
cargo test --test specs ratchet_warn_level
# Regression shows "ratchet: WARN" and exits 0
```

### Phase 5 (Final) Verification
```bash
# Full CI
make check

# Dogfooding passes
cargo run -- check

# All ratchet specs pass
cargo test --test specs ratchet
```

## Exit Criteria

- [ ] JSON output includes `ratchet` object when baseline comparison occurs
- [ ] Per-metric advice messages vary by metric type (escapes, binary_size, etc.)
- [ ] Stale baseline warning appears when baseline > stale_days old
- [ ] Warn level reports regressions without failing (exit 0)
- [ ] `docs/specs/03-output.md` documents ratchet JSON schema
- [ ] `docs/specs/04-ratcheting.md` documents stale_days and warn level
- [ ] All tests pass: `make check`
- [ ] Dogfooding passes: `quench check` on quench
