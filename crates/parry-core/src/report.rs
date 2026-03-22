//! Validation reports and output formatting

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

/// Severity level for rules and issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational notice
    Note,
    /// Warning - doesn't block in non-strict mode
    Warning,
    /// Error - blocks validation
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Note => write!(f, "note"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Issue level in report
pub type IssueLevel = Severity;

/// A single validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Severity level
    pub level: IssueLevel,

    /// Issue code (e.g., "tailwind-invalid-class")
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// File path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Line number (0-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,

    /// Column number (0-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,

    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,

    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl Issue {
    /// Create a new issue
    pub fn new(level: IssueLevel, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
            column: None,
            suggestion: None,
            context: None,
        }
    }

    /// Add file location
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add line location
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Add column location
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Create an error issue
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    /// Create a warning issue
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    /// Create a note issue
    pub fn note(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, code, message)
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,

    /// Issues found
    pub issues: Vec<Issue>,

    /// Number of files checked
    pub files_checked: usize,

    /// Duration of validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            passed: true,
            issues: Vec::new(),
            files_checked: 0,
            duration_ms: None,
        }
    }

    /// Add an issue
    pub fn add_issue(&mut self, issue: Issue) {
        if issue.level >= Severity::Error {
            self.passed = false;
        }
        self.issues.push(issue);
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.passed = self.passed && other.passed;
        self.files_checked += other.files_checked;
        self.issues.extend(other.issues);
    }

    /// Get the count of issues by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.issues.iter().filter(|i| i.level == severity).count()
    }

    /// Get total errors
    pub fn error_count(&self) -> usize {
        self.count_by_severity(Severity::Error)
    }

    /// Get total warnings
    pub fn warning_count(&self) -> usize {
        self.count_by_severity(Severity::Warning)
    }

    /// Finalize validation result with strict mode consideration
    /// In strict mode, warnings are treated as errors
    pub fn finalize_with_strict_mode(&mut self, strict_mode: bool) {
        if strict_mode && self.warning_count() > 0 {
            self.passed = false;
        }
    }

    /// Check if validation passes considering strict mode
    pub fn is_passing_with_strict(&self, strict_mode: bool) -> bool {
        if !self.passed {
            return false;
        }
        if strict_mode && self.warning_count() > 0 {
            return false;
        }
        true
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Parry version
    pub version: String,

    /// Validation results
    pub result: ValidationResult,

    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Report {
    /// Create a new report
    pub fn new(result: ValidationResult) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            result,
            timestamp: Some(chrono::Utc::now()),
        }
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to SARIF format (manual implementation)
    pub fn to_sarif(&self) -> std::result::Result<serde_json::Value, String> {
        use serde_json::json;

        let results: Vec<serde_json::Value> = self
            .result
            .issues
            .iter()
            .map(|issue| {
                let mut result = json!({
                    "ruleId": issue.code,
                    "level": match issue.level {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Note => "note",
                    },
                    "message": {
                        "text": issue.message
                    }
                });

                // Add location if available
                if let (Some(file), Some(line)) = (&issue.file, issue.line) {
                    let location = json!({
                        "physicalLocation": {
                            "artifactLocation": {
                                "filePath": file
                            },
                            "region": {
                                "startLine": line,
                                "startColumn": issue.column.unwrap_or(0)
                            }
                        }
                    });
                    result["locations"] = json!([location]);
                }

                result
            })
            .collect();

        Ok(json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "Parry",
                        "version": env!("CARGO_PKG_VERSION"),
                        "informationUri": "https://github.com/yourusername/parry"
                    }
                },
                "results": results
            }]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_creation() {
        let issue = Issue::error("test-code", "test message")
            .with_file("test.ts")
            .with_line(10)
            .with_column(5)
            .with_suggestion("fix it");

        assert_eq!(issue.code, "test-code");
        assert_eq!(issue.level, Severity::Error);
        assert_eq!(issue.file, Some("test.ts".to_string()));
        assert_eq!(issue.line, Some(10));
        assert_eq!(issue.column, Some(5));
        assert_eq!(issue.suggestion, Some("fix it".to_string()));
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.passed);

        result.add_issue(Issue::warning("warn", "warning"));
        assert!(result.passed); // Still passed with only warning

        result.add_issue(Issue::error("err", "error"));
        assert!(!result.passed); // Failed with error
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_issue_note() {
        let issue = Issue::note("note-code", "just a note");
        assert_eq!(issue.level, Severity::Note);
        assert_eq!(issue.code, "note-code");
    }

    #[test]
    fn test_issue_warning() {
        let issue = Issue::warning("warn-code", "warning message");
        assert_eq!(issue.level, Severity::Warning);
        assert_eq!(issue.code, "warn-code");
    }

    #[test]
    fn test_issue_with_context() {
        let issue = Issue::error("err", "error")
            .with_context("context info");

        assert_eq!(issue.context, Some("context info".to_string()));
    }

    #[test]
    fn test_issue_serialization() {
        let issue = Issue::error("test", "message")
            .with_file("test.ts")
            .with_line(5);

        let json = serde_json::to_string(&issue);
        assert!(json.is_ok());

        let parsed: Issue = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(parsed.code, "test");
        assert_eq!(parsed.level, Severity::Error);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_issue(Issue::error("err1", "error 1"));

        let mut result2 = ValidationResult::new();
        result2.add_issue(Issue::error("err2", "error 2"));

        result1.merge(result2);

        assert_eq!(result1.error_count(), 2);
        assert!(!result1.passed);
    }

    #[test]
    fn test_validation_result_count_by_severity() {
        let mut result = ValidationResult::new();

        result.add_issue(Issue::error("e1", "error 1"));
        result.add_issue(Issue::error("e2", "error 2"));
        result.add_issue(Issue::warning("w1", "warning 1"));
        result.add_issue(Issue::note("n1", "note 1"));

        assert_eq!(result.count_by_severity(Severity::Error), 2);
        assert_eq!(result.count_by_severity(Severity::Warning), 1);
        assert_eq!(result.count_by_severity(Severity::Note), 1);
    }

    #[test]
    fn test_validation_result_files_checked() {
        let mut result = ValidationResult::new();
        result.files_checked = 5;
        assert_eq!(result.files_checked, 5);

        let mut result2 = ValidationResult::new();
        result2.files_checked = 3;

        result.merge(result2);
        assert_eq!(result.files_checked, 8);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Note.to_string(), "note");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Note);
        assert!(Severity::Error > Severity::Note);
    }

    #[test]
    fn test_report_creation() {
        let mut result = ValidationResult::new();
        result.add_issue(Issue::error("test", "test error"));

        let report = Report::new(result);
        assert!(!report.result.passed);
        assert!(report.timestamp.is_some());
        assert!(!report.version.is_empty());
    }

    #[test]
    fn test_report_to_json() {
        let result = ValidationResult::new();
        let report = Report::new(result);

        let json = report.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_report_to_sarif() {
        let mut result = ValidationResult::new();
        result.add_issue(
            Issue::error("test-error", "test message")
                .with_file("test.ts")
                .with_line(10)
                .with_column(5)
        );

        let report = Report::new(result);
        let sarif = report.to_sarif();
        assert!(sarif.is_ok());

        let sarif_value = sarif.unwrap();
        assert_eq!(sarif_value["version"], "2.1.0");
        assert!(sarif_value["runs"].is_array());
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.passed);
        assert!(result.issues.is_empty());
        assert_eq!(result.files_checked, 0);
    }

    #[test]
    fn test_warning_only_still_passes() {
        let mut result = ValidationResult::new();
        result.add_issue(Issue::warning("warn", "warning"));
        result.add_issue(Issue::note("note", "note"));

        assert!(result.passed);
        assert_eq!(result.warning_count(), 1);
    }
}
