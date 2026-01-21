# Escape Hatches Specification

The `escapes` check detects patterns that bypass type safety or error handling.

## Purpose

Track and control "escape hatches" - code patterns that:
- Bypass compile-time safety (`unsafe`, type casts)
- Skip error handling (`.unwrap()`, `.expect()`)
- Silence warnings (`#[allow(...)]`)

## Modes

Each pattern can be configured with one of three modes:

### Count Mode

Just count occurrences. Fail if count exceeds per-pattern threshold (default: 0).

Use for: Patterns you want to monitor with a specific tolerance.

### Require Comment Mode

Pattern is allowed if accompanied by a justification comment.

Use for: Patterns that are sometimes necessary but should be documented.

```rust
// SAFETY: The pointer is valid because we just allocated it above
unsafe { *ptr = value; }
```

### Forbid Mode

Pattern is never allowed in source code. Always allowed in test code.

Use for: Patterns that should not appear in production code.

## Default Patterns

### Rust

| Pattern | Default Mode | Comment Required | Threshold |
|---------|--------------|------------------|-----------|
| `unsafe { }` | require_comment | `// SAFETY:` | 0 |
| `.unwrap()` | forbid | - | 0 |
| `.expect(` | forbid | - | 0 |
| `mem::transmute` | require_comment | `// SAFETY:` | 0 |
| `#[allow(` | require_comment | `// JUSTIFIED:` | 0 |

### Shell

| Pattern | Default Mode | Comment Required | Threshold |
|---------|--------------|------------------|-----------|
| `# shellcheck disable=` | forbid | - | 0 |
| `set +e` | require_comment | `# OK:` | 0 |
| `eval ` | require_comment | `# OK:` | 0 |

## Source vs Test

Escape hatches are **counted separately** for source and test code:
- **Source code**: Thresholds and modes are enforced
- **Test code**: Counted for metrics, but never fails (test code is allowed escape hatches)

Test code identification uses the same patterns as the `loc` check:
- Files matching test patterns (`*_test.rs`, `tests/**`, etc.)
- Lines inside `#[cfg(test)]` blocks (Rust-specific)

## Per-Package Breakdown

When subprojects are configured, counts are tracked per-package:
- Each package has its own source/test counts
- Thresholds apply to **total source count** by default
- Per-package thresholds can be configured via overrides
- `by_package` field is **omitted from JSON** if no subprojects configured

## Comment Detection

For `require_comment` mode, quench searches **upward** for the required comment:
1. On the same line as the pattern
2. On preceding lines, searching upward until a non-blank, non-comment line is found

This allows comments to be separated from the pattern by other comments or blank lines:

```rust
// SAFETY: The lock is held for the duration of this block
unsafe {                    // ✓ Comment found on preceding line
    critical_section();
}

unsafe { quick_op(); }      // SAFETY: Single atomic op  ✓ Comment on same line

// SAFETY: Pointer guaranteed valid by constructor invariant
//
// Additional context about why this is safe...
unsafe {                    // ✓ Comment found (searching upward through blanks/comments)
    indirect_call();
}

fn other_code() {}
unsafe {                    // ✗ No comment (search stopped at `fn other_code`)
    risky_call();
}
```

## Configurable Advice

Each pattern can have custom advice:

```toml
[[checks.escapes.patterns]]
name = "as_uuid"
pattern = " as UUID"
mode = "forbid"
advice = "Use typed UUID fields instead of casting. Mark properties as 'id: UUID' not 'id: string'."
```

## Output

### Fail (require_comment violation)

```
escapes: FAIL
  src/parser.rs:47: unsafe block without // SAFETY: comment
    Add a // SAFETY: comment explaining why this unsafe block is sound.
```

### Fail (forbid violation)

```
escapes: FAIL
  src/client.rs:89: .unwrap() in production code
    Handle the error case or use .expect() with a message if truly infallible.
```

### Fail (threshold exceeded)

```
escapes: FAIL
  "todo": 23 occurrences (max: 10)
    src/parser.rs: 8 occurrences
    src/lexer.rs: 7 occurrences
    src/compiler.rs: 5 occurrences
    (3 more files...)
    Reduce TODO/FIXME comments or increase threshold.
```

### JSON Output

```json
{
  "name": "escapes",
  "passed": false,
  "violations": [
    {
      "file": "src/parser.rs",
      "line": 47,
      "pattern": "unsafe",
      "mode": "require_comment",
      "required_comment": "// SAFETY:",
      "advice": "Add a // SAFETY: comment explaining why this unsafe block is sound."
    }
  ],
  "counts": {
    "source": {
      "unsafe": 3,
      "unwrap": 0,
      "expect": 0,
      "allow": 12
    },
    "test": {
      "unsafe": 0,
      "unwrap": 47,
      "expect": 5,
      "allow": 0
    }
  },
  "by_package": {
    "cli": {
      "source": { "unsafe": 1, "unwrap": 0, "expect": 0, "allow": 4 },
      "test": { "unsafe": 0, "unwrap": 23, "expect": 2, "allow": 0 }
    },
    "core": {
      "source": { "unsafe": 2, "unwrap": 0, "expect": 0, "allow": 8 },
      "test": { "unsafe": 0, "unwrap": 24, "expect": 3, "allow": 0 }
    }
  }
}
```

## Configuration

```toml
[checks.escapes]
enabled = true

# Custom or override patterns
[[checks.escapes.patterns]]
name = "unsafe"
pattern = "unsafe\\s*\\{"
mode = "require_comment"
comment = "// SAFETY:"

[[checks.escapes.patterns]]
name = "unwrap"
pattern = "\\.unwrap\\(\\)"
mode = "forbid"
# Custom advice for this project
advice = "Use .context() from anyhow or handle the error explicitly."

[[checks.escapes.patterns]]
name = "todo"
pattern = "TODO|FIXME|XXX"
mode = "count"
threshold = 10          # Allow up to 10 (default: 0)
advice = "Reduce TODO/FIXME comments before shipping."

# Per-package overrides
[checks.escapes.overrides.cli]
# Stricter for CLI package
[[checks.escapes.overrides.cli.patterns]]
name = "todo"
threshold = 5
```

## Performance

Pattern matching must be fast:
1. Compile regex patterns once at startup
2. Use ripgrep's regex engine (`grep-regex`) for speed
3. Parallel file scanning
4. Early termination for fast checks (non-CI):
   - Stop scanning a file when threshold is exceeded AND max output violations reached
   - Prevents overwhelming agent context with violations
   - Full counts only computed in `--ci` mode for metrics storage
