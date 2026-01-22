# Shell Language Support

Shell-specific behavior for quench checks.

## Detection

Detected when `*.sh` files exist in project root, `bin/`, or `scripts/`.

## Test Code Detection

**Test files** (entire file is test code):
- `*.bats` files (BATS test framework)
- `*_test.sh` files
- Files in `tests/` or `test/` directories

No inline test code convention for shell.

## Default Escape Patterns

| Pattern | Action | Comment Required |
|---------|--------|------------------|
| `set +e` | comment | `# OK:` |
| `eval ` | comment | `# OK:` |

## Suppress

Controls `# shellcheck disable=` comments.

| Setting | Behavior |
|---------|----------|
| `"forbid"` | Never allowed (default) |
| `"comment"` | Requires justification comment |
| `"allow"` | Always allowed |

```toml
[checks.shell.suppress]
check = "forbid"               # forbid | comment | allow
# comment = "# OK:"            # optional: require specific pattern (default: any)

[checks.shell.suppress.source]
allow = ["SC2034"]             # unused variable OK

[checks.shell.suppress.test]
check = "allow"                # tests can suppress freely
```

## Policy

```toml
[checks.shell.policy]
lint_changes = "standalone"
lint_config = [".shellcheckrc"]
```

## CI Mode: Test Metrics

Controlled by `--[no-]tests` flag. In CI mode, runs test suites and collects metrics.

### Test Time

Uses BATS test runner by default. See [11-test-runners.md](../11-test-runners.md) for runner details.

```
tests: time
  total: 4.2s
  avg: 120ms
  max: 850ms (tests/cli/large_input.bats)
```

### Coverage

Shell coverage via `kcov` (optional, not default).

```toml
[checks.shell]
coverage = true
coverage_tool = "kcov"           # kcov, bashcov, or shcov
coverage_min = 70
```

## Configuration

```toml
[checks.shell]
check = "error"

# Lint suppression (# shellcheck disable=)
[checks.shell.suppress]
check = "forbid"               # forbid | comment | allow
# comment = "# OK:"            # optional: require specific pattern
# allow = []                   # codes that don't need comment
# forbid = []                  # codes never allowed

# Policy
[checks.shell.policy]
lint_changes = "standalone"
lint_config = [".shellcheckrc"]

# CI mode metrics
coverage = false               # Disabled by default
test_time = true

# Test suites
[[checks.shell.test_suites]]
runner = "bats"
path = "tests/"
```
