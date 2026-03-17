//! Wrap command - Run Parry wrapper in IPC mode
//!
//! This command starts the wrapper that intercepts Claude Code
//! file operations and validates them in real-time.

use clap::Parser;
use oparry_core::Result;
use std::sync::Arc;

use oparry_wrapper::{ClaudeWrapper, ValidatorEngine, WrapConfig};

/// Wrap command configuration
#[derive(Parser)]
pub struct WrapCommand {
    /// Configuration file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Don't block writes, only warn
    #[arg(long, default_value = "false")]
    pub warn_only: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl WrapCommand {
    pub fn new(config: Option<String>, warn_only: bool, verbose: bool) -> Self {
        Self { config, warn_only, verbose }
    }

    pub fn run(self) -> Result<()> {
        // Setup logging
        if self.verbose {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .init();
        } else {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
        }

        // Load or create config
        let mut config = WrapConfig::default();
        if self.warn_only {
            config.block = false;
        }

        // Create validator engine
        let validator = Arc::new(ValidatorEngine::new());

        // Create wrapper
        let wrapper = ClaudeWrapper::new(validator, config);

        // Run IPC loop
        wrapper.run()
    }
}
