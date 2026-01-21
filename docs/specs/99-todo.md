# Future Work

Features discussed but not yet fully specified. To be designed in later phases.

## Performance Metrics

### Binary Size Tracking

Track compiled artifact sizes over time.

- Release binary size
- Stripped binary size
- Per-binary breakdown (for multi-binary projects)
- Configurable thresholds

### Compile Time

Measure build performance.

- Cold compile (from clean)
- Incremental compile (touch and rebuild)
- Per-package breakdown

### Test Time

Measure test suite performance.

- Cold test run
- Warm test run (cached)
- Per-package breakdown

### Memory Usage

Track peak RSS during operations.

- Memory during `help` command
- Memory during typical operations
- Configurable thresholds

## Coverage Integration

Per-language coverage collection.

- Rust: `cargo llvm-cov` integration
- Configurable coverage threshold (e.g., 85%)
- Per-package coverage breakdown
- Option to include spec/integration tests in coverage

## Metrics Storage

Store metrics over time for trending.

- JSON format for machine processing
- Rolling window (e.g., 52 weeks)
- Append-only history file
- Comparison against historical baselines

## Reporting

### Weekly Reports

Generate trending reports over configurable period.

- Summary table with deltas
- Pass/fail status per metric
- Commit/work tracking summary
- Markdown output

### GitHub Pages Dashboard

Auto-publish metrics to GitHub Pages.

- Latest metrics JSON
- Historical trend data
- Human-readable summary
- CI workflow integration

## Work Tracking

### Commit Analysis

Analyze commits by conventional commit type.

- Count by type: feat, fix, chore, refactor, docs
- Configurable date range
- Per-author breakdown (optional)

### Issue Integration

Track issue/bug counts (requires custom format or integration).

- Open/closed bugs
- Tasks, chores, epics
- Date range filtering

## License Headers

Auto-manage license headers in source files.

- Add missing headers
- Update copyright year
- Configurable header template
- `--fix` support
- Default: disabled

## Git Checks

### Commit Format Validation

Validate conventional commit format.

- `feat:`, `fix:`, `chore:` prefixes
- Configurable patterns
- Default: disabled

### Uncommitted Changes

Detect uncommitted changes in worktree.

- Useful for CI/agent workflows
- Warning vs failure modes

## Baseline Comparison

### Branch Comparison

Compare current metrics against a git branch.

- Stash, checkout, measure, restore
- Default comparison to main/master
- `--compare-branch` flag

### Ratcheting

Prevent quality regressions.

- Fail if escape hatches increase
- Fail if coverage drops (with variance tolerance)
- Fail if files over size limit increase
- Configurable per-metric

## Documentation Checks

### docs-specs

Similar to `docs-agent` but for specification documents (`docs/specs/`).

- Validate spec file existence and structure
- Required sections per spec type
- Size limits (token efficiency)
- Naming conventions (e.g., `00-overview.md`, `99-todo.md`)
- Configurable spec categories and requirements

### docs-correlation

Similar to `test-correlation` but for documentation.

- Code changes should have corresponding doc changes
- Configurable doc categories:
  - `specs/` - design specifications
  - `api/` - API documentation
  - `guides/` - user guides
- Smart detection: significant code changes require doc updates
- Per-category rules (e.g., new feature → update specs)
- Advisory vs strict modes

## Notes from Interview

- Primary users are AI agents ("landing the plane")
- Performance target: sub-second for fast checks
- All output should be agent-friendly (token-efficient)
- Progressive disclosure: only surface failures
- `--fix` should be explicit about what it can/cannot fix
