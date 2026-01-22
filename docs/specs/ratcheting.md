# Ratcheting Specification

Ratcheting prevents quality regressions while allowing gradual improvement.

## Concept

A ratchet only turns one way:
- Metrics can improve (coverage up, escapes down, binary smaller)
- Metrics cannot regress past the baseline
- When metrics improve, the baseline auto-updates

This enables incremental quality improvement without manual threshold maintenance.

## Supported Metrics

| Metric | Good Direction | Ratchet Behavior | Default |
|--------|----------------|------------------|---------|
| Coverage | Higher | Floor rises with improvements | On |
| Escape hatch counts | Lower | Ceiling drops as you clean up | On |
| Binary size | Smaller | Ceiling drops on optimization | Off |
| Compile time (cold) | Faster | Ceiling drops on improvement | Off |
| Compile time (hot) | Faster | Ceiling drops on improvement | Off |
| Test time (total) | Faster | Ceiling drops on improvement | Off |
| Test time (avg) | Faster | Ceiling drops on improvement | Off |
| Test time (max) | Faster | Ceiling drops on improvement | Off |

## Baseline Storage

Ratcheting requires a stored baseline. Two options:

**Committed file** (recommended):
```
.quench/baseline.json
```

**Git notes** (alternative):
```bash
git notes --ref=quench show HEAD
```

See `99-todo.md` for baseline storage details.

## Configuration

Enable ratcheting per-metric:

```toml
[ratchet]
enabled = true

# Which metrics to ratchet (defaults shown)
coverage = true          # Coverage can't drop
escapes = true           # Escape counts can't increase
binary_size = false      # Opt-in: binaries can't grow

# Compile time (granular)
compile_time_cold = false
compile_time_hot = false

# Test time (granular)
test_time_total = false
test_time_avg = false
test_time_max = false    # e.g., ratchet only slowest test
```

### Tolerance

Allow small regressions to handle noise:

```toml
[ratchet]
coverage_tolerance = 0.5       # Allow 0.5% coverage drop
binary_size_tolerance = "100KB" # Allow 100KB size increase
compile_time_tolerance = "5s"   # Allow 5s compile time increase
```

### Per-Package

Ratcheting respects per-package breakdown:

```toml
[ratchet.package.core]
coverage = true          # Ratchet core coverage

[ratchet.package.cli]
coverage = false         # Don't ratchet CLI coverage (still developing)
```

## Behavior

### On Check (quench)

```
escapes: FAIL
  unsafe: 5 (max: 3 from baseline)
    Escape hatch count increased. Clean up or update baseline.
```

### On Fix (quench --fix)

When `--fix` is run and metrics have improved:

```
ratchet: updated baseline
  coverage: 78.4% → 82.1% (new floor)
  escapes.unsafe: 5 → 3 (new ceiling)
```

The baseline file is updated automatically.

### CI Workflow

Typical CI setup:

```yaml
- name: Check quality
  run: quench --ci

- name: Update baseline on main
  if: github.ref == 'refs/heads/main'
  run: |
    quench --fix
    git add .quench/baseline.json
    git commit -m "chore: update quality baseline" || true
    git push
```

## Output

### Pass (within baseline)

```
coverage: 82.1% (baseline: 78.4%) ✓
escapes: 3 unsafe (baseline: 5) ✓
```

### Fail (regression)

```
coverage: FAIL
  76.2% (baseline: 78.4%, min with tolerance: 77.9%)
    Coverage dropped below ratcheted baseline.
```

### JSON Output

```json
{
  "ratchet": {
    "coverage": {
      "current": 76.2,
      "baseline": 78.4,
      "tolerance": 0.5,
      "min_allowed": 77.9,
      "passed": false
    },
    "escapes": {
      "unsafe": {
        "current": 3,
        "baseline": 5,
        "passed": true,
        "improved": true
      }
    }
  }
}
```

## Baseline File Format

```json
{
  "version": 1,
  "updated": "2026-01-21T10:30:00Z",
  "commit": "abc123",
  "metrics": {
    "coverage": {
      "total": 78.4,
      "by_package": {
        "core": 82.3,
        "cli": 68.9
      }
    },
    "escapes": {
      "source": {
        "unsafe": 5,
        "unwrap": 0,
        "allow": 12
      }
    },
    "binary_size": {
      "quench": 4404019
    },
    "compile_time": {
      "cold": 45.2,
      "hot": 1.8
    },
    "test_time": {
      "total": 12.4,
      "avg": 0.045,
      "max": 2.1
    }
  }
}
```

## Notes

- Ratcheting is **opt-in** - must explicitly enable
- Tolerance prevents failing on noise (especially compile time)
- Per-package ratcheting allows different policies for different maturity levels
- `--fix` updates baseline only when metrics improve (never on regression)
- Baseline should be committed to repo for team visibility
