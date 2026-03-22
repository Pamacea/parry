//! Parry Core - The Agentic Linter Library
//!
//! This library contains all the core functionality for Parry:
//! - Configuration management
//! - File parsing (JavaScript/TypeScript, Rust)
//! - Validation rules
//! - File watching
//! - Synchronous wrapper mode
//! - Auto-fix capabilities

pub mod config;
pub mod error;
pub mod report;
pub mod rule;

pub mod parser;
pub mod validators;
pub mod watcher;
pub mod wrapper;
pub mod autofix;

// Re-export common types
pub use config::{Config, OutputFormat};
pub use error::{Error, Result};
pub use report::{Issue, IssueLevel, Report, ValidationResult};
pub use rule::{Rule, RuleEngine};

// Re-exports for convenience
pub mod prelude {
    pub use crate::{Config, Error, Issue, IssueLevel, Report, Result, Rule, RuleEngine};
}

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
