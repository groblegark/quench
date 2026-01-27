# Ruby Language Adapter Implementation Plan

## Overview

Implement a Ruby language adapter for quench that provides:
- Project detection via Gemfile, *.gemspec, config.ru, config/application.rb
- File classification for RSpec, Minitest, and Cucumber test patterns
- Escape patterns for debuggers (binding.pry, byebug, debugger) and metaprogramming (eval, instance_eval, class_eval)
- RuboCop/Standard suppress directive parsing
- Lint config policy enforcement

Reference: `docs/specs/langs/ruby.md`

## Project Structure

```
crates/cli/src/
├── adapter/
│   ├── mod.rs              # Add Ruby variant to ProjectLanguage, detection, registration
│   ├── patterns.rs         # Add LanguageDefaults impl for RubyConfig
│   └── ruby/
│       ├── mod.rs          # RubyAdapter struct and Adapter trait impl
│       ├── mod_tests.rs    # Unit tests for adapter
│       ├── suppress.rs     # RuboCop/Standard directive parsing
│       ├── suppress_tests.rs
│       ├── policy.rs       # Lint policy checking
│       └── policy_tests.rs
├── config/
│   ├── mod.rs              # Add ruby field to Config struct
│   └── ruby.rs             # RubyConfig, RubySuppressConfig, RubyPolicyConfig
└── adapter/common/
    └── suppress.rs         # Add RUBY CommentStyle constant

tests/
├── specs/adapters/ruby.rs  # Already exists with #[ignore] tests
└── fixtures/ruby/          # Already exists with fixture projects
```

## Dependencies

No new external dependencies required. Uses existing crates:
- `globset` for pattern matching
- `serde` for config deserialization
- `regex` for escape pattern matching

## Implementation Phases

### Phase 481: Ruby Configuration

Add Ruby-specific configuration types.

**Files to modify:**
- `crates/cli/src/config/mod.rs` - Add `ruby.rs` module and `pub ruby: RubyConfig` to `Config`
- `crates/cli/src/config/ruby.rs` - New file

**RubyConfig structure:**

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RubyConfig {
    #[serde(default = "RubyConfig::default_source")]
    pub source: Vec<String>,

    #[serde(default = "RubyConfig::default_tests")]
    pub tests: Vec<String>,

    #[serde(default = "RubyConfig::default_ignore")]
    pub ignore: Vec<String>,

    #[serde(default)]
    pub suppress: RubySuppressConfig,

    #[serde(default)]
    pub policy: RubyPolicyConfig,

    #[serde(default)]
    pub cloc: Option<LangClocConfig>,

    #[serde(default)]
    pub cloc_advice: Option<String>,
}

impl RubyConfig {
    pub(crate) fn default_source() -> Vec<String> {
        vec![
            "**/*.rb".to_string(),
            "**/*.rake".to_string(),
            "Rakefile".to_string(),
            "Gemfile".to_string(),
            "*.gemspec".to_string(),
        ]
    }

    pub(crate) fn default_tests() -> Vec<String> {
        vec![
            "spec/**/*_spec.rb".to_string(),
            "test/**/*_test.rb".to_string(),
            "test/**/test_*.rb".to_string(),
            "features/**/*.rb".to_string(),
        ]
    }

    pub(crate) fn default_ignore() -> Vec<String> {
        vec![
            "vendor/".to_string(),
            "tmp/".to_string(),
            "log/".to_string(),
            "coverage/".to_string(),
        ]
    }
}
```

**Verification:**
- [ ] `cargo check` passes
- [ ] Config parses from TOML with `[ruby]` section
- [ ] Defaults are applied when section is empty

---

### Phase 482: Ruby Adapter Core

Implement the RubyAdapter struct and Adapter trait.

**Files to modify:**
- `crates/cli/src/adapter/mod.rs` - Add `pub mod ruby` and exports
- `crates/cli/src/adapter/ruby/mod.rs` - New file
- `crates/cli/src/adapter/patterns.rs` - Add `LanguageDefaults` impl

**RubyAdapter implementation:**

```rust
const RUBY_ESCAPE_PATTERNS: &[EscapePattern] = &[
    EscapePattern {
        name: "binding_pry",
        pattern: r"binding\.pry",
        action: EscapeAction::Forbid,
        comment: None,
        advice: "Remove debugger statement before committing.",
    },
    EscapePattern {
        name: "byebug",
        pattern: r"\bbyebug\b",
        action: EscapeAction::Forbid,
        comment: None,
        advice: "Remove debugger statement before committing.",
    },
    EscapePattern {
        name: "debugger",
        pattern: r"\bdebugger\b",
        action: EscapeAction::Forbid,
        comment: None,
        advice: "Remove debugger statement before committing.",
    },
    EscapePattern {
        name: "eval",
        pattern: r"\beval\s*\(",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining why eval is necessary.",
    },
    EscapePattern {
        name: "instance_eval",
        pattern: r"\.instance_eval\b",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining the DSL or metaprogramming use case.",
    },
    EscapePattern {
        name: "class_eval",
        pattern: r"\.class_eval\b",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining the metaprogramming use case.",
    },
];

pub struct RubyAdapter {
    source_patterns: GlobSet,
    test_patterns: GlobSet,
    ignore_patterns: GlobSet,
}

impl Adapter for RubyAdapter {
    fn name(&self) -> &'static str { "ruby" }
    fn extensions(&self) -> &'static [&'static str] { &["rb", "rake"] }
    fn classify(&self, path: &Path) -> FileKind { ... }
    fn default_escapes(&self) -> &'static [EscapePattern] { RUBY_ESCAPE_PATTERNS }
}
```

**Key classification logic:**
- Test patterns take precedence (spec/, test/, features/)
- Ignore patterns exclude vendor/, tmp/, log/, coverage/
- Source patterns match remaining .rb/.rake files

**Verification:**
- [ ] `cargo test adapter::ruby` passes
- [ ] `FileKind::Test` for spec/**/*_spec.rb
- [ ] `FileKind::Test` for test/**/*_test.rb
- [ ] `FileKind::Test` for features/**/*.rb
- [ ] `FileKind::Other` for vendor/**/*.rb
- [ ] `FileKind::Source` for lib/**/*.rb

---

### Phase 483: Ruby Detection

Add Ruby project detection and adapter registration.

**Files to modify:**
- `crates/cli/src/adapter/mod.rs` - Detection and registry

**Detection logic:**

```rust
pub enum ProjectLanguage {
    Rust,
    Go,
    JavaScript,
    Ruby,      // New variant
    Shell,
    Generic,
}

pub fn detect_language(root: &Path) -> ProjectLanguage {
    // ... existing checks ...

    // Ruby detection (before Shell check)
    if has_ruby_markers(root) {
        return ProjectLanguage::Ruby;
    }

    // ... rest ...
}

fn has_ruby_markers(root: &Path) -> bool {
    root.join("Gemfile").exists()
        || has_gemspec(root)
        || root.join("config.ru").exists()
        || root.join("config/application.rb").exists()
}

fn has_gemspec(root: &Path) -> bool {
    root.read_dir()
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|entry| {
                entry.path().extension().and_then(|e| e.to_str()) == Some("gemspec")
            })
        })
        .unwrap_or(false)
}
```

**Registry integration:**

```rust
impl AdapterRegistry {
    pub fn for_project(root: &Path) -> Self {
        // ...
        ProjectLanguage::Ruby => {
            registry.register(Arc::new(RubyAdapter::new()));
        }
        // ...
    }

    pub fn for_project_with_config(root: &Path, config: &Config) -> Self {
        // ...
        ProjectLanguage::Ruby => {
            let patterns = resolve_ruby_patterns(config, &fallback_test_patterns);
            registry.register(Arc::new(RubyAdapter::with_patterns(patterns)));
        }
        // ...
    }
}
```

**Remove `#[ignore]` from:**
- `auto_detected_when_gemfile_present`
- `auto_detected_when_gemspec_present`
- `auto_detected_when_config_ru_present`
- `auto_detected_when_rails_config_present`
- `default_source_pattern_matches_rb_files`
- `default_source_pattern_matches_rake_files`
- `default_test_pattern_matches_spec_files`
- `default_test_pattern_matches_test_files`
- `default_test_pattern_matches_features`
- `default_ignores_vendor_directory`
- `default_ignores_tmp_directory`
- `default_ignores_log_directory`
- `default_ignores_coverage_directory`

**Verification:**
- [ ] Detection tests in `tests/specs/adapters/ruby.rs` pass
- [ ] Ruby projects detected correctly
- [ ] Pattern resolution hierarchy works

---

### Phase 484: Ruby Comment Style

Add Ruby comment style to common suppress utilities.

**Files to modify:**
- `crates/cli/src/adapter/common/suppress.rs` - Add RUBY constant

**Ruby comment style:**

```rust
impl CommentStyle {
    // ... existing ...

    /// Ruby comment style: `#` prefix, rubocop/standard directives.
    pub const RUBY: Self = Self {
        prefix: "#",
        directive_patterns: &["rubocop:", "standard:", "!"],
    };
}
```

**Verification:**
- [ ] `cargo check` passes
- [ ] RUBY constant available for suppress parsing

---

### Phase 485: Ruby Escape Checking

Enable escape pattern checking in the escapes check for Ruby files.

**Files to review/modify:**
- `crates/cli/src/checks/escapes/mod.rs` - Ensure Ruby escapes are checked

The escape patterns are already defined in Phase 482. This phase ensures:
- Ruby escape patterns are picked up by the escapes check
- `in_tests` behavior works (debuggers still fail, metaprogramming allowed)

**Remove `#[ignore]` from:**
- `eval_without_metaprogramming_comment_fails`
- `eval_with_metaprogramming_comment_passes`
- `instance_eval_without_metaprogramming_comment_fails`
- `class_eval_without_metaprogramming_comment_fails`
- `metaprogramming_in_test_code_allowed`
- `binding_pry_forbidden_in_source`
- `byebug_forbidden_in_source`
- `debugger_forbidden_in_source`
- `debugger_forbidden_even_in_test`
- `debugger_allowed_in_test_when_configured`

**Verification:**
- [ ] Escape tests pass
- [ ] Debuggers fail in source code
- [ ] Metaprogramming requires comment in source
- [ ] Metaprogramming allowed in test code

---

### Phase 486: Ruby Suppress Parsing

Implement RuboCop/Standard suppress directive parsing.

**Files to create:**
- `crates/cli/src/adapter/ruby/suppress.rs`
- `crates/cli/src/adapter/ruby/suppress_tests.rs`

**RubySuppress structure:**

```rust
#[derive(Debug, Clone)]
pub struct RubySuppress {
    /// Line number (0-indexed).
    pub line: usize,
    /// Directive type: "rubocop" or "standard"
    pub kind: RubySuppressKind,
    /// Cop codes being suppressed (e.g., ["Style/StringLiterals"]).
    pub codes: Vec<String>,
    /// Whether a justification comment was found.
    pub has_comment: bool,
    /// The comment text if found.
    pub comment_text: Option<String>,
    /// Whether this is a "todo" directive.
    pub is_todo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RubySuppressKind {
    Rubocop,
    Standard,
}
```

**Parsing patterns:**

```ruby
# rubocop:disable Style/StringLiterals
# rubocop:disable Style/A, Style/B
# rubocop:todo Metrics/MethodLength
# standard:disable Style/StringLiterals
x = foo() # rubocop:disable Lint/UselessAssignment  # inline
```

**Implementation approach:**

```rust
pub fn parse_ruby_suppresses(
    content: &str,
    comment_pattern: Option<&str>,
) -> Vec<RubySuppress> {
    let mut suppresses = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Check for rubocop: or standard: directives
        if let Some(suppress) = parse_rubocop_directive(line, line_idx, &lines, comment_pattern) {
            suppresses.push(suppress);
        }
    }

    suppresses
}

fn parse_rubocop_directive(
    line: &str,
    line_idx: usize,
    lines: &[&str],
    comment_pattern: Option<&str>,
) -> Option<RubySuppress> {
    // Match: # rubocop:disable/todo Cop/Name
    // Match: # standard:disable Cop/Name
    // Handle inline: code # rubocop:disable Cop
    // ...
}
```

**Remove `#[ignore]` from:**
- `rubocop_disable_without_comment_fails`
- `rubocop_disable_with_comment_passes`
- `rubocop_disable_multiple_cops_detected`
- `rubocop_todo_detected`
- `standard_disable_detected`
- `rubocop_disable_inline_detected`
- `rubocop_disable_in_test_allowed`

**Verification:**
- [ ] Suppress parsing tests pass
- [ ] Single cop parsing works
- [ ] Multiple cops parsing works
- [ ] Todo directive detected
- [ ] Standard directive detected
- [ ] Inline directive detected
- [ ] Justification comment detection works

---

### Phase 487: Ruby Policy

Implement lint config policy checking.

**Files to create:**
- `crates/cli/src/adapter/ruby/policy.rs`
- `crates/cli/src/adapter/ruby/policy_tests.rs`

**Policy implementation:**

```rust
// Re-export from common
pub use crate::adapter::common::policy::PolicyCheckResult;

/// Check Ruby lint policy against changed files.
pub fn check_lint_policy(
    changed_files: &[&Path],
    policy: &RubyPolicyConfig,
    classify: impl Fn(&Path) -> FileKind,
) -> PolicyCheckResult {
    crate::adapter::common::policy::check_lint_policy(changed_files, policy, classify)
}
```

**RubyPolicyConfig (from Phase 481):**

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RubyPolicyConfig {
    #[serde(default)]
    pub check: Option<CheckLevel>,

    #[serde(default)]
    pub lint_changes: LintChangesPolicy,

    #[serde(default = "RubyPolicyConfig::default_lint_config")]
    pub lint_config: Vec<String>,
}

impl RubyPolicyConfig {
    pub(crate) fn default_lint_config() -> Vec<String> {
        vec![
            ".rubocop.yml".to_string(),
            ".rubocop_todo.yml".to_string(),
            ".standard.yml".to_string(),
        ]
    }
}

impl PolicyConfig for RubyPolicyConfig {
    fn lint_changes(&self) -> LintChangesPolicy { self.lint_changes }
    fn lint_config(&self) -> &[String] { &self.lint_config }
}
```

**Remove `#[ignore]` from:**
- `lint_config_changes_with_source_fails_standalone_policy`
- `lint_config_standalone_passes`

**Verification:**
- [ ] Policy tests pass
- [ ] Standalone policy violation detected
- [ ] Lint-only changes pass

## Key Implementation Details

### Ignore Pattern Handling

Unlike other adapters, Ruby has explicit ignore patterns (vendor/, tmp/, log/, coverage/). These need to be checked in `classify()`:

```rust
fn classify(&self, path: &Path) -> FileKind {
    // Check ignore patterns first
    let path_str = path.to_string_lossy();
    if self.ignore_patterns.is_match(path)
        || path_str.starts_with("vendor/")
        || path_str.starts_with("tmp/")
        || path_str.starts_with("log/")
        || path_str.starts_with("coverage/")
    {
        return FileKind::Other;
    }

    // Then test patterns
    if self.test_patterns.is_match(path) {
        return FileKind::Test;
    }

    // Finally source patterns
    if self.source_patterns.is_match(path) {
        return FileKind::Source;
    }

    FileKind::Other
}
```

### RuboCop Directive Regex

```rust
// Main pattern: # rubocop:disable/todo Cop/Name, Another/Cop
const RUBOCOP_DIRECTIVE: &str = r"#\s*(rubocop|standard):(disable|todo)\s+(.+)";

// Inline pattern: code # rubocop:disable Cop
// Need to handle comment at end of line
```

### Test Code Escape Behavior

For debuggers, the default `in_tests = "allow"` in the spec actually means we check `[check.escapes.patterns].in_tests` from config. The default escape patterns in the adapter use `Forbid`, but the config can override with `in_tests = "allow"`.

## Verification Plan

### Unit Tests

Each module has a `*_tests.rs` sibling:
- `ruby/mod_tests.rs` - Classification tests
- `ruby/suppress_tests.rs` - Directive parsing tests
- `ruby/policy_tests.rs` - Policy checking tests

### Behavioral Specs

All specs in `tests/specs/adapters/ruby.rs` should pass after removing `#[ignore]`:

```bash
# Run Ruby adapter specs only
cargo test --test specs ruby

# Show which specs are still ignored
cargo test --test specs ruby -- --ignored
```

### Manual Testing

Create a test Ruby project:

```bash
mkdir /tmp/ruby-test && cd /tmp/ruby-test
echo "source 'https://rubygems.org'" > Gemfile
mkdir -p lib spec
echo "class App; binding.pry; end" > lib/app.rb
echo "RSpec.describe App do; end" > spec/app_spec.rb
quench check
# Should fail on binding.pry
```

### Full Test Suite

```bash
make check
```

This runs:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --all`
- `cargo audit`
- `cargo deny check`
