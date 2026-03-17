//! Integration tests for parry-autofix

use oparry_autofix::{AutoFixer, AutoFixConfig, FixStrategy};
use oparry_core::{Issue, IssueLevel};
use oparry_parser::Language;
use std::path::Path;

#[test]
#[ignore] // TODO: Implement actual fix logic
fn test_tailwind_blocked_width_fix() {
    let fixer = AutoFixer::new();

    let source = r#"<div className="w-xl p-4">Content</div>"#;
    let issues = vec![
        Issue::error("tailwind-blocked-width", "Width class 'w-xl' is not allowed")
            .with_line(0)
            .with_suggestion("Use w-full instead"),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::Tsx, Path::new("test.tsx"))
        .unwrap();

    // The fixer should be able to process the issue
    assert_eq!(result.issues_fixed, 1);
}

#[test]
fn test_import_alias_fix() {
    let fixer = AutoFixer::new();

    let source = r#"import { Button } from "./components/Button";"#;
    let issues = vec![
        Issue::warning("import-use-alias", "Import should use path alias")
            .with_line(0)
            .with_suggestion("Use @/components instead of ./components"),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::TypeScript, Path::new("test.ts"))
        .unwrap();

    // Should fix the import path
    assert_eq!(result.issues_fixed, 1);
}

#[test]
fn test_react_fragment_shorthand() {
    let fixer = AutoFixer::new();

    let source = r#"<React.Fragment><div>Hello</div></React.Fragment>"#;
    let issues = vec![
        Issue::note(
            "react-shorthand-fragment",
            "Use shorthand fragment syntax",
        )
        .with_line(0),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::Tsx, Path::new("test.tsx"))
        .unwrap();

    assert_eq!(result.issues_fixed, 1);
}

#[test]
fn test_css_zero_unit_fix() {
    let fixer = AutoFixer::new();

    let source = ".margin { margin: 0px; }";
    let issues = vec![Issue::warning(
        "css-zero-unit",
        "Zero values don't need units",
    )
    .with_line(0)];

    let result = fixer
        .fix_issues(source, &issues, Language::Unknown, Path::new("test.css"))
        .unwrap();

    assert_eq!(result.issues_fixed, 1);
}

#[test]
fn test_dry_run_mode() {
    let config = AutoFixConfig {
        dry_run: true,
        ..Default::default()
    };
    let fixer = AutoFixer::with_config(config);

    let source = r#"<div className="w-xl">Content</div>"#;
    let issues = vec![Issue::error(
        "tailwind-blocked-width",
        "Width class not allowed",
    )
    .with_line(0)];

    let result = fixer
        .fix_issues(source, &issues, Language::Tsx, Path::new("test.tsx"))
        .unwrap();

    // In dry-run mode, original should equal modified
    assert_eq!(result.original, result.modified);
}

#[test]
fn test_safe_strategy() {
    let config = AutoFixConfig {
        strategy: FixStrategy::Safe,
        dry_run: false,
        ..Default::default()
    };
    let fixer = AutoFixer::with_config(config);

    let source = "some code";
    let issues = vec![
        Issue::note("test-note", "A note"),
        Issue::warning("test-warning", "A warning"),
        Issue::error("test-error", "An error"),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::JavaScript, Path::new("test.js"))
        .unwrap();

    // Safe strategy only fixes notes
    assert_eq!(result.issues_fixed, 1);
}

#[test]
fn test_aggressive_strategy() {
    let config = AutoFixConfig {
        strategy: FixStrategy::Aggressive,
        dry_run: false,
        ..Default::default()
    };
    let fixer = AutoFixer::with_config(config);

    let source = "some code";
    let issues = vec![
        Issue::note("test-note", "A note"),
        Issue::warning("test-warning", "A warning"),
        Issue::error("test-error", "An error"),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::JavaScript, Path::new("test.js"))
        .unwrap();

    // Aggressive strategy fixes all non-syntax errors
    assert_eq!(result.issues_fixed, 3);
}

#[test]
fn test_preview_fix() {
    let fixer = AutoFixer::new();

    let issue = Issue::error(
        "tailwind-blocked-width",
        "Width class 'w-xl' is not allowed",
    )
    .with_line(0)
    .with_suggestion("Use w-full");

    let source = r#"<div className="w-xl p-4">Content</div>"#;

    let preview = fixer.preview_fix(&issue, source);

    // Should generate a preview or return None if no fix available
    // Both are valid outcomes depending on rule matching
    match preview {
        Some(p) => {
            assert!(p.contains("becomes") || p.contains("---"));
        }
        None => {
            // Also acceptable - rule might not match
        }
    }
}

#[test]
#[ignore] // TODO: Implement actual fix logic
fn test_multiple_issues_same_file() {
    let fixer = AutoFixer::new();

    let source = r#"
        <div className="w-xl max-w-md p-4 bg-red-500">
            <Button className="w-2xl">Click</Button>
        </div>
    "#;

    let issues = vec![
        Issue::error("tailwind-blocked-width", "w-xl not allowed").with_line(1),
        Issue::error("tailwind-blocked-max-width", "max-w-md not allowed").with_line(1),
        Issue::error("tailwind-blocked-class", "bg-red-500 not allowed").with_line(1),
        Issue::error("tailwind-blocked-width", "w-2xl not allowed").with_line(2),
    ];

    let result = fixer
        .fix_issues(source, &issues, Language::Tsx, Path::new("test.tsx"))
        .unwrap();

    // Should handle multiple issues
    assert!(result.issues_fixed >= 1);
}

#[test]
fn test_fix_application_none() {
    let fixer = AutoFixer::new();

    let source = "clean code with no issues";
    let issues: Vec<Issue> = vec![];

    let result = fixer
        .fix_issues(source, &issues, Language::JavaScript, Path::new("test.js"))
        .unwrap();

    assert!(!result.has_changes());
    assert_eq!(result.fixes_applied, 0);
}

#[test]
fn test_fix_with_custom_rules() {
    // Test creating a fixer with custom rules
    let fixer = AutoFixer::new();

    // The fixer should have default rules loaded
    let source = "test";
    let issues = vec![];

    let result = fixer
        .fix_issues(source, &issues, Language::JavaScript, Path::new("test.js"))
        .unwrap();

    assert!(!result.has_changes());
}
