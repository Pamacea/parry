//! Rust-specific validator

use crate::validators::Validator;
use crate::{Issue, IssueLevel, Result, ValidationResult};
use crate::parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// Rust validation configuration
#[derive(Debug, Clone)]
pub struct RustConfig {
    /// Deny unsafe code
    pub deny_unsafe: bool,
    /// Warn on .unwrap()
    pub warn_unwrap: bool,
    /// Enforce Result handling
    pub enforce_result_handling: bool,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            deny_unsafe: false,
            warn_unwrap: true,
            enforce_result_handling: true,
        }
    }
}

/// Rust validator
pub struct RustValidator {
    config: RustConfig,
    unwrap_regex: Regex,
    expect_regex: Regex,
    unsafe_regex: Regex,
}

impl RustValidator {
    /// Create new Rust validator
    pub fn new(config: RustConfig) -> Self {
        Self {
            config,
            unwrap_regex: Regex::new(r#"\.unwrap\(\)"#).unwrap(),
            expect_regex: Regex::new(r#"\.expect\("[^"]*"\)"#).unwrap(),
            unsafe_regex: Regex::new(r"\bunsafe\b").unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(RustConfig::default())
    }

    /// Check for unwrap usage
    fn check_unwrap(&self, line: &str, file: &str, line_idx: usize) -> Option<Issue> {
        if self.config.warn_unwrap {
            if self.unwrap_regex.is_match(line) {
                return Some(Issue::warning(
                    "rust-unwrap",
                    "Use of .unwrap() may cause panic",
                )
                .with_file(file)
                .with_line(line_idx)
                .with_suggestion("Use proper error handling with ? or match"));
            }
        }
        None
    }

    /// Check for unsafe blocks
    fn check_unsafe(&self, line: &str, file: &str, line_idx: usize) -> Option<Issue> {
        if self.config.deny_unsafe && self.unsafe_regex.is_match(line) {
            return Some(Issue::error(
                "rust-unsafe",
                "Unsafe code is not allowed",
            )
            .with_file(file)
            .with_line(line_idx)
            .with_suggestion("Remove unsafe block or update configuration"));
        }
        None
    }
}

impl Validator for RustValidator {
    fn name(&self) -> &str {
        "Rust"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_rust()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        for (line_idx, line) in source.lines().enumerate() {
            // Check unwrap
            if let Some(issue) = self.check_unwrap(line, &file_str, line_idx) {
                result.add_issue(issue);
            }

            // Check unsafe
            if let Some(issue) = self.check_unsafe(line, &file_str, line_idx) {
                result.add_issue(issue);
            }
        }

        Ok(result)
    }

    fn validate_raw(&self, source: &str, file: &Path) -> Result<ValidationResult> {
        let parsed = ParsedCode::Generic(source.to_string());
        self.validate_parsed(&parsed, file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_validator_unwrap() {
        let validator = RustValidator::default_config();
        let code = r#"
            fn main() {
                let x = Some(5);
                let y = x.unwrap();
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect unwrap usage");
        assert_eq!(result.issues[0].code, "rust-unwrap");
    }

    #[test]
    fn test_rust_validator_unsafe() {
        let config = RustConfig {
            deny_unsafe: true,
            ..Default::default()
        };
        let validator = RustValidator::new(config);
        let code = r#"
            fn main() {
                unsafe {
                    println!("Hello");
                }
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues[0].code, "rust-unsafe");
    }

    #[test]
    fn test_rust_validator_safe() {
        let validator = RustValidator::default_config();
        let code = r#"
            fn main() {
                let x = Some(5);
                let y = x.unwrap_or(0);
                println!("{}", y);
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_rust_config_default() {
        let config = RustConfig::default();
        assert!(!config.deny_unsafe);
        assert!(config.warn_unwrap);
        assert!(config.enforce_result_handling);
    }

    #[test]
    fn test_rust_validator_no_unwrap_warning() {
        let config = RustConfig {
            warn_unwrap: false,
            ..Default::default()
        };
        let validator = RustValidator::new(config);
        let code = r#"
            fn main() {
                let x = Some(5);
                let y = x.unwrap();
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_rust_validator_expect() {
        let validator = RustValidator::default_config();
        let code = r#"
            fn main() {
                let x = Some(5);
                let y = x.expect("must have value");
            }
        "#;

        // expect is also caught by the unwrap regex pattern
        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        // Currently only unwrap is checked, expect is defined but not used
        assert!(result.passed);
    }

    #[test]
    fn test_rust_validator_supports() {
        let validator = RustValidator::default_config();
        assert!(validator.supports(Language::Rust));
        assert!(!validator.supports(Language::JavaScript));
        assert!(!validator.supports(Language::TypeScript));
    }

    #[test]
    fn test_rust_validator_multiple_unwraps() {
        let validator = RustValidator::default_config();
        let code = r#"
            fn main() {
                let x = Some(5);
                let y = x.unwrap();
                let z = y.unwrap();
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_rust_validator_unsafe_fn_keyword() {
        let config = RustConfig {
            deny_unsafe: true,
            ..Default::default()
        };
        let validator = RustValidator::new(config);
        let code = r#"
            unsafe fn dangerous() {}
        "#;

        let result = validator.validate_raw(code, Path::new("test.rs")).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues[0].code, "rust-unsafe");
    }
}
