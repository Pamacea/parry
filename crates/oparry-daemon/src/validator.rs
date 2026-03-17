//! Daemon validator - validates and enforces rules for Claude Code sessions

use crate::claude::ClaudeSession;
use crate::config::DaemonConfig;
use oparry_core::{Result, ValidationResult};
use oparry_parser::parser_for_path;
use oparry_validators::{
    css::CssConfig, imports::ImportConfig, react::ReactConfig, rust::RustConfig,
    tailwind::TailwindConfig,
};
use oparry_validators::{
    CssValidator, ImportValidator, ReactValidator, RustValidator, TailwindValidator, Validators,
};
use tokio::fs;

/// Daemon validator
pub struct DaemonValidator {
    config: DaemonConfig,
    validators: Validators,
}

impl DaemonValidator {
    /// Create new daemon validator
    pub fn new(config: DaemonConfig) -> Self {
        let validators = Self::build_validators(&config);
        Self { config, validators }
    }

    /// Build validators from config
    fn build_validators(_config: &DaemonConfig) -> Validators {
        let mut validators = Validators::new();

        // Tailwind with blocked widths
        let tw_config = TailwindConfig {
            blocked_widths: vec![
                "w-xl".to_string(),
                "w-2xl".to_string(),
                "w-3xl".to_string(),
                "w-4xl".to_string(),
                "w-5xl".to_string(),
                "w-6xl".to_string(),
                "w-7xl".to_string(),
            ],
            blocked_max_widths: vec![
                "max-w-sm".to_string(),
                "max-w-md".to_string(),
                "max-w-lg".to_string(),
                "max-w-xl".to_string(),
                "max-w-2xl".to_string(),
                "max-w-3xl".to_string(),
                "max-w-4xl".to_string(),
                "max-w-5xl".to_string(),
                "max-w-6xl".to_string(),
                "max-w-7xl".to_string(),
            ],
            ..Default::default()
        };
        validators = validators.with_tailwind(TailwindValidator::new(tw_config));

        // Import validator
        let imp_config = ImportConfig {
            enforce_alias: true,
            alias_map: {
                let mut map = std::collections::HashMap::new();
                map.insert("@/".to_string(), "./src".to_string());
                map.insert("@/components".to_string(), "./components".to_string());
                map.insert("@/lib".to_string(), "./lib".to_string());
                map
            },
            ..Default::default()
        };
        validators = validators.with_imports(ImportValidator::new(imp_config));

        // React validator
        let react_config = ReactConfig {
            max_component_lines: 300,
            prefer_function_components: true,
            ..Default::default()
        };
        validators = validators.with_react(ReactValidator::new(react_config));

        // CSS validator
        let css_config = CssConfig {
            max_line_length: 80,
            block_important: true,
            ..Default::default()
        };
        validators = validators.with_css(CssValidator::new(css_config));

        // Rust validator
        let rust_config = RustConfig {
            warn_unwrap: true,
            enforce_result_handling: true,
            ..Default::default()
        };
        validators = validators.with_rust(RustValidator::new(rust_config));

        validators
    }

    /// Validate a Claude Code session
    pub async fn validate_session(&self, session: &ClaudeSession) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Validate all active files in session
        for file in &session.active_files {
            if let Ok(content) = fs::read_to_string(file).await {
                let parser = parser_for_path(file);
                if let Ok(parsed) = parser.parse(&content) {
                    if let Ok(result) = self.validators.validate(&parsed, file) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get validation report for a session
    pub async fn session_report(&self, session: &ClaudeSession) -> Result<String> {
        let results: Vec<oparry_core::ValidationResult> = self.validate_session(session).await?;

        let total_issues: usize = results.iter().map(|r| r.issues.len()).sum();

        if total_issues == 0 {
            Ok("✓ No issues found".to_string())
        } else {
            let mut report = format!("⚠️ Found {} issues:\n\n", total_issues);

            for result in results {
                for issue in &result.issues {
                    report.push_str(&format!(
                        "{}: {} in {:?}\n",
                        issue.level, issue.message, issue.file
                    ));
                    if let Some(ref suggestion) = issue.suggestion {
                        report.push_str(&format!("   Suggestion: {}\n", suggestion));
                    }
                }
            }

            Ok(report)
        }
    }

    /// Auto-correct issues if possible (future feature)
    pub async fn auto_correct(&self, _session: &ClaudeSession) -> Result<Vec<String>> {
        // For MVP, return empty list
        // In future, this would automatically fix fixable issues
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_creation() {
        let config = DaemonConfig::default();
        let validator = DaemonValidator::new(config);
        assert_eq!(validator.validators.validators().len(), 5); // tailwind, imports, react, css, rust
    }
}
