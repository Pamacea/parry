//! Configuration management

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Human,
    Json,
    Sarif,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            _ => Err(Error::Config(format!("Invalid output format: {}", s))),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Human
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable strict mode (warnings as errors)
    #[serde(default)]
    pub strict: bool,

    /// Stop on first error
    #[serde(default)]
    pub fail_fast: bool,

    /// Maximum issues to report
    #[serde(default = "default_max_issues")]
    pub max_issues: usize,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            strict: false,
            fail_fast: false,
            max_issues: default_max_issues(),
        }
    }
}

fn default_max_issues() -> usize {
    100
}

/// Tailwind configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailwindConfig {
    /// Enable Tailwind validator
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Path to Tailwind config
    #[serde(default = "default_tailwind_config")]
    pub config_path: PathBuf,

    /// Safe list patterns
    #[serde(default)]
    pub safe_list: Vec<String>,

    /// Block list patterns
    #[serde(default)]
    pub block_list: Vec<String>,

    /// Maximum arbitrary values
    #[serde(default = "default_max_arbitrary")]
    pub max_arbitrary_values: usize,
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config_path: default_tailwind_config(),
            safe_list: Vec::new(),
            block_list: Vec::new(),
            max_arbitrary_values: default_max_arbitrary(),
        }
    }
}

fn default_tailwind_config() -> PathBuf {
    PathBuf::from("tailwind.config.ts")
}

fn default_max_arbitrary() -> usize {
    5
}

/// Import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Enforce path aliases
    #[serde(default = "default_true")]
    pub enforce_alias: bool,

    /// Alias mappings
    #[serde(default)]
    pub alias_map: HashMap<String, String>,

    /// Require file extensions
    #[serde(default)]
    pub require_extensions: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            enforce_alias: true,
            alias_map: HashMap::new(),
            require_extensions: false,
        }
    }
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Enforce shadcn/ui usage
    #[serde(default = "default_true")]
    pub enforce_shadcn: bool,

    /// shadcn/ui components path
    #[serde(default = "default_shadcn_path")]
    pub shadcn_path: String,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            enforce_shadcn: true,
            shadcn_path: default_shadcn_path(),
        }
    }
}

fn default_shadcn_path() -> String {
    "@/components/ui".to_string()
}

fn default_true() -> bool {
    true
}

/// Rust configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConfig {
    /// Enable Rust validator
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Deny unsafe code
    #[serde(default)]
    pub deny_unsafe: Option<String>,

    /// Warn on unwrap()
    #[serde(default = "default_true")]
    pub warn_unwrap: bool,

    /// Enforce Result handling
    #[serde(default = "default_true")]
    pub enforce_result_handling: bool,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deny_unsafe: None,
            warn_unwrap: true,
            enforce_result_handling: true,
        }
    }
}

/// Next.js configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextJsConfig {
    /// Enable Next.js validator
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enforce App Router conventions
    #[serde(default = "default_true")]
    pub enforce_app_router: bool,

    /// Validate page exports
    #[serde(default = "default_true")]
    pub validate_page_exports: bool,
}

impl Default for NextJsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enforce_app_router: true,
            validate_page_exports: true,
        }
    }
}

/// NestJS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJsConfig {
    /// Enable NestJS validator
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enforce decorator usage
    #[serde(default = "default_true")]
    pub enforce_decorators: bool,

    /// Validate module imports
    #[serde(default = "default_true")]
    pub validate_modules: bool,
}

impl Default for NestJsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enforce_decorators: true,
            validate_modules: true,
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format
    #[serde(default)]
    pub format: OutputFormat,

    /// Show file paths
    #[serde(default = "default_true")]
    pub show_paths: bool,

    /// Color output
    #[serde(default)]
    pub color: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Human,
            show_paths: true,
            color: "auto".to_string(),
        }
    }
}

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Output settings
    #[serde(default)]
    pub output: OutputConfig,

    /// Tailwind settings
    #[serde(default)]
    pub tailwind: TailwindConfig,

    /// Import settings
    #[serde(default)]
    pub imports: ImportConfig,

    /// Component settings
    #[serde(default)]
    pub components: ComponentConfig,

    /// Rust settings
    #[serde(default)]
    pub rust: RustConfig,

    /// Next.js settings
    #[serde(default)]
    pub nextjs: NextJsConfig,

    /// NestJS settings
    #[serde(default)]
    pub nestjs: NestJsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            output: OutputConfig::default(),
            tailwind: TailwindConfig::default(),
            imports: ImportConfig::default(),
            components: ComponentConfig::default(),
            rust: RustConfig::default(),
            nextjs: NextJsConfig::default(),
            nestjs: NestJsConfig::default(),
        }
    }
}

impl Config {
    /// Load config from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::File {
            path: path.to_path_buf(),
            source: e,
        })?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from default locations
    pub fn load() -> Result<Self> {
        // Check .parryrc.toml first
        if let Ok(config) = Self::from_file(Path::new(".parryrc.toml")) {
            return Ok(config);
        }

        // Check parry.toml
        if let Ok(config) = Self::from_file(Path::new("parry.toml")) {
            return Ok(config);
        }

        // Return default
        Ok(Config::default())
    }

    /// Save config to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(e.to_string()))?;
        std::fs::write(path, content).map_err(|e| Error::File {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Merge with another config (other takes precedence)
    pub fn merge(&mut self, other: Config) {
        // Merge general settings
        if other.general.strict {
            self.general.strict = true;
        }
        if other.general.fail_fast {
            self.general.fail_fast = true;
        }

        // Merge tailwind settings
        self.tailwind.enabled = self.tailwind.enabled || other.tailwind.enabled;
        self.tailwind.safe_list.extend(other.tailwind.safe_list);
        self.tailwind.block_list.extend(other.tailwind.block_list);

        // Merge imports
        self.imports.alias_map.extend(other.imports.alias_map);

        // Merge components
        self.components.enforce_shadcn = self.components.enforce_shadcn || other.components.enforce_shadcn;

        // Output takes precedence
        self.output = other.output;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.general.strict);
        assert!(config.tailwind.enabled);
        assert_eq!(config.tailwind.config_path, PathBuf::from("tailwind.config.ts"));
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::default();
        let override_config = Config {
            general: GeneralConfig {
                strict: true,
                ..Default::default()
            },
            tailwind: TailwindConfig {
                safe_list: vec!["p-*".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        base.merge(override_config);
        assert!(base.general.strict);
        assert!(base.tailwind.safe_list.contains(&"p-*".to_string()));
    }

    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert!(OutputFormat::from_str("invalid").is_err());
    }
}
