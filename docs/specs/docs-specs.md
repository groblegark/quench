# Docs Specs Specification

The `docs-specs` check validates specification documents in `docs/specs/` or similar directories.

## Purpose

Specification files document design decisions, feature requirements, and technical details. This check ensures:
- Spec files exist in a configured location
- Content follows consistent conventions
- An index file exists linking to individual specs

## Spec Directory

Default location: `docs/specs/`

```toml
[checks.docs-specs]
# Directory containing spec files
path = "docs/specs"

# File extension (default: .md)
extension = ".md"
```

## Index File

An index file provides an overview and links to individual specs.

**Detection order** (first found wins, case-insensitive):
1. `{path}/[00-]{overview,summary,index}.md` (e.g., `docs/specs/00-overview.md`, `docs/specs/summary.md`)
2. `docs/SPECIFICATIONS.md`
3. `docs/SPECIFICATION.md`
4. `docs/SPECS.md`
5. `docs/SPEC.md`

Configure explicitly:
```toml
[checks.docs-specs]
index_file = "docs/specs/00-overview.md"
```

### Index Validation

The index file is checked for:
- Contains markdown links to spec files
- Links resolve to existing files
- All spec files are linked (optional: `require_all_linked = true`)

```toml
[checks.docs-specs]
require_index = true           # Index file must exist (default: true)
require_all_linked = false     # All specs must be linked in index (default: false)
```

## Section Validation

Same as `agent` check - validates required and forbid sections.

### Required Sections

Sections that must be present (case-insensitive matching):

Simple form (no advice):
```toml
[checks.docs-specs]
sections.required = ["Purpose", "Configuration"]
```

Extended form (with advice):
```toml
[[checks.docs-specs.sections.required]]
name = "Purpose"
advice = "What problem this spec addresses"

[[checks.docs-specs.sections.required]]
name = "Configuration"
advice = "TOML configuration options and examples"
```

### Forbid Sections

Sections that should not be present (case-insensitive, supports globs):

```toml
[checks.docs-specs]
sections.forbid = [
  "TODO",           # Use 99-todo.md instead
  "Draft*",         # Glob: matches "Draft", "Draft Notes", etc.
]
```

## Content Rules

### Markdown Tables

Tables can be useful in specs for configuration options. Default: allowed.

```toml
allow_tables = true  # default (unlike agent files)
```

### Diagrams

Control which diagram types are allowed:

```toml
allow_box_diagrams = true   # ┌─┐ style ASCII diagrams (default: true)
allow_mermaid = true        # ```mermaid blocks (default: true)
```

### Size Limits

Specs can be longer than agent files:

```toml
[checks.docs-specs.limits]
max_lines = 1000
max_tokens = 5000
```

Token estimation: `tokens ≈ characters / 4`

## Output

### Fail (no index file)

```
docs-specs: FAIL
  No index file found in docs/specs/
    Create docs/specs/00-overview.md or docs/SPECS.md with links to spec files.
```

### Fail (missing spec link in index)

```
docs-specs: FAIL
  docs/specs/00-overview.md missing link to: escape-hatches.md
    Add a markdown link to escape-hatches.md in the index file.
```

### Fail (broken link in index)

```
docs-specs: FAIL
  docs/specs/00-overview.md:15 broken link: authentication.md
    File docs/specs/authentication.md does not exist.
```

### Fail (missing required section)

```
docs-specs: FAIL
  docs/specs/escape-hatches.md missing required section: "Purpose"
    Add a "## Purpose" section: What problem this spec addresses
```

### JSON Output

```json
{
  "name": "docs-specs",
  "passed": false,
  "index_file": "docs/specs/00-overview.md",
  "spec_files": ["00-overview.md", "01-config.md", "escape-hatches.md"],
  "violations": [
    {
      "file": "docs/specs/00-overview.md",
      "type": "missing_link",
      "target": "escape-hatches.md",
      "advice": "Add a markdown link to escape-hatches.md in the index file."
    },
    {
      "file": "docs/specs/escape-hatches.md",
      "type": "missing_section",
      "section": "Purpose",
      "advice": "Add a \"## Purpose\" section: What problem this spec addresses"
    }
  ]
}
```

## Configuration

```toml
[checks.docs-specs]
enabled = true

# Directory containing spec files
path = "docs/specs"
extension = ".md"

# Index file (auto-detected if not set)
# index_file = "docs/specs/00-overview.md"

# Index requirements
require_index = true
require_all_linked = false

# Section validation (simple form)
sections.required = ["Purpose"]
sections.forbid = ["TODO"]

# Section validation (extended form with advice)
# [[checks.docs-specs.sections.required]]
# name = "Configuration"
# advice = "TOML configuration options and examples"

# Content rules
allow_tables = true       # Tables allowed in specs (unlike agent files)
allow_box_diagrams = true
allow_mermaid = true

# Size limits
max_lines = 1000
max_tokens = 5000
```

## Comparison to `agent` Check

| Aspect | `agent` | `docs-specs` |
|--------|---------|--------------|
| Purpose | AI agent context | Technical specifications |
| Default path | Project root | `docs/specs/` |
| Tables | Forbidden | Allowed |
| Index file | N/A | Required by default |
| Link validation | N/A | Yes |
| Sync behavior | Yes | No |
| Token limits | Strict (2000) | Relaxed (5000) |
