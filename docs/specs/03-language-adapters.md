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

### Test Code Detection

**Test files** (entire file is test code):
- `*_test.rs`, `*_tests.rs`
- Files in `tests/` directory

**Inline test code** (within source files):
- Lines inside `#[cfg(test)]` blocks are counted as test LOC
- Configurable: `parse_cfg_test = true` (default)

Example:
```rust
pub fn add(a: i32, b: i32) -> i32 {  // ← source LOC
    a + b
}

#[cfg(test)]                          // ← test LOC starts
mod tests {
    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}                                     // ← test LOC ends
```

### Default Escape Patterns

| Pattern | Mode | Comment Required |
|---------|------|------------------|
| `unsafe { }` | require_comment | `// SAFETY:` |
| `.unwrap()` | forbid | - |
| `.expect(` | forbid | - |
| `mem::transmute` | require_comment | `// SAFETY:` |
| `#[allow(` | require_comment | `// JUSTIFIED:` |

### Configuration

```toml
[adapters.rust]
enabled = true
parse_cfg_test = true       # Count #[cfg(test)] as test LOC
```

## Shell Adapter

### Test Code Detection

**Test files**:
- `*.bats` files (BATS test framework)
- `*_test.sh` files
- Files in `tests/` or `test/` directories

No inline test code convention for shell.

### Default Escape Patterns

| Pattern | Mode | Comment Required |
|---------|------|------------------|
| `# shellcheck disable=` | forbid | - |
| `set +e` | require_comment | `# OK:` |
| `eval ` | require_comment | `# OK:` |

### Configuration

```toml
[adapters.shell]
enabled = true
forbid_inline_disables = true   # Forbid # shellcheck disable=
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
[adapters.generic]
enabled = true
source_patterns = ["src/**/*", "lib/**/*"]
test_patterns = ["test/**/*", "tests/**/*"]
```

## Future Adapters

| Adapter | Detection | Test Patterns | Key Escapes |
|---------|-----------|---------------|-------------|
| `typescript` | `tsconfig.json` | `*.test.ts`, `*.spec.ts` | `as unknown`, `@ts-ignore`, `any` |
| `python` | `pyproject.toml` | `test_*.py`, `*_test.py` | `# type: ignore`, `# noqa` |
| `go` | `go.mod` | `*_test.go` | `unsafe.Pointer`, `//nolint` |
