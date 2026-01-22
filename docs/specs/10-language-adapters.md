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
| `shell` | `*.sh` files in root, `bin/`, or `scripts/` | `**/*.sh`, `**/*.bash` |
| `generic` | Always (fallback) | From config |

Multiple adapters can be active. Files match the first applicable adapter.

## Rust Adapter

See [langs/rust.md](langs/rust.md) for full Rust configuration.

### Summary

- **Test detection**: `#[cfg(test)]` blocks, `tests/` directory, `*_test.rs` files
- **Escape patterns**: `unsafe`, `.unwrap()`, `.expect()`, `mem::transmute`
- **Lint suppression**: `#[allow(...)]`, `#[expect(...)]`
- **CI metrics**: Binary size, compile time, coverage, test time

```toml
[checks.rust]
check = "error"
split_cfg_test = true            # Count #[cfg(test)] as test LOC

[checks.rust.suppress]
check = "comment"                # forbid | comment | allow

[checks.rust.policy]
lint_changes = "standalone"
```

## Shell Adapter

See [langs/shell.md](langs/shell.md) for full Shell configuration.

### Summary

- **Test detection**: `*.bats` files, `*_test.sh`, `tests/` directory
- **Escape patterns**: `set +e`, `eval`
- **Lint suppression**: `# shellcheck disable=`
- **CI metrics**: Coverage (optional), test time

```toml
[checks.shell]
check = "error"

[checks.shell.suppress]
check = "forbid"               # forbid | comment | allow

[checks.shell.policy]
lint_changes = "standalone"
```

## Generic Adapter

Fallback for unrecognized languages. Uses only configured patterns.

### Test Code Detection

Pattern-based only (no inline detection):
- Files matching `test_patterns` from config

### Escape Patterns

No defaults. Only user-configured patterns apply.

### Configuration

```toml
[checks.generic]
check = "error"
source_patterns = ["src/**/*", "lib/**/*"]
test_patterns = ["test/**/*", "tests/**/*"]
```

## Future Adapters

| Adapter | Detection | Test Patterns | Key Escapes |
|---------|-----------|---------------|-------------|
| `typescript` | `tsconfig.json` | `*.test.ts`, `*.spec.ts` | `as unknown`, `@ts-ignore`, `any` |
| `python` | `pyproject.toml` | `test_*.py`, `*_test.py` | `# type: ignore`, `# noqa` |
| `go` | `go.mod` | `*_test.go` | `unsafe.Pointer`, `//nolint` |
