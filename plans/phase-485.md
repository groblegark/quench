# Phase 485: Ruby Adapter - Escapes

## Overview

Add default Ruby escape patterns to quench. This phase implements the escapes portion of the Ruby language adapter, covering:

- **Debugger statements** (`binding.pry`, `byebug`, `debugger`) - forbidden in source code
- **Metaprogramming escapes** (`eval(`, `instance_eval`, `class_eval`) - require `# METAPROGRAMMING:` comment
- Test code exemption for all Ruby escapes (test code is never checked for violations)

## Project Structure

```
crates/cli/src/
├── adapter/
│   ├── mod.rs              # Add Ruby to ProjectLanguage enum
│   ├── ruby/               # NEW: Ruby adapter module
│   │   └── mod.rs          # RubyAdapter with default escapes
│   └── ruby_tests.rs       # NEW: Unit tests for Ruby adapter
├── checks/
│   └── escapes/
│       └── patterns.rs     # Add Ruby to get_adapter_escape_patterns()
└── config/
    └── mod.rs              # Add RubyConfig (minimal for now)

tests/fixtures/
└── ruby-gem/               # NEW: Test fixture (if not exists from Phase 484)
    ├── Gemfile
    ├── lib/
    │   └── example.rb
    └── spec/
        └── example_spec.rb
```

## Dependencies

- Phase 205-220: Escapes check framework (completed)
- Phase 201: Generic Language Adapter (completed)
- Phase 484: Ruby Test Fixture (may run in parallel)

No external crate dependencies required.

## Implementation Phases

### Phase 1: Add Ruby to Language Detection

Update `crates/cli/src/adapter/mod.rs`:

1. Add `Ruby` variant to `ProjectLanguage` enum
2. Update `detect_language()` to detect Ruby projects:
   - `Gemfile`
   - `*.gemspec`
   - `config.ru`
   - `config/application.rb`

```rust
// In detect_language():
if root.join("Gemfile").exists()
    || glob::has_gemspec(root)
    || root.join("config.ru").exists()
    || root.join("config/application.rb").exists()
{
    return ProjectLanguage::Ruby;
}
```

Detection order: Ruby should come before Shell (both may have scripts, but Ruby markers are more specific).

### Phase 2: Create Ruby Adapter Module

Create `crates/cli/src/adapter/ruby/mod.rs`:

```rust
//! Ruby language adapter.
//!
//! Provides Ruby-specific behavior for checks:
//! - File classification (source vs test)
//! - Default patterns for Ruby projects
//! - Default escape patterns (debuggers, metaprogramming)
//!
//! See docs/specs/langs/ruby.md for specification.

use std::path::Path;
use globset::GlobSet;

use super::glob::build_glob_set;
use super::{Adapter, EscapeAction, EscapePattern, FileKind};

/// Default escape patterns for Ruby.
///
/// Debugger statements are forbidden (common source of CI failures).
/// Metaprogramming requires METAPROGRAMMING comment to document intent.
const RUBY_ESCAPE_PATTERNS: &[EscapePattern] = &[
    // Debugger statements - forbidden in source
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
    // Metaprogramming - require comment
    EscapePattern {
        name: "eval",
        pattern: r"\beval\s*\(",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining why eval is necessary.",
    },
    EscapePattern {
        name: "instance_eval",
        pattern: r"\binstance_eval\b",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining the DSL or metaprogramming use case.",
    },
    EscapePattern {
        name: "class_eval",
        pattern: r"\bclass_eval\b",
        action: EscapeAction::Comment,
        comment: Some("# METAPROGRAMMING:"),
        advice: "Add a # METAPROGRAMMING: comment explaining the metaprogramming use case.",
    },
];

/// Ruby language adapter.
pub struct RubyAdapter {
    source_patterns: GlobSet,
    test_patterns: GlobSet,
    ignore_patterns: GlobSet,
}

impl RubyAdapter {
    pub fn new() -> Self {
        Self {
            source_patterns: build_glob_set(&[
                "**/*.rb".to_string(),
                "**/*.rake".to_string(),
                "Rakefile".to_string(),
                "Gemfile".to_string(),
            ]),
            test_patterns: build_glob_set(&[
                "spec/**/*_spec.rb".to_string(),
                "test/**/*_test.rb".to_string(),
                "test/**/test_*.rb".to_string(),
                "features/**/*.rb".to_string(),
            ]),
            ignore_patterns: build_glob_set(&[
                "vendor/**".to_string(),
                "tmp/**".to_string(),
                "log/**".to_string(),
                "coverage/**".to_string(),
            ]),
        }
    }

    pub fn with_patterns(patterns: super::ResolvedPatterns) -> Self {
        Self {
            source_patterns: build_glob_set(&patterns.source),
            test_patterns: build_glob_set(&patterns.test),
            ignore_patterns: build_glob_set(&patterns.ignore),
        }
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        self.ignore_patterns.is_match(path)
    }
}

impl Default for RubyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for RubyAdapter {
    fn name(&self) -> &'static str {
        "ruby"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rb", "rake"]
    }

    fn classify(&self, path: &Path) -> FileKind {
        if self.should_ignore(path) {
            return FileKind::Other;
        }

        if self.test_patterns.is_match(path) {
            return FileKind::Test;
        }

        if self.source_patterns.is_match(path) {
            return FileKind::Source;
        }

        FileKind::Other
    }

    fn default_escapes(&self) -> &'static [EscapePattern] {
        RUBY_ESCAPE_PATTERNS
    }
}
```

### Phase 3: Wire Up Ruby Adapter

Update `crates/cli/src/adapter/mod.rs`:

1. Add `pub mod ruby;` module declaration
2. Export `RubyAdapter` from the module
3. Update `AdapterRegistry::for_project()` to register `RubyAdapter` when Ruby is detected
4. Update `AdapterRegistry::for_project_with_config()` similarly

Update `crates/cli/src/checks/escapes/patterns.rs`:

```rust
// In get_adapter_escape_patterns():
ProjectLanguage::Ruby => {
    let ruby_adapter = RubyAdapter::new();
    patterns.extend(convert_adapter_patterns(ruby_adapter.default_escapes()));
}
```

### Phase 4: Add Minimal Ruby Config

Add to `crates/cli/src/config/mod.rs`:

```rust
mod ruby;
pub use ruby::{RubyConfig};

// In Config struct:
#[serde(default)]
pub ruby: RubyConfig,
```

Create `crates/cli/src/config/ruby.rs`:

```rust
//! Ruby-specific configuration.

use serde::Deserialize;

/// Ruby language-specific configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RubyConfig {
    /// Source file patterns.
    #[serde(default)]
    pub source: Vec<String>,

    /// Test file patterns.
    #[serde(default)]
    pub tests: Vec<String>,

    /// Ignore patterns.
    #[serde(default)]
    pub ignore: Vec<String>,
}
```

### Phase 5: Unit Tests

Create `crates/cli/src/adapter/ruby_tests.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use std::path::Path;

#[test]
fn classify_source_file() {
    let adapter = RubyAdapter::new();
    assert_eq!(adapter.classify(Path::new("lib/foo.rb")), FileKind::Source);
    assert_eq!(adapter.classify(Path::new("app/models/user.rb")), FileKind::Source);
}

#[test]
fn classify_test_file() {
    let adapter = RubyAdapter::new();
    assert_eq!(adapter.classify(Path::new("spec/foo_spec.rb")), FileKind::Test);
    assert_eq!(adapter.classify(Path::new("test/foo_test.rb")), FileKind::Test);
    assert_eq!(adapter.classify(Path::new("features/step_definitions/foo.rb")), FileKind::Test);
}

#[test]
fn classify_ignored_file() {
    let adapter = RubyAdapter::new();
    assert_eq!(adapter.classify(Path::new("vendor/bundle/foo.rb")), FileKind::Other);
    assert_eq!(adapter.classify(Path::new("tmp/cache/foo.rb")), FileKind::Other);
}

#[test]
fn default_escapes_contains_debuggers() {
    let adapter = RubyAdapter::new();
    let escapes = adapter.default_escapes();

    let names: Vec<_> = escapes.iter().map(|e| e.name).collect();
    assert!(names.contains(&"binding_pry"));
    assert!(names.contains(&"byebug"));
    assert!(names.contains(&"debugger"));
}

#[test]
fn default_escapes_contains_metaprogramming() {
    let adapter = RubyAdapter::new();
    let escapes = adapter.default_escapes();

    let names: Vec<_> = escapes.iter().map(|e| e.name).collect();
    assert!(names.contains(&"eval"));
    assert!(names.contains(&"instance_eval"));
    assert!(names.contains(&"class_eval"));
}

#[test]
fn debugger_patterns_are_forbidden() {
    let adapter = RubyAdapter::new();
    let escapes = adapter.default_escapes();

    for escape in escapes.iter().filter(|e|
        e.name == "binding_pry" || e.name == "byebug" || e.name == "debugger"
    ) {
        assert_eq!(escape.action, EscapeAction::Forbid);
    }
}

#[test]
fn metaprogramming_patterns_require_comment() {
    let adapter = RubyAdapter::new();
    let escapes = adapter.default_escapes();

    for escape in escapes.iter().filter(|e|
        e.name == "eval" || e.name == "instance_eval" || e.name == "class_eval"
    ) {
        assert_eq!(escape.action, EscapeAction::Comment);
        assert_eq!(escape.comment, Some("# METAPROGRAMMING:"));
    }
}
```

### Phase 6: Integration Test Fixture

Create minimal Ruby fixture (if not created in Phase 484):

```
tests/fixtures/ruby-gem/
├── Gemfile
├── lib/
│   └── example.rb
└── spec/
    └── example_spec.rb
```

**Gemfile:**
```ruby
source 'https://rubygems.org'
gemspec
```

**lib/example.rb:**
```ruby
# Example module for testing
module Example
  def self.greet(name)
    "Hello, #{name}!"
  end
end
```

**spec/example_spec.rb:**
```ruby
require 'example'

RSpec.describe Example do
  describe '.greet' do
    it 'returns greeting' do
      expect(Example.greet('World')).to eq('Hello, World!')
    end
  end
end
```

## Key Implementation Details

### Pattern Design

1. **Word boundaries**: Use `\b` for patterns like `byebug` and `debugger` to avoid matching substrings (e.g., "debugger_mode")

2. **Method call detection**: `eval\s*\(` matches `eval(` with optional whitespace, avoiding matches like `evaluation`

3. **Comment search direction**: The escapes check searches upward for `# METAPROGRAMMING:` comments, following the same pattern as Rust's `// SAFETY:` comments

### Test Code Exemption

The escapes check already skips violations for test code (see `crates/cli/src/checks/escapes/mod.rs:241-244`). This means:

- Metaprogramming in test files: Allowed without comment
- Debuggers in test files: Also allowed (tracked in metrics only)

This matches the Ruby spec: "Metaprogramming: Allowed in tests without comment"

### Language Detection Priority

Ruby detection should come before Shell detection in `detect_language()` because:
- Ruby projects often have shell scripts
- A project with `Gemfile` is definitely Ruby
- Shell detection uses more generic markers (*.sh files)

## Verification Plan

### Unit Tests

1. File classification tests (source/test/ignored)
2. Escape pattern content tests (names, actions, comments)
3. Pattern matching tests (verify regex works correctly)

### Integration Tests

Run `quench check` on fixtures to verify:

```bash
# Should pass - no violations in clean Ruby code
cargo run -- check tests/fixtures/ruby-gem

# Should fail - violations detected
cargo run -- check tests/fixtures/violations  # (add Ruby violations)
```

### Manual Verification Checklist

- [ ] `quench check` on Ruby gem fixture produces no errors
- [ ] Ruby file with `binding.pry` in source fails
- [ ] Ruby file with `binding.pry` in spec/ passes (test exemption)
- [ ] Ruby file with `eval(` without METAPROGRAMMING comment fails
- [ ] Ruby file with `# METAPROGRAMMING: DSL builder\neval(code)` passes
- [ ] Ruby detection works: Gemfile, *.gemspec, config.ru, config/application.rb

### Run CI Checks

```bash
make check  # cargo fmt, clippy, test, build, audit, deny
```

## Notes

- This phase focuses only on escapes. Ruby suppress (`# rubocop:disable`) will be added in Phase 486.
- Ruby policy (`lint_changes = "standalone"`) will be added in Phase 487.
- The `in_tests` config field already exists for user customization if they want different test behavior.
