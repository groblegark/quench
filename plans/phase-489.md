# Phase 489: Ruby Profile Defaults

Add Ruby language profile to the quench init system with detection, defaults, and configuration generation.

## Overview

This phase adds Ruby as a supported language profile in quench, enabling:
- Auto-detection of Ruby projects via Gemfile, *.gemspec, config.ru, and Rails markers
- Profile defaults for escapes, suppress, and policy settings
- Configuration generation via `quench init --with ruby`

## Project Structure

Files to modify:

```
crates/cli/src/
├── init.rs           # Add DetectedLanguage::Ruby and detection logic
├── init_tests.rs     # Unit tests for Ruby detection
├── cmd_init.rs       # Handle Ruby in config generation
└── profiles.rs       # Ruby profile defaults and registry

tests/specs/cli/
└── init.rs           # Behavioral specs for Ruby init
```

## Dependencies

- Phase 1505-1510 (Init Command - Profile System) - Already completed
- No external crate dependencies

## Implementation Phases

### Phase 1: Detection Infrastructure

Add Ruby to the detection system in `init.rs`.

**1.1 Add DetectedLanguage variant**

```rust
// init.rs
pub enum DetectedLanguage {
    Rust,
    Golang,
    JavaScript,
    Shell,
    Ruby,  // Add this
}
```

**1.2 Add Ruby detection logic**

```rust
// init.rs - in detect_languages()
// Ruby: Gemfile, *.gemspec, config.ru, or config/application.rb
if has_ruby_markers(root) {
    languages.push(DetectedLanguage::Ruby);
}
```

```rust
// init.rs - new helper function
fn has_ruby_markers(root: &Path) -> bool {
    // Gemfile
    if root.join("Gemfile").exists() {
        return true;
    }
    // *.gemspec in root
    if let Ok(entries) = root.read_dir() {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("gemspec") {
                return true;
            }
        }
    }
    // config.ru (Rack)
    if root.join("config.ru").exists() {
        return true;
    }
    // config/application.rb (Rails)
    if root.join("config/application.rb").exists() {
        return true;
    }
    false
}
```

**1.3 Unit tests**

Add to `init_tests.rs`:

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
fn detect_ruby_from_rails_application() {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("config")).unwrap();
    fs::write(temp.path().join("config/application.rb"), "module MyApp").unwrap();
    let detected = detect_languages(temp.path());
    assert!(detected.contains(&DetectedLanguage::Ruby));
}
```

### Phase 2: Profile Defaults

Add Ruby profile configuration to `profiles.rs`.

**2.1 Full profile defaults function**

```rust
/// Default Ruby profile configuration for quench init.
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
pattern = "\\bbyebug\\b"
action = "forbid"
in_tests = "allow"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
name = "debugger"
pattern = "\\bdebugger\\b"
action = "forbid"
in_tests = "allow"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
name = "eval"
pattern = "\\beval\\("
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining why eval is necessary."

[[check.escapes.patterns]]
name = "instance_eval"
pattern = "\\binstance_eval\\b"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the DSL or metaprogramming use case."

[[check.escapes.patterns]]
name = "class_eval"
pattern = "\\bclass_eval\\b"
action = "comment"
comment = "# METAPROGRAMMING:"
advice = "Add a # METAPROGRAMMING: comment explaining the metaprogramming use case."
"#
    .to_string()
}
```

**2.2 Detected section (minimal)**

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

**2.3 Landing the Plane items**

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

### Phase 3: Profile Registry

Update `ProfileRegistry` in `profiles.rs` to include Ruby.

**3.1 Update available()**

```rust
pub fn available() -> &'static [&'static str] {
    &["rust", "golang", "javascript", "shell", "ruby", "claude", "cursor"]
}
```

**3.2 Update get()**

```rust
pub fn get(name: &str) -> Option<String> {
    match name.to_lowercase().as_str() {
        "rust" => Some(rust_profile_defaults()),
        "shell" => Some(shell_profile_defaults()),
        "golang" | "go" => Some(golang_profile_defaults()),
        "javascript" | "js" | "typescript" | "ts" => Some(javascript_profile_defaults()),
        "ruby" | "rb" => Some(ruby_profile_defaults()),  // Add this
        "claude" => Some(claude_profile_defaults().to_string()),
        "cursor" => Some(cursor_profile_defaults().to_string()),
        _ => None,
    }
}
```

**3.3 Update suggest()**

```rust
pub fn suggest(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    // ... existing prefix matching ...

    // Check for common aliases
    match lower.as_str() {
        "js" | "ts" | "typescript" | "node" => Some("javascript"),
        "go" => Some("golang"),
        "bash" | "zsh" | "sh" => Some("shell"),
        "rails" | "gem" => Some("ruby"),  // Add this
        _ => None,
    }
}
```

### Phase 4: Init Command Integration

Update `cmd_init.rs` to handle Ruby detection.

**4.1 Add import**

```rust
use crate::profiles::{
    ProfileRegistry, agents_section, default_template_base, default_template_suffix,
    golang_detected_section, javascript_detected_section, rust_detected_section,
    shell_detected_section, ruby_detected_section,  // Add this
};
```

**4.2 Handle Ruby in detection output**

```rust
// In the no --with branch, inside the for lang in &detected_langs loop:
match lang {
    DetectedLanguage::Rust => cfg.push_str(rust_detected_section()),
    DetectedLanguage::Golang => cfg.push_str(golang_detected_section()),
    DetectedLanguage::JavaScript => cfg.push_str(javascript_detected_section()),
    DetectedLanguage::Shell => cfg.push_str(shell_detected_section()),
    DetectedLanguage::Ruby => cfg.push_str(ruby_detected_section()),  // Add this
}
```

**4.3 Update message generation**

```rust
// In the detected_names building:
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

### Phase 5: Behavioral Specs

Add spec tests in `tests/specs/cli/init.rs`.

```rust
// =============================================================================
// Ruby Detection Specs
// =============================================================================

/// Spec: docs/specs/langs/ruby.md#detection
///
/// > Detected when Gemfile exists
#[test]
fn init_detects_ruby_from_gemfile() {
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

/// Spec: docs/specs/langs/ruby.md#detection
///
/// > Detected when *.gemspec exists
#[test]
fn init_detects_ruby_from_gemspec() {
    let temp = Project::empty();
    temp.file("myapp.gemspec", "Gem::Specification.new do |s|\nend\n");

    quench_cmd()
        .args(["init"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
}

/// Spec: docs/specs/langs/ruby.md#detection
///
/// > Detected when config.ru exists (Rack)
#[test]
fn init_detects_ruby_from_config_ru() {
    let temp = Project::empty();
    temp.file("config.ru", "run MyApp\n");

    quench_cmd()
        .args(["init"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
}

/// Spec: docs/specs/langs/ruby.md#detection
///
/// > Detected when config/application.rb exists (Rails)
#[test]
fn init_detects_ruby_from_rails_application() {
    let temp = Project::empty();
    temp.file("config/application.rb", "module MyApp\nend\n");

    quench_cmd()
        .args(["init"])
        .current_dir(temp.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp.path().join("quench.toml")).unwrap();
    assert!(config.contains("[ruby]"));
}

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
    assert!(config.contains("binding"), "should have binding.pry escape pattern");
}

/// Spec: docs/specs/langs/ruby.md#profile-defaults
///
/// > --with rb is alias for ruby profile
#[test]
fn init_with_rb_alias_configures_ruby() {
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

## Key Implementation Details

### Detection Priority

Ruby detection follows the same pattern as other languages - check multiple marker files:

1. `Gemfile` - Standard Ruby dependency file
2. `*.gemspec` - Gem specification (iterate root directory)
3. `config.ru` - Rack application
4. `config/application.rb` - Rails application

Detection is OR logic: any marker triggers Ruby detection.

### Escape Patterns

Ruby escape patterns use word boundaries (`\b`) for precision:

| Pattern | Regex | Rationale |
|---------|-------|-----------|
| binding.pry | `binding\.pry` | Literal dot, no word boundary needed |
| byebug | `\bbyebug\b` | Avoid matching "byebugger" |
| debugger | `\bdebugger\b` | Avoid matching "debugger_helper" |
| eval( | `\beval\(` | Avoid matching "evaluation" |
| instance_eval | `\binstance_eval\b` | Standard word boundary |
| class_eval | `\bclass_eval\b` | Standard word boundary |

### Profile vs Detected Section

- **Profile** (`ruby_profile_defaults()`): Full configuration with all escape patterns, used with `--with ruby`
- **Detected Section** (`ruby_detected_section()`): Minimal section with just check levels, used in auto-detection

This matches the pattern established by Rust, Go, JavaScript, and Shell.

## Verification Plan

### Unit Tests

Run unit tests for detection logic:

```bash
cargo test -p quench-cli init_tests::detect_ruby
```

Expected: All 4 Ruby detection tests pass (Gemfile, gemspec, config.ru, Rails).

### Behavioral Specs

Run init command specs:

```bash
cargo test --test specs cli::init::init_detects_ruby
cargo test --test specs cli::init::init_with_ruby
```

Expected: All 6 Ruby-related spec tests pass.

### Manual Verification

1. Create temp directory with Gemfile:
   ```bash
   mkdir /tmp/ruby-test && cd /tmp/ruby-test
   echo "source 'https://rubygems.org'" > Gemfile
   quench init
   cat quench.toml  # Should contain [ruby] section
   ```

2. Test explicit profile:
   ```bash
   rm quench.toml
   quench init --with ruby
   cat quench.toml  # Should contain full Ruby profile with escapes
   ```

3. Test rb alias:
   ```bash
   rm quench.toml
   quench init --with rb
   cat quench.toml  # Should contain [ruby] section
   ```

### Make Check

Run full verification:

```bash
make check
```

Expected: All checks pass including new Ruby tests.
