# Ruby Profile Defaults and Landing the Plane

Implement Ruby language profile for `quench init --with ruby` and integrate Ruby into the init command's auto-detection and profile system.

## Overview

The Ruby language adapter already exists (`crates/cli/src/adapter/ruby/`), but the profile defaults for `quench init` are missing. This plan adds:

1. Ruby profile defaults function in `profiles.rs`
2. Ruby detected section for auto-detection mode
3. Ruby landing items for agent checklist
4. Ruby detection in `init.rs` and handling in `cmd_init.rs`
5. Integration into `ProfileRegistry`

## Project Structure

Files to modify:
```
crates/cli/src/
├── profiles.rs          # Add ruby_profile_defaults(), ruby_detected_section(), ruby_landing_items()
├── init.rs              # Add DetectedLanguage::Ruby and detect_ruby()
└── cmd_init.rs          # Handle Ruby in auto-detection output

tests/specs/cli/
└── init.rs              # Add Ruby profile and detection specs
```

## Dependencies

No new external dependencies. Uses existing crate infrastructure.

## Implementation Phases

### Phase 1: Add Ruby Profile Defaults

Add `ruby_profile_defaults()` function to `profiles.rs` matching the spec in `docs/specs/langs/ruby.md`.

**File: `crates/cli/src/profiles.rs`**

```rust
/// Ruby profile configuration for quench init.
pub fn ruby_profile_defaults() -> String {
    r#"[ruby]
# No build metrics for interpreted language

[ruby.suppress]
check = "comment"

[ruby.suppress.test]
check = "allow"

[ruby.policy]
lint_changes = "standalone"
lint_config = [".rubocop.yml", ".rubocop_todo.yml", ".standard.yml"]

[[check.escapes.patterns]]
name = "binding_pry"
pattern = "binding\\.pry"
action = "forbid"
in_tests = "allow"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
name = "byebug"
pattern = "byebug"
action = "forbid"
in_tests = "allow"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
name = "debugger"
pattern = "debugger"
action = "forbid"
in_tests = "allow"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
name = "eval"
pattern = "eval\\("
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining why eval is necessary."

[[check.escapes.patterns]]
name = "instance_eval"
pattern = "instance_eval"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the DSL or metaprogramming use case."

[[check.escapes.patterns]]
name = "class_eval"
pattern = "class_eval"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the metaprogramming use case."
"#
    .to_string()
}
```

**Verification:**
- Unit test that profile content contains expected sections
- Unit test that patterns are valid regex

### Phase 2: Add Ruby Landing Items

Add `ruby_landing_items()` function to `profiles.rs`.

**File: `crates/cli/src/profiles.rs`**

```rust
/// Ruby-specific Landing the Plane checklist items.
pub fn ruby_landing_items() -> &'static [&'static str] {
    &[
        "bundle exec rubocop",
        "bundle exec rspec",
        "bundle exec rake test",
    ]
}
```

**Note:** The spec mentions conditional items (standardrb if `.standard.yml` exists, rails test if Rails project), but these require runtime detection. For now, provide the common commands. Agent files can be customized per-project.

**Verification:**
- Unit test that items are non-empty
- Items are valid shell commands

### Phase 3: Add Ruby Detected Section

Add `ruby_detected_section()` for auto-detection mode (minimal config vs full profile).

**File: `crates/cli/src/profiles.rs`**

```rust
/// Minimal Ruby section for auto-detection output.
pub fn ruby_detected_section() -> &'static str {
    r#"[ruby]
ruby.cloc.check = "error"
ruby.policy.check = "error"
ruby.suppress.check = "comment"
"#
}
```

**Verification:**
- Unit test that section uses dotted key format (per spec)

### Phase 4: Add Ruby to ProfileRegistry

Update `ProfileRegistry` to include Ruby.

**File: `crates/cli/src/profiles.rs`**

```rust
impl ProfileRegistry {
    pub fn available() -> &'static [&'static str] {
        &["rust", "golang", "javascript", "ruby", "shell", "claude", "cursor"]
        //                                ^^^^^ add ruby
    }

    pub fn get(name: &str) -> Option<String> {
        match name.to_lowercase().as_str() {
            "ruby" | "rb" => Some(ruby_profile_defaults()),
            // ... existing cases ...
        }
    }

    pub fn suggest(name: &str) -> Option<&'static str> {
        // Add Ruby aliases
        match lower.as_str() {
            "rb" | "rails" | "rake" => Some("ruby"),
            // ... existing cases ...
        }
    }
}
```

**Verification:**
- `ProfileRegistry::is_valid("ruby")` returns true
- `ProfileRegistry::is_valid("rb")` returns true
- `ProfileRegistry::get("ruby")` returns profile content

### Phase 5: Add Ruby Language Detection

Add `DetectedLanguage::Ruby` to `init.rs` and detection logic.

**File: `crates/cli/src/init.rs`**

```rust
pub enum DetectedLanguage {
    Rust,
    Golang,
    JavaScript,
    Shell,
    Ruby,  // Add this variant
}

pub fn detect_languages(root: &Path) -> Vec<DetectedLanguage> {
    // ... existing detections ...

    // Ruby: Gemfile, *.gemspec, config.ru, or config/application.rb exists
    if root.join("Gemfile").exists()
        || has_gemspec(root)
        || root.join("config.ru").exists()
        || root.join("config/application.rb").exists()
    {
        languages.push(DetectedLanguage::Ruby);
    }

    languages
}

fn has_gemspec(root: &Path) -> bool {
    root.read_dir()
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|entry| {
                let path = entry.path();
                path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("gemspec")
            })
        })
        .unwrap_or(false)
}
```

**Verification:**
- Unit test: `detect_languages()` with `Gemfile` returns Ruby
- Unit test: `detect_languages()` with `*.gemspec` returns Ruby
- Unit test: `detect_languages()` with `config.ru` returns Ruby
- Unit test: `detect_languages()` with `config/application.rb` returns Ruby

### Phase 6: Handle Ruby in cmd_init.rs

Update `cmd_init.rs` to handle Ruby in auto-detection output.

**File: `crates/cli/src/cmd_init.rs`**

```rust
use crate::profiles::{
    // ... existing imports ...
    ruby_detected_section,  // Add this
};

// In the auto-detection match:
for lang in &detected_langs {
    cfg.push('\n');
    match lang {
        DetectedLanguage::Rust => cfg.push_str(rust_detected_section()),
        DetectedLanguage::Golang => cfg.push_str(golang_detected_section()),
        DetectedLanguage::JavaScript => cfg.push_str(javascript_detected_section()),
        DetectedLanguage::Shell => cfg.push_str(shell_detected_section()),
        DetectedLanguage::Ruby => cfg.push_str(ruby_detected_section()),  // Add this
    }
}

// In the message building:
for lang in &detected_langs {
    detected_names.push(match lang {
        DetectedLanguage::Rust => "rust",
        DetectedLanguage::Golang => "golang",
        DetectedLanguage::JavaScript => "javascript",
        DetectedLanguage::Shell => "shell",
        DetectedLanguage::Ruby => "ruby",  // Add this
    });
}
```

**Verification:**
- Integration test: `quench init` in directory with `Gemfile` produces `[ruby]` section

## Key Implementation Details

### Escape Pattern Regex

The Ruby escape patterns use regex. Ensure proper escaping:
- `binding\.pry` - escape the dot
- `eval\\(` - escape the parenthesis (double backslash in TOML)

### Profile vs Detected Section Difference

- **Full profile** (`--with ruby`): Complete config with escape patterns, suppress settings, policy
- **Detected section** (auto-detect): Minimal dotted-key config enabling checks at error level

### Detection Order

Ruby detection markers per spec (in priority order for messaging):
1. `Gemfile` - Bundler-managed Ruby project
2. `*.gemspec` - Ruby gem project
3. `config.ru` - Rack application
4. `config/application.rb` - Rails application

## Verification Plan

### Unit Tests (`crates/cli/src/init_tests.rs`)

```rust
#[test]
fn detect_ruby_from_gemfile() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("Gemfile"), "source 'https://rubygems.org'").unwrap();
    let detected = detect_languages(temp.path());
    assert!(detected.contains(&DetectedLanguage::Ruby));
}

#[test]
fn detect_ruby_from_gemspec() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("myapp.gemspec"), "Gem::Specification.new").unwrap();
    let detected = detect_languages(temp.path());
    assert!(detected.contains(&DetectedLanguage::Ruby));
}

#[test]
fn detect_ruby_from_config_ru() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("config.ru"), "run MyApp").unwrap();
    let detected = detect_languages(temp.path());
    assert!(detected.contains(&DetectedLanguage::Ruby));
}

#[test]
fn detect_ruby_from_rails() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("config")).unwrap();
    fs::write(temp.path().join("config/application.rb"), "module MyApp").unwrap();
    let detected = detect_languages(temp.path());
    assert!(detected.contains(&DetectedLanguage::Ruby));
}
```

### Behavioral Specs (`tests/specs/cli/init.rs`)

```rust
/// Spec: docs/specs/langs/ruby.md#profile-defaults
///
/// > --with ruby configures Ruby defaults
#[test]
fn init_with_ruby_configures_ruby_defaults() {
    let temp = Project::empty();

    quench_cmd()
        .args(["init", "--with", "ruby"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("ruby"));

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
    assert!(config.contains("[ruby.suppress]"));
    assert!(config.contains("[ruby.policy]"));
    assert!(config.contains("binding"), "should have debugger escape pattern");
}

/// Spec: docs/specs/langs/ruby.md#detection
///
/// > Auto-detects Ruby from Gemfile
#[test]
fn init_auto_detects_ruby_from_gemfile() {
    let temp = Project::empty();
    temp.file("Gemfile", "source 'https://rubygems.org'\n");

    quench_cmd()
        .args(["init"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
}

/// Spec: docs/specs/01-cli.md#explicit-profiles
///
/// > --with rb is an alias for ruby
#[test]
fn init_with_rb_alias() {
    let temp = Project::empty();

    quench_cmd()
        .args(["init", "--with", "rb"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
}
```

### Landing the Plane Checklist

Before committing:

- [x] Unit tests in sibling `_tests.rs` files (init_tests.rs, profiles_tests.rs if created)
- [x] Run `make check`:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all`
  - `cargo build --all`
  - `cargo audit`
  - `cargo deny check`
- [x] `CACHE_VERSION` does NOT need bumping (profile defaults don't affect check logic)

## Summary

| Phase | Deliverable | Files Modified |
|-------|-------------|----------------|
| 1 | `ruby_profile_defaults()` | `profiles.rs` |
| 2 | `ruby_landing_items()` | `profiles.rs` |
| 3 | `ruby_detected_section()` | `profiles.rs` |
| 4 | `ProfileRegistry` integration | `profiles.rs` |
| 5 | `DetectedLanguage::Ruby` | `init.rs` |
| 6 | Auto-detection handling | `cmd_init.rs` |

All phases are independently verifiable via unit tests. The full integration can be verified with behavioral specs in `tests/specs/cli/init.rs`.
