# License Configuration Guide

Configuration reference for the `license` check.

## Basic Configuration

```toml
[check.license]
# Disabled by default, opt-in check
check = "error"
license = "MIT"
copyright = "Your Organization"
```

## Common Licenses

```toml
# MIT License
[check.license]
check = "error"
license = "MIT"
copyright = "Your Organization"
```

```toml
# Apache 2.0
[check.license]
check = "error"
license = "Apache-2.0"
copyright = "Your Organization"
```

```toml
# Business Source License
[check.license]
check = "error"
license = "BUSL-1.1"
copyright = "Your Organization"
```

```toml
# GPL v3
[check.license]
check = "error"
license = "GPL-3.0-only"
copyright = "Your Organization"
```

## File Patterns

```toml
[check.license]
check = "error"
license = "MIT"
copyright = "Your Organization"

[check.license.patterns]
rust = ["**/*.rs"]
shell = ["**/*.sh", "**/*.bash", "scripts/*"]
typescript = ["**/*.ts", "**/*.tsx"]
go = ["**/*.go"]
python = ["**/*.py"]
ruby = ["**/*.rb"]
```

## Excludes

```toml
[check.license]
check = "error"
license = "MIT"
copyright = "Your Organization"
exclude = [
  "**/generated/**",
  "**/vendor/**",
  "**/node_modules/**",
  "**/target/**",
]
```

## Complete Example

```toml
[check.license]
check = "error"
license = "MIT"
copyright = "Your Organization"

[check.license.patterns]
rust = ["**/*.rs"]
shell = ["**/*.sh", "**/*.bash", "scripts/*"]
typescript = ["**/*.ts", "**/*.tsx"]
go = ["**/*.go"]

exclude = [
  "**/generated/**",
  "**/vendor/**",
  "**/node_modules/**",
  "**/target/**",
]
```
