//! Behavioral specs for config validation.
//!
//! Tests that quench correctly handles:
//! - Unknown config keys (warnings)
//! - Unknown nested keys (warnings)
//! - Valid config (no warnings)
//!
//! Reference: docs/specs/02-config.md#validation

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::prelude::*;

// =============================================================================
// CONFIG WARNING SPECS
// =============================================================================

/// Spec: docs/specs/02-config.md#validation
///
/// > Unknown keys are warnings (forward compatibility)
#[test]
fn unknown_config_key_warns() {
    let temp = Project::empty();
    temp.config(
        r#"unknown_key = true

[check.agents]
required = []
"#,
    );

    quench_cmd()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .success() // Should not fail
        .stderr(predicates::str::contains("unknown").or(predicates::str::contains("unrecognized")));
}

/// Spec: docs/specs/02-config.md#validation
///
/// > Unknown nested keys are warnings
#[test]
fn unknown_nested_config_key_warns() {
    let temp = Project::empty();
    temp.config(&format!(
        r#"{MINIMAL_CONFIG}
[check.unknown]
field = "value"
"#
    ));

    quench_cmd()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("unknown").or(predicates::str::contains("unrecognized")));
}

/// Spec: docs/specs/02-config.md#validation
///
/// > Valid config produces no warnings
#[test]
fn valid_config_no_warnings() {
    let temp = Project::empty();
    temp.config(MINIMAL_CONFIG);

    quench_cmd()
        .arg("check")
        .current_dir(temp.path())
        .assert()
        .success()
        .stderr(predicates::str::is_empty().or(predicates::str::contains("warning").not()));
}
