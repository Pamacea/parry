// Parry Core - Validation engine & abstractions
mod config;
mod error;
mod report;
mod rule;

pub use config::{Config, OutputFormat};
pub use error::{Error, Result};
pub use report::{Issue, IssueLevel, Report, ValidationResult};
pub use rule::{Rule, RuleEngine};

/// Re-exports for convenience
pub mod prelude {
    pub use crate::{Config, Error, Issue, IssueLevel, Report, Result, Rule};
}

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
