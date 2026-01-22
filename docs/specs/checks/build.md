# Build Check Specification

The `build` check validates build outputs and tracks build performance metrics.

## Purpose

Track build health across different project types:
- **Binary size**: Compiled executables, libraries, bundles
- **Build time**: Cold and hot/incremental builds
- **Output validation**: Ensure builds produce expected artifacts

**CI-only.** This check only runs in `--ci` mode. It is skipped in fast mode.

## Concepts

### Targets

Build targets are the outputs being measured. How targets are defined depends on the language adapter:

| Adapter | Target Source | Examples |
|---------|---------------|----------|
| Rust | `[[bin]]` in Cargo.toml | `myapp`, `myserver` |
| TypeScript/JS | `build.output` in config | `dist/bundle.js`, `dist/app.min.js` |
| Go | `main` packages | `cmd/myapp` |

Targets can be auto-detected or explicitly configured.

### Metrics

| Metric | Description | Ratchet Direction |
|--------|-------------|-------------------|
| `size` | Output file size | Smaller is better |
| `time_cold` | Clean build time | Faster is better |
| `time_hot` | Incremental build time | Faster is better |

## Output

### Summary

```
build: metrics
  size:
    myapp: 4.2 MB
    myserver: 12.1 MB
  time:
    cold: 45.2s
    hot: 1.8s
```

### Fail (threshold exceeded)

```
build: FAIL
  myapp: 5.1 MB (max: 5 MB)
    Reduce binary size. Check for unnecessary dependencies.
  cold build: 72s (max: 60s)
    Build time exceeds threshold. Consider incremental compilation or dependency reduction.
```

### JSON Output

```json
{
  "name": "build",
  "passed": false,
  "violations": [
    {
      "file": null,
      "line": null,
      "type": "size_exceeded",
      "target": "myapp",
      "value": 5347737,
      "threshold": 5242880,
      "advice": "Reduce binary size. Check for unnecessary dependencies."
    }
  ],
  "metrics": {
    "size": {
      "myapp": 4404019,
      "myserver": 12692480
    },
    "time": {
      "cold": 45.2,
      "hot": 1.8
    }
  }
}
```

**Violation types**: `size_exceeded`, `time_cold_exceeded`, `time_hot_exceeded`, `missing_target`

## Configuration

```toml
[check.build]
check = "error"                    # error | warn | off

# Targets to measure (auto-detected by language adapter if omitted)
# targets = ["myapp", "myserver"]

# Size thresholds (per-target or global)
size_max = "10 MB"                 # Global default

[check.build.target.myapp]
size_max = "5 MB"                  # Per-target override

[check.build.target.myserver]
size_max = "15 MB"

# Time thresholds
time_cold_max = "60s"
time_hot_max = "5s"
```

## Language-Specific Behavior

Build behavior is provided by language adapters. Each adapter defines:
- How targets are detected
- How builds are executed
- How outputs are measured

### Rust

See [langs/rust.md](../langs/rust.md) for full details.

```toml
[rust]
targets = ["myapp", "myserver"]    # Override auto-detection from Cargo.toml
binary_size = true                 # Enable size tracking
build_time = true                  # Enable time tracking
```

**Build commands:**
- Cold: `cargo clean && cargo build --release`
- Hot: `touch src/lib.rs && cargo build`

**Size measurement:** Release binary size (stripped if `strip = true` in profile).

### TypeScript / JavaScript (Future)

```toml
[typescript]
# Bundler detection: vite.config.ts, webpack.config.js, etc.
bundler = "vite"                   # auto | vite | webpack | esbuild | rollup

# Build outputs to measure
targets = ["dist/index.js", "dist/vendor.js"]

# Or use glob patterns
# targets = ["dist/*.js"]

bundle_size = true
build_time = true
```

**Build commands:**
- Cold: `rm -rf dist && npm run build`
- Hot: `npm run build` (bundler handles caching)

**Size measurement:**
- Raw size and gzipped size reported
- Source maps excluded from size calculation
- Chunk breakdown available in JSON output

**Bundler-specific features:**
- Vite: Reads chunk info from build manifest
- Webpack: Parses stats.json if available
- esbuild: Uses metafile output

### Go (Future)

```toml
[go]
targets = ["cmd/myapp", "cmd/server"]
binary_size = true
build_time = true
```

**Build commands:**
- Cold: `go clean -cache && go build -o bin/ ./cmd/...`
- Hot: `go build -o bin/ ./cmd/...`

### Shell

Shell scripts are not compiled. The build check is not applicable to shell-only projects.

If a project has both Rust and Shell (e.g., Rust CLI with shell test scripts), the build check applies only to Rust targets.

## Bundle Analysis (Future)

For JavaScript/TypeScript projects, extended bundle analysis:

```toml
[check.build.bundle]
# Warn on large dependencies
dep_size_max = "500 KB"

# Fail on known problematic imports
forbid = ["moment", "lodash"]      # Prefer date-fns, lodash-es

# Chunk size limits
chunk_max = "250 KB"
```

Output:
```
build: FAIL
  dist/vendor.js: 1.2 MB (max: 500 KB)
    Large vendor bundle. Consider code splitting or lighter alternatives.
  moment: 289 KB imported
    Use date-fns or dayjs instead (moment is deprecated and large).
```

## Ratcheting

Build metrics can be ratcheted to prevent regressions:

```toml
[ratchet]
binary_size = true                 # Size can't grow
build_time_cold = false            # Too noisy for cold builds
build_time_hot = true              # Hot builds should stay fast
```

When ratcheted, the baseline tracks the best achieved value. Any regression fails:

```
build: FAIL
  myapp: 4.5 MB (baseline: 4.2 MB)
    Binary size increased. Review recent changes for size impact.
```

## CI Integration

```yaml
- name: Build metrics
  run: quench check --ci --build

- name: Track size over time
  run: |
    quench check --ci --save .quench/baseline.json
    # Commit baseline on main branch
```

## Comparison to External Tools

| Tool | Scope | Pros | Cons |
|------|-------|------|------|
| `cargo bloat` | Rust | Detailed function-level | Rust only |
| `webpack-bundle-analyzer` | JS | Visual treemap | Webpack only |
| `size-limit` | JS | Simple CI integration | JS only |
| `hyperfine` | Any | Accurate benchmarking | Manual setup |
| **quench** | Multi-language | Integrated, ratcheting | Less detailed |

Quench provides unified build tracking across languages. Use specialized tools for deep analysis.
