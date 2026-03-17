//! Generic parser for unknown languages (stores raw source)

use oparry_core::Result;

/// Generic parser that just stores the source as-is
pub struct GenericParser;

impl super::Parser for GenericParser {
    fn parse(&self, source: &str) -> Result<super::ParsedCode> {
        Ok(super::ParsedCode::Generic(source.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_generic_parser() {
        let parser = GenericParser;
        let source = "any random text";
        let result = parser.parse(source).unwrap();
        assert_eq!(result.source(), source);
    }
}
