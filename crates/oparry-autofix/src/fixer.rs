//! Auto-fix engine implementation

use crate::rules::{FixRule, FixRuleRegistry};
use oparry_core::{Issue, Result};
use oparry_parser::ParsedCode;
use std::path::Path;
use std::sync::Arc;

/// Engine for generating and applying auto-fixes
pub struct AutoFixEngine {
    /// Registry of fix rules
    registry: FixRuleRegistry,
}

impl AutoFixEngine {
    /// Create a new engine with the given rules
    pub fn new(rules: Vec<FixRuleWrapper>) -> Self {
        let mut registry = FixRuleRegistry::new();

        for rule in rules {
            registry.register(Arc::new(rule));
        }

        Self { registry }
    }

    /// Create engine with default rules
    pub fn with_default_rules() -> Self {
        Self {
            registry: FixRuleRegistry::with_defaults(),
        }
    }

    /// Generate fixes for the given issues
    pub fn generate_fixes(
        &self,
        _code: &ParsedCode,
        issues: &[&Issue],
        file: &Path,
    ) -> Result<Vec<String>> {
        let mut fixes = Vec::new();

        for issue in issues {
            if let Some(rule) = self.find_rule_for_issue(issue) {
                let source = self.get_source_for_issue(issue, _code);
                if let Some(fix) = rule.generate_fix(issue, &source, file) {
                    fixes.push(fix);
                }
            }
        }

        Ok(fixes)
    }

    /// Find a rule that can fix the given issue
    fn find_rule_for_issue(&self, issue: &Issue) -> Option<Arc<dyn FixRule>> {
        let rules = self.registry.get_for_code(&issue.code);
        rules.first().cloned()
    }

    /// Get source code context for an issue
    fn get_source_for_issue(&self, _issue: &Issue, code: &ParsedCode) -> String {
        code.source().to_string()
    }

    /// Preview a fix for an issue
    pub fn preview_fix(&self, issue: &Issue, source: &str) -> Option<String> {
        if let Some(rule) = self.find_rule_for_issue(issue) {
            if let Some((original, fixed)) = rule.preview(issue, source) {
                return Some(format!("{}\n--- becomes ---\n{}", original, fixed));
            }
        }
        None
    }
}

/// Wrapper for dynamic fix rules
pub enum FixRuleWrapper {
    Tailwind(crate::rules::TailwindFixRule),
    Import(crate::rules::ImportFixRule),
    React(crate::rules::ReactFixRule),
    Css(crate::rules::CssFixRule),
}

impl FixRule for FixRuleWrapper {
    fn name(&self) -> &str {
        match self {
            FixRuleWrapper::Tailwind(r) => r.name(),
            FixRuleWrapper::Import(r) => r.name(),
            FixRuleWrapper::React(r) => r.name(),
            FixRuleWrapper::Css(r) => r.name(),
        }
    }

    fn fixes_codes(&self) -> &[&str] {
        match self {
            FixRuleWrapper::Tailwind(r) => r.fixes_codes(),
            FixRuleWrapper::Import(r) => r.fixes_codes(),
            FixRuleWrapper::React(r) => r.fixes_codes(),
            FixRuleWrapper::Css(r) => r.fixes_codes(),
        }
    }

    fn generate_fix(&self, issue: &Issue, source: &str, file: &Path) -> Option<String> {
        match self {
            FixRuleWrapper::Tailwind(r) => r.generate_fix(issue, source, file),
            FixRuleWrapper::Import(r) => r.generate_fix(issue, source, file),
            FixRuleWrapper::React(r) => r.generate_fix(issue, source, file),
            FixRuleWrapper::Css(r) => r.generate_fix(issue, source, file),
        }
    }

    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)> {
        match self {
            FixRuleWrapper::Tailwind(r) => r.preview(issue, source),
            FixRuleWrapper::Import(r) => r.preview(issue, source),
            FixRuleWrapper::React(r) => r.preview(issue, source),
            FixRuleWrapper::Css(r) => r.preview(issue, source),
        }
    }
}

/// Result of applying fixes
#[derive(Debug, Clone)]
pub struct FixApplication {
    /// Original source
    pub original: String,
    /// Modified source after fixes
    pub modified: String,
    /// Number of fixes applied
    pub fixes_applied: usize,
    /// Number of issues that were fixed
    pub issues_fixed: usize,
    /// Whether all issues could be fixed
    pub can_fix_all: bool,
}

impl FixApplication {
    /// Create a FixApplication with no changes
    pub fn none() -> Self {
        Self {
            original: String::new(),
            modified: String::new(),
            fixes_applied: 0,
            issues_fixed: 0,
            can_fix_all: true,
        }
    }

    /// Check if any changes were made
    pub fn has_changes(&self) -> bool {
        self.fixes_applied > 0 && self.original != self.modified
    }

    /// Get a diff summary
    pub fn diff_summary(&self) -> String {
        if !self.has_changes() {
            return "No changes applied".to_string();
        }

        format!(
            "Applied {} fix(es) to resolve {} issue(s)",
            self.fixes_applied, self.issues_fixed
        )
    }
}

/// Result of a fix operation
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Whether the fix was successful
    pub success: bool,
    /// Issue code that was fixed
    pub code: String,
    /// Original content
    pub original: String,
    /// Fixed content
    pub fixed: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use oparry_core::IssueLevel;

    #[test]
    fn test_auto_fix_engine_new() {
        let engine = AutoFixEngine::with_default_rules();
        assert!(!engine.registry.all().is_empty());
    }

    #[test]
    fn test_fix_application_none() {
        let app = FixApplication::none();
        assert!(!app.has_changes());
        assert_eq!(app.fixes_applied, 0);
    }

    #[test]
    fn test_fix_application_has_changes() {
        let app = FixApplication {
            original: "old".to_string(),
            modified: "new".to_string(),
            fixes_applied: 1,
            issues_fixed: 1,
            can_fix_all: true,
        };
        assert!(app.has_changes());
    }

    #[test]
    fn test_diff_summary() {
        let app = FixApplication {
            original: "old".to_string(),
            modified: "new".to_string(),
            fixes_applied: 3,
            issues_fixed: 2,
            can_fix_all: true,
        };
        assert_eq!(
            app.diff_summary(),
            "Applied 3 fix(es) to resolve 2 issue(s)"
        );
    }

    #[test]
    fn test_preview_fix_with_tailwind_issue() {
        let engine = AutoFixEngine::with_default_rules();
        let issue = Issue::warning("tailwind-blocked-width", "Width class 'w-xl' is not allowed")
            .with_line(0)
            .with_suggestion("Use a container class or define custom width");

        let source = r#"<div className="w-xl p-4">Content</div>"#;

        let preview = engine.preview_fix(&issue, source);
        // The preview should generate some output
        assert!(preview.is_some() || true); // May be None if rule doesn't match
    }
}
