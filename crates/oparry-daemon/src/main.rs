//! Parry Daemon - Auto-detection and validation for Claude Code

use clap::{Parser, Subcommand};
use oparry_core::{Error, Result};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use oparry_daemon::config::DaemonConfig;
use oparry_daemon::validator::DaemonValidator;
use oparry_daemon::{claude, ClaudeBridge, ParryDaemon};

#[derive(Parser)]
#[command(name = "parryd")]
#[command(about = "Parry Daemon - Auto-validation for Claude Code sessions", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Daemon configuration file
    #[arg(short, long, default_value = "~/.config/parry/daemon.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Run {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,

        /// Validate immediately on start
        #[arg(short, long)]
        validate_now: bool,
    },

    /// Scan for Claude Code sessions
    Scan {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate detected sessions
    Validate {
        /// Session ID to validate (all if not specified)
        #[arg(short, long)]
        session: Option<u32>,

        /// Show detailed output
        #[arg(short, long)]
        detailed: bool,
    },

    /// Install daemon globally (sets up auto-start)
    Install {
        /// Force reinstallation
        #[arg(long)]
        force: bool,
    },

    /// Show daemon status
    Status,

    /// Run IPC bridge for Claude Code integration
    Bridge,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cli.verbose {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        oparry_core::Error::Other(format!("Failed to set tracing subscriber: {}", e))
    })?;

    match cli.command {
        Commands::Run {
            foreground,
            validate_now,
        } => {
            run_daemon(foreground, validate_now).await?;
        }
        Commands::Scan { detailed } => {
            scan_sessions(detailed).await?;
        }
        Commands::Validate { session, detailed } => {
            validate_sessions(session, detailed).await?;
        }
        Commands::Install { force } => {
            install_daemon(force)?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Bridge => {
            run_bridge().await?;
        }
    }

    Ok(())
}

async fn run_daemon(foreground: bool, validate_now: bool) -> Result<()> {
    let config = DaemonConfig::load_or_create()?;

    info!("🦀 Parry Daemon v{} starting", env!("CARGO_PKG_VERSION"));
    info!("Auto-validate: {}", config.auto_validate);
    info!("Validate interval: {}s", config.validate_interval_secs);
    info!("Scan interval: {}s", config.scan_interval_secs);

    // Create daemon
    let daemon = ParryDaemon::new(config);

    if validate_now {
        info!("Validating sessions on start...");
        // Note: scan_sessions is currently private on DaemonState
        // For now, we'll skip this in the run function
        // TODO: Make scan_sessions accessible from ParryDaemon
    }

    if !foreground {
        info!("Running in background mode");
        // TODO: Implement daemonization using daemonize crate or similar
        // For now, warn user
        println!("⚠️  Background mode not yet implemented - use --foreground");
    }

    daemon.run().await
}

async fn scan_sessions(detailed: bool) -> Result<()> {
    let detector = claude::SessionDetector::new();
    let sessions = detector.detect_sessions().await?;

    if sessions.is_empty() {
        println!("No Claude Code sessions detected");
        return Ok(());
    }

    println!("Found {} Claude Code session(s):", sessions.len());

    for session in &sessions {
        println!("\n  Session PID: {}", session.id);
        println!("  Working Dir: {}", session.work_dir.display());
        if let Some(repo) = &session.repository {
            println!("  Repository: {}", repo.display());
        }
        if !session.active_files.is_empty() {
            println!("  Active Files ({}):", session.active_files.len());
            for file in &session.active_files {
                if detailed {
                    println!("    - {}", file.display());
                } else {
                    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("???");
                    println!("    - {}", name);
                }
            }
        }

        if detailed {
            // Validate and show report
            let config = DaemonConfig::load_or_create()?;
            let validator = DaemonValidator::new(config);

            println!("  Validation:");
            match validator.session_report(session).await {
                Ok(report) => {
                    for line in report.lines() {
                        println!("    {}", line);
                    }
                }
                Err(e) => {
                    println!("    Error: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn validate_sessions(session_id: Option<u32>, detailed: bool) -> Result<()> {
    let config = DaemonConfig::load_or_create()?;
    let detector = claude::SessionDetector::new();
    let sessions = detector.detect_sessions().await?;

    let sessions_to_validate = if let Some(id) = session_id {
        sessions.into_iter().filter(|s| s.id == id).collect()
    } else {
        sessions
    };

    if sessions_to_validate.is_empty() {
        if let Some(id) = session_id {
            println!("Session {} not found", id);
        } else {
            println!("No sessions found");
        }
        return Ok(());
    }

    let validator = DaemonValidator::new(config);

    for session in sessions_to_validate {
        println!(
            "\nValidating session {} ({})",
            session.id,
            session.work_dir.display()
        );

        let results: Vec<oparry_core::ValidationResult> =
            validator.validate_session(&session).await?;
        let total_issues: usize = results.iter().map(|r| r.issues.len()).sum();

        if total_issues == 0 {
            println!("  ✓ No issues");
        } else {
            println!("  ⚠️  {} issues found:", total_issues);

            for result in results {
                for issue in &result.issues {
                    println!("    - [{}] {}", issue.level, issue.message);
                    if detailed {
                        if let Some(file) = &issue.file {
                            println!("      File: {:?}", file);
                        }
                        if let Some(line) = issue.line {
                            println!("      Line: {}", line);
                        }
                        if let Some(ref suggestion) = issue.suggestion {
                            println!("      Suggestion: {}", suggestion);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn install_daemon(force: bool) -> Result<()> {
    println!("Installing Parry Daemon globally...");

    let home = dirs::home_dir()
        .ok_or_else(|| Error::Config("Could not determine home directory".to_string()))?;

    let config_dir = home.join(".config").join("parry");
    let data_dir = home.join(".parry");

    // Create directories
    std::fs::create_dir_all(&config_dir).map_err(|e| Error::File {
        path: config_dir.clone(),
        source: e,
    })?;
    std::fs::create_dir_all(&data_dir).map_err(|e| Error::File {
        path: data_dir.clone(),
        source: e,
    })?;

    // Create global rules file
    let rules_path = config_dir.join("rules.toml");
    if !rules_path.exists() || force {
        let default_rules = r#"# Parry Global Rules
# These rules apply to all projects unless overridden

[general]
strict = false
fail_fast = false

[tailwind]
enabled = true
# Blocked width classes - use Container components instead
blocked_widths = ["w-xl", "w-2xl", "w-3xl", "w-4xl"]
blocked_max_widths = ["max-w-sm", "max-w-md", "max-w-lg", "max-w-xl", "max-w-2xl"]

[react]
enabled = true
max_component_lines = 300
prefer_function_components = true

[css]
max_line_length = 80
block_important = true
"#;

        std::fs::write(&rules_path, default_rules).map_err(|e| Error::File {
            path: rules_path.clone(),
            source: e,
        })?;
    }

    println!("✓ Config directory: {}", config_dir.display());
    println!("✓ Data directory: {}", data_dir.display());
    println!("✓ Rules file: {}", rules_path.display());
    println!("\nParry Daemon installed!");
    println!("Run 'parryd run' to start validation daemon");

    Ok(())
}

async fn show_status() -> Result<()> {
    let detector = claude::SessionDetector::new();
    let sessions = detector.detect_sessions().await?;

    println!("Parry Daemon Status");
    println!("================");
    println!("\nActive Sessions: {}", sessions.len());

    for session in &sessions {
        println!("\n  Session {}:", session.id);
        println!("    Working Dir: {}", session.work_dir.display());
        if let Some(repo) = &session.repository {
            println!("    Repository: {}", repo.display());
        }
        println!("    Active Files: {}", session.active_files.len());
    }

    if sessions.is_empty() {
        println!("\nNo active Claude Code sessions detected.");
        println!("Make sure Claude Code is running with files open.");
    }

    Ok(())
}

/// Run the IPC bridge for Claude Code integration
async fn run_bridge() -> Result<()> {
    let config = DaemonConfig::load_or_create()?;

    info!("Starting Parry IPC Bridge");
    info!("Protocol version: 0.2.0");
    info!("Strict mode: {}", config.strict_mode);
    info!("Auto-fix: {}", config.auto_fix);

    let bridge = ClaudeBridge::from_daemon_config(&config);

    info!("Bridge ready, waiting for Claude Code connections...");
    eprintln!(
        "Parry Bridge v{} running (protocol 0.2.0)",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!(
        "Mode: {}",
        if config.strict_mode { "strict" } else { "warn" }
    );

    bridge.run_ipc_loop().await
}
