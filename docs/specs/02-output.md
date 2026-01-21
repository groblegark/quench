# Output Specification

Quench output is designed for AI agent consumption with progressive disclosure.

## Core Principles

### Progressive Disclosure

Only surface actionable information:
- **Passing checks**: Silent (no output)
- **Failing checks**: Show violation + advice
- **Fixed items** (with `--fix`): Brief confirmation

Rationale: Agents operate under token constraints. Verbose output about passing
checks wastes context and obscures actionable items.

### Actionable Advice

Every failure includes:
1. **Location**: File path and line number (when applicable)
2. **Violation**: What rule was violated and current value
3. **Advice**: Concise guidance on how to fix (not instructions to agents)

Advice is:
- **Specific**: References the actual violation
- **Concise**: One or two sentences
- **Technical**: Assumes reader understands the codebase
- **Not prescriptive**: Says _what_ to fix, not _how_ to approach it

Good: `Add a // SAFETY: comment explaining the invariants.`
Bad: `You should consider adding a safety comment to explain why this unsafe block is sound.`

Good: `Split into smaller modules. Consider extracting TokenStream logic.`
Bad: `This file is too long. Please refactor it into smaller pieces.`

## Output Formats

### Text Format (default)

```
<check-name>: FAIL
  <file>:<line>: <brief violation description>
    <advice>

<check-name>: FAIL
  <file>: <violation> (<value> vs <threshold>)
    <advice>
```

Example:
```
escapes: FAIL
  src/parser.rs:47: unsafe block without // SAFETY: comment
    Add a // SAFETY: comment explaining the invariants.
  src/parser.rs:112: .unwrap() in production code
    Handle the error case or add // OK: comment if infallible.

file-size: FAIL
  src/lexer.rs: 923 lines (max: 900)
    Split into smaller modules. The tokenize() function could be extracted.
```

### JSON Format (`-f json` or `--format json`)

```json
{
  "timestamp": "2026-01-21T10:30:00Z",
  "passed": false,
  "checks": [
    {
      "name": "escapes",
      "passed": false,
      "violations": [
        {
          "file": "src/parser.rs",
          "line": 47,
          "pattern": "unsafe",
          "mode": "require_comment",
          "advice": "Add a // SAFETY: comment explaining the invariants."
        },
        {
          "file": "src/parser.rs",
          "line": 112,
          "pattern": "unwrap",
          "mode": "forbid",
          "advice": "Handle the error case or add // OK: comment if infallible."
        }
      ]
    },
    {
      "name": "file-size",
      "passed": false,
      "violations": [
        {
          "file": "src/lexer.rs",
          "line": null,
          "value": 923,
          "threshold": 900,
          "advice": "Split into smaller modules."
        }
      ]
    }
  ]
}
```

JSON is pipe-friendly: `quench -f json | jq '.checks[] | select(.passed == false)'`

## Colorization

### Detection Logic

```
if --color=always:
    use color
elif --color=never:
    no color
else (--color=auto, default):
    if not stdout.is_tty():
        no color
    elif env.CLAUDE_CODE or env.CODEX or env.CI:
        no color
    else:
        use color
```

### Color Scheme

- **Check name**: Bold
- **FAIL**: Red
- **File path**: Cyan
- **Line number**: Yellow
- **Advice**: Default (no color)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | One or more checks failed |
| 2 | Configuration error or invalid arguments |
| 3 | Internal error (bug in quench) |

## Verbosity Levels

### Default (failures only)

Only failing checks produce output.

### Quiet (`-q`)

Exit code only, no output. Useful for scripting.

### Summary (`--summary`)

Add a one-line summary at the end:
```
escapes: FAIL
  ...

4 checks passed, 2 failed
```

### Verbose (`-v`)

Show passing checks too (for debugging):
```
loc: PASS (4521 source, 3892 test)
file-size: PASS (max 456 lines)
escapes: FAIL
  ...
```

## Fix Mode Output (`--fix`)

When auto-fixing:
```
agent: FIXED
  Synced .cursorrules from CLAUDE.md (3 sections updated)

escapes: FAIL (not auto-fixable)
  src/parser.rs:47: unsafe block without // SAFETY: comment
    Add a // SAFETY: comment explaining the invariants.

1 check fixed, 1 failed
```

## Streaming vs Buffered

- **Default**: Stream output as checks complete (better for slow checks)
- **JSON format**: Buffer and output complete JSON at end
- **`--no-stream`**: Buffer all output, show in consistent order
