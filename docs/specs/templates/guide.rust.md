# Rust Configuration Guide

Configuration reference for Rust language support.

## File Patterns

```toml
[rust]
source = ["**/*.rs"]
tests = ["**/tests/**", "**/*_test.rs", "**/*_tests.rs"]
ignore = ["target/", "examples/"]
```

## CFG Test Split

```toml
[rust]
# How to handle #[cfg(test)] blocks for LOC counting:
# "count" - split into test LOC (default)
# "require" - fail if inline tests found (enforce sibling _tests.rs files)
# "off" - count all as source LOC
cfg_test_split = "count"
```

## Build Metrics

```toml
[rust]
binary_size = true              # Track release binary sizes
build_time = true               # Track build times (cold and hot)
targets = ["myapp", "myserver"] # Override auto-detection from Cargo.toml
```

## CLOC Advice

```toml
[rust.cloc]
check = "error"
advice = "Custom advice for oversized Rust files."
```

## Suppress Directives

```toml
[rust.suppress]
# How to handle #[allow(...)] and #[expect(...)] attributes:
# "forbid" - never allowed
# "comment" - requires justification comment (default for source)
# "allow" - always allowed (default for tests)
check = "comment"

[rust.suppress.test]
check = "allow"
```

## Suppress with Allowlist/Denylist

```toml
[rust.suppress]
check = "comment"

[rust.suppress.source]
allow = ["dead_code"]     # No comment needed for these lints
forbid = ["unsafe_code"]  # Never allowed to suppress

[rust.suppress.test]
check = "allow"
```

## Suppress with Per-Lint Comment Patterns

```toml
[rust.suppress]
check = "comment"

# Require specific comment pattern for dead_code suppressions
[rust.suppress.source.dead_code]
comment = "// LEGACY:"

# Can also use inline array syntax:
# dead_code = ["// KEEP UNTIL:", "// NOTE(compat):"]

[rust.suppress.test]
check = "allow"
```

## Lint Config Policy

```toml
[rust.policy]
check = "error"
# Require lint config changes (rustfmt.toml, clippy.toml) in standalone PRs
lint_changes = "standalone"
lint_config = ["rustfmt.toml", "clippy.toml"]
```

## Escape Patterns

```toml
# Rust-specific escape hatches
[[check.escapes.patterns]]
pattern = "unsafe {"
action = "comment"
comment = "// SAFETY:"
advice = "Add a // SAFETY: comment explaining the invariants."

[[check.escapes.patterns]]
pattern = "mem::transmute"
action = "comment"
comment = "// SAFETY:"
advice = "Add a // SAFETY: comment explaining type compatibility."

[[check.escapes.patterns]]
pattern = "\\.unwrap\\(\\)"
action = "forbid"
advice = "Use .context() from anyhow or handle the error explicitly."
```

## Complete Example

```toml
[rust]
source = ["**/*.rs"]
tests = ["**/tests/**", "**/*_test.rs", "**/*_tests.rs"]
ignore = ["target/"]
cfg_test_split = "count"
targets = ["myapp", "myserver"]
binary_size = true
build_time = true

[rust.cloc]
check = "error"
advice = "Custom advice for Rust source files."

[rust.suppress]
check = "comment"

[rust.suppress.source]
allow = ["dead_code"]
forbid = ["unsafe_code"]

[rust.suppress.test]
check = "allow"

[rust.policy]
check = "error"
lint_changes = "standalone"
lint_config = ["rustfmt.toml", "clippy.toml"]

[[check.escapes.patterns]]
pattern = "unsafe {"
action = "comment"
comment = "// SAFETY:"

[[check.escapes.patterns]]
pattern = "mem::transmute"
action = "comment"
comment = "// SAFETY:"
```
