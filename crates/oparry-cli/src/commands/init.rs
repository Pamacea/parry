//! Init command - initialize configuration

use crate::CliContext;
use oparry_core::{Config, Result};
use std::path::PathBuf;

/// Stack presets
#[derive(Debug, Clone, Copy)]
pub enum StackPreset {
    Auto,
    NextJs,
    RustAxum,
    NestJs,
    ViteJs,
}

impl StackPreset {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "nextjs" => Some(Self::NextJs),
            "rust-axum" | "rust" | "axum" => Some(Self::RustAxum),
            "nestjs" => Some(Self::NestJs),
            "vitejs" | "vite" => Some(Self::ViteJs),
            _ => None,
        }
    }
}

/// Init command
pub struct InitCommand {
    /// Stack preset
    stack: StackPreset,
    /// Force overwrite
    force: bool,
    /// Create sample config
    sample: bool,
}

impl InitCommand {
    /// Create new init command
    pub fn new(stack: StackPreset, force: bool, sample: bool) -> Self {
        Self {
            stack,
            force,
            sample,
        }
    }

    /// Run the init command
    pub fn run(&self, _ctx: &CliContext) -> Result<()> {
        let config_path = PathBuf::from(".parryrc.toml");

        // Check if config exists
        if config_path.exists() && !self.force {
            return Err(oparry_core::Error::Config(
                "Config file already exists. Use --force to overwrite.".to_string()
            ));
        }

        // Create config based on stack
        let config = self.create_config();

        // Save config
        config.save(&config_path)?;

        println!("✓ Created {}", config_path.display());
        println!("  Edit this file to customize Parry's behavior.");

        Ok(())
    }

    /// Create config based on stack preset
    fn create_config(&self) -> Config {
        let mut config = Config::default();

        match self.stack {
            StackPreset::NextJs => {
                config.nextjs.enabled = true;
                config.tailwind.enabled = true;
                config.components.enforce_shadcn = true;
                config.imports.alias_map.insert("@/".to_string(), "./src".to_string());
            }
            StackPreset::RustAxum => {
                config.rust.enabled = true;
                config.rust.deny_unsafe = Some("warn".to_string());
            }
            StackPreset::NestJs => {
                config.nestjs.enabled = true;
            }
            StackPreset::ViteJs => {
                config.tailwind.enabled = true;
                config.components.enforce_shadcn = true;
            }
            StackPreset::Auto => {
                // Auto-detect from project files would go here
                // For now, use defaults
            }
        }

        config
    }
}
