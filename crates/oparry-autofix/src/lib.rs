//! Auto-fix engine for Parry
//!
//! This crate provides automatic code correction capabilities,
//! generating fixes for common issues and reinjecting them into
//! Claude Code's workflow.

mod rules;
mod fixer;
mod config;

pub use config::{AutoFixConfig, FixStrategy};
pub use fixer::{AutoFixEngine, FixResult, FixApplication};
pub use rules::{FixRule, FixRuleRegistry, FixKind};

use oparry_core::{Issue, IssueLevel, Result};
use oparry_parser::{ParsedCode, Language};
use std::path::Path;

/// Main auto-fixer that orchestrates rule application
pub struct AutoFixer {
    /// Fix engine
    engine: AutoFixEngine,
    /// Configuration
    config: AutoFixConfig,
}

impl AutoFixer {
    /// Create a new auto-fixer with default configuration
    pub fn new() -> Self {
        Self {
            engine: AutoFixEngine::with_default_rules(),
            config: AutoFixConfig::default(),
        }
    }

    /// Create auto-fixer with custom configuration
    pub fn with_config(config: AutoFixConfig) -> Self {
        Self {
            engine: AutoFixEngine::with_default_rules(),
            config,
        }
    }

    /// Create auto-fixer with custom rules
    pub fn with_rules(mut self, _rules: Vec<Box<dyn crate::rules::FixRule>>) -> Self {
        // Note: Custom rules require registry access, keeping signature for future use
        self
    }

    /// Generate fixes for the given issues
    ///
    /// Returns a FixApplication containing the corrected content
    pub fn fix_issues(
        &self,
        source: &str,
        issues: &[Issue],
        language: Language,
        file: &Path,
    ) -> Result<FixApplication> {
        // Filter fixable issues based on strategy
        let fixable_issues: Vec<_> = issues
            .iter()
            .filter(|issue| self.is_fixable(issue))
            .collect();

        if fixable_issues.is_empty() {
            return Ok(FixApplication::none());
        }

        // Parse the source code for structured fixes
        let parsed = ParsedCode::Generic(source.to_string());

        // Apply fixes
        let fixes = self.engine.generate_fixes(&parsed, &fixable_issues, file)?;

        // Apply fixes to source
        let modified_content = if self.config.dry_run {
            // In dry-run mode, return original content
            source.to_string()
        } else {
            self.apply_fixes(source, &fixes)?
        };

        Ok(FixApplication {
            original: source.to_string(),
            modified: modified_content,
            fixes_applied: fixes.len(),
            issues_fixed: fixable_issues.len(),
            can_fix_all: fixable_issues.len() == issues.len(),
        })
    }

    /// Check if an issue is fixable
    fn is_fixable(&self, issue: &Issue) -> bool {
        match self.config.strategy {
            FixStrategy::Safe => {
                // Only fix low-risk issues
                matches!(issue.level, IssueLevel::Note)
            }
            FixStrategy::Moderate => {
                // Fix notes and warnings
                matches!(issue.level, IssueLevel::Note | IssueLevel::Warning)
            }
            FixStrategy::Aggressive => {
                // Fix everything including errors (when safe)
                !issue.code.contains("syntax")
                    && !issue.code.contains("parse")
            }
        }
    }

    /// Apply fixes to source code
    fn apply_fixes(&self, source: &str, fixes: &[String]) -> Result<String> {
        let mut result = source.to_string();

        // Apply fixes in reverse order to maintain line numbers
        for fix in fixes.iter().rev() {
            // Parse the fix JSON
            if let Ok(fix_data) = serde_json::from_str::<serde_json::Value>(fix) {
                if let (Some(start), Some(end), Some(replacement)) = (
                    fix_data.get("start").and_then(|v| v.as_u64()),
                    fix_data.get("end").and_then(|v| v.as_u64()),
                    fix_data.get("replacement").and_then(|v| v.as_str()),
                ) {
                    let start = start as usize;
                    let end = end as usize;

                    if end <= result.len() {
                        result.replace_range(start..end, replacement);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Generate a single fix for an issue (for preview)
    pub fn preview_fix(&self, issue: &Issue, source: &str) -> Option<String> {
        self.engine.preview_fix(issue, source)
    }
}

impl Default for AutoFixer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of applying auto-fixes
#[derive(Debug, Clone)]
pub struct AutoFixResult {
    /// Whether any fixes were applied
    pub has_fixes: bool,
    /// Number of issues fixed
    pub fixed_count: usize,
    /// Number of issues that couldn't be fixed
    pub unfixed_count: usize,
    /// Modified content (if any)
    pub modified_content: Option<String>,
    /// Individual fix results
    pub fixes: Vec<SingleFixResult>,
}

/// Result of a single fix application
#[derive(Debug, Clone)]
pub struct SingleFixResult {
    /// Issue code that was fixed
    pub issue_code: String,
    /// Whether the fix was successful
    pub success: bool,
    /// Original content
    pub original: String,
    /// Fixed content
    pub fixed: String,
    /// Line number of the fix
    pub line: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autofixer_new() {
        let fixer = AutoFixer::new();
        assert_eq!(fixer.config.strategy, FixStrategy::Moderate);
    }

    #[test]
    fn test_autofixer_default() {
        let fixer = AutoFixer::default();
        assert_eq!(fixer.config.strategy, FixStrategy::Moderate);
    }

    #[test]
    fn test_is_fixable_safe_strategy() {
        let fixer = AutoFixer::with_config(AutoFixConfig {
            strategy: FixStrategy::Safe,
            ..Default::default()
        });

        let note = Issue::note("test", "test");
        let warning = Issue::warning("test", "test");
        let error = Issue::error("test", "test");

        assert!(fixer.is_fixable(&note));
        assert!(!fixer.is_fixable(&warning));
        assert!(!fixer.is_fixable(&error));
    }

    #[test]
    fn test_is_fixable_moderate_strategy() {
        let fixer = AutoFixer::with_config(AutoFixConfig {
            strategy: FixStrategy::Moderate,
            ..Default::default()
        });

        let note = Issue::note("test", "test");
        let warning = Issue::warning("test", "test");
        let error = Issue::error("test", "test");

        assert!(fixer.is_fixable(&note));
        assert!(fixer.is_fixable(&warning));
        assert!(!fixer.is_fixable(&error));
    }

    #[test]
    fn test_is_fixable_aggressive_strategy() {
        let fixer = AutoFixer::with_config(AutoFixConfig {
            strategy: FixStrategy::Aggressive,
            dry_run: false,
            max_fixes: 100,
            preserve_formatting: true,
        });

        let normal_error = Issue::error("test", "test");
        let syntax_error = Issue::error("syntax-error", "test");

        assert!(fixer.is_fixable(&normal_error));
        assert!(!fixer.is_fixable(&syntax_error));
    }

    #[test]
    fn test_fix_application_none() {
        let application = FixApplication::none();
        assert!(application.fixes_applied == 0);
        assert!(!application.has_changes());
    }

    #[test]
    fn test_fix_result_empty() {
        let result = AutoFixResult {
            has_fixes: false,
            fixed_count: 0,
            unfixed_count: 0,
            modified_content: None,
            fixes: vec![],
        };
        assert!(!result.has_fixes);
    }
}
