# Language Adapters Specification

Language adapters provide language-specific behavior for checks.

## Overview

Adapters enhance generic checks with language-specific knowledge:
- **LOC**: Separate inline test code (e.g., `#[cfg(test)]` in Rust)
- **Escapes**: Language-specific default patterns
- **Test detection**: Language conventions for test files

## Adapter Selection

Adapters are auto-detected based on project files:

| Adapter | Detection | File Patterns |
|---------|-----------|---------------|
| `rust` | `Cargo.toml` exists | `**/*.rs` |
| `golang` | `go.mod` exists | `**/*.go` |
| `javascript` | `package.json`, `tsconfig.json`, or `jsconfig.json` exists | `**/*.js`, `**/*.ts`, `**/*.jsx`, `**/*.tsx` |
| `shell` | `*.sh` files in root, `bin/`, or `scripts/` | `**/*.sh`, `**/*.bash` |
| `generic` | Always (fallback) | From config |

Multiple adapters can be active. Files match the first applicable adapter.

## Rust Adapter

See [langs/rust.md](langs/rust.md) for full Rust configuration.

### Summary

- **Test detection**: `#[cfg(test)]` blocks, `tests/` directory, `*_test.rs` files
- **Escape patterns**: `unsafe`, `.unwrap()`, `.expect()`, `mem::transmute`
- **Lint suppression**: `#[allow(...)]`, `#[expect(...)]`
- **Build metrics**: Binary size, build time

```toml
[rust]
cfg_test_split = "count"         # count | require | off (default: "count")
binary_size = true
build_time = true

[rust.suppress]
check = "comment"                # forbid | comment | allow

[rust.policy]
lint_changes = "standalone"
```

## Go Adapter

See [langs/golang.md](langs/golang.md) for full Go configuration.

### Summary

- **Test detection**: `*_test.go` files (Go convention)
- **Escape patterns**: `unsafe.Pointer`, `//go:linkname`, `//go:noescape`
- **Lint suppression**: `//nolint` directives
- **Build metrics**: Binary size, build time

```toml
[golang]
binary_size = true
build_time = true

[golang.suppress]
check = "comment"                # forbid | comment | allow

[golang.policy]
lint_changes = "standalone"
```

## JavaScript / TypeScript Adapter

See [langs/javascript.md](langs/javascript.md) for full JavaScript/TypeScript configuration.

### Summary

- **Test detection**: `*.test.ts`, `*.spec.ts`, `__tests__/` directories
- **Escape patterns**: `as unknown`, `@ts-ignore` (forbid)
- **Lint suppression**: `eslint-disable`, `biome-ignore`
- **Build metrics**: Bundle size, build time

```toml
[javascript]
bundle_size = true
build_time = true

[javascript.suppress]
check = "comment"              # forbid | comment | allow

[javascript.policy]
lint_changes = "standalone"
```

## Shell Adapter

See [langs/shell.md](langs/shell.md) for full Shell configuration.

### Summary

- **Test detection**: `*.bats` files, `*_test.sh`, `tests/` directory
- **Escape patterns**: `set +e`, `eval`
- **Lint suppression**: `# shellcheck disable=`

```toml
[shell]
# source = ["**/*.sh", "**/*.bash"]
# tests = ["tests/**/*.bats", "test/**/*.bats", "**/*_test.sh"]

[shell.suppress]
check = "forbid"               # forbid | comment | allow

[shell.policy]
lint_changes = "standalone"
```

## Generic / Fallback

For unrecognized languages, quench uses patterns from `[project]`:

### Test Code Detection

Pattern-based only (no inline detection):
- Files matching `tests` patterns from `[project]`

### Escape Patterns

No defaults. Only user-configured patterns in `[check.escapes]` apply.

### Configuration

```toml
[project]
source = ["src/**/*", "lib/**/*"]
tests = ["test/**/*", "tests/**/*", "**/*_test.*", "**/*.spec.*"]
```

## Future Adapters

| Adapter | Detection | Test Patterns | Key Escapes |
|---------|-----------|---------------|-------------|
| `python` | `pyproject.toml` | `test_*.py`, `*_test.py` | `# type: ignore`, `# noqa` |

Future adapters will also provide build metrics. See [checks/build.md](checks/build.md) for how adapters integrate with the build check (bundle size, build time, etc.).
