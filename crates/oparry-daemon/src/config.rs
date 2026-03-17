//! Daemon configuration

use dirs::home_dir;
use oparry_core::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Auto-validate detected sessions
    pub auto_validate: bool,
    /// Validation interval in seconds
    pub validate_interval_secs: u64,
    /// Session scan interval in seconds
    pub scan_interval_secs: u64,
    /// Claude Code config directory
    pub claude_config_dir: PathBuf,
    /// Global rules file
    pub global_rules_path: PathBuf,
    /// Log file path
    pub log_file: PathBuf,
    /// Enable strict mode (block on validation errors)
    #[serde(default)]
    pub strict_mode: bool,
    /// Enable auto-fix for fixable issues
    #[serde(default = "default_true")]
    pub auto_fix: bool,
    /// Maximum file size to validate (bytes, None for unlimited)
    #[serde(default)]
    pub max_file_size: Option<usize>,
    /// Validation timeout (seconds, None for unlimited)
    #[serde(default)]
    pub validation_timeout: Option<u64>,
    /// Enable IPC bridge for Claude Code integration
    #[serde(default = "default_true")]
    pub enable_bridge: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            auto_validate: true,
            validate_interval_secs: 5,
            scan_interval_secs: 30,
            claude_config_dir: home.join(".claude"),
            global_rules_path: home.join(".config").join("parry").join("rules.toml"),
            log_file: home.join(".parry").join("daemon.log"),
            strict_mode: false,
            auto_fix: true,
            max_file_size: Some(1024 * 1024), // 1MB default
            validation_timeout: Some(30),
            enable_bridge: true,
        }
    }
}

impl DaemonConfig {
    /// Load from file or create default
    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).map_err(|e| oparry_core::Error::File {
                    path: config_path.clone(),
                    source: e,
                })?;
            toml::from_str(&content).map_err(|e| oparry_core::Error::Config(e.to_string()))
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| oparry_core::Error::File {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e: toml::ser::Error| oparry_core::Error::Config(e.to_string()))?;

        std::fs::write(&config_path, content).map_err(|e| oparry_core::Error::File {
            path: config_path.clone(),
            source: e,
        })?;

        tracing::info!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Get config path
    fn config_path() -> Result<PathBuf> {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".config").join("parry");

        Ok(config_dir.join("daemon.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = DaemonConfig::default();
        assert!(config.auto_validate);
        assert_eq!(config.validate_interval_secs, 5);
        assert_eq!(config.scan_interval_secs, 30);
    }

    #[test]
    fn test_config_paths() {
        let config = DaemonConfig::default();
        assert!(!config.claude_config_dir.as_os_str().is_empty());
        assert!(!config.global_rules_path.as_os_str().is_empty());
        assert!(!config.log_file.as_os_str().is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = DaemonConfig::default();
        let toml_str = toml::to_string_pretty(&config);
        assert!(toml_str.is_ok());
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            auto_validate = false
            validate_interval_secs = 10
            scan_interval_secs = 60
            claude_config_dir = "/path/to/.claude"
            global_rules_path = "/path/to/rules.toml"
            log_file = "/path/to/daemon.log"
        "#;

        let config: DaemonConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.auto_validate);
        assert_eq!(config.validate_interval_secs, 10);
        assert_eq!(config.scan_interval_secs, 60);
    }

    #[test]
    fn test_config_with_custom_values() {
        let mut config = DaemonConfig::default();
        config.auto_validate = false;
        config.validate_interval_secs = 15;

        assert!(!config.auto_validate);
        assert_eq!(config.validate_interval_secs, 15);
    }
}
