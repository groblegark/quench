# Quench

A fast linting tool for AI agents that measures quality signals.

## Installation

```bash
# macOS
brew install alfredjeanlab/tap/quench

# Linux / macOS (manual)
curl -fsSL https://github.com/alfredjeanlab/quench/releases/latest/download/install.sh | bash
```

## Supported Languages

- **Rust** - `.rs` files, `#[allow]` suppression tracking, cargo integration
- **Go** - `.go` files, `//nolint` suppression tracking, go test integration
- **JavaScript/TypeScript** - `.js`/`.ts` files, eslint suppression tracking, vitest/jest/bun
- **Python** - `.py` files, `# noqa`/`# type: ignore` suppression, pytest integration
- **Ruby** - `.rb` files, rubocop suppression tracking, RSpec/Minitest integration
- **Shell** - `.sh`/`.bash` files, shellcheck directive tracking, bats integration
- **Other** - Basic cloc checks work on any text file

## Quick Start

```bash
quench init    # Create quench.toml
quench check   # Run quality checks
```

## Configuration

```toml
version = 1

[check.cloc]
exclude = ["tests/fixtures/**"]
advice_test = "Use table-driven tests with t.Run()."  # customize advice for project

[check.agents]
required = ["CLAUDE.md"]  # default: ["*"] any agent file
tables = "forbid"         # default: "allow"
# mermaid = "allow/forbid"
# box_diagrams = "allow/forbid"

# Per-scope config:
#   .root (project, monorepo or workspace)
#   .package (crates/, packages/, etc)
#   .module (package subdirs)
[check.agents.root]
max_lines = 750  # default: 500

[check.agents.package]
required = ["CLAUDE.md"]  # require agent file in each package
sections.required = ["API"]  # with an "API" section

[[check.agents.sections.required]]
name = "Error Handling"
advice = "Document expected errors and recovery strategies"
```

## Checks

| Check | What it does |
|-------|--------------|
| `cloc` | Enforces file size limits (lines, tokens) |
| `escapes` | Flags escape hatches (`.unwrap()`, `unsafe`, `set +e`) and validates lint suppressions (`#[allow]`, `//nolint`) |
| `agents` | Validates AI context files (CLAUDE.md, .cursorrules) |
| `docs` | Validates documentation structure, TOC entries, and markdown links |
| `tests` | Ensures test coverage for source changes, collects test metrics |
| `git` | Validates commit message format (disabled by default) |
| `build` | Tracks binary/bundle size and build time (CI mode only) |
| `license` | Validates license headers in source files (CI mode only) |

## License

MIT - Copyright (c) 2026 Alfred Jean LLC
