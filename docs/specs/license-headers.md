# License Check Specification

The `license` check validates and auto-fixes license headers in source files.

## Purpose

Ensure all source files have proper license headers:
- SPDX license identifier for machine readability
- Copyright notice with current year
- Consistent formatting across the codebase

**Disabled by default.** Enable explicitly when your project requires license headers.

**CI-only.** This check only runs in `--ci` mode. It is skipped in fast mode.

## Header Format

Uses SPDX license identifiers for standardization:

**Rust/TypeScript/Go** (// comments):
```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Your Organization
```

**Shell/Python** (# comments):
```bash
#!/bin/bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Your Organization
```

**Note:** Shebangs are preserved at the top of shell scripts.

## Validation Rules

### Missing Header

File has no SPDX or copyright line:

```
license: FAIL
  src/parser.rs: missing license header
    Add SPDX-License-Identifier and Copyright at file start.
```

### Wrong License

File has different SPDX identifier than configured:

```
license: FAIL
  src/parser.rs:1: wrong license identifier
    Expected: MIT, found: Apache-2.0
```

### Outdated Copyright Year

Copyright year doesn't include current year:

```
license: FAIL
  src/parser.rs:2: outdated copyright year
    Expected: 2026, found: 2025
```

## Auto-Fix (`--fix`)

When running `quench --ci --fix`:

1. **Add missing headers**: Insert header at file start (after shebang if present)
2. **Update copyright year**: Change year to current year
3. **Preserve content**: Only modify header lines, not file content

```
license: FIXED
  src/parser.rs: added license header
  src/lexer.rs: updated copyright year (2025 → 2026)
  3 files unchanged
```

## Configuration

```toml
[checks.license]
enabled = false              # Disabled by default, opt-in check

# SPDX license identifier
license = "MIT"              # or "Apache-2.0", "BUSL-1.1", etc.

# Copyright holder
copyright = "Your Organization"

# File patterns to check (by extension)
[checks.license-headers.patterns]
rust = ["**/*.rs"]
shell = ["**/*.sh", "**/*.bash", "scripts/*"]
typescript = ["**/*.ts", "**/*.tsx"]
go = ["**/*.go"]

# Exclude patterns
exclude = [
  "**/generated/**",
  "**/vendor/**",
  "**/node_modules/**",
  "**/target/**",
]
```

### Comment Syntax by Language

Automatically determined by file extension:

| Extensions | Comment Style |
|------------|---------------|
| `.rs`, `.ts`, `.tsx`, `.js`, `.go`, `.c`, `.cpp`, `.h` | `// ` |
| `.sh`, `.bash`, `.py`, `.rb`, `.yaml`, `.yml` | `# ` |
| `.html`, `.xml` | `<!-- -->` |

Override with explicit config:

```toml
[checks.license-headers.syntax]
# Custom comment syntax for specific patterns
"scripts/*" = "#"
"*.config.js" = "//"
```

## Output

### Pass (silent)

No output when all files have correct headers.

### Fail (missing headers)

```
license: FAIL
  src/parser.rs: missing license header
    Add SPDX-License-Identifier and Copyright at file start.
  src/lexer.rs: missing license header
    Add SPDX-License-Identifier and Copyright at file start.
```

### Fail (wrong license)

```
license: FAIL
  src/parser.rs:1: wrong license identifier
    Expected: MIT, found: Apache-2.0
    Update or run --fix to correct.
```

### Fixed

```
license: FIXED
  Added headers: 3 files
  Updated years: 2 files
```

### JSON Output

```json
{
  "name": "license-headers",
  "passed": false,
  "violations": [
    {
      "file": "src/parser.rs",
      "type": "missing_header",
      "advice": "Add SPDX-License-Identifier and Copyright at file start."
    },
    {
      "file": "src/lexer.rs",
      "line": 2,
      "type": "outdated_year",
      "found": "2025",
      "expected": "2026",
      "advice": "Update copyright year or run --fix."
    }
  ],
  "summary": {
    "files_checked": 47,
    "files_with_headers": 45,
    "files_missing_headers": 2,
    "files_outdated_year": 1
  }
}
```

## Supported Licenses

Common SPDX identifiers:

| Identifier | License |
|------------|---------|
| `MIT` | MIT License |
| `Apache-2.0` | Apache License 2.0 |
| `GPL-3.0-only` | GNU GPL v3 |
| `BUSL-1.1` | Business Source License 1.1 |
| `BSD-3-Clause` | BSD 3-Clause |
| `ISC` | ISC License |

Full list: https://spdx.org/licenses/

## CI Usage

```yaml
- name: Check license headers
  run: quench --ci --check license-headers

- name: Fix license headers (main only)
  if: github.ref == 'refs/heads/main'
  run: |
    quench --ci --fix --check license-headers
    git add -A
    git commit -m "chore: update license headers" || true
    git push
```

## Comparison to External Tools

| Tool | Pros | Cons |
|------|------|------|
| `addlicense` | Go tool, fast | Separate install, limited config |
| `license-header-checker` | npm, many languages | Node dependency |
| `scripts/license` | Shell script, simple | Manual maintenance |
| **quench** | Integrated, configurable | Part of larger tool |

Quench integrates license header checking with other quality checks, avoiding separate tooling.
