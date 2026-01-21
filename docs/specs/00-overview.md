# Quench Overview

Quench (quality bench) is a fast, configurable quality linting CLI for software projects.

## Problem Statement

Quality checks across projects suffer from:
- **Speed**: Existing checks are too slow, blocking fast iteration
- **Setup time**: Each new project requires copy-pasting and adapting scripts
- **Consistency**: Different projects enforce different standards
- **Agent compatibility**: Checks aren't designed for AI agent consumption

## Design Philosophy

### Agent-First

Quench is designed primarily for consumption by AI coding agents (Claude, Codex, Cursor).
Human developers benefit from the same output, but agents are the primary audience.

This means:
- **Progressive disclosure**: Only surface failures, never passing checks
- **Token efficiency**: Concise output, no verbose explanations
- **Actionable advice**: Each failure includes specific, fixable guidance
- **No instructions**: Advice tells _what_ to fix, not _how_ to approach fixing

### Fast by Design

Performance is a core constraint, not an afterthought:
- **Target**: Sub-second for fast checks on typical projects
- **Acceptable**: A few seconds (1-5s) for full checks
- **Unacceptable**: Anything over 30 seconds

Design decisions prioritize speed: parallel file walking, memory-mapped I/O,
compiled pattern matching, early termination where possible.

### Convention Over Configuration

Quench should work out-of-the-box on most projects with zero configuration:
- Smart defaults for common patterns (src/, test/, *_test.*, etc.)
- Language detection enables language-specific defaults
- Configuration only needed for project-specific conventions

When configuration is needed, it's hierarchical (monorepo-friendly) and minimal.

## Core Capabilities

### Built-in Checks (Fast)

| Check | Description |
|-------|-------------|
| `loc` | Lines of code, file size limits (750 source, 1100 test) |
| `escapes` | Pattern detection with count/require-comment/forbid modes |
| `agent` | Agent file validation (CLAUDE.md, .cursorrules, sync) |
| `test-correlation` | Source changes require corresponding test changes |
| `docs-correlation` | Feature commits require doc changes (disabled by default) |

### CI Mode Metrics (Per Adapter)

| Metric | Description |
|--------|-------------|
| `coverage` | Code coverage via test suites (llvm-cov for Rust) |
| `binary_size` | Release binary sizes |
| `compile_time` | Cold (release) and hot (debug) compile times |
| `test_time` | Total, average, and max individual test times |

### Language Adapters

| Adapter | Capabilities |
|---------|--------------|
| `rust` | `#[cfg(test)]` parsing, cargo integration, llvm-cov coverage |
| `shell` | Shellcheck integration, bats test runner |
| `generic` | Glob-based source/test detection, pattern matching |

### Test Runners

Shared across adapters for test time and coverage:
- `cargo`, `bats`, `pytest`, `vitest`, `bun`, `jest`, `go`

See `04-test-runners.md` for details.

### Ratcheting

Prevent quality regressions (enabled by default for coverage and escapes):
- Coverage can't drop
- Escape counts can't increase
- Optional: binary size, compile time, test time

See `ratcheting.md` for details.

## Output Design

### Default: Minimal Failures

```
loc: FAIL
  src/parser.rs: 812 lines (max: 750)
    Split into smaller modules.

escapes: FAIL
  src/main.rs:47: unsafe block without // SAFETY: comment
    Add a // SAFETY: comment explaining the invariants.
```

### JSON Mode (`-f json`)

```json
{
  "checks": [
    {
      "name": "loc",
      "passed": false,
      "violations": [
        {
          "file": "src/parser.rs",
          "lines": 812,
          "threshold": 750,
          "advice": "Split into smaller modules."
        }
      ]
    }
  ]
}
```

### Colorization

- TTY detection: colorize if stdout is a terminal
- Agent detection: check `CLAUDE_CODE` or `CODEX` env vars, disable color if set
- Override: `--color=always|never|auto`

## Configuration

Single `quench.toml` at project root. Per-package overrides configured inline.

```
project-root/
├── quench.toml              # Single config file (optional)
├── crates/
│   ├── cli/                 # No config here - use overrides in root
│   └── core/
```

Per-package and per-module behavior defined in root config via `[checks.*.overrides.package_name]`.

## Modes

### Fast Mode (default)

Quick checks suitable for frequent runs:
- LOC counting with file size limits
- Escape hatch detection
- Agent file validation (CLAUDE.md, .cursorrules)
- Test correlation (staged changes only)

### CI Mode (`--ci`)

Full checks with multiple behavior changes:

**Enables slow metrics:**
- Coverage collection
- Binary size measurement
- Compile times (cold/hot)
- Test times (total/avg/max)

**Changes behavior:**
- Full file scanning (no early termination)
- Complete violation counts (not limited)
- Metrics storage to baseline file or git notes

**Metrics storage:**
```bash
quench --ci --save .quench/baseline.json    # Save to committed file
quench --ci --save-notes                     # Save to git notes
```

### Fix Mode (`--fix`)

Auto-fix what can be fixed:
- CLAUDE.md / .cursorrules sync
- Ratchet baseline updates (when metrics improve)
- License headers (if enabled)
- Report what was fixed, what remains

## Success Criteria

Quench succeeds if:
1. A new project can run `quench` with zero config and get useful feedback
2. Fast checks complete in under 1 second on a 50k LOC codebase
3. AI agents can parse output and take action without additional prompting
4. Configuration is only needed for project-specific conventions
