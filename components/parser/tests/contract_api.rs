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
    assert!(matches!(token, Token::Identifier(_, _)));
    if let Token::Identifier(name, _has_escapes) = token {
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

#[test]
fn test_rest_destructuring_pattern() {
    let code = "function empty(...[]) {}";
    let result = parser::Parser::new(code).parse();
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed with: {:?}", result);
}

#[test]
fn test_rest_destructuring_bytecode() {
    let code = "function empty(...[]) {}";
    let ast = parser::Parser::new(code).parse().expect("parse failed");
    println!("AST: {:?}", ast);
    
    let mut gen = parser::BytecodeGenerator::new();
    let result = gen.generate(&ast);
    println!("Bytecode result: {:?}", result);
    assert!(result.is_ok(), "Bytecode gen failed: {:?}", result);
}

#[test]
fn test_all_rest_patterns() {
    let patterns = [
        ("empty", "function empty(...[]) {}"),
        ("emptyWithArray", "function emptyWithArray(...[[]]) {}"),
        ("emptyWithObject", "function emptyWithObject(...[{}]) {}"),
        ("emptyWithRest", "function emptyWithRest(...[...[]]) {}"),
        ("emptyWithLeading", "function emptyWithLeading(x, ...[]) {}"),
        ("singleElement", "function singleElement(...[a]) {}"),
        ("singleElementWithInitializer", "function singleElementWithInitializer(...[a = 0]) {}"),
        ("singleElementWithArray", "function singleElementWithArray(...[[a]]) {}"),
        ("singleElementWithObject", r#"function singleElementWithObject(...[{p: q}]) {}"#),
        ("singleElementWithRest", "function singleElementWithRest(...[...a]) {}"),
        ("singleElementWithLeading", "function singleElementWithLeading(x, ...[a]) {}"),
        ("multiElement", "function multiElement(...[a, b, c]) {}"),
        ("multiElementWithInitializer", "function multiElementWithInitializer(...[a = 0, b, c = 1]) {}"),
        ("multiElementWithArray", "function multiElementWithArray(...[[a], b, [c]]) {}"),
        ("multiElementWithObject", r#"function multiElementWithObject(...[{p: q}, {r}, {s = 0}]) {}"#),
        ("multiElementWithRest", "function multiElementWithRest(...[a, b, ...c]) {}"),
        ("multiElementWithLeading", "function multiElementWithLeading(x, y, ...[a, b, c]) {}"),
    ];
    
    for (name, source) in patterns {
        let result = parser::Parser::new(source).parse();
        if result.is_err() {
            println!("✗ {} - {:?}", name, result);
        }
        assert!(result.is_ok(), "Pattern {} failed: {:?}", name, result);
    }
}

#[test]
fn test_private_generator_method() {
    let code = r#"class C {
  * #method() {
    return 1;
  }
}"#;
    let result = parser::Parser::new(code).parse();
    println!("Private generator method: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_private_method_access() {
    let code = "class C { #x; getX() { return this.#x; } }";
    let result = parser::Parser::new(code).parse();
    println!("Private member access: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_setter_with_underscore_param() {
    let code = r#"({
      set foo(_v) { }
    })"#;
    let result = parser::Parser::new(code).parse();
    println!("Setter with underscore: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_arguments_accessor() {
    let code = r#"(function(a) {
  let setCalls = 0;
  Object.defineProperty(arguments, "0", {
    set(_v) { setCalls += 1; },
    enumerable: true,
    configurable: true,
  });
})(0);"#;
    let result = parser::Parser::new(code).parse();
    println!("Arguments accessor: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_iife() {
    let code = "(function() { })()";;
    let result = parser::Parser::new(code).parse();
    println!("IIFE: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_iife_with_args() {
    let code = "(function(a) { return a; })(0);";
    let result = parser::Parser::new(code).parse();
    println!("IIFE with args: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_let_in_function() {
    let code = "(function(a) { let setCalls = 0; })(0);";
    let result = parser::Parser::new(code).parse();
    println!("Let in function: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_object_define_property() {
    let code = r#"Object.defineProperty(arguments, "0", {})"#;
    let result = parser::Parser::new(code).parse();
    println!("defineProperty: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_setter_shorthand() {
    let code = r#"({
      set(_v) { }
    })"#;
    let result = parser::Parser::new(code).parse();
    println!("Setter shorthand: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_object_with_setter() {
    let code = r#"({
      set foo(_v) { },
      enumerable: true,
    })"#;
    let result = parser::Parser::new(code).parse();
    println!("Object with setter: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_keyword_as_property_name() {
    let code = "var obj = { break: 1, case: 2, if: 3 }";
    let result = parser::Parser::new(code).parse();
    println!("Keyword property name: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_keyword_member_access() {
    let code = "obj.break = 1; obj.if = 2;";
    let result = parser::Parser::new(code).parse();
    println!("Keyword member access: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_keyword_accessor_methods() {
    // Keywords should be allowed as accessor method names
    let code = r#"
        var obj = {
            get await() { return 1; },
            set break(v) { },
            get if() { return 2; }
        };
    "#;
    let result = parser::Parser::new(code).parse();
    println!("Keyword accessor: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_asi_labeled_statement() {
    // ASI with labeled statements inside a block
    let code = "{\na:\n1 \n} \n3";
    let result = parser::Parser::new(code).parse();
    println!("ASI labeled: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_object_property_getter() {
    // Test step by step
    let code0 = "Object;";
    let result0 = parser::Parser::new(code0).parse();
    println!("Step 0: {:?}", result0);
    assert!(result0.is_ok(), "Step 0 failed: {:?}", result0);

    let code1 = "Object.foo;";
    let result1 = parser::Parser::new(code1).parse();
    println!("Step 1: {:?}", result1);
    assert!(result1.is_ok(), "Step 1 failed: {:?}", result1);

    let code2 = "Object.defineProperty;";
    let result2 = parser::Parser::new(code2).parse();
    println!("Step 2: {:?}", result2);
    assert!(result2.is_ok(), "Step 2 failed: {:?}", result2);

    let code3 = "Object.defineProperty();";
    let result3 = parser::Parser::new(code3).parse();
    println!("Step 3: {:?}", result3);
    assert!(result3.is_ok(), "Step 3 failed: {:?}", result3);

    let code4 = "Object.defineProperty({});";
    let result4 = parser::Parser::new(code4).parse();
    println!("Step 4: {:?}", result4);
    assert!(result4.is_ok(), "Step 4 failed: {:?}", result4);
}

#[test]
fn test_asi_prefix_increment() {
    // x\n++y should be parsed as x; ++y; not x++; y;
    let code = "var x = 0;\nvar y = 0;\nx\n++y";
    let result = parser::Parser::new(code).parse();
    println!("ASI prefix increment: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_asi_multiline_increment() {
    // Simpler test
    let code = "var x=0, y=0;\nx\n++\ny";
    let result = parser::Parser::new(code).parse();
    println!("ASI multiline increment: {:?}", result);
    assert!(result.is_ok(), "ASI multiline increment failed: {:?}", result);
}

#[test]
fn test_bitwise_and() {
    let code = "var x = 1 & 2;";
    let result = parser::Parser::new(code).parse();
    println!("Bitwise AND: {:?}", result);
    assert!(result.is_ok(), "Bitwise AND failed: {:?}", result);
}

#[test]
fn test_continue_with_label() {
    // continue with label
    let code = r#"label1: for (var i = 0; i <= 0; i++) {
  for (var j = 0; j <= 1; j++) {
    if (j === 0) {
      continue label1;
    }
  }
}"#;
    let result = parser::Parser::new(code).parse();
    println!("Continue with label: {:?}", result);
    assert!(result.is_ok(), "Continue with label failed: {:?}", result);
}
