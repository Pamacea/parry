//! Parry CLI - The Agentic Linter for AI-Generated Code

use clap::{Parser, Subcommand};
use oparry_cli::{CliContext, commands::{CheckCommand, WatchCommand, WrapCommand, InitCommand, ConfigCommand, InstallCommand, init::StackPreset, hook::{HookCommand, HookAction}}};
use oparry_core::OutputFormat;
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
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,

        /// Force bypass of strict mode (even in strict mode, allow warnings)
        #[arg(long)]
        force: bool,
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

    /// Wrap a command and intercept file writes (or run in IPC mode if no command)
    Wrap {
        /// Block violating writes
        #[arg(long)]
        block: bool,

        /// Command to wrap (if empty, runs in IPC mode)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Initialize configuration
    Init {
        /// Stack preset
        #[arg(long, value_name = "STACK")]
        stack: Option<String>,

        /// Force overwrite existing config
        #[arg(long)]
        force: bool,

        /// Create sample config with comments
        #[arg(long)]
        sample: bool,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },

    /// Manage Claude Code integration hooks
    Hook {
        #[command(subcommand)]
        subcommand: HookAction,
    },

    /// Install Parry (one-command setup)
    Install {
        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,
        /// Global install (system-wide)
        #[arg(long)]
        global: bool,
        /// Skip daemon setup
        #[arg(long)]
        no_daemon: bool,
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

    // Load context
    let ctx = if let Some(config_path) = cli.config {
        CliContext::with_config_path(config_path)?
    } else {
        CliContext::new()?
    };

    // Run command
    match cli.command {
        Commands::Check { paths, validators, output, fix, strict, force } => {
            let validators_list = validators.unwrap_or_default()
                .split(',')
                .map(String::from)
                .filter(|s| !s.is_empty())
                .collect();

            let format = if let Some(fmt) = output {
                OutputFormat::from_str(&fmt)?
            } else {
                ctx.config.output.format
            };

            // In force mode, disable strict even if config or flag says otherwise
            let effective_strict = strict && !force;

            let cmd = CheckCommand::new(paths, validators_list, format, fix, effective_strict);
            cmd.run(&ctx).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Watch { paths, debounce, clear } => {
            let cmd = WatchCommand::new(paths, std::time::Duration::from_millis(debounce), clear);
            cmd.run(&ctx).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Wrap { block, mut command } => {
            if command.is_empty() {
                // IPC mode - run wrapper for Claude Code integration
                let cmd = WrapCommand::new(None, !block, cli.verbose);
                cmd.run().map_err(|e| anyhow::anyhow!("{}", e))?;
            } else {
                // Legacy mode - wrap an external command
                let cmd_name = command.remove(0);
                // Use old StdioWrapper for external command wrapping
                let config = oparry_wrapper::WrapConfig {
                    block,
                    ..Default::default()
                };
                let wrapper = oparry_wrapper::StdioWrapper::new(config);
                let exit_code = wrapper.wrap_command(&cmd_name, &command)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                std::process::exit(exit_code);
            }
        }
        Commands::Init { stack, force, sample } => {
            let stack_preset = if let Some(s) = stack {
                StackPreset::from_str(&s)
                    .ok_or_else(|| anyhow::anyhow!("Unknown stack preset: {}", s))?
            } else {
                StackPreset::Auto
            };

            let cmd = InitCommand::new(stack_preset, force, sample);
            cmd.run(&ctx).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Config { subcommand } => {
            let config_cmd = match subcommand {
                ConfigCommands::Get { key } => {
                    ConfigCommand::new(oparry_cli::commands::config::ConfigSubCommand::Get { key })
                }
                ConfigCommands::Set { key, value } => {
                    ConfigCommand::new(oparry_cli::commands::config::ConfigSubCommand::Set { key, value })
                }
                ConfigCommands::List => {
                    ConfigCommand::new(oparry_cli::commands::config::ConfigSubCommand::List)
                }
                ConfigCommands::Validate => {
                    ConfigCommand::new(oparry_cli::commands::config::ConfigSubCommand::Validate)
                }
            };

            config_cmd.run(&ctx).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Hook { subcommand } => {
            let hook_cmd = HookCommand::new(subcommand);
            hook_cmd.run().map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Install { force, global, no_daemon } => {
            let cmd = InstallCommand::new(force, global, no_daemon);
            cmd.run(&ctx).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}
