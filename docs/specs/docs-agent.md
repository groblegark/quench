# Agent Docs Specification

The `agent` check validates AI agent context files (CLAUDE.md, .cursorrules, etc.).

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
[checks.agent]
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
[checks.agent]
# Sync check is ON by default when multiple files exist
sync = true

# Which file is the source of truth for --fix
# Default: first file in the `files` list
# If not configured: --fix is NOT available (must reconcile manually)
sync_source = "CLAUDE.md"
```

**Auto-fix requires explicit `sync_source`**:
- If `sync_source` is set → `--fix` syncs other files from it
- If `sync_source` is not set → out-of-sync is reported but `--fix` won't modify files

## Existence Requirements

Configure which files must exist at each scope:

```toml
[checks.agent.root]
# At project root
required = ["CLAUDE.md"]              # Must exist
optional = [".cursorrules"]           # Checked if present, not required
forbidden = []                        # Must not exist

[checks.agent.package]
# At each package/subproject
required = []                         # None required by default
optional = ["CLAUDE.md"]

[checks.agent.module]
# At subdirectories
required = []
optional = ["CLAUDE.md"]
```

Simpler flat config (root only):

```toml
[checks.agent]
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

Simple form:
```toml
require_sections = ["Project Structure", "Development"]
```

Extended form (with descriptions for advice):
```toml
[[checks.agent.require_sections]]
name = "Project Structure"
description = "Overview of directory layout and key files"

[[checks.agent.require_sections]]
name = "Development"
description = "How to build, test, and run the project"

[[checks.agent.require_sections]]
name = "Landing the Plane"
description = "Checklist for AI agents before finishing work"
```

When a section is missing, the description becomes actionable advice:
```
agent: FAIL
  CLAUDE.md missing required section: "Landing the Plane"
    Add a "## Landing the Plane" section: Checklist for AI agents before finishing work
```

### Forbidden Sections

Sections that should not be present (case-insensitive, supports globs):

```toml
forbid_sections = [
  "API Keys",           # Exact match (case-insensitive)
  "Secrets",
  "Test*",              # Glob: matches "Testing", "Test Plan", etc.
]
```

## Content Rules

### Markdown Tables

Tables are verbose and not token-efficient. Default: forbidden.

```toml
forbid_tables = true  # default
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
[checks.agent.limits]
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
agent: FAIL
  CLAUDE.md not found at project root
    Create a CLAUDE.md with project context for AI agents.
```

### Fail (files out of sync, sync_source configured)

```
agent: FAIL
  CLAUDE.md and .cursorrules are out of sync
    Section "Code Style" differs between files.
    Use --fix to sync from CLAUDE.md.
```

### Fail (files out of sync, no sync_source)

```
agent: FAIL
  CLAUDE.md and .cursorrules are out of sync
    Section "Code Style" differs between files.
    Set sync_source in quench.toml to enable --fix, or reconcile manually.
```

### Fail (missing section)

```
agent: FAIL
  CLAUDE.md missing required section: "Landing the Plane"
    Add a "## Landing the Plane" section: Checklist for AI agents before finishing work
```

### Fail (forbidden content)

```
agent: FAIL
  CLAUDE.md:45: Markdown table detected
    Tables are not token-efficient. Convert to a list or prose.
```

### Fixed (with --fix)

```
agent: FIXED
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
      "advice": "Set sync_source to enable --fix, or reconcile manually."
    },
    {
      "file": "CLAUDE.md",
      "line": 45,
      "type": "forbidden_table",
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
[checks.agent]
enabled = true

# Which agent files to check
files = ["CLAUDE.md", ".cursorrules"]

# Sync behavior (default: true if multiple files exist)
sync = true

# Source of truth for --fix (default: not set, --fix disabled)
# Set this to enable auto-fix when files are out of sync
sync_source = "CLAUDE.md"

# Existence requirements (root scope)
required = ["CLAUDE.md"]
optional = [".cursorrules"]

# Section validation
require_sections = ["Project Structure", "Development"]
forbid_sections = ["Secrets", "API Keys"]

# Extended sections with descriptions
# [[checks.agent.require_sections]]
# name = "Landing the Plane"
# description = "Checklist for AI agents before finishing work"

# Content rules
forbid_tables = true
allow_box_diagrams = true
allow_mermaid = true

# Size limits
max_lines = 500
max_tokens = 2000

# Per-scope overrides
[checks.agent.root]
required = ["CLAUDE.md"]
max_tokens = 2000

[checks.agent.package]
required = []
optional = ["CLAUDE.md"]
max_tokens = 800

[checks.agent.module]
required = []
max_tokens = 400
```
