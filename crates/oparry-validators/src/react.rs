//! React validator - hooks rules, patterns, and best practices

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// React validation configuration
#[derive(Debug, Clone)]
pub struct ReactConfig {
    /// Enforce hooks rules
    pub enforce_hooks_rules: bool,
    /// Maximum component lines
    pub max_component_lines: usize,
    /// Require prop-types or TypeScript
    pub require_prop_types: bool,
    /// Enforce function components
    pub prefer_function_components: bool,
    /// Block specific patterns
    pub blocked_patterns: Vec<String>,
}

impl Default for ReactConfig {
    fn default() -> Self {
        Self {
            enforce_hooks_rules: true,
            max_component_lines: 300,
            require_prop_types: false,
            prefer_function_components: true,
            blocked_patterns: vec![
                "propTypes".to_string(),
                "createClass".to_string(),
            ],
        }
    }
}

/// React validator
pub struct ReactValidator {
    config: ReactConfig,
    class_component_regex: Regex,
}

impl ReactValidator {
    /// Create new React validator
    pub fn new(config: ReactConfig) -> Self {
        Self {
            config,
            class_component_regex: Regex::new(r"class\s+(\w+)\s+extends\s+React\.Component").unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(ReactConfig::default())
    }

    /// Check component size
    fn check_component_size(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        let line_count = source.lines().count();

        if line_count > self.config.max_component_lines {
            issues.push(Issue::warning(
                "react-component-size",
                format!(
                    "Component too large: {} lines (max: {})",
                    line_count, self.config.max_component_lines
                ),
            )
            .with_file(file)
            .with_suggestion("Split component into smaller pieces"));
        }

        issues
    }

    /// Check for class components (if function preferred)
    fn check_class_components(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        if self.config.prefer_function_components {
            for caps in self.class_component_regex.captures_iter(source) {
                if let Some(component) = caps.get(1) {
                    issues.push(Issue::warning(
                        "react-class-component",
                        format!("Class component '{}' should be a function component", component.as_str()),
                    )
                    .with_file(file)
                    .with_suggestion("Convert to function component with hooks"));
                }
            }
        }

        issues
    }
}

impl Validator for ReactValidator {
    fn name(&self) -> &str {
        "React"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        // Check all React rules
        for issue in self.check_component_size(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_class_components(source, &file_str) {
            result.add_issue(issue);
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
    fn test_react_validator_valid() {
        let validator = ReactValidator::default_config();
        let code = r#"
            function Button({ children }) {
                return <button>{children}</button>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("Button.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_react_class_component() {
        let validator = ReactValidator::default_config();
        let code = r#"
            class Button extends React.Component {
                render() {
                    return <button>Click</button>;
                }
            }
        "#;

        let result = validator.validate_raw(code, Path::new("Button.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect class component");
        assert_eq!(result.issues[0].code, "react-class-component");
    }

    #[test]
    fn test_react_config_default() {
        let config = ReactConfig::default();
        assert!(config.enforce_hooks_rules);
        assert_eq!(config.max_component_lines, 300);
        assert!(config.prefer_function_components);
    }

    #[test]
    fn test_react_component_size_warning() {
        let validator = ReactValidator::default_config();
        let large_component = "fn main() { }\n".repeat(301);

        let result = validator.validate_raw(&large_component, Path::new("Large.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect large component");
        assert_eq!(result.issues[0].code, "react-component-size");
    }

    #[test]
    fn test_react_custom_config() {
        let config = ReactConfig {
            max_component_lines: 50,
            prefer_function_components: false,
            ..Default::default()
        };

        let validator = ReactValidator::new(config);
        let code = r#"
            class Button extends React.Component {
                render() {
                    return <button>Click</button>;
                }
            }
        "#;

        let result = validator.validate_raw(code, Path::new("Button.tsx")).unwrap();
        // Should pass when not enforcing function components
        assert!(result.passed);
    }

    #[test]
    fn test_react_arrow_function_component() {
        let validator = ReactValidator::default_config();
        let code = r#"
            const Button = ({ children }) => {
                return <button>{children}</button>;
            };
        "#;

        let result = validator.validate_raw(code, Path::new("Button.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_react_validator_supports() {
        let validator = ReactValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(validator.supports(Language::TypeScript));
        assert!(validator.supports(Language::Jsx));
        assert!(validator.supports(Language::Tsx));
        assert!(!validator.supports(Language::Rust));
    }

    #[test]
    fn test_react_multiple_class_components() {
        let validator = ReactValidator::default_config();
        let code = r#"
            class Button extends React.Component {
                render() { return <button>Click</button>; }
            }
            class Input extends React.Component {
                render() { return <input />; }
            }
        "#;

        let result = validator.validate_raw(code, Path::new("Components.tsx")).unwrap();
        assert_eq!(result.issues.len(), 2);
    }
}
