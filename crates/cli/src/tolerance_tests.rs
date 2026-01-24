// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parse_duration_seconds() {
    assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
    assert_eq!(
        parse_duration("1.5s").unwrap(),
        Duration::from_secs_f64(1.5)
    );
}

#[test]
fn parse_duration_milliseconds() {
    assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
    assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
}

#[test]
fn parse_duration_combined() {
    assert_eq!(parse_duration("1m30s").unwrap(), Duration::from_secs(90));
    assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
    assert_eq!(parse_duration("1m0s").unwrap(), Duration::from_secs(60));
}

#[test]
fn parse_duration_plain_number() {
    assert_eq!(parse_duration("5").unwrap(), Duration::from_secs(5));
    assert_eq!(parse_duration("2.5").unwrap(), Duration::from_secs_f64(2.5));
}

#[test]
fn parse_duration_with_whitespace() {
    assert_eq!(parse_duration("  5s  ").unwrap(), Duration::from_secs(5));
}

#[test]
fn parse_size_bytes() {
    assert_eq!(parse_size("1024").unwrap(), 1024);
    assert_eq!(parse_size("1024B").unwrap(), 1024);
}

#[test]
fn parse_size_kilobytes() {
    assert_eq!(parse_size("100KB").unwrap(), 100 * 1024);
    assert_eq!(parse_size("1kb").unwrap(), 1024); // case insensitive
}

#[test]
fn parse_size_megabytes() {
    assert_eq!(parse_size("5MB").unwrap(), 5 * 1024 * 1024);
    assert_eq!(parse_size("1.5MB").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
}

#[test]
fn parse_size_gigabytes() {
    assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
}

#[test]
fn parse_size_with_whitespace() {
    assert_eq!(parse_size("  100KB  ").unwrap(), 100 * 1024);
}

#[test]
fn parse_duration_invalid() {
    assert!(parse_duration("invalid").is_err());
    assert!(parse_duration("").is_err());
}

#[test]
fn parse_size_invalid() {
    assert!(parse_size("invalid").is_err());
    assert!(parse_size("").is_err());
}
