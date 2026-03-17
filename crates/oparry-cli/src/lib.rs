//! Parry CLI - Main library

pub mod commands;
pub mod output;

pub use commands::{check, watch, wrap, init, config, hook};
pub use output::{OutputFormatter, HumanFormatter, JsonFormatter, SarifFormatter};

use oparry_core::Config;
use std::path::PathBuf;

/// CLI context
#[derive(Debug, Clone)]
pub struct CliContext {
    /// Configuration
    pub config: Config,
    /// Working directory
    pub work_dir: PathBuf,
    /// Verbose mode
    pub verbose: bool,
    /// Quiet mode
    pub quiet: bool,
}

impl CliContext {
    /// Create new CLI context
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

        let work_dir = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get work dir: {}", e))?;

        Ok(Self {
            config,
            work_dir,
            verbose: false,
            quiet: false,
        })
    }

    /// Load config from specific path
    pub fn with_config_path(config_path: PathBuf) -> anyhow::Result<Self> {
        let config = Config::from_file(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load config from {:?}: {}", config_path, e))?;

        let work_dir = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get work dir: {}", e))?;

        Ok(Self {
            config,
            work_dir,
            verbose: false,
            quiet: false,
        })
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set quiet mode
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
}

impl Default for CliContext {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to initialize context: {}", e);
            Self {
                config: Config::default(),
                work_dir: PathBuf::from("."),
                verbose: false,
                quiet: false,
            }
        })
    }
}
