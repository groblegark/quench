//! Quench CLI entry point.

use clap::{CommandFactory, Parser};
use tracing_subscriber::{EnvFilter, fmt};

use quench::cli::{CheckArgs, Cli, Command, InitArgs, OutputFormat, ReportArgs};
use quench::config;
use quench::discovery;
use quench::error::ExitCode;

fn init_logging() {
    let filter = EnvFilter::try_from_env("QUENCH_LOG").unwrap_or_else(|_| EnvFilter::new("off"));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}

fn main() {
    init_logging();

    if let Err(e) = run() {
        eprintln!("quench: {}", e);
        let code = match e.downcast_ref::<quench::Error>() {
            Some(err) => ExitCode::from(err) as i32,
            None => ExitCode::InternalError as i32,
        };
        std::process::exit(code);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        None => {
            // Show help for bare invocation
            Cli::command().print_help()?;
            println!();
        }
        Some(Command::Check(args)) => run_check(&cli, args)?,
        Some(Command::Report(args)) => run_report(&cli, args)?,
        Some(Command::Init(args)) => run_init(&cli, args)?,
    }

    Ok(())
}

fn run_check(cli: &Cli, args: &CheckArgs) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Resolve config
    let config_path = discovery::resolve_config(cli.config.as_deref(), &cwd)?;

    let _config = match &config_path {
        Some(path) => {
            tracing::debug!("loading config from {}", path.display());
            config::load_with_warnings(path)?
        }
        None => {
            tracing::debug!("no config found, using defaults");
            config::Config::default()
        }
    };

    tracing::trace!("check command starting");

    // For now, just output success
    match args.output {
        OutputFormat::Text => {
            // Silent on success per spec
        }
        OutputFormat::Json => {
            println!(r#"{{"passed": true, "violations": []}}"#);
        }
    }

    Ok(())
}

fn run_report(_cli: &Cli, args: &ReportArgs) -> anyhow::Result<()> {
    match args.output {
        OutputFormat::Text => println!("No metrics collected yet."),
        OutputFormat::Json => println!(r#"{{"metrics": {{}}}}"#),
    }
    Ok(())
}

fn run_init(_cli: &Cli, args: &InitArgs) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("quench.toml");

    if config_path.exists() && !args.force {
        eprintln!("quench.toml already exists. Use --force to overwrite.");
        std::process::exit(ExitCode::ConfigError as i32);
    }

    std::fs::write(&config_path, "version = 1\n")?;
    println!("Created quench.toml");
    Ok(())
}
