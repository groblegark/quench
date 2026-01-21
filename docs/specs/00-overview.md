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

### Built-in Checks (Language Agnostic)

| Check | Description |
|-------|-------------|
| `loc` | Lines of code (source vs test separation) |
| `file-size` | File size limits (avg/max thresholds) |
| `escapes` | Pattern detection with count/require-comment/forbid modes |
| `agent` | Agent file validation (CLAUDE.md, .cursorrules, sync) |
| `test-correlation` | Changes to source require corresponding test changes |

### Language Adapters

| Adapter | Capabilities |
|---------|--------------|
| `rust` | Parse `#[cfg(test)]` blocks, separate `*_tests.rs`, cargo integration |
| `shell` | Shellcheck integration, inline disable detection |
| `generic` | Glob-based source/test detection, pattern matching |

### Future Capabilities (Later Phases)

- Coverage integration (per-language)
- Binary size tracking
- License header management (auto-fix)
- Metrics storage and trending
- GitHub Pages dashboard publishing
- Commit format validation

## Output Design

### Default: Minimal Failures

```
file-size: FAIL
  crates/core/src/parser.rs: 912 lines (max: 900)
    Split into smaller modules. Consider extracting the `TokenStream` logic.

escapes: FAIL
  crates/cli/src/main.rs:47: unsafe block without // SAFETY: comment
    Add a // SAFETY: comment explaining the invariants.
```

### JSON Mode (`-f json`)

```json
{
  "checks": [
    {
      "name": "file-size",
      "passed": false,
      "violations": [
        {
          "file": "crates/core/src/parser.rs",
          "line": null,
          "value": 912,
          "threshold": 900,
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
- LOC counting
- File size limits
- Escape hatch detection
- Agent file validation (CLAUDE.md, .cursorrules)
- Test correlation (staged changes only)

### CI Mode (`--ci`)

Full checks including slower operations:
- All fast checks
- Coverage collection
- Binary size measurement
- Full branch comparison (not just staged)

### Fix Mode (`--fix`)

Auto-fix what can be fixed:
- CLAUDE.md / .cursorrules alignment
- License headers (if enabled)
- Report what was fixed, what remains

## Success Criteria

Quench succeeds if:
1. A new project can run `quench` with zero config and get useful feedback
2. Fast checks complete in under 1 second on a 50k LOC codebase
3. AI agents can parse output and take action without additional prompting
4. Configuration is only needed for project-specific conventions
