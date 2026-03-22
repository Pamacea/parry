//! Rule engine and rule definitions

use crate::{Issue, IssueLevel, Result, ValidationResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Rule trait - all validators implement this
pub trait Rule: Send + Sync {
    /// Get the rule name
    fn name(&self) -> &str;

    /// Get the rule code
    fn code(&self) -> &str;

    /// Get the default severity
    fn severity(&self) -> IssueLevel;

    /// Validate input
    fn validate(&self, input: &str) -> Result<Vec<Issue>>;
}

/// Compiled rule with regex
#[derive(Clone)]
pub struct PatternRule {
    name: String,
    code: String,
    severity: IssueLevel,
    pattern: Regex,
    message: String,
    suggestion: Option<String>,
}

impl PatternRule {
    /// Create a new pattern rule
    pub fn new(
        name: impl Into<String>,
        code: impl Into<String>,
        severity: IssueLevel,
        pattern: &str,
        message: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            name: name.into(),
            code: code.into(),
            severity,
            pattern: Regex::new(pattern)?,
            message: message.into(),
            suggestion: None,
        })
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl Rule for PatternRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn severity(&self) -> IssueLevel {
        self.severity
    }

    fn validate(&self, input: &str) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        for mat in self.pattern.find_iter(input) {
            let mut issue = Issue::new(self.severity, self.code.clone(), self.message.clone())
                .with_context(mat.as_str().to_string());

            if let Some(ref suggestion) = self.suggestion {
                issue = issue.with_suggestion(suggestion.clone());
            }

            issues.push(issue);
        }

        Ok(issues)
    }
}

/// Rule engine that runs multiple rules
#[derive(Clone)]
pub struct RuleEngine {
    rules: Vec<Arc<dyn Rule>>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Arc<dyn Rule>) -> &mut Self {
        self.rules.push(rule);
        self
    }

    /// Add multiple rules
    pub fn extend_rules(&mut self, rules: impl IntoIterator<Item = Arc<dyn Rule>>) -> &mut Self {
        self.rules.extend(rules);
        self
    }

    /// Run all rules on input
    pub fn validate(&self, input: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        for rule in &self.rules {
            let issues = rule.validate(input)?;
            for mut issue in issues {
                // Ensure severity from rule
                if issue.level < rule.severity() {
                    issue.level = rule.severity();
                }
                result.add_issue(issue);
            }
        }

        Ok(result)
    }

    /// Get all rules
    pub fn rules(&self) -> &[Arc<dyn Rule>] {
        &self.rules
    }

    /// Count rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Rule name/code
    pub name: String,

    /// Enable/disable rule
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Severity override
    pub severity: Option<IssueLevel>,

    /// Custom patterns
    pub patterns: Vec<String>,

    /// Custom message
    pub message: Option<String>,

    /// Custom suggestion
    pub suggestion: Option<String>,
}

fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_rule() {
        let rule = PatternRule::new(
            "no-console",
            "no-console",
            IssueLevel::Warning,
            r"console\.(log|error|warn)\(",
            "Don't use console in production",
        )
        .unwrap()
        .with_suggestion("Use a proper logging library");

        let code = r#"
            function test() {
                console.log("debug");
                console.error("error");
            }
        "#;

        let issues = rule.validate(code).unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].code, "no-console");
    }

    #[test]
    fn test_rule_engine() {
        let mut engine = RuleEngine::new();

        let rule1 = PatternRule::new(
            "no-debugger",
            "no-debugger",
            IssueLevel::Error,
            r"debugger;",
            "Remove debugger statement",
        )
        .unwrap();

        let rule2 = PatternRule::new(
            "no-alert",
            "no-alert",
            IssueLevel::Warning,
            r"alert\(",
            "Don't use alert()",
        )
        .unwrap();

        engine.add_rule(Arc::new(rule1));
        engine.add_rule(Arc::new(rule2));

        let code = r#"
            debugger;
            alert("hello");
        "#;

        let result = engine.validate(code).unwrap();
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert!(!result.passed);
    }
}
