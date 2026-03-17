//! Rust parser using Syn

use oparry_core::{Error, Result};
use syn::{File, parse_file};

/// Rust AST
#[derive(Debug)]
pub struct RustAst {
    source: String,
    file: File,
}

impl RustAst {
    /// Create new Rust AST
    pub fn new(source: String, file: File) -> Self {
        Self { source, file }
    }

    /// Get source code
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the syn File
    pub fn file(&self) -> &File {
        &self.file
    }
}

/// Rust parser
pub struct RustParser;

impl RustParser {
    /// Create new parser
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Parser for RustParser {
    fn parse(&self, source: &str) -> Result<super::ParsedCode> {
        let file = parse_file(source).map_err(|e| oparry_core::Error::Parse {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            message: e.to_string(),
        })?;

        Ok(super::ParsedCode::Rust(RustAst::new(source.to_string(), file)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_rust() {
        let parser = RustParser::new();
        let source = r#"
            fn hello(name: &str) -> String {
                format!("Hello {}", name)
            }
        "#;

        let result = parser.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_rust_struct() {
        let parser = RustParser::new();
        let source = r#"
            struct User {
                name: String,
                age: u32,
            }

            impl User {
                fn new(name: String) -> Self {
                    Self { name, age: 0 }
                }
            }
        "#;

        let result = parser.parse(source);
        assert!(result.is_ok());
    }
}
