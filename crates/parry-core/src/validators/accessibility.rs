//! Accessibility validator

use crate::validators::Validator;
use crate::{Issue, IssueLevel, Result, ValidationResult};
use crate::parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct A11yConfig {
    pub require_alt: bool,
    pub require_aria_label: bool,
    pub allow_aria_hidden: bool,
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            require_alt: true,
            require_aria_label: true,
            allow_aria_hidden: true,
        }
    }
}

pub struct A11yValidator {
    config: A11yConfig,
    img_tag_regex: Regex,
    button_no_text_regex: Regex,
}

impl A11yValidator {
    pub fn new(config: A11yConfig) -> Self {
        Self {
            config,
            img_tag_regex: Regex::new(r"<img\b[^>]*>").unwrap(),
            button_no_text_regex: Regex::new(r"<button\b[^>]*>(?:\s*</button>)?").unwrap(),
        }
    }

    pub fn default_config() -> Self {
        Self::new(A11yConfig::default())
    }

    /// Check if img tag has alt attribute
    fn has_alt_attribute(img_tag: &str) -> bool {
        // Match alt= with or without quotes, handling various formats
        let alt_regex = Regex::new(r#"alt\s*=\s*("[^"]*"|'[^']*'|[^"'\s>]+)"#).unwrap();
        alt_regex.is_match(img_tag)
    }

    /// Check if button has accessible content (text or aria-label)
    fn has_accessible_label(button_tag: &str) -> bool {
        // Check for aria-label or aria-labelledby
        let aria_regex = Regex::new(r#"aria-(label|labelledby)\s*="#).unwrap();
        if aria_regex.is_match(button_tag) {
            return true;
        }
        // Extract content between tags
        if let Some(start) = button_tag.find('>') {
            let content = &button_tag[start + 1..];
            if let Some(end) = content.find("</button>") {
                let text = &content[..end];
                return !text.trim().is_empty();
            }
        }
        false
    }

    fn validate_internal(&self, source: &str, file: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if self.config.require_alt {
            for mat in self.img_tag_regex.find_iter(source) {
                if !Self::has_alt_attribute(mat.as_str()) {
                    let line = source[..mat.start()].lines().count() + 1;
                    result.add_issue(Issue::warning("a11y-img-no-alt", "Image missing alt text")
                        .with_line(line)
                        .with_suggestion("Add alt attribute"));
                }
            }
        }

        if self.config.require_aria_label {
            for mat in self.button_no_text_regex.find_iter(source) {
                if !Self::has_accessible_label(mat.as_str()) {
                    let line = source[..mat.start()].lines().count() + 1;
                    result.add_issue(Issue::warning("a11y-button-no-label", "Button has no accessible label")
                        .with_line(line)
                        .with_suggestion("Add aria-label or text content"));
                }
            }
        }

        Ok(result)
    }
}

impl Validator for A11yValidator {
    fn name(&self) -> &str {
        "Accessibility"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let source = code.source();
        let file_str = file.to_string_lossy().to_string();
        self.validate_internal(source, &file_str)
    }

    fn validate_raw(&self, source: &str, file: &Path) -> Result<ValidationResult> {
        let file_str = file.to_string_lossy().to_string();
        self.validate_internal(source, &file_str)
    }
}

impl Default for A11yValidator {
    fn default() -> Self {
        Self::new(A11yConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_img_without_alt() {
        let validator = A11yValidator::new(A11yConfig::default());
        let source = r#"<img src="logo.png" />"#;
        let result = validator.validate_raw(source, std::path::Path::new("test.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect missing alt attribute");
    }

    #[test]
    fn test_img_with_alt() {
        let validator = A11yValidator::new(A11yConfig::default());
        let source = r#"<img src="logo.png" alt="Company Logo" />"#;
        let result = validator.validate_raw(source, std::path::Path::new("test.tsx")).unwrap();
        assert!(result.passed);
        assert_eq!(result.warning_count(), 0);
    }
}
