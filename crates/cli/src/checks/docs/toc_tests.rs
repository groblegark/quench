#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

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
    };
    assert!(looks_like_tree(&block));
}

#[test]
fn looks_like_tree_detects_paths() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec!["src/".to_string(), "  lib.rs".to_string()],
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
    };
    assert!(!looks_like_tree(&block));
}

#[test]
fn empty_block_not_tree() {
    let block = FencedBlock {
        start_line: 1,
        lines: vec![],
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
    };
    let entries = parse_tree_block(&block);
    assert!(
        entries
            .iter()
            .any(|e| e.path == "crates/cli/src/checks/docs/toc.rs")
    );
}
