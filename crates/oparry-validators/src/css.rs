//! CSS validator - line length, selectors, and best practices

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use std::path::Path;

/// CSS validation configuration
#[derive(Debug, Clone)]
pub struct CssConfig {
    /// Maximum line length
    pub max_line_length: usize,
    /// Enforce selector specificity limits
    pub max_selector_specificity: u8,
    /// Block !important
    pub block_important: bool,
}

impl Default for CssConfig {
    fn default() -> Self {
        Self {
            max_line_length: 80,
            max_selector_specificity: 0,
            block_important: true,
        }
    }
}

/// CSS validator
pub struct CssValidator {
    config: CssConfig,
}

impl CssValidator {
    /// Create new CSS validator
    pub fn new(config: CssConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(CssConfig::default())
    }

    /// Check line length
    fn check_line_length(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (idx, line) in source.lines().enumerate() {
            if line.len() > self.config.max_line_length {
                issues.push(Issue::warning(
                    "css-line-too-long",
                    format!("Line too long: {} chars (max: {})", line.len(), self.config.max_line_length),
                )
                .with_file(file)
                .with_line(idx)
                .with_suggestion("Break line or use shorter selector"));
            }
        }

        issues
    }

    /// Check for !important
    fn check_important(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        if !self.config.block_important {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if line.contains("!important") {
                issues.push(Issue::error(
                    "css-important",
                    "!important should not be used",
                )
                .with_file(file)
                .with_line(idx)
                .with_suggestion("Increase specificity or refactor CSS cascade"));
            }
        }

        issues
    }
}

impl Validator for CssValidator {
    fn name(&self) -> &str {
        "CSS"
    }

    fn supports(&self, language: Language) -> bool {
        matches!(language, Language::Unknown)
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        // Only validate CSS files
        if file.to_string_lossy().ends_with(".css") {
            for issue in self.check_line_length(source, &file_str) {
                result.add_issue(issue);
            }
            for issue in self.check_important(source, &file_str) {
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
    fn test_css_validator_valid() {
        let validator = CssValidator::default_config();
        let code = r#"
            .button {
                padding: 8px;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_css_validator_important() {
        let validator = CssValidator::default_config();
        let code = r#"
            .button {
                padding: 8px !important;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues[0].code, "css-important");
    }

    #[test]
    fn test_css_validator_line_too_long() {
        let validator = CssValidator::default_config();
        let code = ".button { padding: 8px; margin: 16px; border: 1px solid red; color: blue; background: white; }";

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect line too long");
        assert_eq!(result.issues[0].code, "css-line-too-long");
    }

    #[test]
    fn test_css_validator_non_css_file() {
        let validator = CssValidator::default_config();
        let code = r#"
            .button {
                padding: 8px !important;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        // Should not validate non-CSS files
        assert!(result.passed);
    }

    #[test]
    fn test_css_config_no_important_blocking() {
        let config = CssConfig {
            block_important: false,
            ..Default::default()
        };
        let validator = CssValidator::new(config);
        let code = r#"
            .button {
                padding: 8px !important;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_css_validator_multiple_issues() {
        let validator = CssValidator::default_config();
        let code = r#"
            .button {
                padding: 8px !important;
            }
            .alert {
                background: red !important;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_css_config_custom_max_line_length() {
        let config = CssConfig {
            max_line_length: 20,
            ..Default::default()
        };
        let validator = CssValidator::new(config);
        let code = ".button { padding: 8px; }";

        let result = validator.validate_raw(code, Path::new("test.css")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect line too long with custom limit");
        assert_eq!(result.issues[0].code, "css-line-too-long");
    }

    #[test]
    fn test_css_config_default() {
        let config = CssConfig::default();
        assert_eq!(config.max_line_length, 80);
        assert!(config.block_important);
    }

    #[test]
    fn test_css_validator_supports() {
        let validator = CssValidator::default_config();
        // CSS validator should not "support" specific languages (used as fallback)
        assert!(!validator.supports(Language::JavaScript));
        assert!(!validator.supports(Language::Rust));
    }
}
