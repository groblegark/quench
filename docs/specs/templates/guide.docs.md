# Docs Configuration Guide

Configuration reference for the `docs` check.

## TOC Validation

```toml
[check.docs.toc]
check = "error"
include = ["**/*.md", "**/*.mdc"]
exclude = ["plans/**", "plan.md", "*_plan.md", "plan_*"]
```

## Link Validation

```toml
[check.docs.links]
check = "error"
include = ["**/*.md", "**/*.mdc"]
exclude = ["plans/**", "plan.md", "*_plan.md", "plan_*"]
```

## Specs Validation

```toml
[check.docs.specs]
check = "error"
path = "docs/specs"
extension = ".md"
# How to validate index:
# "auto" - try toc first, fall back to linked (default)
# "toc" - parse directory tree in index file
# "linked" - all specs reachable via markdown links
# "exists" - index file must exist, no reachability check
index = "auto"
```

## Specs with Index File

```toml
[check.docs.specs]
check = "error"
path = "docs/specs"
index_file = "docs/specs/CLAUDE.md"  # Or auto-detect
index = "auto"
```

## Specs with Required Sections

```toml
[check.docs.specs]
check = "error"
path = "docs/specs"
# Case-insensitive matching
sections.required = ["Purpose", "Configuration"]
sections.forbid = ["TODO", "Draft*"]
```

## Specs with Section Advice

```toml
[check.docs.specs]
check = "error"

[[check.docs.specs.sections.required]]
name = "Purpose"
advice = "What problem this spec addresses"

[[check.docs.specs.sections.required]]
name = "Configuration"
advice = "How to configure this feature"
```

## Specs with Content Rules

```toml
[check.docs.specs]
check = "error"
tables = "allow"       # Allow markdown tables (default)
box_diagrams = "allow" # Allow ASCII diagrams (default)
mermaid = "allow"      # Allow mermaid blocks (default)
max_lines = 1000       # Or false to disable
max_tokens = 20000     # Or false to disable
```

## Commit Checking (CI Mode)

```toml
[check.docs.commit]
# Disabled by default, enable explicitly
check = "error"
# Which commit types require doc changes (default shown)
types = ["feat", "feature", "story", "breaking"]
```

## Area Mappings

```toml
# Define areas for scoped commits
[check.docs.area.api]
docs = "docs/api/**"
source = "src/api/**"  # Changes here also require docs in docs/api/**

[check.docs.area.cli]
docs = "docs/usage/**"
source = "src/cli/**"

[check.docs.area.parser]
docs = "docs/specs/parser.md"
source = "crates/parser/**"
```

## Complete Example

```toml
[check.docs]
check = "error"

[check.docs.toc]
check = "error"
include = ["**/*.md", "**/*.mdc"]
exclude = ["plans/**", "node_modules/**"]

[check.docs.links]
check = "error"
include = ["**/*.md", "**/*.mdc"]
exclude = ["plans/**"]

[check.docs.specs]
check = "error"
path = "docs/specs"
index_file = "docs/specs/CLAUDE.md"
index = "auto"
tables = "allow"
box_diagrams = "allow"
mermaid = "allow"
max_lines = 1000
max_tokens = 20000
sections.forbid = ["TODO", "Draft*"]

[[check.docs.specs.sections.required]]
name = "Purpose"
advice = "What problem this spec addresses"

[[check.docs.specs.sections.required]]
name = "Configuration"
advice = "How to configure this feature"

[check.docs.commit]
check = "error"
types = ["feat", "feature", "story", "breaking"]

[check.docs.area.api]
docs = "docs/api/**"
source = "src/api/**"

[check.docs.area.cli]
docs = "docs/usage/**"
source = "src/cli/**"
```
