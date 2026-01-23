//! Escapes (escape hatches) check.
//!
//! Detects patterns that bypass type safety or error handling.
//! See docs/specs/checks/escape-hatches.md.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;

use crate::adapter::{FileKind, GenericAdapter};
use crate::check::{Check, CheckContext, CheckResult, Violation};
use crate::config::{CheckLevel, EscapeAction, EscapesConfig};
use crate::pattern::{CompiledPattern, PatternError};

/// Compiled escape pattern ready for matching.
struct CompiledEscapePattern {
    name: String,
    matcher: CompiledPattern,
    action: EscapeAction,
    advice: String,
    /// Required comment pattern for action = comment.
    comment: Option<String>,
    /// Count threshold for action = count (default: 0).
    threshold: usize,
}

/// Track counts per pattern.
struct PatternCounts {
    counts: HashMap<String, usize>,
}

impl PatternCounts {
    fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    fn increment(&mut self, pattern_name: &str) -> usize {
        let count = self.counts.entry(pattern_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    fn get(&self, pattern_name: &str) -> usize {
        self.counts.get(pattern_name).copied().unwrap_or(0)
    }
}

/// The escapes check detects escape hatch patterns.
pub struct EscapesCheck;

impl Check for EscapesCheck {
    fn name(&self) -> &'static str {
        "escapes"
    }

    fn description(&self) -> &'static str {
        "Escape hatch detection"
    }

    fn run(&self, ctx: &CheckContext) -> CheckResult {
        let config = &ctx.config.check.escapes;

        if config.check == CheckLevel::Off {
            return CheckResult::passed(self.name());
        }

        // No patterns configured = nothing to check
        if config.patterns.is_empty() {
            return CheckResult::passed(self.name());
        }

        // Compile patterns once
        let patterns = match compile_patterns(config) {
            Ok(p) => p,
            Err(e) => return CheckResult::skipped(self.name(), e.to_string()),
        };

        // Use project test patterns or defaults
        let test_patterns = if ctx.config.project.tests.is_empty() {
            default_test_patterns()
        } else {
            ctx.config.project.tests.clone()
        };

        let mut violations = Vec::new();
        let mut source_counts = PatternCounts::new();
        let mut limit_reached = false;

        for file in ctx.files {
            if limit_reached {
                break;
            }

            // Skip non-source files (configs, docs, etc.)
            if !is_source_file(&file.path) {
                continue;
            }

            // Read file content
            let content = match std::fs::read_to_string(&file.path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative = file.path.strip_prefix(ctx.root).unwrap_or(&file.path);

            // Classify file as source or test
            let is_test = classify_file(&file.path, ctx.root, &test_patterns) == FileKind::Test;

            // Find matches for each pattern
            for pattern in &patterns {
                let matches = pattern.matcher.find_all_with_lines(&content);

                for m in matches {
                    // Test code: count but don't report violations
                    // (Metrics will be added in Phase 220)
                    if is_test {
                        continue;
                    }

                    // Source code: apply action logic
                    match pattern.action {
                        EscapeAction::Count => {
                            // Just count - threshold check happens after all files
                            source_counts.increment(&pattern.name);
                        }
                        EscapeAction::Comment => {
                            let comment_pattern =
                                pattern.comment.as_deref().unwrap_or("// JUSTIFIED:");

                            if !has_justification_comment(&content, m.line, comment_pattern) {
                                let advice =
                                    format_comment_advice(&pattern.advice, comment_pattern);
                                if let Some(v) = try_create_violation(
                                    ctx,
                                    relative,
                                    m.line,
                                    "missing_comment",
                                    &advice,
                                    &pattern.name,
                                ) {
                                    violations.push(v);
                                } else {
                                    limit_reached = true;
                                    break;
                                }
                            }
                        }
                        EscapeAction::Forbid => {
                            if let Some(v) = try_create_violation(
                                ctx,
                                relative,
                                m.line,
                                "forbidden",
                                &pattern.advice,
                                &pattern.name,
                            ) {
                                violations.push(v);
                            } else {
                                limit_reached = true;
                                break;
                            }
                        }
                    }
                }

                if limit_reached {
                    break;
                }
            }
        }

        // Check count thresholds after scanning all files
        for pattern in &patterns {
            if pattern.action == EscapeAction::Count {
                let count = source_counts.get(&pattern.name);
                if count > pattern.threshold
                    && let Some(v) = create_threshold_violation(
                        ctx,
                        &pattern.name,
                        count,
                        pattern.threshold,
                        &pattern.advice,
                    )
                {
                    violations.push(v);
                }
            }
        }

        if violations.is_empty() {
            CheckResult::passed(self.name())
        } else {
            CheckResult::failed(self.name(), violations)
        }
    }

    fn default_enabled(&self) -> bool {
        true
    }
}

fn compile_patterns(config: &EscapesConfig) -> Result<Vec<CompiledEscapePattern>, PatternError> {
    config
        .patterns
        .iter()
        .map(|p| {
            let matcher = CompiledPattern::compile(&p.pattern)?;
            let advice = p
                .advice
                .clone()
                .unwrap_or_else(|| default_advice(&p.action));
            Ok(CompiledEscapePattern {
                name: p.name.clone(),
                matcher,
                action: p.action,
                advice,
                comment: p.comment.clone(),
                threshold: p.threshold,
            })
        })
        .collect()
}

fn default_advice(action: &EscapeAction) -> String {
    match action {
        EscapeAction::Forbid => "Remove this escape hatch from production code.".to_string(),
        EscapeAction::Comment => "Add a justification comment.".to_string(),
        EscapeAction::Count => "Reduce escape hatch usage.".to_string(),
    }
}

fn try_create_violation(
    ctx: &CheckContext,
    path: &std::path::Path,
    line: u32,
    violation_type: &str,
    advice: &str,
    pattern_name: &str,
) -> Option<Violation> {
    let current = ctx.violation_count.fetch_add(1, Ordering::SeqCst);
    if let Some(limit) = ctx.limit
        && current >= limit
    {
        return None;
    }

    Some(Violation::file(path, line, violation_type, advice).with_pattern(pattern_name))
}

fn create_threshold_violation(
    ctx: &CheckContext,
    pattern_name: &str,
    count: usize,
    threshold: usize,
    advice: &str,
) -> Option<Violation> {
    let current = ctx.violation_count.fetch_add(1, Ordering::SeqCst);
    if let Some(limit) = ctx.limit
        && current >= limit
    {
        return None;
    }

    Some(Violation {
        file: None,
        line: None,
        violation_type: "threshold_exceeded".to_string(),
        advice: advice.to_string(),
        value: Some(count as i64),
        threshold: Some(threshold as i64),
        pattern: Some(pattern_name.to_string()),
    })
}

fn format_comment_advice(custom_advice: &str, comment_pattern: &str) -> String {
    if custom_advice.is_empty() || custom_advice == default_advice(&EscapeAction::Comment) {
        format!(
            "Add a {} comment explaining why this is necessary.",
            comment_pattern
        )
    } else {
        custom_advice.to_string()
    }
}

/// Search upward from a line for a required comment pattern.
///
/// Searches:
/// 1. Same line as the match
/// 2. Preceding lines, stopping at non-blank/non-comment lines
///
/// Returns true if comment pattern is found.
fn has_justification_comment(content: &str, match_line: u32, comment_pattern: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = (match_line - 1) as usize;

    // Check same line first
    if line_idx < lines.len() && lines[line_idx].contains(comment_pattern) {
        return true;
    }

    // Search upward through comments and blank lines
    if line_idx > 0 {
        for i in (0..line_idx).rev() {
            let line = lines[i].trim();

            // Check for comment pattern
            if line.contains(comment_pattern) {
                return true;
            }

            // Stop at non-blank, non-comment lines
            if !line.is_empty() && !is_comment_line(line) {
                break;
            }
        }
    }

    false
}

/// Check if a line is a comment line (language-agnostic heuristics).
fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("//")      // C-style
        || trimmed.starts_with('#')   // Shell/Python/Ruby
        || trimmed.starts_with("/*")  // C block comment start
        || trimmed.starts_with('*')   // C block comment continuation
        || trimmed.starts_with("--")  // SQL/Lua
        || trimmed.starts_with(";;") // Lisp
}

/// Classify file as source or test.
fn classify_file(path: &Path, root: &Path, test_patterns: &[String]) -> FileKind {
    use crate::adapter::Adapter;
    let adapter = GenericAdapter::new(&[], test_patterns);
    let relative = path.strip_prefix(root).unwrap_or(path);
    adapter.classify(relative)
}

/// Default test patterns for file classification.
fn default_test_patterns() -> Vec<String> {
    vec![
        "**/tests/**".to_string(),
        "**/test/**".to_string(),
        "**/*_test.*".to_string(),
        "**/*_tests.*".to_string(),
        "**/*.test.*".to_string(),
        "**/*.spec.*".to_string(),
    ]
}

/// Check if a file is a source code file (for escape pattern checking).
/// Excludes configuration files, documentation, and data files.
fn is_source_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        ext.as_str(),
        // Systems languages
        "rs" | "c" | "cpp" | "h" | "hpp" | "go"
        // JVM languages
        | "java" | "kt" | "scala"
        // Dynamic languages
        | "py" | "rb" | "php" | "lua" | "pl" | "pm" | "r"
        // JavaScript/TypeScript
        | "js" | "ts" | "jsx" | "tsx"
        // Apple platforms
        | "swift" | "m" | "mm"
        // .NET
        | "cs"
        // Shell scripts
        | "sh" | "bash" | "zsh"
        // Web
        | "html" | "css" | "vue" | "svelte"
        // Other
        | "sql" | "ex" | "exs" | "erl" | "clj" | "hs" | "ml"
    )
}

#[cfg(test)]
#[path = "escapes_tests.rs"]
mod tests;
