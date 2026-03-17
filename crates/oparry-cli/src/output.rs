//! Output formatters for different formats

use oparry_core::{Report, ValidationResult};
use colored::Colorize;

/// Output formatter trait
pub trait OutputFormatter {
    /// Format validation result
    fn format_result(&self, result: &ValidationResult) -> String;
    /// Format report
    fn format_report(&self, report: &Report) -> String;
}

/// Human-readable output formatter
pub struct HumanFormatter {
    show_paths: bool,
    use_colors: bool,
}

impl HumanFormatter {
    /// Create new human formatter
    pub fn new(show_paths: bool, use_colors: bool) -> Self {
        Self {
            show_paths,
            use_colors,
        }
    }
}

impl OutputFormatter for HumanFormatter {
    fn format_result(&self, result: &ValidationResult) -> String {
        let mut output = String::new();

        for issue in &result.issues {
            let icon = match issue.level {
                oparry_core::IssueLevel::Error => {
                    if self.use_colors {
                        "✗".red()
                    } else {
                        "✗".normal()
                    }
                }
                oparry_core::IssueLevel::Warning => {
                    if self.use_colors {
                        "⚠".yellow()
                    } else {
                        "⚠".normal()
                    }
                }
                oparry_core::IssueLevel::Note => {
                    if self.use_colors {
                        "ℹ".blue()
                    } else {
                        "ℹ".normal()
                    }
                }
            };

            let level_str = match issue.level {
                oparry_core::IssueLevel::Error => "error",
                oparry_core::IssueLevel::Warning => "warning",
                oparry_core::IssueLevel::Note => "note",
            };

            output.push_str(&format!("{} ", icon));

            if let Some(ref file) = issue.file {
                output.push_str(&format!("{} ", file));
            }

            output.push_str(&format!("{}\n", level_str));

            output.push_str(&format!("  --> {}\n", issue.message));

            if let Some(ref suggestion) = issue.suggestion {
                output.push_str(&format!("     suggestion: {}\n", suggestion));
            }

            output.push('\n');
        }

        // Summary
        let errors = result.error_count();
        let warnings = result.warning_count();

        if errors > 0 || warnings > 0 {
            output.push_str(&format!(
                "{} {}, {} {}\n",
                errors.to_string().red().bold(),
                if errors == 1 { "error" } else { "errors" },
                warnings.to_string().yellow().bold(),
                if warnings == 1 { "warning" } else { "warnings" }
            ));
        } else {
            output.push_str(&format!("{} No issues found\n", "✓".green().bold()));
        }

        output
    }

    fn format_report(&self, report: &Report) -> String {
        self.format_result(&report.result)
    }
}

/// JSON output formatter
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create new JSON formatter
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_result(&self, result: &ValidationResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_report(&self, report: &Report) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}

/// SARIF output formatter
pub struct SarifFormatter;

impl SarifFormatter {
    /// Create new SARIF formatter
    pub fn new() -> Self {
        Self
    }
}

impl Default for SarifFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for SarifFormatter {
    fn format_result(&self, result: &ValidationResult) -> String {
        let report = Report::new(result.clone());
        self.format_report(&report)
    }

    fn format_report(&self, report: &Report) -> String {
        match report.to_sarif() {
            Ok(sarif) => serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!(r#"{{"error": "{}"}}"#, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oparry_core::{Issue, IssueLevel};

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let mut result = ValidationResult::new();
        result.add_issue(Issue::error("test", "test message"));

        let output = formatter.format_result(&result);
        assert!(output.contains("\"error\""));
        assert!(output.contains("test message"));
    }
}
