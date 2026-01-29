# Escape Hatches Configuration Guide

Configuration reference for the `escapes` check.

## Basic Pattern (Comment)

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
name = "unsafe"
pattern = "unsafe\\s*\\{"
# Require comment justification
action = "comment"
comment = "// SAFETY:"
advice = "Add a // SAFETY: comment explaining the invariants."
```

## Basic Pattern (Forbid)

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
name = "unwrap"
pattern = "\\.unwrap\\(\\)"
# Never allowed in source code (always allowed in tests)
action = "forbid"
advice = "Handle the error case or use .expect() with a message."
```

## Basic Pattern (Count)

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
name = "todo"
pattern = "TODO|FIXME|XXX"
# Just count occurrences
action = "count"
threshold = 10  # Fail if more than 10 (default: 0)
advice = "Reduce TODO/FIXME comments before shipping."
```

## Override for Tests

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
name = "debugger"
pattern = "breakpoint\\(\\)"
action = "forbid"
# Also forbid in test code (default: allow in tests)
in_tests = "forbid"
advice = "Remove debugger before committing."
```

## Rust Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "unsafe {"
action = "comment"
comment = "// SAFETY:"

[[check.escapes.patterns]]
pattern = "mem::transmute"
action = "comment"
comment = "// SAFETY:"

[[check.escapes.patterns]]
pattern = "\\.unwrap\\(\\)"
action = "forbid"
```

## Shell Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "set \\+e"
action = "comment"
comment = "# OK:"

[[check.escapes.patterns]]
pattern = "eval "
action = "comment"
comment = "# OK:"

[[check.escapes.patterns]]
pattern = "# shellcheck disable="
action = "forbid"
in_tests = "allow"
```

## Go Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "unsafe\\.Pointer"
action = "comment"
comment = "// SAFETY:"

[[check.escapes.patterns]]
pattern = "//go:linkname"
action = "comment"
comment = "// LINKNAME:"

[[check.escapes.patterns]]
pattern = "//go:noescape"
action = "comment"
comment = "// NOESCAPE:"
```

## JavaScript/TypeScript Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "as unknown"
action = "comment"
comment = "// CAST:"

[[check.escapes.patterns]]
pattern = "@ts-ignore"
action = "forbid"
advice = "Use @ts-expect-error instead."
```

## Python Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "breakpoint\\(\\)"
action = "forbid"
in_tests = "forbid"

[[check.escapes.patterns]]
pattern = "eval\\("
action = "comment"
comment = "# EVAL:"

[[check.escapes.patterns]]
pattern = "exec\\("
action = "comment"
comment = "# EXEC:"
```

## Ruby Patterns

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
pattern = "binding\\.pry"
action = "forbid"
in_tests = "forbid"

[[check.escapes.patterns]]
pattern = "eval\\("
action = "comment"
comment = "# METAPROGRAMMING:"
```

## Per-Package Overrides

```toml
[check.escapes]
check = "error"

[[check.escapes.patterns]]
name = "todo"
pattern = "TODO|FIXME"
action = "count"
threshold = 10

# Stricter for CLI package
[check.escapes.package.cli]
[[check.escapes.package.cli.patterns]]
name = "todo"
threshold = 5
```
