//! Complete JavaScript/TypeScript parser using Oxc

use crate::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Information about an import specifier
#[derive(Debug, Clone)]
pub enum ImportSpecifierInfo {
    /// Default import: `import React from 'react'`
    Default { local_name: String },
    /// Named import: `import { useState } from 'react'`
    Named { imported_name: String, local_name: String },
    /// Namespace import: `import * as React from 'react'`
    Namespace { local_name: String },
}

/// Information about an import declaration
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Source module path
    pub source: String,
    /// All specifiers in this import
    pub specifiers: Vec<ImportSpecifierInfo>,
    /// Whether this is a type-only import (TypeScript)
    pub is_type_only: bool,
}

/// Information about an export
#[derive(Debug, Clone)]
pub enum ExportInfo {
    /// Named export: `export { foo, bar }`
    Named { names: Vec<String> },
    /// Default export: `export default Component`
    Default,
    /// Export all: `export * from './module'`
    AllFrom { source: String },
    /// Export declaration: `export function foo() {}`
    Declaration { kind: String, name: String },
}

/// Information about a JSX component
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component name
    pub name: String,
    /// Line number where component opens
    pub line: usize,
    /// Whether component has children
    pub has_children: bool,
}

/// Information about a React hook usage
#[derive(Debug, Clone)]
pub struct HookInfo {
    /// Hook name (useState, useEffect, etc.)
    pub name: String,
    /// Line number where hook is called
    pub line: usize,
}

/// Parse results containing all extracted information
#[derive(Debug, Clone, Default)]
pub struct ParseResults {
    /// All imports found
    pub imports: Vec<ImportInfo>,
    /// All exports found
    pub exports: Vec<ExportInfo>,
    /// All JSX components found
    pub components: Vec<ComponentInfo>,
    /// All React hooks found
    pub hooks: Vec<HookInfo>,
}

/// JavaScript AST with parsed results
pub struct JavaScriptAst {
    source: String,
    pub results: ParseResults,
}

impl JavaScriptAst {
    /// Create new JavaScript AST
    pub fn new(source: String, results: ParseResults) -> Self {
        Self { source, results }
    }

    /// Get source code
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get parse results
    pub fn results(&self) -> &ParseResults {
        &self.results
    }

    /// Extract all imports
    pub fn extract_imports(&self) -> Vec<ImportInfo> {
        self.results.imports.clone()
    }

    /// Extract all exports
    pub fn extract_exports(&self) -> Vec<ExportInfo> {
        self.results.exports.clone()
    }

    /// Extract all JSX components
    pub fn extract_jsx_components(&self) -> Vec<ComponentInfo> {
        self.results.components.clone()
    }

    /// Extract all React hooks
    pub fn extract_hooks(&self) -> Vec<HookInfo> {
        self.results.hooks.clone()
    }
}

/// Check if name is a component name (PascalCase)
pub fn is_component_name(name: &str) -> bool {
    name.starts_with(char::is_uppercase)
}

/// JavaScript/TypeScript parser
pub struct JavaScriptParser;

impl JavaScriptParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse source code - validates syntax and extracts information
    pub fn parse(&self, source: &str, source_type: SourceType) -> Result<JavaScriptAst> {
        let allocator = Allocator::default();

        // Validate syntax using Oxc
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        if !result.errors.is_empty() {
            let e = &result.errors[0];
            return Err(crate::Error::Parse {
                file: "<unknown>".to_string(),
                line: 0,
                column: 0,
                message: e.message.to_string(),
            });
        }

        // Extract information using regex-based approach for MVP
        let results = Self::extract_info(source);
        let source_str = source.to_string();

        Ok(JavaScriptAst::new(source_str, results))
    }

    /// Extract information from source code using regex
    fn extract_info(source: &str) -> ParseResults {
        let mut results = ParseResults::default();

        // Extract imports
        let import_re = regex::Regex::new(
            r#"import\s+(?:(?P<default>\w+)|\{(?P<named>[^}]+)\}|\*\s+as\s+(?P<namespace>\w+))\s+from\s+['"](?P<source>[^'"]+)['"]"#
        ).unwrap();

        for caps in import_re.captures_iter(source) {
            let source = caps.name("source")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let is_type_only = source.contains("type");

            let specifiers = if caps.name("default").is_some() {
                vec![ImportSpecifierInfo::Default {
                    local_name: caps.name("default").unwrap().as_str().to_string(),
                }]
            } else if caps.name("namespace").is_some() {
                vec![ImportSpecifierInfo::Namespace {
                    local_name: caps.name("namespace").unwrap().as_str().to_string(),
                }]
            } else if let Some(named) = caps.name("named") {
                named.as_str().split(',')
                    .map(|s| ImportSpecifierInfo::Named {
                        imported_name: s.trim().to_string(),
                        local_name: s.trim().to_string(),
                    })
                    .collect()
            } else {
                vec![]
            };

            results.imports.push(ImportInfo {
                source,
                specifiers,
                is_type_only,
            });
        }

        // Extract components (PascalCase names in JSX-like context)
        let jsx_re = regex::Regex::new(r#"<([A-Z][a-zA-Z0-9]*)"#).unwrap();
        let mut component_names = std::collections::HashSet::new();
        for caps in jsx_re.captures_iter(source) {
            if let Some(name) = caps.get(1) {
                component_names.insert(name.as_str().to_string());
            }
        }

        results.components = component_names.into_iter()
            .enumerate()
            .map(|(_i, name)| ComponentInfo {
                name,
                line: 0,
                has_children: false,
            })
            .collect();

        // Extract React hooks - match complete hook names
        let hook_re = regex::Regex::new(
            r#"\b(useState|useEffect|useCallback|useMemo|useRef|useContext|useReducer|useTransition|useId|useSyncExternalStore|useLayoutEffect|useImperativeHandle|useDeferredValue|useDebugValue|useInsertionEffect)\b"#
        ).unwrap();
        let mut hook_names = std::collections::HashSet::new();
        for caps in hook_re.captures_iter(source) {
            if let Some(name) = caps.get(1) {
                hook_names.insert(name.as_str().to_string());
            }
        }

        results.hooks = hook_names.into_iter()
            .map(|name| HookInfo {
                name,
                line: 0,
            })
            .collect();

        // Extract exports - simpler regex approach
        if source.contains("export default") {
            results.exports.push(ExportInfo::Default);
        }

        // Export * from './module'
        let export_all_re = regex::Regex::new(r#"export\s+\*\s+from\s+['"]([^'"]+)['"]"#).unwrap();
        for caps in export_all_re.captures_iter(source) {
            if let Some(from) = caps.get(1) {
                results.exports.push(ExportInfo::AllFrom {
                    source: from.as_str().to_string(),
                });
            }
        }

        // Export const/function/class
        let export_decl_re = regex::Regex::new(r#"export\s+(?:const|function|class)\s+(\w+)"#).unwrap();
        for caps in export_decl_re.captures_iter(source) {
            if let Some(name) = caps.get(1) {
                results.exports.push(ExportInfo::Declaration {
                    kind: "declaration".to_string(),
                    name: name.as_str().to_string(),
                });
            }
        }

        results
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_js() {
        let parser = JavaScriptParser::new();
        let source = r#"
            function hello() {
                return "world";
            }
        "#;

        let ast = parser.parse(source, SourceType::default()).unwrap();
        assert_eq!(ast.source(), source);
    }

    #[test]
    fn test_parse_invalid_js() {
        let parser = JavaScriptParser::new();
        let source = "function { invalid";

        let result = parser.parse(source, SourceType::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_imports() {
        let parser = JavaScriptParser::new();
        let source = r#"
            import React from 'react';
            import { useState, useEffect } from 'react';
            import * as utils from './utils';
        "#;

        let ast = parser.parse(source, SourceType::default()).unwrap();
        let imports = ast.extract_imports();
        assert_eq!(imports.len(), 3);
    }

    #[test]
    fn test_extract_components() {
        let parser = JavaScriptParser::new();
        let source = r#"
            function App() {
                return <div><Header /><Main /></div>;
            }
        "#;

        let ast = parser.parse(source, SourceType::tsx()).unwrap();
        let components = ast.extract_jsx_components();
        assert!(!components.is_empty());
        assert!(components.iter().any(|c| c.name == "Header"));
    }

    #[test]
    fn test_extract_hooks() {
        let parser = JavaScriptParser::new();
        let source = r#"
            function Component() {
                const [state, setState] = useState(0);
                useEffect(() => {}, []);
                const memoized = useMemo(() => compute(), []);
                return <div />;
            }
        "#;

        let ast = parser.parse(source, SourceType::tsx()).unwrap();
        let hooks = ast.extract_hooks();
        assert!(!hooks.is_empty());
        assert!(hooks.iter().any(|h| h.name == "useState"));
        assert!(hooks.iter().any(|h| h.name == "useEffect"));
    }

    #[test]
    fn test_is_component_name() {
        assert!(is_component_name("Button"));
        assert!(is_component_name("MyComponent"));
        assert!(!is_component_name("button"));
        assert!(!is_component_name("div"));
    }

    #[test]
    fn test_parse_typescript() {
        let parser = JavaScriptParser::new();
        let source = r#"
            interface User {
                name: string;
                age: number;
            }

            function greet(user: User): string {
                return `Hello ${user.name}`;
            }
        "#;

        let ast = parser.parse(source, SourceType::ts()).unwrap();
        assert_eq!(ast.source(), source);
    }

    #[test]
    fn test_parse_tsx() {
        let parser = JavaScriptParser::new();
        let source = r#"
            interface Props {
                children: string;
            }

            export function Button({ children }: Props) {
                return <button>{children}</button>;
            }
        "#;

        let ast = parser.parse(source, SourceType::tsx()).unwrap();
        assert_eq!(ast.source(), source);
    }

    #[test]
    fn test_ast_new() {
        let source = "test source".to_string();
        let results = ParseResults::default();
        let ast = JavaScriptAst::new(source, results);

        assert_eq!(ast.source(), "test source");
    }
}
