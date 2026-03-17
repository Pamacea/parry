//! Import structure validator

use crate::Validator;
use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Import configuration
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Enforce alias usage
    pub enforce_alias: bool,
    /// Alias mappings (e.g., "@/" -> "./src")
    pub alias_map: HashMap<String, String>,
    /// Require file extensions
    pub require_extensions: bool,
    /// Allowed import sources
    pub allowed_sources: Vec<String>,
}

impl Default for ImportConfig {
    fn default() -> Self {
        let mut alias_map = HashMap::new();
        alias_map.insert("@/".to_string(), "./src".to_string());
        alias_map.insert("@/components".to_string(), "./components".to_string());
        alias_map.insert("@/lib".to_string(), "./lib".to_string());

        Self {
            enforce_alias: true,
            alias_map,
            require_extensions: false,
            allowed_sources: vec![
                "react".to_string(),
                "react-dom".to_string(),
                "next".to_string(),
                "@radix-ui/*".to_string(),
                "class-variance-authority".to_string(),
                "clsx".to_string(),
                "tailwind-merge".to_string(),
            ],
        }
    }
}

/// Import validator
pub struct ImportValidator {
    config: ImportConfig,
    import_regex: Regex,
    require_regex: Regex,
}

impl ImportValidator {
    /// Create new import validator
    pub fn new(config: ImportConfig) -> Self {
        Self {
            config,
            // Match import statements - simplified to catch from... imports
            import_regex: Regex::new(
                r#"from\s+['"]([^'"]+)['"]"#
            ).unwrap(),
            // Match require statements
            require_regex: Regex::new(r#"require\(['"]([^'"]+)['"]\)"#).unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(ImportConfig::default())
    }

    /// Validate import path
    fn validate_import_path(
        &self,
        path: &str,
        file: &str,
        line: usize,
    ) -> Option<Issue> {
        // Check file extensions first (for non-node_modules)
        if self.config.require_extensions {
            let is_node_module = !path.starts_with('.') && !path.starts_with('/');
            if !is_node_module {
                // Check if it has a valid file extension (not just the relative path dot)
                let has_extension = path.ends_with(".ts")
                    || path.ends_with(".tsx")
                    || path.ends_with(".js")
                    || path.ends_with(".jsx")
                    || path.ends_with(".mts")
                    || path.ends_with(".cjs")
                    || path.ends_with(".mjs");
                if !has_extension {
                    return Some(Issue::error(
                        "import-missing-extension",
                        format!("Import '{}' is missing file extension", path),
                    )
                    .with_file(file)
                    .with_line(line)
                    .with_suggestion("Add file extension (e.g., '.ts', '.tsx')"));
                }
            }
        }

        // Check if it's a relative import - skip alias check for relative imports
        if path.starts_with("./") || path.starts_with("../") {
            return None;
        }

        // Check if it should use an alias
        for (alias, _target) in &self.config.alias_map {
            // This is simplified - real implementation would check if path
            // matches the target and suggest using the alias instead
            if path.contains("/src/") || path.contains("/components/") {
                if self.config.enforce_alias {
                    return Some(Issue::warning(
                        "import-use-alias",
                        format!("Import '{}' should use path alias", path),
                    )
                    .with_file(file)
                    .with_line(line)
                    .with_suggestion(&format!("Use {} instead", alias)));
                }
            }
        }

        None
    }
}

impl Validator for ImportValidator {
    fn name(&self) -> &str {
        "Imports"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        // Check imports
        for (line_idx, line) in source.lines().enumerate() {
            // ES imports
            if let Some(caps) = self.import_regex.captures(line) {
                if let Some(path) = caps.get(1) {
                    let path_str = path.as_str();
                    if let Some(issue) = self.validate_import_path(path_str, &file_str, line_idx) {
                        result.add_issue(issue);
                    }
                }
            }

            // CommonJS requires
            if let Some(caps) = self.require_regex.captures(line) {
                if let Some(path) = caps.get(1) {
                    let path_str = path.as_str();
                    if let Some(issue) = self.validate_import_path(path_str, &file_str, line_idx) {
                        result.add_issue(issue);
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
    fn test_import_validator_valid() {
        let validator = ImportValidator::default_config();
        let code = r#"
            import React from 'react';
            import { Button } from '@/components/ui/button';
            import { utils } from '@/lib/utils';
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_import_validator_relative() {
        let validator = ImportValidator::default_config();
        let code = r#"
            import { Button } from './Button';
            import { utils } from '../utils';
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        assert!(result.passed); // Relative imports are OK
    }

    #[test]
    fn test_import_config_default() {
        let config = ImportConfig::default();
        assert!(config.enforce_alias);
        assert!(!config.require_extensions);
        assert!(!config.allowed_sources.is_empty());
    }

    #[test]
    fn test_import_config_alias_map() {
        let config = ImportConfig::default();
        assert!(config.alias_map.contains_key("@/"));
        assert!(config.alias_map.contains_key("@/components"));
        assert!(config.alias_map.contains_key("@/lib"));
    }

    #[test]
    fn test_import_validator_require_extensions() {
        let config = ImportConfig {
            require_extensions: true,
            ..Default::default()
        };
        let validator = ImportValidator::new(config);
        let code = r#"
            import { Component } from './Component';
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        assert!(!result.passed, "Should fail with missing extension error");
        assert_eq!(result.issues[0].code, "import-missing-extension");
    }

    #[test]
    fn test_import_validator_node_modules() {
        let config = ImportConfig {
            require_extensions: true,
            ..Default::default()
        };
        let validator = ImportValidator::new(config);
        let code = r#"
            import React from 'react';
            import { useState } from 'react';
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        // Node modules don't need extensions
        assert!(result.passed);
    }

    #[test]
    fn test_import_validator_commonjs() {
        let validator = ImportValidator::default_config();
        let code = r#"
            const React = require('react');
            const utils = require('./utils');
        "#;

        let result = validator.validate_raw(code, Path::new("test.js")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_import_validator_supports() {
        let validator = ImportValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(validator.supports(Language::TypeScript));
        assert!(validator.supports(Language::Jsx));
        assert!(validator.supports(Language::Tsx));
        assert!(!validator.supports(Language::Rust));
    }

    #[test]
    fn test_import_config_custom_alias_map() {
        let mut alias_map = std::collections::HashMap::new();
        alias_map.insert("@lib".to_string(), "./lib".to_string());

        let config = ImportConfig {
            alias_map,
            enforce_alias: true,
            ..Default::default()
        };

        assert!(config.alias_map.contains_key("@lib"));
    }

    #[test]
    fn test_import_multiple_issues() {
        let validator = ImportValidator::default_config();
        let code = r#"
            import React from 'react';
            import { Button } from './Button';
            import { utils } from '../utils';
        "#;

        let result = validator.validate_raw(code, Path::new("test.ts")).unwrap();
        // All imports should be valid (relative or standard)
        assert!(result.passed);
    }
}
