# Agents Check Specification

The `agents` check validates AI agent context files (CLAUDE.md, .cursorrules, etc.).

## Purpose

Agent markdown files provide context to AI coding agents. This check ensures:
- Configured files exist where expected
- Content follows token-efficient conventions
- Files stay in sync with each other (default behavior)

## Agent Files

Quench recognizes these agent context files:

| File | Description |
|------|-------------|
| `CLAUDE.md` | Claude Code / Anthropic agents |
| `.cursorrules` | Cursor IDE |
| `.cursorignore` | Cursor IDE ignore patterns |
| `COPILOT.md` | GitHub Copilot (future) |
| `AGENTS.md` | Generic agent instructions |

Configure which files are recognized:

```toml
[checks.agents]
# Files to check (default: all recognized)
files = ["CLAUDE.md", ".cursorrules"]
```

## Sync Behavior (Default)

**If multiple agent files exist, they are checked for sync by default.**

This happens regardless of whether files are `required` or optional:
- If CLAUDE.md and .cursorrules both exist → check they're in sync
- If only one exists → no sync check needed
- If neither exists but one is required → fail on missing file

```toml
[checks.agents]
# Sync check is ON by default when multiple files exist
sync = true

# Which file is the source of truth for --fix
# Default: first file in the `files` list
sync_source = "CLAUDE.md"
```

**Auto-fix behavior**:
- `sync_source` defaults to first file in the `files` list
- `--fix` syncs other files from the source
- `--fix` is only unavailable if `files` list is empty

## Existence Requirements

Configure which files must exist at each scope:

```toml
[checks.agents.root]
# At project root
required = ["CLAUDE.md"]              # Must exist
optional = [".cursorrules"]           # Checked if present, not required
forbid = []                        # Must not exist

[checks.agents.package]
# At each package
required = []                         # None required by default
optional = ["CLAUDE.md"]

[checks.agents.module]
# At subdirectories
required = []
optional = ["CLAUDE.md"]
```

Simpler flat config (root only):

```toml
[checks.agents]
required = ["CLAUDE.md"]
optional = [".cursorrules"]
```

## Scope Levels

### Root (Project Root)

The main agent files at project root. Typically:
- Project overview and architecture
- Development setup and conventions
- "Landing the Plane" checklist

### Package (Subproject/Crate)

Agent files in a package directory (e.g., `crates/cli/CLAUDE.md`). Typically:
- Package-specific context
- Local conventions differing from root
- Shorter, focused content

### Module (Subdirectory)

Agent files in a subdirectory (e.g., `src/parser/CLAUDE.md`). Typically:
- Module-specific implementation notes
- Very brief, focused context
- Usually optional

## Section Validation

### Required Sections

Sections that must be present. Matching is **case-insensitive**.

Simple form (no advice):
```toml
[checks.agents]
sections.required = ["Project Structure", "Development"]
```

Extended form (with advice for agents):
```toml
[[checks.agents.sections.required]]
name = "Project Structure"
advice = "Overview of directory layout and key files"

[[checks.agents.sections.required]]
name = "Development"
advice = "How to build, test, and run the project"

[[checks.agents.sections.required]]
name = "Landing the Plane"
advice = "Checklist for AI agents before finishing work"
```

When a section is missing, the advice becomes actionable output:
```
agents: FAIL
  CLAUDE.md missing required section: "Landing the Plane"
    Add a "## Landing the Plane" section: Checklist for AI agents before finishing work
```

### Forbid Sections

Sections that should not be present (case-insensitive, supports globs):

```toml
[checks.agents]
sections.forbid = [
  "API Keys",           # Exact match (case-insensitive)
  "Secrets",
  "Test*",              # Glob: matches "Testing", "Test Plan", etc.
]
```

## Content Rules

### Markdown Tables

Tables are verbose and not token-efficient. Default: forbid.

```toml
allow_tables = false  # default (tables forbid)
```

Advice: Use lists or prose instead.

### Diagrams

Control which diagram types are allowed (both default to true):

```toml
allow_box_diagrams = true   # ┌─┐ style ASCII diagrams
allow_mermaid = true        # ```mermaid blocks
```

### Size Limits

Keep agent files concise:

```toml
[checks.agents.limits]
root_max_lines = 500
root_max_tokens = 2000
package_max_lines = 200
package_max_tokens = 800
module_max_lines = 100
module_max_tokens = 400
```

Token estimation: `tokens ≈ characters / 4`

## Output

### Fail (missing required file)

```
agents: FAIL
  CLAUDE.md not found at project root
    Create a CLAUDE.md with project context for AI agents.
```

### Fail (files out of sync, sync_source configured)

```
agents: FAIL
  CLAUDE.md and .cursorrules are out of sync
    Section "Code Style" differs between files.
    Use --fix to sync from CLAUDE.md.
```

### Fail (files out of sync)

```
agents: FAIL
  CLAUDE.md and .cursorrules are out of sync
    Section "Code Style" differs between files.
    Run --fix to sync from CLAUDE.md, or reconcile manually.
```

### Fail (missing section)

```
agents: FAIL
  CLAUDE.md missing required section: "Landing the Plane"
    Add a "## Landing the Plane" section: Checklist for AI agents before finishing work
```

### Fail (forbid content)

```
agents: FAIL
  CLAUDE.md:45: Markdown table detected
    Tables are not token-efficient. Convert to a list or prose.
```

### Fixed (with --fix)

```
agents: FIXED
  Synced .cursorrules from CLAUDE.md (3 sections updated)
```

### JSON Output

```json
{
  "name": "agent",
  "passed": false,
  "violations": [
    {
      "file": "CLAUDE.md",
      "type": "out_of_sync",
      "other_file": ".cursorrules",
      "section": "Code Style",
      "advice": "Run --fix to sync from CLAUDE.md, or reconcile manually."
    },
    {
      "file": "CLAUDE.md",
      "line": 45,
      "type": "forbid_table",
      "advice": "Tables are not token-efficient. Convert to a list."
    }
  ],
  "files_found": ["CLAUDE.md", ".cursorrules"],
  "files_missing": [],
  "in_sync": false
}
```

## Configuration

```toml
[checks.agents]
enabled = true

# Which agent files to check
files = ["CLAUDE.md", ".cursorrules"]

# Sync behavior (default: true if multiple files exist)
sync = true

# Source of truth for --fix (default: first file in `files` list)
sync_source = "CLAUDE.md"

# Existence requirements (root scope)
required = ["CLAUDE.md"]
optional = [".cursorrules"]

# Section validation (simple form)
sections.required = ["Project Structure", "Development"]
sections.forbid = ["Secrets", "API Keys"]

# Section validation (extended form with advice)
# [[checks.agents.sections.required]]
# name = "Landing the Plane"
# advice = "Checklist for AI agents before finishing work"

# Content rules
allow_tables = false
allow_box_diagrams = true
allow_mermaid = true

# Size limits
max_lines = 500 # default: 500
max_tokens = 5000 # default: unlimited

# Per-scope overrides
[checks.agents.root]
required = ["CLAUDE.md"]
max_tokens = 2000

[checks.agents.package]
required = []
optional = ["CLAUDE.md"]
max_tokens = 800
# max_lines: # default 250

[checks.agents.module]
required = []
max_tokens = 400
# max_lines: # default 150
```
