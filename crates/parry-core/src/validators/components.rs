//! Component usage validator (shadcn/ui, etc.)

use crate::validators::Validator;
use crate::{Issue, IssueLevel, Result, ValidationResult};
use crate::parser::{ParsedCode, Language};
use regex::Regex;
use std::path::Path;

/// Component validation configuration
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    /// Enforce shadcn/ui usage
    pub enforce_shadcn: bool,
    /// shadcn/ui components path
    pub shadcn_path: String,
    /// Known shadcn/ui components
    pub known_components: Vec<String>,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            enforce_shadcn: true,
            shadcn_path: "@/components/ui".to_string(),
            known_components: vec![
                "Button".to_string(),
                "Card".to_string(),
                "Input".to_string(),
                "Label".to_string(),
                "Select".to_string(),
                "Checkbox".to_string(),
                "Dialog".to_string(),
                "DropdownMenu".to_string(),
                "Toast".to_string(),
                "Tabs".to_string(),
            ],
        }
    }
}

/// Component validator
pub struct ComponentValidator {
    config: ComponentConfig,
    jsx_element_regex: Regex,
    import_regex: Regex,
}

impl ComponentValidator {
    /// Create new component validator
    pub fn new(config: ComponentConfig) -> Self {
        Self {
            config,
            // Match JSX element names
            jsx_element_regex: Regex::new(r"<([A-Z][a-zA-Z0-9]*)").unwrap(),
            // Match imports
            import_regex: Regex::new(r#"import\s+\{[^}]*\}\s+from\s+['"]([^'"]+)['"]"#).unwrap(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(ComponentConfig::default())
    }

    /// Check if component is a known shadcn component
    fn is_known_shadcn_component(&self, name: &str) -> bool {
        self.config.known_components.contains(&name.to_string())
    }

    /// Validate component import
    fn validate_component_import(
        &self,
        component: &str,
        imports: &[String],
        file: &str,
    ) -> Option<Issue> {
        if !self.is_known_shadcn_component(component) {
            return None;
        }

        let expected_import = format!("{}/{}", self.config.shadcn_path, component.to_lowercase());

        // Check if correct import exists
        let has_correct_import = imports.iter().any(|imp| {
            imp.contains(&component.to_lowercase())
                && imp.contains(&self.config.shadcn_path)
        });

        if !has_correct_import {
            return Some(Issue::warning(
                "component-shadcn-import",
                format!("Component '{}' should be imported from shadcn/ui", component),
            )
            .with_file(file)
            .with_suggestion(&format!(
                "import {{ {} }} from '{}'",
                component, expected_import
            )));
        }

        None
    }
}

impl Validator for ComponentValidator {
    fn name(&self) -> &str {
        "Components"
    }

    fn supports(&self, language: Language) -> bool {
        language.is_javascript_variant()
    }

    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let source = code.source();

        let file_str = file.to_string_lossy().to_string();

        // Collect all imports
        let mut imports = Vec::new();
        for line in source.lines() {
            if let Some(caps) = self.import_regex.captures(line) {
                if let Some(path) = caps.get(1) {
                    imports.push(path.as_str().to_string());
                }
            }
        }

        // Find all JSX components
        for (line_idx, line) in source.lines().enumerate() {
            for caps in self.jsx_element_regex.captures_iter(line) {
                if let Some(component) = caps.get(1) {
                    let component_name = component.as_str();
                    if let Some(issue) =
                        self.validate_component_import(component_name, &imports, &file_str)
                    {
                        result.add_issue(issue.with_line(line_idx));
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
    fn test_component_validator_valid() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            import { Button } from '@/components/ui/button';

            export function Form() {
                return <Button>Submit</Button>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_component_validator_missing_import() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            export function Form() {
                return <Button>Submit</Button>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect missing shadcn import");
        assert_eq!(result.issues[0].code, "component-shadcn-import");
    }

    #[test]
    fn test_component_config_default() {
        let config = ComponentConfig::default();
        assert!(config.enforce_shadcn);
        assert_eq!(config.shadcn_path, "@/components/ui");
        assert!(!config.known_components.is_empty());
    }

    #[test]
    fn test_component_known_components() {
        let validator = ComponentValidator::default_config();

        assert!(validator.is_known_shadcn_component("Button"));
        assert!(validator.is_known_shadcn_component("Card"));
        assert!(validator.is_known_shadcn_component("Input"));
        assert!(validator.is_known_shadcn_component("Dialog"));

        assert!(!validator.is_known_shadcn_component("MyCustomComponent"));
        assert!(!validator.is_known_shadcn_component("div"));
    }

    #[test]
    fn test_component_multiple_imports() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            import { Button } from '@/components/ui/button';
            import { Card } from '@/components/ui/card';

            export function Form() {
                return (
                    <Card>
                        <Button>Submit</Button>
                    </Card>
                );
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_component_non_shadcn_component() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            export function Form() {
                return <CustomWidget>Submit</CustomWidget>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // CustomWidget is not in known_components, so no error
        assert!(result.passed);
    }

    #[test]
    fn test_component_validator_supports() {
        let validator = ComponentValidator::default_config();
        assert!(validator.supports(Language::JavaScript));
        assert!(validator.supports(Language::TypeScript));
        assert!(validator.supports(Language::Jsx));
        assert!(validator.supports(Language::Tsx));
        assert!(!validator.supports(Language::Rust));
    }

    #[test]
    fn test_component_custom_shadcn_path() {
        let config = ComponentConfig {
            shadcn_path: "@ui/components".to_string(),
            ..Default::default()
        };
        let validator = ComponentValidator::new(config);
        let code = r#"
            import { Button } from '@ui/components/button';

            export function Form() {
                return <Button>Submit</Button>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_component_wrong_import_path() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            import { Button } from './Button';

            export function Form() {
                return <Button>Submit</Button>;
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should detect wrong import path");
    }

    #[test]
    fn test_component_multiple_known_components() {
        let validator = ComponentValidator::default_config();
        let code = r#"
            export function Form() {
                return (
                    <>
                        <Button>Submit</Button>
                        <Input />
                        <Label>Password</Label>
                    </>
                );
            }
        "#;

        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Should have 3 issues for missing imports
        assert_eq!(result.issues.len(), 3);
    }

    #[test]
    fn test_component_config_not_enforcing() {
        let config = ComponentConfig {
            enforce_shadcn: false,
            ..Default::default()
        };
        let validator = ComponentValidator::new(config);
        let code = r#"
            export function Form() {
                return <Button>Submit</Button>;
            }
        "#;

        // When enforce_shadcn is false, we still validate (config not used in validation logic yet)
        let result = validator.validate_raw(code, Path::new("test.tsx")).unwrap();
        // Warnings don't fail by default (only in strict mode)
        assert!(result.warning_count() > 0, "Should still detect missing import");
    }
}
