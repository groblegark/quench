// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn parse_bare_invocation() {
    let cli = Cli::parse_from(["quench"]);
    assert!(cli.command.is_none());
    assert!(cli.config.is_none());
}

#[test]
fn parse_check_command() {
    let cli = Cli::parse_from(["quench", "check"]);
    assert!(matches!(cli.command, Some(Command::Check(_))));
}

#[test]
fn parse_check_with_paths() {
    let cli = Cli::parse_from(["quench", "check", "src/", "lib/"]);
    if let Some(Command::Check(args)) = cli.command {
        assert_eq!(args.paths.len(), 2);
    } else {
        panic!("expected check command");
    }
}

#[test]
fn parse_check_with_output_format() {
    let cli = Cli::parse_from(["quench", "check", "-o", "json"]);
    if let Some(Command::Check(args)) = cli.command {
        assert!(matches!(args.output, OutputFormat::Json));
    } else {
        panic!("expected check command");
    }
}

#[test]
fn parse_report_command() {
    let cli = Cli::parse_from(["quench", "report"]);
    assert!(matches!(cli.command, Some(Command::Report(_))));
}

#[test]
fn parse_init_command() {
    let cli = Cli::parse_from(["quench", "init"]);
    assert!(matches!(cli.command, Some(Command::Init(_))));
}

#[test]
fn parse_init_with_force() {
    let cli = Cli::parse_from(["quench", "init", "--force"]);
    if let Some(Command::Init(args)) = cli.command {
        assert!(args.force);
    } else {
        panic!("expected init command");
    }
}

#[test]
fn parse_global_config_flag() {
    let cli = Cli::parse_from(["quench", "-C", "custom.toml", "check"]);
    assert_eq!(cli.config, Some(PathBuf::from("custom.toml")));
}

#[test]
fn parse_global_config_long_flag() {
    let cli = Cli::parse_from(["quench", "--config", "custom.toml", "check"]);
    assert_eq!(cli.config, Some(PathBuf::from("custom.toml")));
}

#[test]
fn default_template_contains_required_sections() {
    let template = default_template();
    assert!(template.contains("version = 1"));
    assert!(template.contains("[check.cloc]"));
    assert!(template.contains("[check.escapes]"));
    assert!(template.contains("[check.agents]"));
    assert!(template.contains("[check.docs]"));
    assert!(template.contains("[check.tests]"));
    assert!(template.contains("[check.license]"));
    assert!(template.contains("[git.commit]"));
    assert!(template.contains("# Supported Languages:"));
    assert!(template.contains("# [rust], [golang], [javascript], [shell]"));
}

#[test]
fn default_template_has_explicit_check_levels() {
    let template = default_template();
    // Enabled checks
    assert!(template.contains("[check.cloc]\ncheck = \"error\""));
    assert!(template.contains("[check.escapes]\ncheck = \"error\""));
    assert!(template.contains("[check.agents]\ncheck = \"error\""));
    assert!(template.contains("[check.docs]\ncheck = \"error\""));
    // Disabled checks with stub comments
    assert!(template.contains("check = \"off\"  # stub in quench v0.3.0"));
}
