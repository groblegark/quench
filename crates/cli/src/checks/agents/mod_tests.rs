// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn agents_check_has_correct_name() {
    let check = AgentsCheck;
    assert_eq!(check.name(), "agents");
}

#[test]
fn agents_check_has_description() {
    let check = AgentsCheck;
    assert!(!check.description().is_empty());
}

#[test]
fn agents_check_is_enabled_by_default() {
    let check = AgentsCheck;
    assert!(check.default_enabled());
}
