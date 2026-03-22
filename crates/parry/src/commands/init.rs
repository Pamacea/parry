//! Init command - initialize configuration

use oalacea_parry_core::Config;
use std::path::PathBuf;

/// Stack presets
#[derive(Debug, Clone, Copy)]
pub enum StackPreset {
    Auto,
    NextJs,
    Rust,
    Vite,
}

impl StackPreset {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "nextjs" | "next" => Some(Self::NextJs),
            "rust" => Some(Self::Rust),
            "vite" => Some(Self::Vite),
            _ => None,
        }
    }
}

/// Run the `parry init` command
pub fn run(force: bool, stack: Option<String>) -> anyhow::Result<()> {
    let config_path = PathBuf::from(".parryrc.toml");

    // Check if config exists
    if config_path.exists() && !force {
        anyhow::bail!("Config file already exists. Use --force to overwrite.");
    }

    // Parse stack preset
    let stack_preset = if let Some(s) = stack {
        StackPreset::from_str(&s)
            .ok_or_else(|| anyhow::anyhow!("Unknown stack preset: {}", s))?
    } else {
        StackPreset::Auto
    };

    // Create config based on stack
    let config = create_config(stack_preset);

    // Save config
    let toml = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, toml)?;

    println!("✓ Created {}", config_path.display());
    println!("  Edit this file to customize Parry's behavior.");

    Ok(())
}

/// Create config based on stack preset
fn create_config(stack: StackPreset) -> Config {
    let mut config = Config::default();

    match stack {
        StackPreset::NextJs => {
            // Enable Next.js-specific validators
            config.tailwind.enabled = true;
        }
        StackPreset::Rust => {
            // Enable Rust-specific validators
            config.rust.enabled = true;
        }
        StackPreset::Vite => {
            // Enable Vite-specific validators
            config.tailwind.enabled = true;
        }
        StackPreset::Auto => {
            // Auto-detect from project files would go here
            // For now, use defaults
        }
    }

    config
}
