//! Parser to Bytecode Integration Tests
//!
//! Tests the integration between the parser and bytecode_system components.
//! Verifies that JavaScript source code is correctly parsed into AST
//! and then correctly compiled into bytecode.

use bytecode_system::{BytecodeChunk, Opcode};
use parser::{BytecodeGenerator, Parser};

/// Test: Parse simple number and generate bytecode
#[test]
fn test_parse_number_to_bytecode() {
    let source = "42;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Verify bytecode contains LoadConstant for the number
    assert!(!chunk.instructions.is_empty(), "Bytecode should not be empty");
    assert!(!chunk.constants.is_empty(), "Constants should contain 42");

    let has_load_constant = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::LoadConstant(_)));
    assert!(has_load_constant, "Should have LoadConstant instruction");

    let has_return = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::Return));
    assert!(has_return, "Should have Return instruction");
}

/// Test: Parse addition expression and verify bytecode
#[test]
fn test_parse_addition_to_bytecode() {
    let source = "1 + 2;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have: LoadConstant(1), LoadConstant(2), Add, Return
    let has_add = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::Add));
    assert!(has_add, "Should have Add instruction");

    // Should have 2 constants (1 and 2)
    assert!(
        chunk.constants.len() >= 2,
        "Should have at least 2 constants"
    );
}

/// Test: Parse variable declaration
#[test]
fn test_parse_variable_declaration_to_bytecode() {
    let source = "let x = 100;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have StoreLocal for variable assignment
    let has_store_local = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::StoreLocal(_)));
    assert!(has_store_local, "Should have StoreLocal instruction");

    // Register count should be at least 1
    assert!(chunk.register_count >= 1, "Should allocate at least 1 register");
}

/// Test: Parse boolean literals
#[test]
fn test_parse_boolean_literals() {
    let source = "true; false;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    let has_load_true = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::LoadTrue));
    let has_load_false = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::LoadFalse));

    assert!(has_load_true, "Should have LoadTrue instruction");
    assert!(has_load_false, "Should have LoadFalse instruction");
}

/// Test: Parse multiplication expression
#[test]
fn test_parse_multiplication_to_bytecode() {
    let source = "3 * 4;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    let has_mul = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::Mul));
    assert!(has_mul, "Should have Mul instruction");
}

/// Test: Parse comparison operators
#[test]
fn test_parse_comparison_to_bytecode() {
    let source = "10 < 20;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    let has_less_than = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::LessThan));
    assert!(has_less_than, "Should have LessThan instruction");
}

/// Test: Parse if statement generates control flow bytecode
#[test]
fn test_parse_if_statement_to_bytecode() {
    let source = "if (true) { 1; } else { 2; }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have conditional jumps
    let has_jump_if_false = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::JumpIfFalse(_)));
    let has_jump = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::Jump(_)));

    assert!(has_jump_if_false, "Should have JumpIfFalse for if condition");
    assert!(has_jump, "Should have Jump for skipping else branch");
}

/// Test: Parse while loop generates loop bytecode
#[test]
fn test_parse_while_loop_to_bytecode() {
    let source = "while (false) { 1; }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have both conditional jump and unconditional jump (loop back)
    let has_jump_if_false = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::JumpIfFalse(_)));
    let has_jump = chunk
        .instructions
        .iter()
        .any(|i| matches!(i.opcode, Opcode::Jump(_)));

    assert!(has_jump_if_false, "Should have JumpIfFalse for loop condition");
    assert!(has_jump, "Should have Jump for loop back");
}

/// Test: Parse string literal
#[test]
fn test_parse_string_literal() {
    let source = r#""hello";"#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have string constant
    let has_string = chunk.constants.iter().any(|c| {
        matches!(c, bytecode_system::Value::String(s) if s == "hello")
    });
    assert!(has_string, "Should have string constant 'hello'");
}

/// Test: Parse complex expression
#[test]
fn test_parse_complex_expression() {
    let source = "let result = (10 + 20) * 2 - 5;";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Should have multiple arithmetic operations
    let has_add = chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::Add));
    let has_mul = chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::Mul));
    let has_sub = chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::Sub));

    assert!(has_add, "Should have Add instruction");
    assert!(has_mul, "Should have Mul instruction");
    assert!(has_sub, "Should have Sub instruction");
}

/// Test: Empty program compiles correctly
#[test]
fn test_parse_empty_program() {
    let source = "";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse empty program");

    let mut gen = BytecodeGenerator::new();
    let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

    // Empty program should still have a return
    let has_return = chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::Return));
    assert!(has_return, "Empty program should have Return instruction");
}
