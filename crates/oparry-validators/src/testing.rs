//! Testing validator - Test coverage and quality checks

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::path::{Path, PathBuf};

/// Testing validation configuration
#[derive(Debug, Clone)]
pub struct TestingConfig {
    /// Minimum test coverage percentage
    pub min_coverage: usize,
    /// Require test files for source files
    pub require_test_files: bool,
    /// Block skipping tests
    pub block_skip_tests: bool,
    /// Require assertions in tests
    pub require_assertions: bool,
    /// Test file patterns
    pub test_file_patterns: Vec<String>,
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            min_coverage: 80,
            require_test_files: true,
            block_skip_tests: true,
            require_assertions: true,
            test_file_patterns: vec![
                "*.test.ts".to_string(),
                "*.test.tsx".to_string(),
                "*.spec.ts".to_string(),
                "_test.rs".to_string(),
            ],
        }
    }
}

/// Testing validator
pub struct TestingValidator {
    config: TestingConfig,
    skip_regex: Regex,
    assertion_regex: Regex,
    test_fn_regex: Regex,
}

impl TestingValidator {
    pub fn new(config: TestingConfig) -> Self {
        Self {
            config,
            skip_regex: Regex::new(r"(skip|describe\.skip|test\.skip|it\.skip)").unwrap(),
            assertion_regex: Regex::new(r"(expect|assert|assertEq)").unwrap(),
            test_fn_regex: Regex::new(r#"(test|it|describe)\s*\(['"]"#).unwrap(),
        }
    }

    pub fn default_config() -> Self {
        Self::new(TestingConfig::default())
    }

    fn check_skipped_tests(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.block_skip_tests {
            return issues;
        }

        for (idx, line) in source.lines().enumerate() {
            if self.skip_regex.is_match(line) {
                issues.push(Issue::warning(
                    "test-skipped",
                    "Skipped test detected",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Remove skip or fix the test"));
            }
        }
        issues
    }

    fn check_missing_assertions(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.require_assertions {
            return issues;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut in_test = false;
        let mut test_has_assertion = false;

        for (idx, line) in lines.iter().enumerate() {
            if self.test_fn_regex.is_match(line) {
                in_test = true;
                test_has_assertion = false;
            } else if in_test {
                if self.assertion_regex.is_match(line) {
                    test_has_assertion = true;
                }

                if line.contains("});") || line.contains(");") {
                    if !test_has_assertion {
                        issues.push(Issue::warning(
                            "test-no-assertion",
                            "Test without any assertion detected",
                        )
                        .with_file(file)
                        .with_line(idx + 1)
                        .with_suggestion("Add expect/assert to verify test behavior"));
                    }
                    in_test = false;
                }
            }
        }
        issues
    }

    fn is_test_file(&self, path: &str) -> bool {
        path.contains(".test.") || path.contains(".spec.") || path.contains("_test.")
    }
}

impl Validator for TestingValidator {
    fn name(&self) -> &str {
        "Testing"
    }

    fn supports(&self, _language: Language) -> bool {
        true
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();
        let file_str = file.to_string_lossy().to_string();

        if self.is_test_file(&file_str) {
            for issue in self.check_skipped_tests(source, &file_str) {
                result.add_issue(issue);
            }
            for issue in self.check_missing_assertions(source, &file_str) {
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
    fn test_testing_validator_valid() {
        let validator = TestingValidator::default_config();
        let code = r#"test("adds numbers", () => { expect(add(1, 2)).toBe(3); });"#;
        let result = validator.validate_raw(code, Path::new("sum.test.ts")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_testing_skipped() {
        let validator = TestingValidator::default_config();
        let code = r#"test.skip("skipped test", () => { });"#;
        let result = validator.validate_raw(code, Path::new("test.test.ts")).unwrap();
        assert!(!result.passed || result.warning_count() >= 1);
    }

    #[test]
    fn test_testing_config_default() {
        let config = TestingConfig::default();
        assert_eq!(config.min_coverage, 80);
        assert!(config.require_test_files);
    }

    #[test]
    fn test_testing_validator_supports() {
        let validator = TestingValidator::default_config();
        assert!(validator.supports(Language::TypeScript));
        assert!(validator.supports(Language::Rust));
    }
}

