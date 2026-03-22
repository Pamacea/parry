//! Check command - validate codebase

use oalacea_parry_core::{
    Config, OutputFormat, ValidationResult, Issue, IssueLevel,
    parser::parser_for_path, parser::Language,
    validators::{Validators, TailwindValidator, ImportValidator, RustValidator},
    autofix::{AutoFixer, AutoFixConfig, FixStrategy},
};
use std::path::PathBuf;
use colored::Colorize;

/// Run the `parry check` command
pub fn run(
    config: Config,
    paths: Vec<PathBuf>,
    validators_list: Option<String>,
    output: Option<String>,
    fix: bool,
    strict: bool,
) -> anyhow::Result<()> {
    let validators = validators_list.unwrap_or_default();
    let validators: Vec<String> = validators
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let format = if let Some(fmt) = output {
        OutputFormat::from_str(&fmt)?
    } else {
        config.output.format
    };

    // Build validators
    let validators = build_validators(&config, &validators);

    // Collect files
    let files = collect_files(&paths)?;

    if files.is_empty() {
        println!("{}", "No files to check".yellow());
        return Ok(());
    }

    println!("{}", format!("Checking {} file(s)...", files.len()).dimmed());
    println!();

    let mut total_result = ValidationResult::new();
    let mut files_to_fix = Vec::new();

    // Check each file
    for file in &files {
        let content = std::fs::read_to_string(file)?;

        let parser = parser_for_path(file);
        let parsed = parser.parse(&content)?;

        let result = validators.validate(&parsed, file)?;
        total_result.merge(result.clone());

        if fix && !result.passed && !result.issues.is_empty() {
            files_to_fix.push((file.clone(), content, result.issues));
        }
    }

    // Print results
    print_results(&total_result);

    // Apply fixes if requested
    if fix && !files_to_fix.is_empty() {
        apply_fixes(&files_to_fix)?;
    }

    // Exit with error if validation failed
    if !total_result.passed || (strict && !total_result.issues.is_empty()) {
        std::process::exit(1);
    }

    Ok(())
}

/// Build validators from config
fn build_validators(config: &Config, validators_list: &[String]) -> Validators {
    let mut validators = Validators::new();

    let should_run = |name: &str| -> bool {
        validators_list.is_empty() || validators_list.iter().any(|v| v == name)
    };

    // Add Tailwind validator
    if should_run("tailwind") {
        validators = validators.with_tailwind(TailwindValidator::default_config());
    }

    // Add Import validator
    if should_run("imports") {
        validators = validators.with_imports(ImportValidator::default_config());
    }

    // Add Rust validator
    if should_run("rust") {
        validators = validators.with_rust(RustValidator::default_config());
    }

    validators
}

/// Collect all files to check
fn collect_files(paths: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let extensions = ["rs", "ts", "tsx", "js", "jsx"];

    if paths.is_empty() {
        // Default: search from current directory
        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap()) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    } else {
        // Process each path
        for path in paths {
            if path.is_file() {
                files.push(path.clone());
            } else if path.is_dir() {
                // Walk directory
                for entry in walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension() {
                            if extensions.contains(&ext.to_str().unwrap()) {
                                files.push(p.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

/// Print validation results
fn print_results(result: &ValidationResult) {
    if result.passed {
        println!("{}", "✓ All checks passed!".green());
    } else {
        println!("{}", "✕ Validation failed".red());
        println!();

        for issue in &result.issues {
            let icon = match issue.level {
                IssueLevel::Error => "✖".red(),
                IssueLevel::Warning => "⚠".yellow(),
                IssueLevel::Note => "ℹ".blue(),
            };
            println!("{} {}: {}", icon, issue.code, issue.message);
        }
    }

    println!();
    println!("Files checked: {}", result.files_checked);
    println!("Issues found: {}", result.issues.len());
}

/// Apply auto-fixes to files
fn apply_fixes(files_to_fix: &[(PathBuf, String, Vec<Issue>)]) -> anyhow::Result<()> {
    let config = AutoFixConfig {
        strategy: FixStrategy::Aggressive,
        dry_run: false,
        ..Default::default()
    };
    let autofixer = AutoFixer::with_config(config);

    let mut total_fixed = 0;
    let mut total_files = 0;

    for (file, original_content, issues) in files_to_fix {
        let language = Language::from_path(file);

        let fix_app = autofixer.fix_issues(original_content, issues, language, file)?;

        if fix_app.has_changes() {
            std::fs::write(file, &fix_app.modified)?;
            total_fixed += fix_app.issues_fixed;
            total_files += 1;
            println!("✓ Fixed {} issue(s) in {}", fix_app.issues_fixed, file.display());
        }
    }

    if total_files > 0 {
        println!("\n✓ Applied fixes across {} file(s)", total_files);
    }

    Ok(())
}
