//! Tests for javascript_complete module

use oparry_parser::{
    is_component_name, ImportSpecifierInfo, CompleteJavaScriptParser,
};
use oxc_span::SourceType;

#[test]
fn test_parse_simple_js() {
    let parser = CompleteJavaScriptParser::new();
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
    let parser = CompleteJavaScriptParser::new();
    let source = "function { invalid";

    let result = parser.parse(source, SourceType::default());
    assert!(result.is_err());
}

#[test]
fn test_extract_imports() {
    let parser = CompleteJavaScriptParser::new();
    let source = r#"
        import React from 'react';
        import { useState, useEffect } from 'react';
        import * as utils from './utils';
    "#;

    let ast = parser.parse(source, SourceType::default()).unwrap();
    let imports = ast.extract_imports();
    assert_eq!(imports.len(), 3);

    // Default import
    assert_eq!(imports[0].source, "react");
    assert!(matches!(
        imports[0].specifiers[0],
        ImportSpecifierInfo::Default { .. }
    ));

    // Named imports
    assert_eq!(imports[1].specifiers.len(), 2);
    assert!(matches!(
        imports[1].specifiers[0],
        ImportSpecifierInfo::Named { .. }
    ));

    // Namespace import
    assert!(matches!(
        imports[2].specifiers[0],
        ImportSpecifierInfo::Namespace { .. }
    ));
}

#[test]
fn test_extract_exports() {
    let parser = CompleteJavaScriptParser::new();
    let source = r#"
        export const foo = 'bar';
        export function baz() {}
        export default App;
        export * from './module';
    "#;

    let ast = parser.parse(source, SourceType::default()).unwrap();
    let exports = ast.extract_exports();
    assert!(!exports.is_empty());
}

#[test]
fn test_jsx_components() {
    let parser = CompleteJavaScriptParser::new();
    let source = r#"
        function App() {
            return (
                <div>
                    <Header />
                    <Main>
                        <Content />
                    </Main>
                </div>
            );
        }
    "#;

    let ast = parser.parse(source, SourceType::tsx()).unwrap();
    let components = ast.extract_jsx_components();
    assert!(!components.is_empty());

    // Should find Header, Main, Content
    let component_names: Vec<_> = components.iter().map(|c| &c.name).collect();
    assert!(component_names.contains(&&"Header".to_string()));
    assert!(component_names.contains(&&"Main".to_string()));
    assert!(component_names.contains(&&"Content".to_string()));
}

#[test]
fn test_react_hooks() {
    let parser = CompleteJavaScriptParser::new();
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
    let hook_names: Vec<_> = hooks.iter().map(|h| &h.name).collect();
    assert!(hook_names.contains(&&"useState".to_string()));
    assert!(hook_names.contains(&&"useEffect".to_string()));
    assert!(hook_names.contains(&&"useMemo".to_string()));
}

#[test]
fn test_is_component_name() {
    assert!(is_component_name("Component"));
    assert!(is_component_name("App"));
    assert!(is_component_name("UserProfile"));
    assert!(!is_component_name("div"));
    assert!(!is_component_name("span"));
    assert!(!is_component_name("button"));
}
