//! Behavioral specs for configuration.
//!
//! Tests that quench correctly handles:
//! - Config file validation
//! - Environment variables
//!
//! Reference: docs/specs/02-config.md

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "validation.rs"]
mod validation;

#[path = "env.rs"]
mod env;
