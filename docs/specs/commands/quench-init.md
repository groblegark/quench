# quench init

Initialize quench configuration for a project.

## Default Output

Base config generated for all projects: [templates/init.default.toml](../templates/init.default.toml)

Language and agent profile sections are appended based on detection or explicitly from `--with`.

## `--with` Flag

Explicitly specify profiles, skipping auto-detection:

```bash
quench init --with rust           # Rust only
quench init --with rust,shell     # Multiple languages
quench init --with claude         # Agent only
quench init --with rust,claude    # Language + agent
```

Profiles are comma-separated. Combining languages and agents configures both.

## Language Detection

Auto-detect languages from project root markers:

| Language | Marker Files |
|----------|-------------|
| `rust` | `Cargo.toml` |
| `golang` | `go.mod` |
| `javascript` | `package.json`, `tsconfig.json`, `jsconfig.json` |
| `shell` | `*.sh` in root, `bin/`, or `scripts/` |

Detection is additive: a project with `Cargo.toml` and `scripts/*.sh` detects both `rust` and `shell`.

When a language is detected, its section is appended. Example for Rust:

```toml
# Supported Languages:
# [rust], [golang], [javascript], [shell]

[rust]
rust.cloc.check = "error"
rust.policy.check = "error"
rust.suppress.check = "comment"
```

When `--with` is specified, language detection is skipped and only the specified profiles are used.

## Agent Detection

Auto-detect agent files from project root:

| Agent | Marker Files |
|-------|-------------|
| `claude` | `CLAUDE.md` |
| `cursor` | `.cursorrules`, `.cursor/rules/*.md[c]` |

Both language and agent detection run by default. When `--with` is specified, all auto-detection is skipped.

When an agent is detected, the `[check.agents]` section is updated. Example for Claude:

```toml
[check.agents]
check = "error"
required = ["CLAUDE.md"]
```
