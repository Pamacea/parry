//! Check command - validate codebase

use crate::{CliContext, OutputFormatter, HumanFormatter, JsonFormatter, SarifFormatter};
use oparry_core::{OutputFormat, Result, ValidationResult};
use oparry_parser::parser_for_path;
use oparry_validators::{Validators, TailwindValidator, ImportValidator, RustValidator, ComponentValidator};
use oparry_validators::{A11yValidator, SecurityValidator, PerformanceValidator, TypeScriptValidator, TestingValidator};
use oparry_validators::{tailwind::TailwindConfig, imports::ImportConfig, rust::RustConfig, components::ComponentConfig};
use oparry_validators::{accessibility::A11yConfig, security::SecurityConfig, performance::PerformanceConfig, typescript::TypeScriptConfig, testing::TestingConfig};
use std::path::PathBuf;
use glob::glob;

/// Check command
pub struct CheckCommand {
    /// Paths to check
    paths: Vec<PathBuf>,
    /// Validators to run
    validators: Vec<String>,
    /// Output format
    format: OutputFormat,
    /// Auto-fix
    fix: bool,
    /// Strict mode
    strict: bool,
}

impl CheckCommand {
    /// Create new check command
    pub fn new(
        paths: Vec<PathBuf>,
        validators: Vec<String>,
        format: OutputFormat,
        fix: bool,
        strict: bool,
    ) -> Self {
        Self {
            paths,
            validators,
            format,
            fix,
            strict,
        }
    }

    /// Run the check command
    pub fn run(&self, ctx: &CliContext) -> Result<()> {
        // Build validators
        let validators = self.build_validators(ctx)?;

        // Collect files
        let files = self.collect_files()?;

        let mut total_result = ValidationResult::new();

        // Check each file
        for file in files {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| oparry_core::Error::File {
                    path: file.clone(),
                    source: e,
                })?;

            let parser = parser_for_path(&file);
            let parsed = parser.parse(&content)?;

            let result = validators.validate(&parsed, &file)?;
            total_result.merge(result);
        }

        // Format and print output
        let formatter = self.create_formatter();
        println!("{}", formatter.format_result(&total_result));

        // Apply strict mode if enabled
        total_result.finalize_with_strict_mode(self.strict);

        // Return appropriate exit code
        if total_result.is_passing_with_strict(self.strict) {
            Ok(())
        } else {
            Err(oparry_core::Error::Validation(
                "Validation failed".to_string()
            ))
        }
    }

    /// Build validators from context
    fn build_validators(&self, ctx: &CliContext) -> Result<Validators> {
        let mut validators = Validators::new();

        // Add Tailwind validator
        if self.should_run_validator("tailwind") && ctx.config.tailwind.enabled {
            let tw_config = TailwindConfig {
                safe_list: ctx.config.tailwind.safe_list.clone(),
                block_list: ctx.config.tailwind.block_list.clone(),
                max_arbitrary: ctx.config.tailwind.max_arbitrary_values,
                ..Default::default()
            };
            validators = validators.with_tailwind(TailwindValidator::new(tw_config));
        }

        // Add Import validator
        if self.should_run_validator("imports") {
            let imp_config = ImportConfig {
                enforce_alias: ctx.config.imports.enforce_alias,
                alias_map: ctx.config.imports.alias_map.clone(),
                require_extensions: ctx.config.imports.require_extensions,
                ..Default::default()
            };
            validators = validators.with_imports(ImportValidator::new(imp_config));
        }

        // Add Rust validator
        if self.should_run_validator("rust") && ctx.config.rust.enabled {
            let rust_config = RustConfig {
                deny_unsafe: ctx.config.rust.deny_unsafe.is_some(),
                warn_unwrap: ctx.config.rust.warn_unwrap,
                enforce_result_handling: ctx.config.rust.enforce_result_handling,
            };
            validators = validators.with_rust(RustValidator::new(rust_config));
        }

        // Add Component validator
        if self.should_run_validator("components") && ctx.config.components.enforce_shadcn {
            let comp_config = ComponentConfig {
                enforce_shadcn: ctx.config.components.enforce_shadcn,
                shadcn_path: ctx.config.components.shadcn_path.clone(),
                ..Default::default()
            };
            validators = validators.with_components(ComponentValidator::new(comp_config));
        }

        // Add Accessibility validator
        if self.should_run_validator("accessibility") || self.should_run_validator("a11y") {
            let a11y_config = A11yConfig::default();
            validators = validators.with_accessibility(A11yValidator::new(a11y_config));
        }

        // Add Security validator
        if self.should_run_validator("security") {
            let sec_config = SecurityConfig::default();
            validators = validators.with_security(SecurityValidator::new(sec_config));
        }

        // Add Performance validator
        if self.should_run_validator("performance") || self.should_run_validator("perf") {
            let perf_config = PerformanceConfig::default();
            validators = validators.with_performance(PerformanceValidator::new(perf_config));
        }

        // Add TypeScript validator
        if self.should_run_validator("typescript") || self.should_run_validator("ts") {
            let ts_config = TypeScriptConfig::default();
            validators = validators.with_typescript(TypeScriptValidator::new(ts_config));
        }

        // Add Testing validator
        if self.should_run_validator("testing") || self.should_run_validator("test") {
            let test_config = TestingConfig::default();
            validators = validators.with_testing(TestingValidator::new(test_config));
        }

        Ok(validators)
    }

    /// Check if validator should run
    fn should_run_validator(&self, name: &str) -> bool {
        self.validators.is_empty() || self.validators.iter().any(|v| v == name)
    }

    /// Collect all files to check
    fn collect_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let patterns = if self.paths.is_empty() {
            vec!["**/*.{ts,tsx,js,jsx,rs}"]
        } else {
            self.paths.iter().map(|p| p.to_str().unwrap()).collect()
        };

        for pattern in patterns {
            if let Ok(entries) = glob(pattern) {
                for entry in entries.flatten() {
                    if entry.is_file() {
                        files.push(entry);
                    }
                }
            } else {
                // Single file
                files.push(PathBuf::from(pattern));
            }
        }

        Ok(files)
    }

    /// Create output formatter
    fn create_formatter(&self) -> Box<dyn OutputFormatter> {
        match self.format {
            OutputFormat::Human => Box::new(HumanFormatter::new(true, true)),
            OutputFormat::Json => Box::new(JsonFormatter::new()),
            OutputFormat::Sarif => Box::new(SarifFormatter::new()),
        }
    }
}
