//! Auto-fix engine for Parry

mod rules;
mod fixer;
mod config;

pub use config::{AutoFixConfig, FixStrategy};
pub use fixer::{AutoFixEngine, FixResult, FixApplication};
pub use rules::{FixRule, FixRuleRegistry, FixKind};

use crate::{Issue, IssueLevel, Result};
use crate::parser::{ParsedCode, Language};
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

    /// Generate fixes for the given issues
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
                matches!(issue.level, IssueLevel::Note)
            }
            FixStrategy::Moderate => {
                matches!(issue.level, IssueLevel::Note | IssueLevel::Warning)
            }
            FixStrategy::Aggressive => {
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
}
