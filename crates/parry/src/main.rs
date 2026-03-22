//! Parry CLI - The Agentic Linter for AI-Generated Code

use clap::{Parser, Subcommand};
use oalacea_parry_core::{Config, OutputFormat};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "parry")]
#[command(about = "The Agentic Linter for AI-Generated Code", long_about = None)]
#[command(version)]
struct Cli {
    /// Increase verbosity
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Decrease verbosity
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Config file path
    #[arg(short = 'c', long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate codebase
    Check {
        /// Paths to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,

        /// Validators to run (comma-separated)
        #[arg(short, long)]
        validators: Option<String>,

        /// Output format
        #[arg(short = 'o', long, value_name = "FORMAT")]
        output: Option<String>,

        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },

    /// Watch files and validate on changes
    Watch {
        /// Paths to watch
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,

        /// Debounce delay in milliseconds
        #[arg(short, long, default_value = "300")]
        debounce: u64,

        /// Clear screen between runs
        #[arg(long)]
        clear: bool,
    },

    /// Run a command with file write interception
    Run {
        /// Command to run
        #[arg(required = true)]
        command: String,

        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Block violating writes (exit with error on violations)
        #[arg(long)]
        block: bool,

        /// Validators to run (comma-separated)
        #[arg(short, long)]
        validators: Option<String>,
    },

    /// Initialize configuration
    Init {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,

        /// Stack preset (nextjs, rust, generic)
        #[arg(long, value_name = "STACK")]
        stack: Option<String>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
    /// List all config values
    List,
    /// Validate configuration
    Validate,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else if cli.quiet {
            tracing::Level::ERROR
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set tracing subscriber: {}", e))?;

    // Load config
    let config = if let Some(config_path) = cli.config {
        Config::from_file(&config_path)?
    } else {
        Config::load().unwrap_or_default()
    };

    // Run command
    match cli.command {
        Commands::Check { paths, validators, output, fix, strict } => {
            commands::check::run(config, paths, validators, output, fix, strict)?;
        }
        Commands::Watch { paths, debounce, clear } => {
            commands::watch::run(config, paths, debounce, clear)?;
        }
        Commands::Run { command, args, block, validators } => {
            commands::run::run(config, command, args, block, validators)?;
        }
        Commands::Init { force, stack } => {
            commands::init::run(force, stack)?;
        }
        Commands::Config { subcommand } => {
            commands::config::run(config, subcommand)?;
        }
    }

    Ok(())
}

mod commands {
    pub mod check;
    pub mod watch;
    pub mod run;
    pub mod init;
    pub mod config;
}
