//! Full Pipeline Integration Tests
//!
//! Tests the complete flow: Source -> Parser -> AST -> BytecodeGenerator -> Bytecode -> VM -> Result
//! This is the most critical integration test suite.

use core_types::Value;
use interpreter::VM;
use parser::{BytecodeGenerator, Parser};

/// Helper function to execute JavaScript source code
fn execute_js(source: &str) -> Result<Value, String> {
    let mut parser = Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut gen = BytecodeGenerator::new();
    let bytecode = gen
        .generate(&ast)
        .map_err(|e| format!("Bytecode generation error: {:?}", e))?;

    let mut vm = VM::new();
    vm.execute(&bytecode)
        .map_err(|e| format!("Execution error: {:?}", e))
}

/// Test: Simple number literal
#[test]
fn test_full_pipeline_number() {
    let result = execute_js("42;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number, got {:?}", result),
    }
}

/// Test: Addition expression
#[test]
fn test_full_pipeline_addition() {
    let result = execute_js("1 + 2;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 3),
        Value::Double(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected 3, got {:?}", result),
    }
}

/// Test: Subtraction expression
#[test]
fn test_full_pipeline_subtraction() {
    let result = execute_js("10 - 3;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 7),
        Value::Double(n) => assert_eq!(n, 7.0),
        _ => panic!("Expected 7, got {:?}", result),
    }
}

/// Test: Multiplication expression
#[test]
fn test_full_pipeline_multiplication() {
    let result = execute_js("6 * 7;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected 42, got {:?}", result),
    }
}

/// Test: Division expression
#[test]
fn test_full_pipeline_division() {
    let result = execute_js("100 / 5;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 20),
        Value::Double(n) => assert_eq!(n, 20.0),
        _ => panic!("Expected 20, got {:?}", result),
    }
}

/// Test: Complex arithmetic
#[test]
fn test_full_pipeline_complex_arithmetic() {
    let result = execute_js("(10 + 20) * 2 - 18;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected 42, got {:?}", result),
    }
}

/// Test: Boolean true
#[test]
fn test_full_pipeline_boolean_true() {
    let result = execute_js("true;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Boolean false
#[test]
fn test_full_pipeline_boolean_false() {
    let result = execute_js("false;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(!b),
        _ => panic!("Expected false, got {:?}", result),
    }
}

/// Test: Null value
#[test]
fn test_full_pipeline_null() {
    let result = execute_js("null;").expect("Execution failed");

    assert!(matches!(result, Value::Null));
}

/// Test: Undefined value
#[test]
fn test_full_pipeline_undefined() {
    let result = execute_js("undefined;").expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Variable declaration and usage
#[test]
fn test_full_pipeline_variable() {
    let result = execute_js("let x = 50; x;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 50),
        Value::Double(n) => assert_eq!(n, 50.0),
        _ => panic!("Expected 50, got {:?}", result),
    }
}

/// Test: Multiple variable declarations
#[test]
fn test_full_pipeline_multiple_variables() {
    let result = execute_js("let a = 10; let b = 20; a + b;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 30),
        Value::Double(n) => assert_eq!(n, 30.0),
        _ => panic!("Expected 30, got {:?}", result),
    }
}

/// Test: Variable with expression
#[test]
fn test_full_pipeline_variable_expression() {
    let result = execute_js("let x = 10 + 5; let y = x * 2; y;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 30),
        Value::Double(n) => assert_eq!(n, 30.0),
        _ => panic!("Expected 30, got {:?}", result),
    }
}

/// Test: Comparison less than
#[test]
fn test_full_pipeline_less_than() {
    let result = execute_js("5 < 10;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Comparison greater than
#[test]
fn test_full_pipeline_greater_than() {
    let result = execute_js("10 > 5;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Strict equality
#[test]
fn test_full_pipeline_strict_equal() {
    let result = execute_js("42 === 42;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Strict inequality
#[test]
fn test_full_pipeline_strict_not_equal() {
    let result = execute_js("10 !== 20;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Empty program returns undefined
#[test]
fn test_full_pipeline_empty_program() {
    let result = execute_js("").expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Multiple statements, return last
#[test]
fn test_full_pipeline_multiple_statements() {
    let result = execute_js("1; 2; 3;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 3),
        Value::Double(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected 3, got {:?}", result),
    }
}

/// Test: Modulo operation
#[test]
fn test_full_pipeline_modulo() {
    let result = execute_js("17 % 5;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 2),
        Value::Double(n) => assert_eq!(n, 2.0),
        _ => panic!("Expected 2, got {:?}", result),
    }
}

/// Test: Negation
#[test]
fn test_full_pipeline_negation() {
    let result = execute_js("-42;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, -42),
        Value::Double(n) => assert_eq!(n, -42.0),
        _ => panic!("Expected -42, got {:?}", result),
    }
}

/// Test: Floating point arithmetic
#[test]
fn test_full_pipeline_float_arithmetic() {
    let result = execute_js("3.14 * 2;").expect("Execution failed");

    match result {
        Value::Double(n) => {
            assert!((n - 6.28).abs() < 0.001, "Expected ~6.28, got {}", n)
        }
        _ => panic!("Expected double, got {:?}", result),
    }
}

/// Test: Nested arithmetic with parentheses
#[test]
fn test_full_pipeline_nested_parentheses() {
    let result = execute_js("((2 + 3) * (4 + 5));").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 45),
        Value::Double(n) => assert_eq!(n, 45.0),
        _ => panic!("Expected 45, got {:?}", result),
    }
}

/// Test: Const declaration
#[test]
fn test_full_pipeline_const_declaration() {
    let result = execute_js("const PI = 3.14; PI;").expect("Execution failed");

    match result {
        Value::Double(n) => assert!((n - 3.14).abs() < 0.001),
        _ => panic!("Expected 3.14, got {:?}", result),
    }
}

/// Test: Var declaration
#[test]
fn test_full_pipeline_var_declaration() {
    let result = execute_js("var x = 99; x;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 99),
        Value::Double(n) => assert_eq!(n, 99.0),
        _ => panic!("Expected 99, got {:?}", result),
    }
}

/// Test: Less than or equal
#[test]
fn test_full_pipeline_less_than_equal() {
    let result = execute_js("5 <= 5;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Greater than or equal
#[test]
fn test_full_pipeline_greater_than_equal() {
    let result = execute_js("10 >= 10;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Order of operations (operator precedence)
#[test]
fn test_full_pipeline_operator_precedence() {
    let result = execute_js("2 + 3 * 4;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 14), // 3*4=12, 12+2=14
        Value::Double(n) => assert_eq!(n, 14.0),
        _ => panic!("Expected 14, got {:?}", result),
    }
}

/// Test: Chain of additions
#[test]
fn test_full_pipeline_chain_additions() {
    let result = execute_js("1 + 2 + 3 + 4 + 5;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 15),
        Value::Double(n) => assert_eq!(n, 15.0),
        _ => panic!("Expected 15, got {:?}", result),
    }
}

/// Test: Multiple operations in sequence
#[test]
fn test_full_pipeline_sequential_operations() {
    let result = execute_js("let x = 10; let y = x + 5; let z = y * 2; z - 10;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 20), // x=10, y=15, z=30, z-10=20
        Value::Double(n) => assert_eq!(n, 20.0),
        _ => panic!("Expected 20, got {:?}", result),
    }
}
