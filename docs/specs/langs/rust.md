# Rust Language Support

Rust-specific behavior for quench checks.

## Detection

Detected when `Cargo.toml` exists in project root.

## Test Code Detection

**Test files** (entire file is test code):
- `*_test.rs`, `*_tests.rs`
- Files in `tests/` directory

**Inline test code** (within source files):
- Lines inside `#[cfg(test)]` blocks are counted as test LOC
- Configurable: `split_cfg_test = true` (default)

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

For test correlation, changes to `#[cfg(test)]` blocks satisfy the test requirement.

## Default Escape Patterns

| Pattern | Action | Comment Required |
|---------|--------|------------------|
| `unsafe { }` | comment | `// SAFETY:` |
| `.unwrap()` | forbid | - |
| `.expect(` | forbid | - |
| `mem::transmute` | comment | `// SAFETY:` |

## Suppress

Controls `#[allow(...)]` and `#[expect(...)]` attributes.

| Setting | Behavior |
|---------|----------|
| `"forbid"` | Never allowed |
| `"comment"` | Requires justification comment |
| `"allow"` | Always allowed |

Default: `"comment"` for source, `"allow"` for test code.

```toml
[checks.rust.suppress]
check = "comment"              # forbid | comment | allow
# comment = "// JUSTIFIED:"    # optional: require specific pattern (default: any)

[checks.rust.suppress.source]
allow = ["dead_code"]          # no comment needed
forbid = ["unsafe_code"]       # never allowed

[checks.rust.suppress.test]
check = "allow"                # tests can suppress freely
```

## Policy

Enforce lint configuration hygiene.

```toml
[checks.rust.policy]
lint_changes = "standalone"    # lint config changes must be standalone PRs
lint_config = [                # files that trigger standalone requirement
  "rustfmt.toml",
  ".rustfmt.toml",
  "clippy.toml",
  ".clippy.toml",
]
```

When `lint_changes = "standalone"`, changing any `lint_config` files alongside source/test changes fails:

```
rust: FAIL
  lint config changes must be standalone
    Changed: rustfmt.toml
    Also changed: src/parser.rs, src/lexer.rs
  Submit lint config changes in a separate PR.
```

## CI Mode: Compile Metrics

Controlled by `--[no-]compile` flag. Only runs in CI mode.

### Binary Size

Track release binary sizes. Binaries auto-detected from `[[bin]]` in Cargo.toml.

```
compile: binary size
  quench: 4.2 MB
  server: 12.1 MB
```

With threshold:
```
compile: FAIL
  quench: 5.1 MB (max: 5 MB)
```

### Compile Time

Track compile times:

- **Cold**: `cargo clean && cargo build --release`
- **Hot**: Incremental debug rebuild

```
compile: time
  cold (release): 45.2s
  hot (debug): 1.8s
```

## CI Mode: Test Metrics

Controlled by `--[no-]tests` flag. In CI mode, `--tests` runs test suites and collects metrics.

### Test Time

Track test execution times across configured test suites.

- **Total**: Sum of all suite times
- **Average**: Average per-test time
- **Max**: Slowest individual test

```
tests: time
  total: 12.4s
  avg: 45ms
  max: 2.1s (tests::integration::large_file_parse)
  suites:
    cargo: 8.2s
    bats: 4.2s
```

### Coverage

Uses `cargo llvm-cov` by default.

```
tests: coverage 78.4%
  core: 82.3%
  cli: 68.9%
```

Multiple test suites contribute to coverage via LLVM profile merging.

## Configuration

```toml
[checks.rust]
check = "error"
split_cfg_test = true            # Count #[cfg(test)] as test LOC

# Compile metrics (CI mode)
binary_size = true
compile_time = true

# Compile thresholds (fail if exceeded)
binary_size_max = "5 MB"
compile_time_cold_max = "60s"
compile_time_hot_max = "2s"

# Test/coverage metrics (CI mode)
coverage = true
test_time = true

# Test thresholds
coverage_min = 75
test_time_total_max = "30s"
test_time_max = "1s"

# Test suites (see [11-test-runners.md](../11-test-runners.md))
[[checks.rust.test_suites]]
runner = "cargo"

[[checks.rust.test_suites]]
runner = "bats"
path = "tests/cli/"
setup = "cargo build"
```

### Per-Package Coverage

```toml
[checks.rust.coverage.package.core]
min = 90                         # Higher coverage for core

[checks.rust.coverage.package.cli]
min = 60
exclude_files = ["src/main.rs"]

[checks.rust.coverage.package.experimental]
check = "off"                  # Skip coverage for experimental
```
