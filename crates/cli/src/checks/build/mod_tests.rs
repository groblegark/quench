// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use crate::adapter::ProjectLanguage;
use tempfile::TempDir;

#[test]
fn get_rust_targets_from_cargo_toml() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "myapp"
version = "0.1.0"

[[bin]]
name = "myapp"
path = "src/main.rs"
"#,
    )
    .unwrap();

    let targets = get_rust_targets(root);
    assert_eq!(targets, vec!["myapp"]);
}

#[test]
fn get_rust_targets_default_binary() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "myapp"
version = "0.1.0"
"#,
    )
    .unwrap();

    let targets = get_rust_targets(root);
    assert_eq!(targets, vec!["myapp"]);
}

#[test]
fn get_go_targets_from_go_mod() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    std::fs::write(root.join("go.mod"), "module github.com/example/myapp\n").unwrap();

    let targets = get_go_targets(root);
    assert_eq!(targets, vec!["myapp"]);
}

#[test]
fn measure_binary_size_rust() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create fake binary
    let release_dir = root.join("target/release");
    std::fs::create_dir_all(&release_dir).unwrap();
    std::fs::write(release_dir.join("myapp"), vec![0u8; 1024]).unwrap();

    let size = measure_binary_size(root, "myapp", ProjectLanguage::Rust);
    assert_eq!(size, Some(1024));
}

#[test]
fn measure_binary_size_missing() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let size = measure_binary_size(root, "nonexistent", ProjectLanguage::Rust);
    assert_eq!(size, None);
}

#[test]
fn build_metrics_to_json() {
    let mut metrics = BuildMetrics::default();
    metrics.sizes.insert("myapp".to_string(), 1_000_000);
    metrics.time_cold = Some(Duration::from_secs(10));
    metrics.time_hot = Some(Duration::from_secs(2));

    let json = metrics.to_json();

    assert_eq!(json["size"]["myapp"], 1_000_000);
    assert_eq!(json["time"]["cold"], 10.0);
    assert_eq!(json["time"]["hot"], 2.0);
}
