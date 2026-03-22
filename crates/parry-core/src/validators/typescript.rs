//! TypeScript validator - TypeScript strict mode and type safety

use crate::validators::Validator;
use crate::{Issue, IssueLevel, Result, ValidationResult};
use crate::parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// TypeScript validation configuration
#[derive(Debug, Clone)]
pub struct TypeScriptConfig {
    /// Block 'any' type usage
    pub block_any: bool,
    /// Block type assertions (as)
    pub block_assertions: bool,
    /// Require explicit return types
    pub require_return_types: bool,
    /// Require strict null checks
    pub require_strict_nulls: bool,
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self {
            block_any: true,
            block_assertions: false,
            require_return_types: false,
            require_strict_nulls: true,
        }
    }
}

/// TypeScript validator
pub struct TypeScriptValidator {
    config: TypeScriptConfig,
    any_regex: Regex,
    type_assertion_regex: Regex,
    non_null_regex: Regex,
}

impl TypeScriptValidator {
    /// Create new TypeScript validator
    pub fn new(config: TypeScriptConfig) -> Self {
        Self {
            config,
            any_regex: Regex::new(r":\s*any\b|<any>").unwrap(),
            type_assertion_regex: Regex::new(r"\s+as\s+\w+").unwrap(),
            non_null_regex: Regex::new(r"!\s*[,\);]").unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(TypeScriptConfig::default())
    }

    /// Check for 'any' type usage
    fn check_any_usage(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.block_any {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            if self.any_regex.is_match(line) {
                issues.push(Issue::warning(
                    "ts-any-type",
                    "'any' type detected - defeats type safety",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Use specific type or unknown with type guards"));
            }
        }
        issues
    }

    /// Check for non-null assertions
    fn check_non_null_assertions(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.require_strict_nulls {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if self.non_null_regex.is_match(line) {
                issues.push(Issue::warning(
                    "ts-non-null-assertion",
                    "Non-null assertion (!) detected",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Use optional chaining (?.) or nullish coalescing (??)"));
            }
        }
        issues
    }
}

impl Validator for TypeScriptValidator {
    fn name(&self) -> &str {
        "TypeScript"
    }

    fn supports(&self, language: Language) -> bool {
        matches!(language, Language::TypeScript | Language::Tsx | Language::Jsx)
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();
        let file_str = file.to_string_lossy().to_string();

        for issue in self.check_any_usage(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_non_null_assertions(source, &file_str) {
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
    fn test_ts_validator_valid() {
        let validator = TypeScriptValidator::default_config();
        let code = r#"function greet(name: string): string { return name; }"#;
        let result = validator.validate_raw(code, Path::new("greet.ts")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_ts_any_type() {
        let validator = TypeScriptValidator::default_config();
        let code = r#"function process(data: any): void { }"#;
        let result = validator.validate_raw(code, Path::new("process.ts")).unwrap();
        assert!(!result.passed || result.warning_count() >= 1);
    }

    #[test]
    fn test_ts_validator_supports() {
        let validator = TypeScriptValidator::default_config();
        assert!(validator.supports(Language::TypeScript));
        assert!(!validator.supports(Language::Rust));
    }
}
