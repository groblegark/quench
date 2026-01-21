# Docs Correlation Specification

The `docs-correlation` check ensures code changes are accompanied by documentation updates.

## Purpose

Verify that significant code changes include corresponding documentation:
- New features should be documented
- API changes should update relevant docs
- Prevents shipping undocumented functionality

**Disabled by default.** Enable explicitly in `quench.toml` when your project is ready for doc correlation enforcement.

## Scope

Correlation can be checked at different scopes:

### Branch Scope (Default)

Checks all changes on the branch together:
- All source and doc changes across all commits count
- Order doesn't matter (docs first or code first both work)
- Ideal for PR checks

```bash
quench --compare-branch main
```

### Commit Scope

Checks individual commits with **asymmetric rules**:
- Docs without code = **OK** (spec-first recognized)
- Code without docs = **FAIL** (when commit triggers match)
- Supports "docs first" / "spec-driven" workflows naturally

```bash
quench --staged          # Pre-commit
quench --since HEAD~5    # Recent commits
```

```toml
[checks.docs-correlation]
scope = "branch"  # or "commit"
```

## Modes

### Advisory Mode (Default)

Warn but don't fail:
- Report when docs appear needed but missing
- Exit code 0 regardless
- Good default for most projects

### Require Mode

Commit message triggers determine when docs are required:
- `feat:` or `feature:` commits → require doc changes
- Other commits → no doc requirement
- Deletions → no doc requirement

### Strict Mode

Explicit requirements:
- Any triggering commit requires doc changes
- New public modules require documentation
- No exceptions

## Change Detection

### Git Integration

```bash
# Staged changes (pre-commit)
quench --staged

# Branch changes (PR/CI)
quench --compare-branch main

# Specific commits
quench --since HEAD~5
```

### Commit Message Triggers

Commit messages matching these patterns require doc updates:

```toml
[checks.docs-correlation]
# Commit prefixes that require doc updates (case-insensitive)
commit_triggers = [
  "feat:",
  "feat(",       # e.g., feat(parser):
  "feature:",
  "feature(",
]
```

## Doc Categories

Configure which documentation categories correlate with which code:

```toml
[checks.docs-correlation.categories]
# Code in src/api/ should have docs in docs/api/
api = { source = "src/api/**", docs = "docs/api/**" }

# Code in src/cli/ should have docs in docs/usage/
cli = { source = "src/cli/**", docs = "docs/usage/**" }

# Everything else correlates with README or docs/
default = { source = "src/**", docs = ["README.md", "docs/**"] }
```

### Category Matching

When source files change, quench looks for corresponding doc changes:
1. Match source file to a category by pattern
2. Check if any file in that category's doc pattern was also changed
3. If not, report violation

## Placeholder Docs (Future)

Support for placeholder documentation that indicates planned content:

**Markdown:**
```markdown
## Export Command

<!-- TODO: document export command -->
```

**Or dedicated placeholder files:**
```
docs/usage/export.md.todo
```

When placeholder docs exist for a feature, correlation is satisfied—the doc intent is recorded.

```toml
[checks.docs-correlation]
# Recognize placeholder patterns as valid correlation
allow_placeholders = true  # default: true
```

## Output

### Pass (silent)

No output when correlation is satisfied.

### Fail (commit trigger)

```
docs-correlation: FAIL
  Commit "feat: add export command" requires documentation
    No changes found in docs/. Add or update relevant documentation.
```

### Fail (category mismatch)

```
docs-correlation: FAIL
  src/api/auth.rs: changed in feat commit, no doc changes in docs/api/
    Update docs/api/authentication.md with the new auth functionality.
```

### Advisory (warn mode)

```
docs-correlation: WARN
  Commit "feat(parser): improve error messages" may need docs
    Consider updating docs/ if user-facing behavior changed.
```

### JSON Output

```json
{
  "name": "docs-correlation",
  "passed": true,
  "mode": "advisory",
  "scope": "branch",
  "violations": [
    {
      "source_file": "src/api/auth.rs",
      "change_type": "modified",
      "trigger": "commit_message",
      "commit": "abc123",
      "commit_message": "feat: add export command",
      "category": "api",
      "expected_docs": "docs/api/**",
      "doc_changes": false,
      "advice": "Update docs/api/ with the new functionality."
    }
  ],
  "summary": {
    "commits_requiring_docs": 2,
    "with_doc_changes": 1,
    "without_doc_changes": 1
  }
}
```

## Configuration

```toml
[checks.docs-correlation]
enabled = false  # Disabled by default, opt-in check

# Mode: advisory | require | strict
mode = "advisory"

# Scope: branch | commit
# branch = all changes on branch count together (order doesn't matter)
# commit = per-commit checking with asymmetric rules (docs-first OK)
scope = "branch"

# Commit message prefixes that require docs (case-insensitive)
commit_triggers = ["feat:", "feat(", "feature:", "feature("]

# Placeholder docs
allow_placeholders = true   # <!-- TODO --> comments, .md.todo files count

# Doc file patterns (where to look for doc changes)
doc_patterns = [
  "README.md",
  "docs/**/*.md",
]

# Source patterns to check
source_patterns = ["src/**/*.rs"]

# Exclude patterns (never require doc correlation)
exclude = [
  "**/tests/**",
  "**/test/**",
  "**/benches/**",
  "**/examples/**",
  "**/generated/**",
]

# Category mappings (optional, for more precise correlation)
[checks.docs-correlation.categories]
api = { source = "src/api/**", docs = "docs/api/**" }
cli = { source = "src/cli/**", docs = "docs/usage/**" }
```

## Comparison to `test-correlation`

| Aspect | `test-correlation` | `docs-correlation` |
|--------|-------------------|-------------------|
| Purpose | Code has tests | Code has docs |
| Default mode | require | advisory |
| Default scope | branch | branch |
| Trigger | Any source change | Commit message (feat:) |
| Asymmetric (commit scope) | Tests-first OK | Docs-first OK |
| Category mapping | Implicit (by name) | Explicit config |
| Inline detection | Yes (#[cfg(test)]) | No |
| Placeholders | #[ignore], test.todo() | <!-- TODO -->, .md.todo |
