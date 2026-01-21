# Future Work

Features discussed but not yet fully specified. To be designed in later phases.

## Reporting

### Weekly Reports

Generate trending reports over configurable period.

- Summary table with deltas
- Pass/fail status per metric
- Commit/work tracking summary
- Markdown output

### GitHub Pages Dashboard

Auto-publish metrics to GitHub Pages.

- Latest metrics JSON
- Historical trend data
- Human-readable summary
- CI workflow integration

## License Headers

Auto-manage license headers in source files.

- Add missing headers
- Update copyright year
- Configurable header template
- `--fix` support
- Default: disabled

## Git Checks

### Commit Format Validation

Validate conventional commit format.

- `feat:`, `fix:`, `chore:` prefixes (or `feat(...):`)
- Configurable patterns
- Default: disabled

## Future Adapters

| Adapter | Detection | Notes |
|---------|-----------|-------|
| `typescript` | `tsconfig.json` | `as unknown`, `@ts-ignore`, `any` escapes |
| `python` | `pyproject.toml` | `# type: ignore`, `# noqa` escapes |
| `go` | `go.mod` | `unsafe.Pointer`, `//nolint` escapes |

## Notes from Interview

- Primary users are AI agents ("landing the plane")
- Performance target: sub-second for fast checks
- All output should be agent-friendly (token-efficient)
- Progressive disclosure: only surface failures
- `--fix` should be explicit about what it can/cannot fix
