//! Multi-language parser for Parry

mod javascript;
mod rust;
mod generic;
mod javascript_complete;

// Simple JS parser (implements Parser trait)
pub use javascript::JavaScriptParser as SimpleJavaScriptParser;
// Re-exports from javascript_complete (full AST parser)
pub use javascript_complete::{
    ComponentInfo, ExportInfo, HookInfo, ImportInfo, ImportSpecifierInfo,
    JavaScriptAst as CompleteJavaScriptAst, JavaScriptParser as CompleteJavaScriptParser,
    ParseResults, is_component_name,
};
pub use rust::{RustParser, RustAst};
pub use generic::GenericParser;

use crate::Result;
use std::path::Path;

/// Language detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
    Rust,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("js") | Some("mjs") | Some("cjs") => Language::JavaScript,
            Some("jsx") => Language::Jsx,
            Some("ts") => Language::TypeScript,
            Some("tsx") => Language::Tsx,
            Some("rs") => Language::Rust,
            _ => Language::Unknown,
        }
    }

    /// Check if this is a JavaScript/TypeScript variant
    pub fn is_javascript_variant(&self) -> bool {
        matches!(
            self,
            Language::JavaScript | Language::TypeScript | Language::Jsx | Language::Tsx
        )
    }

    /// Check if this is Rust
    pub fn is_rust(&self) -> bool {
        matches!(self, Language::Rust)
    }
}

/// Parsed code representation
#[derive(Debug)]
pub enum ParsedCode {
    Rust(RustAst),
    Generic(String),
}

impl ParsedCode {
    /// Get the raw source
    pub fn source(&self) -> &str {
        match self {
            ParsedCode::Rust(ast) => ast.source(),
            ParsedCode::Generic(s) => s,
        }
    }
}

/// Parser trait
pub trait Parser: Send + Sync {
    /// Parse source code
    fn parse(&self, source: &str) -> Result<ParsedCode>;
}

/// Get parser for language
pub fn parser_for_language(language: Language) -> Box<dyn Parser> {
    match language {
        Language::JavaScript | Language::TypeScript | Language::Jsx | Language::Tsx => {
            Box::new(SimpleJavaScriptParser::new())
        }
        Language::Rust => Box::new(RustParser::new()),
        Language::Unknown => Box::new(GenericParser),
    }
}

/// Get parser for file path
pub fn parser_for_path(path: &Path) -> Box<dyn Parser> {
    parser_for_language(Language::from_path(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path(Path::new("test.js")), Language::JavaScript);
        assert_eq!(Language::from_path(Path::new("test.ts")), Language::TypeScript);
        assert_eq!(Language::from_path(Path::new("test.tsx")), Language::Tsx);
        assert_eq!(Language::from_path(Path::new("test.rs")), Language::Rust);
        assert_eq!(Language::from_path(Path::new("test.unknown")), Language::Unknown);
    }
}
