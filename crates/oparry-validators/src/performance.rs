//! Performance validator - React performance patterns and best practices

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// Performance validation configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Require React.memo for components passed as props
    pub require_memo_for_props: bool,
    /// Check for missing useMemo/useCallback
    pub check_hook_usage: bool,
    /// Detect missing lazy loading
    pub check_lazy_loading: bool,
    /// Warn against inline object/function creation in render
    pub warn_inline_creation: bool,
    /// Maximum component re-renders threshold
    pub max_render_complexity: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            require_memo_for_props: true,
            check_hook_usage: true,
            check_lazy_loading: true,
            warn_inline_creation: true,
            max_render_complexity: 100,
        }
    }
}

/// Performance validator
pub struct PerformanceValidator {
    config: PerformanceConfig,
    component_regex: Regex,
    useeffect_regex: Regex,
}

impl PerformanceValidator {
    /// Create new performance validator
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            component_regex: Regex::new(r"(?:function|const)\s+(\w+)\s*(?:\(|=|\(\))").unwrap(),
            useeffect_regex: Regex::new(r"useEffect\s*\(").unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(PerformanceConfig::default())
    }

    /// Check for data fetching in useEffect
    fn check_useeffect_fetching(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if !self.config.check_hook_usage {
            return issues;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut in_useeffect = false;
        let mut useEffect_depth = 0;

        for (idx, line) in lines.iter().enumerate() {
            if self.useeffect_regex.is_match(line) {
                in_useeffect = true;
                useEffect_depth = 1;
                // Also check this line for fetch/axios
                if line.contains("fetch(") || line.contains("axios.") {
                    issues.push(Issue::warning(
                        "perf-useeffect-fetch",
                        "Data fetching in useEffect - prefer React Query/SWR",
                    )
                    .with_file(file)
                    .with_line(idx + 1)
                    .with_suggestion("Use @tanstack/react-query for data fetching"));
                }
            } else if in_useeffect {
                useEffect_depth += line.matches('{').count() as i32;
                useEffect_depth -= line.matches('}').count() as i32;

                if line.contains("fetch(") || line.contains("axios.") {
                    issues.push(Issue::warning(
                        "perf-useeffect-fetch",
                        "Data fetching in useEffect - prefer React Query/SWR",
                    )
                    .with_file(file)
                    .with_line(idx + 1)
                    .with_suggestion("Use @tanstack/react-query for data fetching"));
                }

                if useEffect_depth <= 0 {
                    in_useeffect = false;
                }
            }
        }
        issues
    }

    /// Check for missing key props in lists
    fn check_missing_keys(&self, source: &str, file: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        let key_regex = Regex::new(r#"\.map\s*\([^)]*\)\s*=>\s*<"#).unwrap();

        for (idx, line) in source.lines().enumerate() {
            if key_regex.is_match(line) && !line.contains("key=") {
                issues.push(Issue::warning(
                    "perf-missing-key",
                    "Missing 'key' prop in list rendering",
                )
                .with_file(file)
                .with_line(idx + 1)
                .with_suggestion("Add unique key prop for efficient rendering"));
            }
        }
        issues
    }
}

impl Validator for PerformanceValidator {
    fn name(&self) -> &str {
        "Performance"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();
        let file_str = file.to_string_lossy().to_string();

        for issue in self.check_useeffect_fetching(source, &file_str) {
            result.add_issue(issue);
        }
        for issue in self.check_missing_keys(source, &file_str) {
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
    fn test_performance_validator_valid() {
        let validator = PerformanceValidator::default_config();
        let code = r#"function Button({ children }) { return <button>{children}</button>; }"#;
        let result = validator.validate_raw(code, Path::new("Button.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_perf_useeffect_fetch() {
        let validator = PerformanceValidator::default_config();
        let code = r#"useEffect(() => { fetch('/api/data').then(r => r.json()); }, []);"#;
        let result = validator.validate_raw(code, Path::new("Data.tsx")).unwrap();
        assert!(!result.passed || result.warning_count() >= 1);
    }

    #[test]
    fn test_perf_validator_supports() {
        let validator = PerformanceValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(!validator.supports(Language::Rust));
    }
}
