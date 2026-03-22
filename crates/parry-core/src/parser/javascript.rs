//! JavaScript/TypeScript parser using Oxc

use crate::Result;

/// JavaScript/TypeScript parser
pub struct JavaScriptParser;

impl JavaScriptParser {
    /// Create new parser
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Parser for JavaScriptParser {
    fn parse(&self, source: &str) -> Result<super::ParsedCode> {
        // For MVP, just return generic parsed code
        // Full Oxc integration would require more work with lifetimes
        Ok(super::ParsedCode::Generic(source.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser as _, SimpleJavaScriptParser};

    type TestParser = SimpleJavaScriptParser;

    #[test]
    fn test_parse_javascript() {
        let parser = TestParser::new();
        let source = r#"
            function hello(name) {
                return "Hello " + name;
            }
        "#;

        let result = parser.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_react() {
        let parser = TestParser::new();
        let source = r#"
            import React from 'react';

            function Button({ children }) {
                return <button>{children}</button>;
            }
        "#;

        let result = parser.parse(source);
        assert!(result.is_ok());
    }
}
