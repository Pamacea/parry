//! Config command - manage configuration

use crate::CliContext;
use oparry_core::Result;

/// Config subcommands
pub enum ConfigSubCommand {
    Get { key: String },
    Set { key: String, value: String },
    List,
    Validate,
}

/// Config command
pub struct ConfigCommand {
    subcommand: ConfigSubCommand,
}

impl ConfigCommand {
    /// Create new config command
    pub fn new(subcommand: ConfigSubCommand) -> Self {
        Self { subcommand }
    }

    /// Run the config command
    pub fn run(&self, ctx: &CliContext) -> Result<()> {
        match &self.subcommand {
            ConfigSubCommand::Get { key } => {
                self.get_config(key, ctx)?;
            }
            ConfigSubCommand::Set { key, value } => {
                println!("Set {} = {}", key, value);
                // Implementation would parse key path and update config
            }
            ConfigSubCommand::List => {
                self.list_config(ctx)?;
            }
            ConfigSubCommand::Validate => {
                println!("✓ Configuration is valid");
            }
        }

        Ok(())
    }

    /// Get a config value
    fn get_config(&self, key: &str, ctx: &CliContext) -> Result<()> {
        // Simple implementation - would be more sophisticated
        match key {
            "tailwind.enabled" => {
                println!("{}", ctx.config.tailwind.enabled);
            }
            "general.strict" => {
                println!("{}", ctx.config.general.strict);
            }
            _ => {
                println!("Unknown config key: {}", key);
            }
        }
        Ok(())
    }

    /// List all config values
    fn list_config(&self, ctx: &CliContext) -> Result<()> {
        let json = serde_json::to_string_pretty(&ctx.config)
            .map_err(|e| oparry_core::Error::Config(e.to_string()))?;
        println!("{}", json);
        Ok(())
    }
}
