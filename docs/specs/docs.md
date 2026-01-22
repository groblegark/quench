# Docs Check Specification

The `docs` check validates documentation: spec files in fast mode, doc correlation in CI mode.

## Purpose

Ensure documentation is well-organized and kept in sync with code:
- **Fast mode**: Validate spec files (structure, index, sections)
- **CI mode**: Also check that feature commits have corresponding doc updates

## Fast Mode: Specs Validation

Validates specification documents in `docs/specs/` or similar directories.

### Spec Directory

Default location: `docs/specs/`

```toml
[checks.docs]
path = "docs/specs"
extension = ".md"
```

### Index File

An index file provides an overview and links to individual specs.

**Detection order** (first found wins, case-insensitive):
1. `{path}/[00-]{overview,summary,index}.md`
2. `docs/SPECIFICATIONS.md`
3. `docs/SPECS.md`

```toml
[checks.docs]
index_file = "docs/specs/00-overview.md"  # Or auto-detect
require_index = true                       # Index must exist (default)
require_all_linked = false                 # All specs linked in index
```

### Section Validation

Required sections (case-insensitive matching):

```toml
[checks.docs]
sections.required = ["Purpose", "Configuration"]
sections.forbid = ["TODO", "Draft*"]
```

With advice:

```toml
[[checks.docs.sections.required]]
name = "Purpose"
advice = "What problem this spec addresses"
```

### Content Rules

```toml
[checks.docs]
allow_tables = true         # Tables allowed (default)
allow_box_diagrams = true   # ASCII diagrams (default)
allow_mermaid = true        # Mermaid blocks (default)
max_lines = 1000
max_tokens = 5000
```

### Output (Fast Mode)

```
docs: FAIL
  docs/specs/00-overview.md missing link to: escapes.md
    Add a markdown link to escapes.md in the index file.
  docs/specs/escapes.md missing required section: "Purpose"
    Add a "## Purpose" section: What problem this spec addresses
```

## CI Mode: Doc Correlation

In `--ci` mode, also checks that feature commits have corresponding documentation updates.

**Disabled by default.** Enable with `correlation = true`.

### How It Works

1. Examine commits on the branch (compared to base branch)
2. Identify commits with `feat:` or `feat(area):` prefixes
3. Check if the branch includes changes to documentation
4. Report when feature commits lack corresponding doc changes

### Commit Triggers

```toml
[checks.docs]
correlation = true  # Enable correlation checking in CI mode

# Commit prefixes that require doc updates
triggers = ["feat:", "feat(", "feature:", "feature("]
```

### Area Mapping

Map scoped commits to specific documentation:

```toml
[checks.docs.areas]
api = "docs/api/**"              # feat(api): → docs/api/
cli = "docs/usage/**"            # feat(cli): → docs/usage/
parser = "docs/specs/parser.md"  # feat(parser): → specific file
default = ["README.md", "docs/**"]  # feat: (no scope) → anywhere
```

### Modes

| Mode | Behavior |
|------|----------|
| `require` | Feature commits must have doc changes (default) |
| `advisory` | Warn but don't fail |

```toml
[checks.docs]
correlation_mode = "require"  # or "advisory"
```

### Output (CI Mode)

```
docs: FAIL
  Branch has feature commits without documentation:
    abc123: feat(api): add export endpoint
    def456: feat: new user settings
  Update docs/ or add area mapping in quench.toml.
```

With area mapping:

```
docs: FAIL
  feat(api) commits require changes in docs/api/**
    No changes found in docs/api/
    Update docs/api/ with the new API functionality.
```

## Configuration

```toml
[checks.docs]
enabled = true

# Specs validation (fast mode)
path = "docs/specs"
extension = ".md"
require_index = true
require_all_linked = false
sections.required = ["Purpose"]
sections.forbid = ["TODO"]
allow_tables = true
max_lines = 1000

# Correlation (CI mode)
correlation = false              # Disabled by default
correlation_mode = "require"     # require | advisory
triggers = ["feat:", "feat(", "feature:", "feature("]
doc_patterns = ["README.md", "docs/**/*.md"]

# Area mappings
[checks.docs.areas]
api = "docs/api/**"
default = ["README.md", "docs/**"]
```

## JSON Output

```json
{
  "name": "docs",
  "passed": false,
  "specs": {
    "index_file": "docs/specs/00-overview.md",
    "spec_files": ["00-overview.md", "01-cli.md", "escapes.md"],
    "violations": [
      {
        "file": "docs/specs/escapes.md",
        "type": "missing_section",
        "section": "Purpose"
      }
    ]
  },
  "correlation": {
    "enabled": true,
    "feature_commits": [
      {
        "sha": "abc123",
        "message": "feat(api): add export endpoint",
        "scope": "api",
        "expected_docs": "docs/api/**",
        "doc_changes_found": false
      }
    ]
  }
}
```

## Comparison to `agents` Check

| Aspect | `agents` | `docs` |
|--------|----------|--------|
| Purpose | AI agent context | Technical specifications |
| Default path | Project root | `docs/specs/` |
| Tables | Forbidden | Allowed |
| Index file | N/A | Required by default |
| Sync behavior | Yes | No |
| Token limits | Strict (2000) | Relaxed (5000) |
| Correlation | N/A | CI mode (opt-in) |
