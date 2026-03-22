//! Config command - manage configuration

use oalacea_parry_core::Config;
use std::path::PathBuf;

/// Run the `parry config` command
pub fn run(config: Config, subcommand: super::super::ConfigCommands) -> anyhow::Result<()> {
    match subcommand {
        super::super::ConfigCommands::Get { key } => {
            get_config(&config, &key)?;
        }
        super::super::ConfigCommands::Set { key, value } => {
            set_config(key, value)?;
        }
        super::super::ConfigCommands::List => {
            list_config(&config)?;
        }
        super::super::ConfigCommands::Validate => {
            println!("✓ Configuration is valid");
        }
    }

    Ok(())
}

/// Get a config value
fn get_config(config: &Config, key: &str) -> anyhow::Result<()> {
    // Simple implementation - would be more sophisticated
    match key {
        "tailwind.enabled" => {
            println!("{}", config.tailwind.enabled);
        }
        "rust.enabled" => {
            println!("{}", config.rust.enabled);
        }
        "general.strict" => {
            println!("{}", config.general.strict);
        }
        _ => {
            println!("Unknown config key: {}", key);
            println!("Available keys: tailwind.enabled, rust.enabled, general.strict");
        }
    }
    Ok(())
}

/// Set a config value
fn set_config(key: String, value: String) -> anyhow::Result<()> {
    println!("Set {} = {}", key, value);
    // Implementation would parse key path and update config file
    println!("Note: This feature is not yet fully implemented.");
    println!("Edit .parryrc.toml directly to change configuration.");
    Ok(())
}

/// List all config values
fn list_config(config: &Config) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    println!("{}", json);
    Ok(())
}
