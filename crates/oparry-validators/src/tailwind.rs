//! Tailwind CSS class validator

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::fs;

/// Known Tailwind utility classes (subset for MVP)
const TAILWIND_CLASSES: &[&str] = &[
    // Spacing
    "p-0", "p-1", "p-2", "p-3", "p-4", "p-5", "p-6", "p-8", "p-10", "p-12",
    "px-0", "px-1", "px-2", "px-3", "px-4", "px-5", "px-6", "px-8",
    "py-0", "py-1", "py-2", "py-3", "py-4", "py-5", "py-6", "py-8",
    "m-0", "m-1", "m-2", "m-3", "m-4", "m-auto", "mx-auto", "my-auto",
    // Layout
    "flex", "inline-flex", "grid", "inline-grid",
    "flex-row", "flex-col", "flex-row-reverse", "flex-col-reverse",
    "justify-start", "justify-end", "justify-center", "justify-between", "justify-around",
    "items-start", "items-end", "items-center", "items-stretch",
    "gap-1", "gap-2", "gap-3", "gap-4", "gap-6", "gap-8",
    // Colors (background, text, border)
    "bg-white", "bg-black", "bg-transparent",
    "text-white", "text-black", "text-transparent",
    "border-white", "border-black", "border-transparent",
    // Typography
    "text-xs", "text-sm", "text-base", "text-lg", "text-xl", "text-2xl", "text-3xl",
    "font-light", "font-normal", "font-medium", "font-semibold", "font-bold",
    "text-left", "text-center", "text-right",
    // Borders
    "border", "border-0", "border-2", "border-4",
    "rounded", "rounded-none", "rounded-sm", "rounded-md", "rounded-lg", "rounded-xl", "rounded-full",
    // Sizing
    "w-full", "w-auto", "w-fit", "w-screen", "w-1/2", "w-1/3", "w-2/3",
    "h-full", "h-auto", "h-fit", "h-screen",
    "max-w-full", "max-w-md", "max-w-lg", "max-w-xl", "max-w-2xl", "max-w-4xl", "max-w-6xl",
    // Display
    "block", "inline-block", "hidden",
    // Position
    "relative", "absolute", "fixed", "sticky",
    // Effects
    "shadow", "shadow-sm", "shadow-md", "shadow-lg", "shadow-xl", "shadow-none",
    "opacity-0", "opacity-50", "opacity-100",
];

/// Tailwind validator configuration
#[derive(Debug, Clone)]
pub struct TailwindConfig {
    /// Safe list patterns (e.g., "p-*", "m-*")
    pub safe_list: Vec<String>,
    /// Block list patterns
    pub block_list: Vec<String>,
    /// Maximum arbitrary values
    pub max_arbitrary: usize,
    /// Custom classes from tailwind.config.ts
    pub custom_classes: HashSet<String>,
    /// Maximum width classes (blocked for consistency)
    pub blocked_max_widths: Vec<String>,
    /// Width classes (blocked for consistency)
    pub blocked_widths: Vec<String>,
    /// Enforce spacing scale
    pub enforce_spacing_scale: bool,
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            safe_list: vec![
                "p-*".to_string(),
                "m-*".to_string(),
                "w-*".to_string(),
                "h-*".to_string(),
                "text-*".to_string(),
                "bg-*".to_string(),
            ],
            block_list: vec![
                "bg-red-500".to_string(),
                "bg-yellow-500".to_string(),
                "w-xl".to_string(),
                "w-2xl".to_string(),
                "w-3xl".to_string(),
                "max-w-xl".to_string(),
                "max-w-2xl".to_string(),
                "max-w-3xl".to_string(),
                "max-w-4xl".to_string(),
                "max-w-5xl".to_string(),
                "max-w-6xl".to_string(),
                "max-w-7xl".to_string(),
            ],
            max_arbitrary: 5,
            custom_classes: HashSet::new(),
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
            blocked_widths: vec![
                "w-sm".to_string(),
                "w-md".to_string(),
                "w-lg".to_string(),
                "w-xl".to_string(),
                "w-2xl".to_string(),
                "w-3xl".to_string(),
                "w-4xl".to_string(),
                "w-5xl".to_string(),
                "w-6xl".to_string(),
                "w-7xl".to_string(),
            ],
            enforce_spacing_scale: true,
        }
    }
}

/// Tailwind CSS validator
pub struct TailwindValidator {
    config: TailwindConfig,
    class_regex: Regex,
    arbitrary_regex: Regex,
}

impl TailwindValidator {
    /// Create new Tailwind validator
    pub fn new(config: TailwindConfig) -> Self {
        Self {
            config,
            // Match className="..." or class="..."
            class_regex: Regex::new(r#"class(?:Name)?\s*=\s*["']([^"']+)["']"#).unwrap(),
            // Match arbitrary values like [color] or [size:...]
            arbitrary_regex: Regex::new(r"\[[^\]]+\]").unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(TailwindConfig::default())
    }

    /// Validate a single class name
    fn validate_class(&self, class: &str, file: &str, line: usize) -> Option<Issue> {
        let class = class.trim();

        // Empty class - no validation needed
        if class.is_empty() {
            return None;
        }

        // Check for variants (hover:, focus:, etc.) - process first
        if class.contains(':') {
            let parts: Vec<&str> = class.split(':').collect();
            if parts.len() == 2 {
                return self.validate_class(parts[1], file, line);
            }
        }

        // Check for arbitrary values - check BEFORE safe list
        // Arbitrary values should always be warned even if they match safe patterns
        if self.arbitrary_regex.is_match(class) {
            return Some(Issue::warning(
                "tailwind-arbitrary-value",
                format!("Arbitrary value '{}' may indicate design inconsistency", class),
            )
            .with_file(file)
            .with_line(line)
            .with_suggestion("Define a custom class in tailwind.config.ts"));
        }

        // Check blocked widths (w-xl, w-2xl, etc.)
        for blocked in &self.config.blocked_widths {
            if class == blocked || class.starts_with(&format!("{}:", blocked)) {
                return Some(Issue::error(
                    "tailwind-blocked-width",
                    format!("Width class '{}' is not allowed - use container or component", class),
                )
                .with_file(file)
                .with_line(line)
                .with_suggestion("Use a container class or define custom width in tailwind.config.ts"));
            }
        }

        // Check blocked max-widths (max-w-xl, max-w-2xl, etc.)
        for blocked in &self.config.blocked_max_widths {
            if class == blocked || class.starts_with(&format!("{}:", blocked)) {
                return Some(Issue::error(
                    "tailwind-blocked-max-width",
                    format!("Max-width class '{}' is not allowed - use container", class),
                )
                .with_file(file)
                .with_line(line)
                .with_suggestion("Use Container component or define custom max-width"));
            }
        }

        // Check block list
        for blocked in &self.config.block_list {
            if self.matches_pattern(class, blocked) {
                return Some(Issue::error(
                    "tailwind-blocked-class",
                    format!("Class '{}' is blocked by configuration", class),
                )
                .with_file(file)
                .with_line(line)
                .with_suggestion("Remove this class or update block_list"));
            }
        }

        // Check safe list
        for safe in &self.config.safe_list {
            if self.matches_pattern(class, safe) {
                return None; // Allowed
            }
        }

        // Check if it's a known Tailwind class
        if TAILWIND_CLASSES.contains(&class) {
            return None; // Valid
        }

        // Check custom classes
        if self.config.custom_classes.contains(class) {
            return None; // Valid custom class
        }

        // Unknown class
        Some(Issue::warning(
            "tailwind-unknown-class",
            format!("Unknown Tailwind class '{}'", class),
        )
        .with_file(file)
        .with_line(line)
        .with_suggestion("Check tailwind.config.ts or add to safe_list"))
    }

    /// Match class against pattern (supports * wildcard)
    fn matches_pattern(&self, class: &str, pattern: &str) -> bool {
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            class.starts_with(prefix)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            class.ends_with(suffix)
        } else {
            class == pattern
        }
    }

    /// Parse tailwind.config.ts for custom classes (simplified)
    fn load_custom_classes(&mut self, config_path: &Path) -> Result<()> {
        if !config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| oparry_core::Error::File {
                path: config_path.to_path_buf(),
                source: e,
            })?;

        // Very basic parsing - in production, would use proper TS parser
        if content.contains("extend") {
            // This is where custom classes would be defined
            // For MVP, we'll just note that customization exists
        }

        Ok(())
    }
}

impl Validator for TailwindValidator {
    fn name(&self) -> &str {
        "Tailwind"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        // Find all className attributes
        for (line_idx, line) in source.lines().enumerate() {
            if let Some(caps) = self.class_regex.captures(line) {
                if let Some(classes_str) = caps.get(1) {
                    let classes = classes_str.as_str().split_whitespace();
                    for class in classes {
                        if let Some(issue) = self.validate_class(class, &file_str, line_idx) {
                            result.add_issue(issue);
                        }
                    }
                }
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
    fn test_tailwind_validator_valid() {
        let validator = TailwindValidator::default_config();
        let code = r#"
            <div className="flex items-center gap-4 p-4">
                <button className="px-4 py-2 bg-white rounded">Click</button>
            </div>
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_tailwind_validator_invalid() {
        let validator = TailwindValidator::default_config();
        let code = r#"
            <div className="flex invalid-class">
                <button className="bg-red-500">Click</button>
            </div>
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect invalid classes");
        assert_eq!(result.issues.len(), 2); // invalid-class + bg-red-500
    }

    #[test]
    fn test_pattern_matching() {
        let validator = TailwindValidator::default_config();
        assert!(validator.matches_pattern("p-4", "p-*"));
        assert!(validator.matches_pattern("text-xl", "text-*"));
        assert!(!validator.matches_pattern("bg-red-500", "p-*"));
    }

    #[test]
    fn test_blocked_width_classes() {
        let validator = TailwindValidator::default_config();
        let code = r#"<div className="w-xl"></div>"#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues[0].code, "tailwind-blocked-width");
    }

    #[test]
    fn test_blocked_max_width_classes() {
        let validator = TailwindValidator::default_config();
        let code = r#"<div className="max-w-md"></div>"#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues[0].code, "tailwind-blocked-max-width");
    }

    #[test]
    fn test_class_regex() {
        let validator = TailwindValidator::default_config();
        let line = r#"<div className="w-[123px]"></div>"#;
        assert!(validator.class_regex.is_match(line));
        if let Some(caps) = validator.class_regex.captures(line) {
            assert_eq!(caps.get(1).map(|m| m.as_str()), Some("w-[123px]"));
        }
    }

    #[test]
    fn test_arbitrary_values() {
        let validator = TailwindValidator::default_config();
        let code = r#"<div className="w-[123px]"></div>"#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        println!("Issues: {:?}", result.issues);
        println!("Warning count: {}", result.warning_count());
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect arbitrary value");
        assert_eq!(result.issues[0].code, "tailwind-arbitrary-value");
    }

    #[test]
    fn test_variant_classes() {
        let validator = TailwindValidator::default_config();
        let code = r#"<div className="hover:bg-white"></div>"#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Should pass - hover: prefix is valid
        assert!(result.passed);
    }

    #[test]
    fn test_custom_classes() {
        let mut config = TailwindConfig::default();
        config.custom_classes = vec!["my-custom-class".to_string()].into_iter().collect();
        let validator = TailwindValidator::new(config);
        let code = r#"<div className="my-custom-class"></div>"#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_multiple_class_attributes() {
        let validator = TailwindValidator::default_config();
        let code = r#"
            <div className="flex">
                <div className="invalid-1"></div>
                <div className="invalid-2"></div>
            </div>
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_class_attribute_vs_classname() {
        let validator = TailwindValidator::default_config();
        let code = r#"
            <div class="flex items-center"></div>
            <div className="flex items-center"></div>
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_tailwind_config_default() {
        let config = TailwindConfig::default();
        assert!(config.enforce_spacing_scale);
        assert_eq!(config.max_arbitrary, 5);
        assert!(!config.block_list.is_empty());
    }

    #[test]
    fn test_validator_supports() {
        let validator = TailwindValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(validator.supports(Language::TypeScript));
        assert!(validator.supports(Language::Jsx));
        assert!(validator.supports(Language::Tsx));
        assert!(!validator.supports(Language::Rust));
    }

    #[test]
    fn test_validate_class_edge_cases() {
        let validator = TailwindValidator::default_config();

        // Empty class
        assert!(validator.validate_class("", "test.tsx", 0).is_none());

        // Valid spacing classes
        assert!(validator.validate_class("p-0", "test.tsx", 0).is_none());
        assert!(validator.validate_class("m-auto", "test.tsx", 0).is_none());

        // Valid display classes
        assert!(validator.validate_class("flex", "test.tsx", 0).is_none());
        assert!(validator.validate_class("hidden", "test.tsx", 0).is_none());
    }
}
