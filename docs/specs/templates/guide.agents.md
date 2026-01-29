# Agent Files Configuration Guide

Configuration reference for the `agents` check.

## Basic Setup

```toml
[check.agents]
check = "error"
files = ["CLAUDE.md", ".cursorrules"]
sync = true
sync_from = "CLAUDE.md"
```

## Content Rules

```toml
[check.agents]
check = "error"
# Control token-inefficient content:
tables = "forbid"       # "allow" | "forbid" (default: allow)
box_diagrams = "allow"  # ASCII box diagrams (default: allow)
mermaid = "allow"       # Mermaid code blocks (default: allow)
max_lines = 500         # Or false to disable
max_tokens = 20000      # Or false to disable
```

## Required Sections (Simple)

```toml
[check.agents]
check = "error"
# Case-insensitive matching
sections.required = ["Directory Structure", "Landing the Plane"]
```

## Required Sections (With Advice)

```toml
[check.agents]
check = "error"

[[check.agents.sections.required]]
name = "Directory Structure"
advice = "Overview of project layout and key directories"

[[check.agents.sections.required]]
name = "Landing the Plane"
advice = "Checklist for AI agents before completing work"
```

## Forbid Sections

```toml
[check.agents]
check = "error"
# Case-insensitive, supports globs
sections.forbid = ["API Keys", "Secrets", "Test*"]
```

## Scope-Based Configuration

```toml
[check.agents]
check = "error"

# Project root
[check.agents.root]
required = ["CLAUDE.md"]
optional = [".cursorrules"]
forbid = []
max_lines = 500
max_tokens = 20000
sections.required = ["Directory Structure", "Landing the Plane"]

# Each package directory
[check.agents.package]
required = []
optional = ["CLAUDE.md"]
max_lines = 200
max_tokens = 800

# Subdirectories
[check.agents.module]
required = []
max_lines = 100
max_tokens = 400
```

## Sync Behavior

```toml
[check.agents]
check = "error"
files = ["CLAUDE.md", ".cursorrules"]
# Keep files in sync (default: true)
sync = true
# Source of truth for --fix (default: first in files list)
sync_from = "CLAUDE.md"
```

## Disable Sync

```toml
[check.agents]
check = "error"
files = ["CLAUDE.md", ".cursorrules"]
# Allow files to have different content
sync = false
```

## Claude Profile

```toml
[check.agents]
check = "error"
files = ["CLAUDE.md"]
required = ["CLAUDE.md"]
sync = true
sync_from = "CLAUDE.md"
tables = "forbid"
max_lines = 500
max_tokens = 20000

[[check.agents.sections.required]]
name = "Directory Structure"
advice = "Overview of project layout and key directories"

[[check.agents.sections.required]]
name = "Landing the Plane"
advice = "Checklist for AI agents before completing work"
```

## Combined Claude and Cursor

```toml
[check.agents]
check = "error"
files = ["CLAUDE.md", ".cursorrules"]
required = ["CLAUDE.md"]       # CLAUDE.md is required
optional = [".cursorrules"]    # .cursorrules is optional
sync = true
sync_from = "CLAUDE.md"        # Sync from CLAUDE.md if both exist
tables = "forbid"
max_lines = 500
max_tokens = 20000
```
