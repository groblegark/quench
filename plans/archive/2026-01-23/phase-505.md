# Phase 505: Agents Check - File Detection

**Root Feature:** `quench-93a5`

## Overview

Implement the file detection foundation for the `agents` check, which validates AI agent context files (CLAUDE.md, .cursorrules, etc.). This phase focuses on:
- Recognizing agent files at different scopes
- Configurable file lists
- File existence checking (required/optional/forbid)
- Scope detection (root, package, module)

Later phases will add content validation, sync checking, and --fix support.

## Project Structure

```
crates/cli/src/
├── checks/
│   ├── agents/
│   │   ├── mod.rs           # AgentsCheck implementation
│   │   ├── config.rs        # AgentsConfig parsing
│   │   ├── config_tests.rs
│   │   ├── detection.rs     # File detection logic
│   │   └── detection_tests.rs
│   └── mod.rs               # Register AgentsCheck (replace stub)
├── config/
│   ├── mod.rs               # Add AgentsConfig to CheckConfig
│   └── parse.rs             # Add parse_agents_config()
```

## Dependencies

No new external dependencies. Uses existing:
- `serde` for config deserialization
- `glob` for `.cursor/rules/*.md[c]` pattern matching
- `serde_json` for metrics output

## Implementation Phases

### Phase 1: Configuration Schema (AgentsConfig)

Add configuration parsing for the agents check.

**Tasks:**
1. Create `crates/cli/src/checks/agents/config.rs` with:
   - `AgentsConfig` struct
   - `AgentsScopeConfig` for per-scope settings
   - `FileRequirement` enum (required/optional/forbid)
2. Add `parse_agents_config()` to `crates/cli/src/config/parse.rs`
3. Add `AgentsConfig` to `CheckConfig` in `crates/cli/src/config/mod.rs`

**Config structures:**
```rust
/// Configuration for the agents check.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AgentsConfig {
    /// Check level: error, warn, or off.
    #[serde(default)]
    pub check: CheckLevel,

    /// Agent files to check (default: all recognized).
    #[serde(default = "AgentsConfig::default_files")]
    pub files: Vec<String>,

    /// Files that must exist (root scope).
    #[serde(default)]
    pub required: Vec<String>,

    /// Files checked if present (root scope).
    #[serde(default)]
    pub optional: Vec<String>,

    /// Files that must not exist (root scope).
    #[serde(default)]
    pub forbid: Vec<String>,

    /// Root scope settings (overrides flat config).
    #[serde(default)]
    pub root: Option<AgentsScopeConfig>,

    /// Package scope settings.
    #[serde(default)]
    pub package: Option<AgentsScopeConfig>,

    /// Module scope settings.
    #[serde(default)]
    pub module: Option<AgentsScopeConfig>,
}

impl AgentsConfig {
    fn default_files() -> Vec<String> {
        vec![
            "CLAUDE.md".to_string(),
            "AGENTS.md".to_string(),
            ".cursorrules".to_string(),
            ".cursorignore".to_string(),
            ".cursor/rules/*.md".to_string(),
            ".cursor/rules/*.mdc".to_string(),
        ]
    }
}

/// Per-scope configuration for agent files.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AgentsScopeConfig {
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub forbid: Vec<String>,
    pub max_lines: Option<usize>,
    pub max_tokens: Option<usize>,
}
```

**Verification:**
```bash
cargo test config -- agents
```

### Phase 2: File Detection Logic

Implement the core file detection and scope classification.

**Tasks:**
1. Create `crates/cli/src/checks/agents/detection.rs` with:
   - `detect_agent_files()` - find agent files at a path
   - `classify_scope()` - determine if path is root/package/module
   - `match_agent_pattern()` - handle glob patterns for `.cursor/rules/*.md`
2. Create unit tests in `detection_tests.rs`

**Detection logic:**
```rust
/// Recognized agent file names.
pub const AGENT_FILES: &[&str] = &[
    "CLAUDE.md",
    "AGENTS.md",
    ".cursorrules",
    ".cursorignore",
];

/// Recognized agent file glob patterns.
pub const AGENT_PATTERNS: &[&str] = &[
    ".cursor/rules/*.md",
    ".cursor/rules/*.mdc",
];

/// Detected agent file with its scope.
#[derive(Debug)]
pub struct DetectedFile {
    pub path: PathBuf,
    pub scope: Scope,
    pub pattern: String,  // which config pattern matched
}

/// Scope at which an agent file was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Root,
    Package(String),  // package name
    Module,
}

/// Detect agent files under a directory.
pub fn detect_agent_files(
    root: &Path,
    packages: &[String],
    patterns: &[String],
) -> Vec<DetectedFile> {
    // 1. Check root for exact matches and glob patterns
    // 2. Check package directories
    // 3. Walk for module-level files (if configured)
}

/// Classify the scope of a file path.
pub fn classify_scope(
    file_path: &Path,
    root: &Path,
    packages: &[String],
) -> Scope {
    let relative = file_path.strip_prefix(root).unwrap_or(file_path);

    // Check if under a package directory
    for pkg_pattern in packages {
        // Handle wildcard patterns like "crates/*"
        if is_in_package(relative, pkg_pattern) {
            return Scope::Package(extract_package_name(relative, pkg_pattern));
        }
    }

    // Direct child of root = Root scope
    // Deeper nesting = Module scope
    if relative.components().count() == 1 {
        Scope::Root
    } else {
        Scope::Module
    }
}
```

**Verification:**
```bash
cargo test checks::agents::detection
```

### Phase 3: AgentsCheck Implementation

Implement the Check trait for the agents check, focusing on file existence validation.

**Tasks:**
1. Create `crates/cli/src/checks/agents/mod.rs` with `AgentsCheck`
2. Replace stub in `crates/cli/src/checks/mod.rs`
3. Implement file existence checking:
   - Required files must exist → `missing_file` violation
   - Forbidden files must not exist → `forbidden_file` violation
   - Optional files are checked if present (for later content validation)

**Check implementation:**
```rust
pub struct AgentsCheck;

impl Check for AgentsCheck {
    fn name(&self) -> &'static str {
        "agents"
    }

    fn description(&self) -> &'static str {
        "Agent file validation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let config = &ctx.config.check.agents;

        if config.check == CheckLevel::Off {
            return CheckResult::passed(self.name());
        }

        let packages = &ctx.config.workspace.packages;

        // Detect all agent files
        let detected = detect_agent_files(ctx.root, packages, &config.files);

        let mut violations = Vec::new();

        // Check required files exist (per scope)
        check_required_files(ctx, config, &detected, &mut violations);

        // Check forbidden files don't exist
        check_forbidden_files(ctx, config, &detected, &mut violations);

        // Build metrics
        let files_found: Vec<String> = detected
            .iter()
            .map(|f| f.path.strip_prefix(ctx.root)
                .unwrap_or(&f.path)
                .to_string_lossy()
                .to_string())
            .collect();

        let metrics = json!({
            "files_found": files_found,
            "files_missing": [], // populated by check_required_files
            "in_sync": true,     // placeholder for sync phase
        });

        let result = if violations.is_empty() {
            CheckResult::passed(self.name())
        } else {
            CheckResult::failed(self.name(), violations)
        };

        result.with_metrics(metrics)
    }

    fn default_enabled(&self) -> bool {
        true
    }
}
```

**Violation types (file detection subset):**
- `missing_file` - required file not found
- `forbidden_file` - forbidden file exists

**Verification:**
```bash
cargo test checks::agents
```

### Phase 4: Update Behavioral Specs

Update the existing behavioral specs to pass for file detection features and add any missing specs.

**Tasks:**
1. Remove `#[ignore]` from specs that should now pass:
   - `agents_detects_claude_md_at_project_root`
   - `agents_detects_cursorrules_at_project_root`
   - `agents_passes_on_valid_project`
   - `agents_missing_required_file_generates_violation`
   - `agents_json_includes_files_found_and_in_sync_metrics`
   - `agents_violation_type_is_valid`
2. Add new spec for forbidden file detection:
   - `agents_forbidden_file_generates_violation`
3. Create `tests/fixtures/agents/forbidden-file/` fixture

**Verification:**
```bash
cargo test --test specs agents
```

## Key Implementation Details

### File Pattern Matching

Agent files can be:
1. **Exact names**: `CLAUDE.md`, `.cursorrules`
2. **Glob patterns**: `.cursor/rules/*.md`, `.cursor/rules/*.mdc`

Use the existing `glob` crate for pattern matching:
```rust
fn match_pattern(pattern: &str, root: &Path) -> Vec<PathBuf> {
    if pattern.contains('*') {
        let full_pattern = root.join(pattern);
        glob::glob(&full_pattern.to_string_lossy())
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .collect()
    } else {
        let path = root.join(pattern);
        if path.exists() {
            vec![path]
        } else {
            vec![]
        }
    }
}
```

### Scope Resolution Priority

When a file could belong to multiple scopes, resolve with:
1. Root scope (direct children of project root)
2. Package scope (under configured workspace.packages)
3. Module scope (any other nested location)

```rust
// Example: crates/cli/CLAUDE.md with packages = ["crates/*"]
// → Scope::Package("cli")

// Example: src/parser/CLAUDE.md with no matching package
// → Scope::Module
```

### Default File List vs Configured

The `files` config option:
- Default: all recognized agent files
- When set: only check specified files

```toml
# Default behavior - checks all
[check.agents]

# Explicit list - only these files
[check.agents]
files = ["CLAUDE.md"]  # ignores .cursorrules even if present
```

### Required/Optional/Forbid Logic

Per-scope file requirements:
```rust
fn get_effective_requirements(config: &AgentsConfig, scope: Scope) -> ScopeRequirements {
    match scope {
        Scope::Root => {
            // Use root config if present, else flat config
            config.root.as_ref().map_or_else(
                || ScopeRequirements {
                    required: &config.required,
                    optional: &config.optional,
                    forbid: &config.forbid,
                },
                |root| ScopeRequirements {
                    required: &root.required,
                    optional: &root.optional,
                    forbid: &root.forbid,
                }
            )
        }
        Scope::Package(_) => config.package.as_ref().map(|p| /* ... */),
        Scope::Module => config.module.as_ref().map(|m| /* ... */),
    }
}
```

## Verification Plan

### Unit Tests

```bash
# Config parsing
cargo test config -- agents

# Detection logic
cargo test checks::agents::detection

# Full check
cargo test checks::agents
```

### Behavioral Specs

```bash
# Run agents specs
cargo test --test specs agents

# Show remaining ignored count
cargo test --test specs agents -- --ignored
```

### Full Validation

```bash
make check
```

### Acceptance Criteria

1. `check.agents` config section parses correctly
2. All recognized agent files are detected at root
3. Required/optional/forbid validation works at root scope
4. Package scope detection works with workspace.packages
5. JSON metrics include `files_found` and `files_missing`
6. All Phase 505 behavioral specs pass
7. `make check` passes

## Spec Status (After Implementation)

| Spec | Phase 505 Status |
|------|------------------|
| agents_detects_claude_md_at_project_root | ✅ Pass |
| agents_detects_cursorrules_at_project_root | ✅ Pass |
| agents_passes_on_valid_project | ✅ Pass |
| agents_missing_required_file_generates_violation | ✅ Pass |
| agents_out_of_sync_generates_violation | ⏳ Later |
| agents_missing_section_generates_violation_with_advice | ⏳ Later |
| agents_forbidden_section_generates_violation | ⏳ Later |
| agents_markdown_table_generates_violation | ⏳ Later |
| agents_file_over_max_lines_generates_violation | ⏳ Later |
| agents_file_over_max_tokens_generates_violation | ⏳ Later |
| agents_json_includes_files_found_and_in_sync_metrics | ✅ Pass |
| agents_violation_type_is_valid | ✅ Pass |
| agents_fix_syncs_files_from_sync_source | ⏳ Later |
