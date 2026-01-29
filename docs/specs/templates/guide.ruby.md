# Ruby Configuration Guide

Configuration reference for Ruby language support.

## File Patterns

```toml
[ruby]
source = ["**/*.rb", "**/*.rake", "Rakefile", "Gemfile", "*.gemspec"]
tests = ["spec/**/*_spec.rb", "test/**/*_test.rb", "test/**/test_*.rb", "features/**/*.rb"]
ignore = ["vendor/", "tmp/", "log/", "coverage/"]
```

## CLOC Advice

```toml
[ruby.cloc]
check = "error"
advice = "Custom advice for oversized Ruby files."
```

## Suppress Directives

```toml
[ruby.suppress]
# How to handle # rubocop:disable and # standard:disable comments:
# "forbid" - never allowed
# "comment" - requires justification comment (default for source)
# "allow" - always allowed (default for tests)
check = "comment"

[ruby.suppress.test]
check = "allow"
```

## Suppress with Allowlist/Denylist

```toml
[ruby.suppress]
check = "comment"

[ruby.suppress.source]
allow = ["Style/FrozenStringLiteralComment"]  # No comment needed
forbid = ["Security/Eval"]                     # Never suppress

# Require specific comment for method length suppressions
[ruby.suppress.source."Metrics/MethodLength"]
comment = "# TODO(refactor):"

[ruby.suppress.test]
check = "allow"
```

## Lint Config Policy

```toml
[ruby.policy]
check = "error"
# Require RuboCop/Standard config changes in standalone PRs
lint_changes = "standalone"
lint_config = [".rubocop.yml", ".rubocop_todo.yml", ".standard.yml"]
```

## Escape Patterns

```toml
# Ruby-specific escape hatches
[[check.escapes.patterns]]
pattern = "binding\\.pry"
action = "forbid"
in_tests = "forbid"  # Forbidden even in tests (breaks CI)
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
pattern = "byebug"
action = "forbid"
in_tests = "forbid"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
pattern = "debugger"
action = "forbid"
in_tests = "forbid"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
pattern = "eval\\("
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining why eval is necessary."

[[check.escapes.patterns]]
pattern = "instance_eval"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the DSL or metaprogramming use case."

[[check.escapes.patterns]]
pattern = "class_eval"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the metaprogramming use case."
```

## Coverage

```toml
# RSpec or Minitest with SimpleCov
[[check.tests.suite]]
runner = "rspec"

# Or for Minitest
[[check.tests.suite]]
runner = "minitest"
```

## Complete Example

```toml
[ruby]
source = ["**/*.rb", "**/*.rake", "Rakefile", "Gemfile"]
tests = ["spec/**/*_spec.rb", "test/**/*_test.rb", "features/**/*.rb"]
ignore = ["vendor/", "tmp/", "log/"]

[ruby.cloc]
check = "error"
advice = "Custom advice for Ruby files."

[ruby.suppress]
check = "comment"

[ruby.suppress.source]
allow = ["Style/FrozenStringLiteralComment"]
forbid = ["Security/Eval"]

[ruby.suppress.source."Metrics/MethodLength"]
comment = "# TODO(refactor):"

[ruby.suppress.test]
check = "allow"

[ruby.policy]
check = "error"
lint_changes = "standalone"
lint_config = [".rubocop.yml", ".standard.yml"]

[[check.escapes.patterns]]
pattern = "binding\\.pry"
action = "forbid"
in_tests = "forbid"

[[check.escapes.patterns]]
pattern = "eval\\("
action = "comment"
comment = "# METAPROGRAMMING:"

[[check.tests.suite]]
runner = "rspec"
```
