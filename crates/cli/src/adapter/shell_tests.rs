//! Unit tests for the Shell adapter.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use yare::parameterized;

use super::ShellAdapter;
use crate::adapter::{Adapter, FileKind};

// =============================================================================
// FILE CLASSIFICATION
// =============================================================================

#[parameterized(
    // Source files
    src_root_sh = { "build.sh", FileKind::Source },
    src_scripts_sh = { "scripts/deploy.sh", FileKind::Source },
    src_nested_sh = { "scripts/ci/build.sh", FileKind::Source },
    src_bash = { "scripts/setup.bash", FileKind::Source },
    src_bin_sh = { "bin/run.sh", FileKind::Source },
    // Test files
    test_bats = { "tests/integration.bats", FileKind::Test },
    test_dir_bats = { "test/unit.bats", FileKind::Test },
    test_nested_bats = { "tests/e2e/smoke.bats", FileKind::Test },
    test_suffix = { "build_test.sh", FileKind::Test },
    test_suffix_nested = { "scripts/build_test.sh", FileKind::Test },
    // Other files
    other_toml = { "quench.toml", FileKind::Other },
    other_md = { "README.md", FileKind::Other },
    other_rs = { "src/lib.rs", FileKind::Other },
)]
fn classify_path(path: &str, expected: FileKind) {
    let adapter = ShellAdapter::new();
    assert_eq!(
        adapter.classify(Path::new(path)),
        expected,
        "path {:?} should be {:?}",
        path,
        expected
    );
}

#[test]
fn name_returns_shell() {
    let adapter = ShellAdapter::new();
    assert_eq!(adapter.name(), "shell");
}

#[test]
fn extensions_include_sh_bash_bats() {
    let adapter = ShellAdapter::new();
    let exts = adapter.extensions();
    assert!(exts.contains(&"sh"), "should include sh");
    assert!(exts.contains(&"bash"), "should include bash");
    assert!(exts.contains(&"bats"), "should include bats");
}
