//! Contract tests for parser API
//!
//! These tests verify the parser component implements its contract correctly.

use core_types::JsError;
use parser::{ASTNode, BytecodeGenerator, Lexer, Parser, ScopeAnalyzer, ScopeInfo, Token};

// =============================================================================
// Lexer Contract Tests
// =============================================================================

#[test]
fn test_lexer_new_creates_lexer() {
    let source = "let x = 42;";
    let _lexer = Lexer::new(source);
    // Should compile and create lexer
}

#[test]
fn test_lexer_next_token_returns_result() {
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source);
    let result: Result<Token, JsError> = lexer.next_token();
    assert!(result.is_ok());
}

#[test]
fn test_lexer_peek_token_returns_ref() {
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source);
    let result: Result<&Token, JsError> = lexer.peek_token();
    assert!(result.is_ok());
}

#[test]
fn test_token_identifier_variant() {
    let source = "myVar";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::Identifier(_)));
    if let Token::Identifier(name) = token {
        assert_eq!(name, "myVar");
    }
}

#[test]
fn test_token_number_variant() {
    let source = "42.5";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::Number(_)));
    if let Token::Number(n) = token {
        assert_eq!(n, 42.5);
    }
}

#[test]
fn test_token_string_variant() {
    let source = r#""hello""#;
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::String(_)));
    if let Token::String(s) = token {
        assert_eq!(s, "hello");
    }
}

#[test]
fn test_token_keyword_variant() {
    let source = "let";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::Keyword(_)));
}

#[test]
fn test_token_punctuator_variant() {
    let source = "=";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::Punctuator(_)));
}

#[test]
fn test_token_eof_variant() {
    let source = "";
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token().unwrap();
    assert!(matches!(token, Token::EOF));
}

// =============================================================================
// Parser Contract Tests
// =============================================================================

#[test]
fn test_parser_new_creates_parser() {
    let source = "let x = 42;";
    let _parser = Parser::new(source);
    // Should compile and create parser
}

#[test]
fn test_parser_parse_returns_ast_result() {
    let source = "let x = 42;";
    let mut parser = Parser::new(source);
    let result: Result<ASTNode, JsError> = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parser_parse_lazy_returns_lazy_ast() {
    let source = "function foo() { return 1; }";
    let mut parser = Parser::new(source);
    let result = parser.parse_lazy();
    assert!(result.is_ok());
}

#[test]
fn test_ast_node_is_enum() {
    let source = "let x = 1;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    // ASTNode should be an enum
    match ast {
        ASTNode::Program(_) => {}
        _ => {}
    }
}

// =============================================================================
// BytecodeGenerator Contract Tests
// =============================================================================

#[test]
fn test_bytecode_generator_new_creates_generator() {
    let _gen = BytecodeGenerator::new();
    // Should compile and create generator
}

#[test]
fn test_bytecode_generator_generate_returns_chunk() {
    let source = "let x = 42;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut gen = BytecodeGenerator::new();
    let result = gen.generate(&ast);
    assert!(result.is_ok());

    let chunk = result.unwrap();
    // Should return BytecodeChunk from bytecode_system
    let _insts = chunk.instructions;
    let _consts = chunk.constants;
}

// =============================================================================
// ScopeAnalyzer Contract Tests
// =============================================================================

#[test]
fn test_scope_analyzer_analyze_returns_scope_info() {
    let source = "let x = 42;";
    let mut parser = Parser::new(source);
    let mut ast = parser.parse().unwrap();

    let analyzer = ScopeAnalyzer::new();
    let result: Result<ScopeInfo, JsError> = analyzer.analyze(&mut ast);
    assert!(result.is_ok());
}

// =============================================================================
// Error Handling Contract Tests
// =============================================================================

#[test]
fn test_lexer_reports_invalid_token_error() {
    let source = "@@@";
    let mut lexer = Lexer::new(source);
    let result = lexer.next_token();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.source_position.is_some());
    }
}

#[test]
fn test_parser_reports_syntax_error() {
    let source = "let = ;"; // Invalid syntax
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.source_position.is_some());
    }
}

// =============================================================================
// ES2024 Feature Tests
// =============================================================================

#[test]
fn test_parse_arrow_function() {
    let source = "const add = (a, b) => a + b;";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_const_declaration() {
    let source = "const x = 10;";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_template_literal() {
    let source = r#"const msg = `hello ${name}`;"#;
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_destructuring() {
    let source = "const { a, b } = obj;";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_spread_operator() {
    let source = "const arr = [...other, 1];";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_async_function() {
    let source = "async function fetch() { await data; }";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_class_declaration() {
    let source = "class Foo { constructor() {} }";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_optional_chaining() {
    let source = "const x = obj?.prop?.value;";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_nullish_coalescing() {
    let source = "const x = a ?? b;";
    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok());
}
