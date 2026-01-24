#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

// === Phase 1: Language tag extraction ===

#[test]
fn extract_language_tag_rust() {
    let content = "```rust\nfn main() {}\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
}

#[test]
fn extract_language_tag_with_attributes() {
    let content = "```rust,linenos\ncode\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
}

#[test]
fn extract_no_language_tag() {
    let content = "```\nplain block\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, None);
}

#[test]
fn extract_language_tag_normalizes_case() {
    let content = "```RUST\ncode\n```";
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks[0].language, Some("rust".to_string()));
}

// === Phase 2: Language-tagged blocks skipped ===

#[test]
fn bash_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["scripts/deploy.sh:23: error".to_string()],
        language: Some("bash".to_string()),
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn text_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["foo.rs".to_string(), "bar.rs".to_string()],
        language: Some("text".to_string()),
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn unlabeled_tree_still_detected() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "├── lib.rs".to_string(),
            "└── main.rs".to_string(),
        ],
        language: None,
    };
    assert!(looks_like_tree(&block));
}

// === Phase 3: Error output detection ===

#[test]
fn error_output_file_line() {
    assert!(looks_like_error_output("scripts/deploy.sh:23:"));
    assert!(looks_like_error_output("src/main.rs:45:12:"));
    assert!(looks_like_error_output("foo.go:100: undefined"));
}

#[test]
fn not_error_output() {
    assert!(!looks_like_error_output("foo.rs")); // no line number
    assert!(!looks_like_error_output("src/")); // directory
    assert!(!looks_like_error_output("Cargo.toml")); // no colon
    assert!(!looks_like_error_output("README")); // no extension
}

#[test]
fn error_output_in_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "scripts/deploy.sh:23: shellcheck_missing_comment:".to_string(),
            "  Lint suppression requires justification.".to_string(),
            "scripts/build.sh:45: shellcheck_missing_comment:".to_string(),
        ],
        language: None,
    };
    assert!(!looks_like_tree(&block));
}

// === Phase 4: Strengthened heuristics regression tests ===

#[test]
fn indentation_tree_still_detected() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "docs/".to_string(),
            "  README.md".to_string(),
            "  overview.md".to_string(),
        ],
        language: None,
    };
    assert!(looks_like_tree(&block));
}

// === Existing tests (updated with language field) ===

#[test]
fn extract_single_fenced_block() {
    let content = r#"# Header

```
foo/
  bar.rs
```

More text.
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].start_line, 4);
    assert_eq!(blocks[0].lines, vec!["foo/", "  bar.rs"]);
    assert_eq!(blocks[0].language, None);
}

#[test]
fn extract_multiple_fenced_blocks() {
    let content = r#"
```
block1
```

```
block2
```
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].lines, vec!["block1"]);
    assert_eq!(blocks[1].lines, vec!["block2"]);
}

#[test]
fn extract_unclosed_block_no_panic() {
    let content = r#"
```
unclosed block
no end fence
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 0); // Unclosed blocks are not included
}

#[test]
fn extract_empty_block() {
    let content = r#"
```
```
"#;
    let blocks = extract_fenced_blocks(content);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].lines.is_empty());
}

#[test]
fn parse_box_drawing_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "├── lib.rs".to_string(),
            "└── main.rs".to_string(),
        ],
        language: None,
    };
    let entries = parse_tree_block(&block);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "src");
    assert!(entries[0].is_dir);
    assert_eq!(entries[1].path, "src/lib.rs");
    assert!(!entries[1].is_dir);
    assert_eq!(entries[2].path, "src/main.rs");
    assert!(!entries[2].is_dir);
}

#[test]
fn parse_indentation_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "src/".to_string(),
            "  lib.rs".to_string(),
            "  main.rs".to_string(),
        ],
        language: None,
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "src/lib.rs"));
    assert!(entries.iter().any(|e| e.path == "src/main.rs"));
}

#[test]
fn strip_comments_from_entries() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["src/".to_string(), "├── lib.rs  # Main library".to_string()],
        language: None,
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "src/lib.rs"));
    assert!(!entries.iter().any(|e| e.path.contains('#')));
}

#[test]
fn nested_directories() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "docs/".to_string(),
            "├── specs/".to_string(),
            "│   ├── overview.md".to_string(),
            "│   └── config.md".to_string(),
            "└── README.md".to_string(),
        ],
        language: None,
    };
    let entries = parse_tree_block(&block);
    assert!(entries.iter().any(|e| e.path == "docs/specs/overview.md"));
    assert!(entries.iter().any(|e| e.path == "docs/specs/config.md"));
    assert!(entries.iter().any(|e| e.path == "docs/README.md"));
}

#[test]
fn looks_like_tree_detects_box_drawing() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["src/".to_string(), "├── file.rs".to_string()],
        language: None,
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn looks_like_tree_detects_paths() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["src/".to_string(), "  lib.rs".to_string()],
        language: None,
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn looks_like_tree_rejects_code() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "fn main() {".to_string(),
            "    println!(\"hi\");".to_string(),
        ],
        language: None,
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn empty_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![],
        language: None,
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn deeply_nested_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![
            "crates/".to_string(),
            "├── cli/".to_string(),
            "│   └── src/".to_string(),
            "│       └── checks/".to_string(),
            "│           └── docs/".to_string(),
            "│               └── toc.rs".to_string(),
        ],
        language: None,
    };
    let entries = parse_tree_block(&block);
    assert!(
        entries
            .iter()
            .any(|e| e.path == "crates/cli/src/checks/docs/toc.rs")
    );
}
