# Shell Configuration Guide

Configuration reference for Shell language support.

## File Patterns

```toml
[shell]
source = ["**/*.sh", "**/*.bash", "bin/*", "scripts/*"]
tests = ["**/tests/**/*.bats", "**/test/**/*.bats", "**/*_test.sh"]
```

## CLOC Advice

```toml
[shell.cloc]
check = "error"
advice = "Custom advice for oversized shell scripts."
```

## Suppress Directives

```toml
[shell.suppress]
# How to handle # shellcheck disable= comments:
# "forbid" - never allowed (default for source)
# "comment" - requires justification comment
# "allow" - always allowed (default for tests)
check = "forbid"

[shell.suppress.test]
check = "allow"
```

## Suppress with Comment Requirement

```toml
[shell.suppress]
check = "comment"

[shell.suppress.source]
allow = ["SC2034"]  # Unused variable OK without comment

# Require specific comment for word splitting suppressions
[shell.suppress.source.SC2086]
comment = "# INTENTIONAL:"

[shell.suppress.test]
check = "allow"
```

## Lint Config Policy

```toml
[shell.policy]
check = "error"
# Require .shellcheckrc changes in standalone PRs
lint_changes = "standalone"
lint_config = [".shellcheckrc"]
```

## Escape Patterns

```toml
# Shell-specific escape hatches
[[check.escapes.patterns]]
pattern = "set \\+e"
action = "comment"
comment = "# OK:"
advice = "Add a # OK: comment explaining why error checking is disabled."

[[check.escapes.patterns]]
pattern = "eval "
action = "comment"
comment = "# OK:"
advice = "Add a # OK: comment explaining why eval is safe here."

[[check.escapes.patterns]]
pattern = "# shellcheck disable="
action = "forbid"
in_tests = "allow"
advice = "Fix the shellcheck warning instead of disabling it."
```

## Coverage via kcov

```toml
# Shell coverage requires kcov and explicit targets
[[check.tests.suite]]
runner = "bats"
path = "tests/"
targets = ["scripts/*.sh", "bin/*"]  # Shell scripts to instrument
```

## Complete Example

```toml
[shell]
source = ["**/*.sh", "**/*.bash", "bin/*", "scripts/*"]
tests = ["**/tests/**/*.bats", "**/*_test.sh"]

[shell.cloc]
check = "error"
advice = "Custom advice for shell scripts."

[shell.suppress]
check = "forbid"

[shell.suppress.test]
check = "allow"

[shell.policy]
check = "error"
lint_changes = "standalone"
lint_config = [".shellcheckrc"]

[[check.escapes.patterns]]
pattern = "set \\+e"
action = "comment"
comment = "# OK:"

[[check.escapes.patterns]]
pattern = "eval "
action = "comment"
comment = "# OK:"

[[check.tests.suite]]
runner = "bats"
path = "tests/"
targets = ["scripts/*.sh", "bin/*"]
```
