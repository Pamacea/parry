//! Security validator - Security patterns and vulnerabilities

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// Security validation configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Block dangerous innerHTML usage
    pub block_innerhtml: bool,
    /// Block eval usage
    pub block_eval: bool,
    /// Block dangerous APIs
    pub block_dangerous_apis: bool,
    /// Check for hardcoded secrets
    pub check_secrets: bool,
    /// Warn on unsafe DOM manipulation
    pub warn_unsafe_dom: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            block_innerhtml: true,
            block_eval: true,
            block_dangerous_apis: true,
            check_secrets: true,
            warn_unsafe_dom: true,
        }
    }
}

/// Security validator
pub struct SecurityValidator {
    config: SecurityConfig,
    innerhtml_regex: Regex,
    eval_regex: Regex,
    dangerous_dom_regex: Regex,
    secret_pattern_regex: Regex,
}

impl SecurityValidator {
    /// Create new security validator
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            innerhtml_regex: Regex::new(r"dangerouslySetInnerHTML|innerHTML\s*=").unwrap(),
            eval_regex: Regex::new(r#"\beval\s*\(|new\s+Function\s*\("#).unwrap(),
            dangerous_dom_regex: Regex::new(r#"\b(document\.write|outerHTML\s*=)"#).unwrap(),
            secret_pattern_regex: Regex::new(
                r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"][\w-]{20,}"#
            ).unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(SecurityConfig::default())
    }

    /// Check for dangerous innerHTML usage
    fn check_innerhtml(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.block_innerhtml {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if self.innerhtml_regex.is_match(line) {
                issues.push(Issue::error(
                    "sec-dangerous-innerhtml",
                    "Dangerous innerHTML usage detected - XSS vulnerability",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Use React text or DOMParser with sanitization"));
            }
        }
        issues
    }

    /// Check for eval usage
    fn check_eval(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.block_eval {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if self.eval_regex.is_match(line) {
                issues.push(Issue::error(
                    "sec-eval-usage",
                    "eval() or new Function() detected - code injection risk",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Never use eval - find safer alternative"));
            }
        }
        issues
    }

    /// Check for hardcoded secrets
    fn check_secrets(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.check_secrets {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            // Skip comment lines
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#") {
                continue;
            }

            if self.secret_pattern_regex.is_match(line) {
                issues.push(Issue::error(
                    "sec-hardcoded-secret",
                    "Possible hardcoded secret detected",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Move secrets to environment variables"));
            }
        }
        issues
    }

    /// Check for unsafe DOM manipulation
    fn check_unsafe_dom(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.warn_unsafe_dom {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if self.dangerous_dom_regex.is_match(line) {
                issues.push(Issue::warning(
                    "sec-unsafe-dom",
                    "Unsafe DOM manipulation detected",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Use React state and refs instead"));
            }
        }
        issues
    }
}

impl Validator for SecurityValidator {
    fn name(&self) -> &str {
        "Security"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();
        let file_str = file.to_string_lossy().to_string();

        for issue in self.check_innerhtml(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_eval(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_secrets(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_unsafe_dom(source, &file_str) {
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
    fn test_security_validator_valid() {
        let validator = SecurityValidator::default_config();
        let code = r#"function Safe() { return <div>{content}</div>; }"#;
        let result = validator.validate_raw(code, Path::new("Safe.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_sec_innerhtml() {
        let validator = SecurityValidator::default_config();
        let code = r#"<div dangerouslySetInnerHTML={{ __html: html }} />"#;
        let result = validator.validate_raw(code, Path::new("Unsafe.tsx")).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_sec_eval() {
        let validator = SecurityValidator::default_config();
        let code = r#"eval(userInput)"#;
        let result = validator.validate_raw(code, Path::new("Bad.ts")).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_sec_hardcoded_secret() {
        let validator = SecurityValidator::default_config();
        let code = r#"const apiKey = 'sk-1234567890abcdef1234567890abcdef'"#;
        let result = validator.validate_raw(code, Path::new("config.ts")).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_security_validator_supports() {
        let validator = SecurityValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(!validator.supports(Language::Rust));
    }
}
