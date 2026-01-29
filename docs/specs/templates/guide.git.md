# Git Configuration Guide

Configuration reference for git integration.

## Basic Conventional Commits

```toml
[git.commit]
check = "error"
format = "conventional"  # Or "none" (default: conventional)
skip_merge = true        # Skip merge commits (default: true)
agents = true            # Check format is documented (default: true)
template = true          # Create .gitmessage (default: true)
```

## Restrict Commit Types

```toml
[git.commit]
check = "error"
# Only allow these types (default: common conventional types)
types = ["feat", "fix", "chore", "docs", "test", "refactor"]
```

## Restrict Commit Scopes

```toml
[git.commit]
check = "error"
# Only allow these scopes (default: any scope allowed)
scopes = ["api", "cli", "core"]
```

## Allow Any Type (Structure Only)

```toml
[git.commit]
check = "error"
# Empty array = accept any type, just check structure
types = []
```

## Disable Features

```toml
[git.commit]
check = "error"
agents = false    # Don't check agent file documentation
template = false  # Don't create .gitmessage
```

## Allow Merge Commits

```toml
[git.commit]
check = "error"
# Validate merge commits against format
skip_merge = false
```

## Git Configuration

```toml
[git]
base = "main"       # Default for --base (auto: main > master > develop)
# Baseline storage:
# "notes" - use git notes (refs/notes/quench)
# "<path>" - use file at path (e.g., ".quench/baseline.json")
baseline = "notes"
```

## Complete Example

```toml
[git]
base = "main"
baseline = "notes"

[git.commit]
check = "error"
format = "conventional"
types = ["feat", "fix", "chore", "docs", "test", "refactor", "perf", "ci", "build", "style"]
scopes = ["api", "cli", "core"]
skip_merge = true
agents = true
template = true
```
