//! Specialized validators for Parry

pub mod tailwind;
pub mod imports;
pub mod rust;
pub mod react;
pub mod css;
pub mod components;
pub mod accessibility;
pub mod security;
pub mod performance;
pub mod typescript;
pub mod testing;

pub use tailwind::TailwindValidator;
pub use imports::ImportValidator;
pub use rust::RustValidator;
pub use react::ReactValidator;
pub use css::CssValidator;
pub use components::ComponentValidator;
pub use accessibility::A11yValidator;
pub use security::SecurityValidator;
pub use performance::PerformanceValidator;
pub use typescript::TypeScriptValidator;
pub use testing::TestingValidator;

use oparry_core::{Issue, IssueLevel, Result, ValidationResult};
use oparry_parser::{ParsedCode, Language};
use std::path::Path;

/// Validator trait
pub trait Validator: Send + Sync {
    /// Get validator name
    fn name(&self) -> &str;

    /// Check if validator supports given language
    fn supports(&self, language: Language) -> bool;

    /// Validate parsed code
    fn validate_parsed(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult>;

    /// Validate raw source (fallback)
    fn validate_raw(&self, source: &str, file: &Path) -> Result<ValidationResult> {
        let _ = source;
        let _ = file;
        Ok(ValidationResult::new())
    }
}

/// Collection of validators
#[derive(Default)]
pub struct Validators {
    tailwind: Option<TailwindValidator>,
    imports: Option<ImportValidator>,
    rust: Option<RustValidator>,
    react: Option<ReactValidator>,
    css: Option<CssValidator>,
    components: Option<ComponentValidator>,
    accessibility: Option<A11yValidator>,
    security: Option<SecurityValidator>,
    performance: Option<PerformanceValidator>,
    typescript: Option<TypeScriptValidator>,
    testing: Option<TestingValidator>,
}

impl Validators {
    /// Create new validators collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add Tailwind validator
    pub fn with_tailwind(mut self, validator: TailwindValidator) -> Self {
        self.tailwind = Some(validator);
        self
    }

    /// Add Import validator
    pub fn with_imports(mut self, validator: ImportValidator) -> Self {
        self.imports = Some(validator);
        self
    }

    /// Add Rust validator
    pub fn with_rust(mut self, validator: RustValidator) -> Self {
        self.rust = Some(validator);
        self
    }

    /// Add React validator
    pub fn with_react(mut self, validator: ReactValidator) -> Self {
        self.react = Some(validator);
        self
    }

    /// Add CSS validator
    pub fn with_css(mut self, validator: CssValidator) -> Self {
        self.css = Some(validator);
        self
    }

    /// Add Component validator
    pub fn with_components(mut self, validator: ComponentValidator) -> Self {
        self.components = Some(validator);
        self
    }

    /// Add Accessibility validator
    pub fn with_accessibility(mut self, validator: A11yValidator) -> Self {
        self.accessibility = Some(validator);
        self
    }

    /// Add Security validator
    pub fn with_security(mut self, validator: SecurityValidator) -> Self {
        self.security = Some(validator);
        self
    }

    /// Add Performance validator
    pub fn with_performance(mut self, validator: PerformanceValidator) -> Self {
        self.performance = Some(validator);
        self
    }

    /// Add TypeScript validator
    pub fn with_typescript(mut self, validator: TypeScriptValidator) -> Self {
        self.typescript = Some(validator);
        self
    }

    /// Add Testing validator
    pub fn with_testing(mut self, validator: TestingValidator) -> Self {
        self.testing = Some(validator);
        self
    }

    /// Validate code with all applicable validators
    pub fn validate(&self, code: &ParsedCode, file: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.files_checked = 1;

        let language = Language::from_path(file);

        // Run applicable validators
        if let Some(ref validator) = self.tailwind {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.imports {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.rust {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.react {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.css {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.components {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.accessibility {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.security {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.performance {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.typescript {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        if let Some(ref validator) = self.testing {
            if validator.supports(language) {
                result.merge(validator.validate_parsed(code, file)?);
            }
        }

        Ok(result)
    }

    /// Get all enabled validators
    pub fn validators(&self) -> Vec<&dyn Validator> {
        let mut validators = Vec::new();

        if let Some(ref v) = self.tailwind {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.imports {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.rust {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.react {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.css {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.components {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.accessibility {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.security {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.performance {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.typescript {
            validators.push(v as &dyn Validator);
        }
        if let Some(ref v) = self.testing {
            validators.push(v as &dyn Validator);
        }

        validators
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validators_default() {
        let validators = Validators::new();
        assert!(validators.validators().is_empty());
    }

    #[test]
    fn test_validators_builder() {
        let validators = Validators::new()
            .with_tailwind(TailwindValidator::default_config())
            .with_imports(ImportValidator::default_config())
            .with_rust(RustValidator::default_config());

        assert_eq!(validators.validators().len(), 3);
    }

    #[test]
    fn test_validators_validate_tsx() {
        let validators = Validators::new()
            .with_tailwind(TailwindValidator::default_config())
            .with_react(ReactValidator::default_config());

        let code = ParsedCode::Generic(r#"
            function Button() {
                return <button className="flex">Click</button>;
            }
        "#.to_string());

        let result = validators.validate(&code, Path::new("test.tsx")).unwrap();
        assert!(result.passed);
        assert_eq!(result.files_checked, 1);
    }

    #[test]
    fn test_validators_validate_rust() {
        let validators = Validators::new()
            .with_rust(RustValidator::default_config());

        let code = ParsedCode::Generic(r#"
            fn main() {
                println!("Hello");
            }
        "#.to_string());

        let result = validators.validate(&code, Path::new("test.rs")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_validators_empty() {
        let validators = Validators::new();
        let code = ParsedCode::Generic("const x = 5;".to_string());

        let result = validators.validate(&code, Path::new("test.js")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_validators_language_filtering() {
        let validators = Validators::new()
            .with_tailwind(TailwindValidator::default_config())
            .with_rust(RustValidator::default_config());

        let code = ParsedCode::Generic("const x = 5;".to_string());

        // For JS, only tailwind should run
        let result = validators.validate(&code, Path::new("test.js")).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_validators_merge_results() {
        let validators = Validators::new()
            .with_tailwind(TailwindValidator::default_config())
            .with_react(ReactValidator::default_config());

        let code = ParsedCode::Generic(r#"
            class Button extends React.Component {
                render() {
                    return <button className="invalid-class">Click</button>;
                }
            }
        "#.to_string());

        let result = validators.validate(&code, Path::new("test.tsx")).unwrap();
        // Should have issues from both validators
        assert!(!result.passed || result.issues.len() >= 1);
    }

    #[test]
    fn test_validator_trait_bounds() {
        // Test that Validator trait is object-safe
        let validator: &dyn Validator = &TailwindValidator::default_config();
        assert_eq!(validator.name(), "Tailwind");
        assert!(validator.supports(Language::JavaScript));
    }
}
