//! Auto-fix rules
//!
//! Defines fix rules for common code issues.

use oparry_core::Issue;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Kind of fix to apply
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixKind {
    /// Replace text with new text
    Replace { old: String, new: String },
    /// Remove a line or portion
    Remove { start: usize, end: usize },
    /// Insert text at position
    Insert { position: usize, text: String },
    /// Regex-based replacement
    RegexReplace { pattern: String, replacement: String },
    /// Multi-line replacement
    MultiLineReplace {
        start_line: usize,
        end_line: usize,
        new_content: String,
    },
}

/// A fix rule that can correct specific issues
pub trait FixRule: Send + Sync {
    /// Get the rule name
    fn name(&self) -> &str;

    /// Get issue codes this rule can fix
    fn fixes_codes(&self) -> &[&str];

    /// Check if this rule can fix the given issue
    fn can_fix(&self, issue: &Issue) -> bool {
        self.fixes_codes().contains(&issue.code.as_str())
    }

    /// Generate a fix for the issue
    fn generate_fix(&self, issue: &Issue, source: &str, file: &Path) -> Option<String>;

    /// Preview the fix without applying it
    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)>;
}

/// Registry of fix rules
pub struct FixRuleRegistry {
    rules: Vec<Arc<dyn FixRule>>,
    by_code: HashMap<String, Vec<Arc<dyn FixRule>>>,
}

impl FixRuleRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            by_code: HashMap::new(),
        }
    }

    /// Add a rule to the registry
    pub fn register(&mut self, rule: Arc<dyn FixRule>) {
        for code in rule.fixes_codes() {
            self.by_code
                .entry(code.to_string())
                .or_default()
                .push(Arc::clone(&rule));
        }
        self.rules.push(rule);
    }

    /// Get rules that can fix a specific issue code
    pub fn get_for_code(&self, code: &str) -> Vec<Arc<dyn FixRule>> {
        self.by_code
            .get(code)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all registered rules
    pub fn all(&self) -> &[Arc<dyn FixRule>] {
        &self.rules
    }

    /// Create registry with default rules
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register default fix rules
        registry.register(Arc::new(TailwindFixRule::default()));
        registry.register(Arc::new(ImportFixRule::default()));
        registry.register(Arc::new(ReactFixRule::default()));
        registry.register(Arc::new(CssFixRule::default()));

        registry
    }
}

impl Default for FixRuleRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Tailwind CSS fix rule
pub struct TailwindFixRule {
    blocked_replacements: HashMap<String, String>,
    class_regex: Regex,
}

impl Default for TailwindFixRule {
    fn default() -> Self {
        let mut blocked_replacements = HashMap::new();

        // Width class replacements
        blocked_replacements.insert("w-xl".to_string(), "w-full".to_string());
        blocked_replacements.insert("w-2xl".to_string(), "w-full".to_string());
        blocked_replacements.insert("w-3xl".to_string(), "w-full".to_string());

        // Max-width replacements
        blocked_replacements.insert("max-w-sm".to_string(), "".to_string());
        blocked_replacements.insert("max-w-md".to_string(), "".to_string());
        blocked_replacements.insert("max-w-lg".to_string(), "".to_string());
        blocked_replacements.insert("max-w-xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-2xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-3xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-4xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-5xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-6xl".to_string(), "".to_string());
        blocked_replacements.insert("max-w-7xl".to_string(), "".to_string());

        // Blocked colors
        blocked_replacements.insert("bg-red-500".to_string(), "bg-red-600".to_string());
        blocked_replacements.insert("bg-yellow-500".to_string(), "bg-yellow-600".to_string());

        Self {
            blocked_replacements,
            class_regex: Regex::new(r#"class(?:Name)?\s*=\s*["']([^"']+)["']"#).unwrap(),
        }
    }
}

impl FixRule for TailwindFixRule {
    fn name(&self) -> &str {
        "TailwindFixer"
    }

    fn fixes_codes(&self) -> &[&str] {
        &[
            "tailwind-blocked-width",
            "tailwind-blocked-max-width",
            "tailwind-blocked-class",
        ]
    }

    fn generate_fix(&self, issue: &Issue, source: &str, _file: &Path) -> Option<String> {
        let line = issue.line?;

        // Check if the suggestion contains a class to replace
        if let Some(_suggestion) = &issue.suggestion {
            // Parse the source to find the class
            let source_lines: Vec<&str> = source.lines().collect();
            if line < source_lines.len() {
                let line_content = source_lines[line];

                if let Some(caps) = self.class_regex.captures(line_content) {
                    if let Some(classes) = caps.get(1) {
                        let classes_str = classes.as_str();
                        let fixed_classes = self.apply_replacements(classes_str);

                        if fixed_classes != classes_str {
                            // Generate JSON fix
                            let start = line_content.find(classes_str)?;
                            let end = start + classes_str.len();

                            return Some(serde_json::json!({
                                "type": "replace",
                                "start": start,
                                "end": end,
                                "replacement": fixed_classes,
                                "line": line,
                            }).to_string());
                        }
                    }
                }
            }
        }

        None
    }

    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)> {
        let fix_json = self.generate_fix(issue, source, Path::new(""))?;
        if let Ok(fix_data) = serde_json::from_str::<serde_json::Value>(&fix_json) {
            let replacement = fix_data.get("replacement")?.as_str()?;
            let line = issue.line?;
            let source_lines: Vec<&str> = source.lines().collect();
            let original = source_lines.get(line)?.to_string();
            Some((original, replacement.to_string()))
        } else {
            None
        }
    }
}

impl TailwindFixRule {
    /// Apply replacements to a class string
    fn apply_replacements(&self, classes: &str) -> String {
        classes
            .split_whitespace()
            .filter_map(|class| {
                // Check if this class should be replaced
                if let Some(replacement) = self.blocked_replacements.get(class) {
                    if replacement.is_empty() {
                        None // Remove the class
                    } else {
                        Some(replacement.as_str())
                    }
                } else {
                    Some(class) // Keep the class
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Import fix rule
pub struct ImportFixRule {
    alias_replacements: HashMap<String, String>,
    import_regex: Regex,
}

impl Default for ImportFixRule {
    fn default() -> Self {
        let mut alias_replacements = HashMap::new();
        alias_replacements.insert("./src/".to_string(), "@/".to_string());
        alias_replacements.insert("./components/".to_string(), "@/components/".to_string());
        alias_replacements.insert("./lib/".to_string(), "@/lib/".to_string());

        Self {
            alias_replacements,
            import_regex: Regex::new(
                r#"import\s+(?:(?:\{[^}]*\}|\w+|\*\s+as\s+\w+)\s+from\s+)?['"]([^'"]+)['"]"#
            ).unwrap(),
        }
    }
}

impl FixRule for ImportFixRule {
    fn name(&self) -> &str {
        "ImportFixer"
    }

    fn fixes_codes(&self) -> &[&str] {
        &["import-use-alias", "import-missing-extension"]
    }

    fn generate_fix(&self, issue: &Issue, source: &str, _file: &Path) -> Option<String> {
        let line = issue.line?;
        let source_lines: Vec<&str> = source.lines().collect();
        let line_content = source_lines.get(line)?;

        if let Some(caps) = self.import_regex.captures(line_content) {
            if let Some(import_path) = caps.get(1) {
                let path_str = import_path.as_str();

                // Check if we should replace with an alias
                for (old_prefix, new_alias) in &self.alias_replacements {
                    if path_str.starts_with(old_prefix) {
                        let new_path = path_str.replacen(old_prefix, new_alias, 1);

                        let start = import_path.start();
                        let end = import_path.end();

                        return Some(serde_json::json!({
                            "type": "replace",
                            "start": start,
                            "end": end,
                            "replacement": new_path,
                            "line": line,
                        }).to_string());
                    }
                }

                // Add missing extension for local imports
                if issue.code == "import-missing-extension" {
                    let new_path = if path_str.ends_with(".js") {
                        path_str.replacen(".js", ".ts", 1)
                    } else if !path_str.contains('.') && (path_str.starts_with("./") || path_str.starts_with("../")) {
                        format!("{}.ts", path_str)
                    } else {
                        return None;
                    };

                    let start = import_path.start();
                    let end = import_path.end();

                    return Some(serde_json::json!({
                        "type": "replace",
                        "start": start,
                        "end": end,
                        "replacement": new_path,
                        "line": line,
                    }).to_string());
                }
            }
        }

        None
    }

    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)> {
        let fix_json = self.generate_fix(issue, source, Path::new(""))?;
        if let Ok(fix_data) = serde_json::from_str::<serde_json::Value>(&fix_json) {
            let replacement = fix_data.get("replacement")?.as_str()?;
            let line = issue.line?;
            let source_lines: Vec<&str> = source.lines().collect();
            let original = source_lines.get(line)?.to_string();
            Some((original, replacement.to_string()))
        } else {
            None
        }
    }
}

/// React fix rule
pub struct ReactFixRule {
    jsx_regex: Regex,
    shorthand_regex: Regex,
}

impl Default for ReactFixRule {
    fn default() -> Self {
        Self {
            jsx_regex: Regex::new(r#"<([A-Z]\w+)([^>]*)>"#).unwrap(),
            shorthand_regex: Regex::new(r#"className=\{["']([^"']+)["']\}"#).unwrap(),
        }
    }
}

impl FixRule for ReactFixRule {
    fn name(&self) -> &str {
        "ReactFixer"
    }

    fn fixes_codes(&self) -> &[&str] {
        &[
            "react-missing-fragment",
            "react-bool-attribute",
            "react-shorthand-fragment",
        ]
    }

    fn generate_fix(&self, issue: &Issue, source: &str, _file: &Path) -> Option<String> {
        let line = issue.line?;
        let source_lines: Vec<&str> = source.lines().collect();
        let line_content = source_lines.get(line)?;

        match issue.code.as_str() {
            "react-missing-fragment" => {
                // Wrap in <Fragment>
                let indentation = line_content.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                let fixed = format!("{}<>{}</>", indentation, line_content.trim());

                Some(serde_json::json!({
                    "type": "replace",
                    "replacement": fixed,
                    "line": line,
                }).to_string())
            }
            "react-shorthand-fragment" => {
                // Replace <React.Fragment> with <>
                let fixed = line_content.replace("<React.Fragment>", "<>")
                    .replace("</React.Fragment>", "</>");

                Some(serde_json::json!({
                    "type": "replace",
                    "replacement": fixed,
                    "line": line,
                }).to_string())
            }
            "react-bool-attribute" => {
                // Convert disabled={true} to disabled
                let fixed = self.shorthand_regex.replace_all(line_content, "$1");

                Some(serde_json::json!({
                    "type": "replace",
                    "replacement": fixed,
                    "line": line,
                }).to_string())
            }
            _ => None,
        }
    }

    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)> {
        let fix_json = self.generate_fix(issue, source, Path::new(""))?;
        if let Ok(fix_data) = serde_json::from_str::<serde_json::Value>(&fix_json) {
            let replacement = fix_data.get("replacement")?.as_str()?.to_string();
            let line = issue.line?;
            let source_lines: Vec<&str> = source.lines().collect();
            let original = source_lines.get(line)?.to_string();
            Some((original, replacement))
        } else {
            None
        }
    }
}

/// CSS fix rule
pub struct CssFixRule {
    unit_replacements: HashMap<String, String>,
    color_replacements: HashMap<String, String>,
}

impl Default for CssFixRule {
    fn default() -> Self {
        let mut unit_replacements = HashMap::new();
        unit_replacements.insert(".0px".to_string(), "".to_string());

        let mut color_replacements = HashMap::new();
        color_replacements.insert("#f00".to_string(), "red".to_string());
        color_replacements.insert("#00f".to_string(), "blue".to_string());

        Self {
            unit_replacements,
            color_replacements,
        }
    }
}

impl FixRule for CssFixRule {
    fn name(&self) -> &str {
        "CssFixer"
    }

    fn fixes_codes(&self) -> &[&str] {
        &["css-zero-unit", "css-hard-color"]
    }

    fn generate_fix(&self, issue: &Issue, source: &str, _file: &Path) -> Option<String> {
        let line = issue.line?;
        let source_lines: Vec<&str> = source.lines().collect();
        let line_content = source_lines.get(line)?;

        match issue.code.as_str() {
            "css-zero-unit" => {
                // Remove px from zero values
                let fixed = line_content.replace("0px", "0");

                Some(serde_json::json!({
                    "type": "replace",
                    "replacement": fixed,
                    "line": line,
                }).to_string())
            }
            "css-hard-color" => {
                // Convert hex to named colors where possible
                let mut fixed = line_content.to_string();
                for (hex, name) in &self.color_replacements {
                    fixed = fixed.replace(hex, name);
                }

                Some(serde_json::json!({
                    "type": "replace",
                    "replacement": fixed,
                    "line": line,
                }).to_string())
            }
            _ => None,
        }
    }

    fn preview(&self, issue: &Issue, source: &str) -> Option<(String, String)> {
        let fix_json = self.generate_fix(issue, source, Path::new(""))?;
        if let Ok(fix_data) = serde_json::from_str::<serde_json::Value>(&fix_json) {
            let replacement = fix_data.get("replacement")?.as_str()?.to_string();
            let line = issue.line?;
            let source_lines: Vec<&str> = source.lines().collect();
            let original = source_lines.get(line)?.to_string();
            Some((original, replacement))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_rule_registry_new() {
        let registry = FixRuleRegistry::new();
        assert!(registry.all().is_empty());
    }

    #[test]
    fn test_fix_rule_registry_defaults() {
        let registry = FixRuleRegistry::with_defaults();
        assert!(!registry.all().is_empty());
        assert!(registry.get_for_code("tailwind-blocked-width").len() > 0);
    }

    #[test]
    fn test_tailwind_fix_rule_codes() {
        let rule = TailwindFixRule::default();
        assert!(rule.fixes_codes().contains(&"tailwind-blocked-width"));
        assert!(rule.fixes_codes().contains(&"tailwind-blocked-max-width"));
    }

    #[test]
    fn test_import_fix_rule_codes() {
        let rule = ImportFixRule::default();
        assert!(rule.fixes_codes().contains(&"import-use-alias"));
        assert!(rule.fixes_codes().contains(&"import-missing-extension"));
    }

    #[test]
    fn test_react_fix_rule_codes() {
        let rule = ReactFixRule::default();
        assert!(rule.fixes_codes().contains(&"react-missing-fragment"));
        assert!(rule.fixes_codes().contains(&"react-shorthand-fragment"));
    }

    #[test]
    fn test_css_fix_rule_codes() {
        let rule = CssFixRule::default();
        assert!(rule.fixes_codes().contains(&"css-zero-unit"));
        assert!(rule.fixes_codes().contains(&"css-hard-color"));
    }

    #[test]
    fn test_tailwind_apply_replacements() {
        let rule = TailwindFixRule::default();
        let classes = "w-xl max-w-md p-4 bg-red-500";
        let fixed = rule.apply_replacements(classes);
        assert!(!fixed.contains("w-xl"));
        assert!(!fixed.contains("max-w-md"));
        assert!(fixed.contains("p-4"));
    }

    #[test]
    fn test_fix_kind_variants() {
        let replace = FixKind::Replace {
            old: "old".to_string(),
            new: "new".to_string(),
        };
        assert!(matches!(replace, FixKind::Replace { .. }));

        let remove = FixKind::Remove { start: 0, end: 10 };
        assert!(matches!(remove, FixKind::Remove { .. }));
    }
}
