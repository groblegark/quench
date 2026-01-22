# Future Work

Features discussed but not yet fully specified. To be designed in later phases.

## Report Command

The `quench report` command generates human-readable reports from stored metrics.

### Formats

- **Markdown** (default): Summary tables, metric values, trend indicators
- **JSON**: Machine-readable for external tools
- **HTML**: Static dashboard page

### Data Sources

Reports read from:
- `.quench/baseline.json` (committed file)
- Git notes (`git notes --ref=quench`)

History is derived from git notes history or git log/blame (for committed baseline).

### Weekly Summary (Future)

Generate trending reports over configurable period:
- Summary table with deltas from previous period
- Pass/fail status per metric
- Trend direction indicators (↑ ↓ →)

## GitHub Pages Dashboard

Auto-publish metrics to GitHub Pages. Based on pattern from wok project.

### Components

- `docs/reports/index.html` - Static dashboard with JavaScript
- `docs/reports/quality/latest.json` - Current metrics
- `docs/reports/quality/latest.md` - Human-readable summary

### CI Integration

```yaml
- name: Generate reports
  run: |
    quench check --ci --save .quench/baseline.json
    quench report -o json > docs/reports/quality/latest.json
    quench report -o html > docs/reports/index.html

- name: Deploy to GitHub Pages
  uses: actions/deploy-pages@v2
```

### Dashboard Features

- Metric cards with current values
- Color coding (green/yellow/red based on thresholds)
- Links to CI runs
- Responsive design

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
