//! End-to-End CLI Integration Tests
//!
//! Tests the complete JavaScript runtime through the js_cli Runtime API.
//! This is the highest level integration test - source code to final result.

use core_types::Value;
use js_cli::Runtime;

/// Test: Simple number execution
#[test]
fn test_e2e_simple_number() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("42;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number, got {:?}", result),
    }
}

/// Test: Addition
#[test]
fn test_e2e_addition() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("1 + 2;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 3),
        Value::Double(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected 3, got {:?}", result),
    }
}

/// Test: Complex arithmetic
#[test]
fn test_e2e_complex_arithmetic() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("(10 + 20) * 2 - 18;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected 42, got {:?}", result),
    }
}

/// Test: Variable declaration and usage
#[test]
fn test_e2e_variable() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("let x = 100; x;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 100),
        Value::Double(n) => assert_eq!(n, 100.0),
        _ => panic!("Expected 100, got {:?}", result),
    }
}

/// Test: Multiple variables
#[test]
fn test_e2e_multiple_variables() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("let a = 10; let b = 20; let c = 30; a + b + c;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 60),
        Value::Double(n) => assert_eq!(n, 60.0),
        _ => panic!("Expected 60, got {:?}", result),
    }
}

/// Test: Boolean operations
#[test]
fn test_e2e_boolean_true() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("true;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Comparison operations
#[test]
fn test_e2e_comparison() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("5 < 10;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Empty program
#[test]
fn test_e2e_empty_program() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("").expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Runtime with JIT enabled (configuration test)
#[test]
fn test_e2e_jit_enabled_config() {
    let runtime = Runtime::new(true);
    assert!(runtime.is_jit_enabled(), "JIT should be enabled");
}

/// Test: Runtime with JIT disabled (configuration test)
#[test]
fn test_e2e_jit_disabled_config() {
    let runtime = Runtime::new(false);
    assert!(!runtime.is_jit_enabled(), "JIT should be disabled");
}

/// Test: Runtime with bytecode printing enabled
#[test]
fn test_e2e_bytecode_printing_config() {
    let runtime = Runtime::new(false).with_print_bytecode(true);
    assert!(
        runtime.is_print_bytecode_enabled(),
        "Bytecode printing should be enabled"
    );
}

/// Test: Runtime with AST printing enabled
#[test]
fn test_e2e_ast_printing_config() {
    let runtime = Runtime::new(false).with_print_ast(true);
    assert!(
        runtime.is_print_ast_enabled(),
        "AST printing should be enabled"
    );
}

/// Test: Chained builder pattern
#[test]
fn test_e2e_builder_pattern() {
    let runtime = Runtime::new(true)
        .with_print_bytecode(true)
        .with_print_ast(true);

    assert!(runtime.is_jit_enabled());
    assert!(runtime.is_print_bytecode_enabled());
    assert!(runtime.is_print_ast_enabled());
}

/// Test: Null value
#[test]
fn test_e2e_null() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("null;").expect("Execution failed");

    assert!(matches!(result, Value::Null));
}

/// Test: Undefined value
#[test]
fn test_e2e_undefined() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("undefined;")
        .expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Division
#[test]
fn test_e2e_division() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("100 / 4;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 25),
        Value::Double(n) => assert_eq!(n, 25.0),
        _ => panic!("Expected 25, got {:?}", result),
    }
}

/// Test: Modulo
#[test]
fn test_e2e_modulo() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("17 % 5;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 2),
        Value::Double(n) => assert_eq!(n, 2.0),
        _ => panic!("Expected 2, got {:?}", result),
    }
}

/// Test: Negation
#[test]
fn test_e2e_negation() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("-42;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, -42),
        Value::Double(n) => assert_eq!(n, -42.0),
        _ => panic!("Expected -42, got {:?}", result),
    }
}

/// Test: Strict equality
#[test]
fn test_e2e_strict_equality() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("42 === 42;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Strict inequality
#[test]
fn test_e2e_strict_inequality() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("10 !== 20;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Multiple statement program
#[test]
fn test_e2e_multiple_statements() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("let x = 10; let y = x * 2; let z = y + 5; z;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 25), // x=10, y=20, z=25
        Value::Double(n) => assert_eq!(n, 25.0),
        _ => panic!("Expected 25, got {:?}", result),
    }
}

/// Test: Const declaration
#[test]
fn test_e2e_const_declaration() {
    let mut runtime = Runtime::new(false);
    let result = runtime
        .execute_string("const MAX = 1000; MAX;")
        .expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 1000),
        Value::Double(n) => assert_eq!(n, 1000.0),
        _ => panic!("Expected 1000, got {:?}", result),
    }
}

/// Test: Float arithmetic
#[test]
fn test_e2e_float_arithmetic() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("3.5 + 2.5;").expect("Execution failed");

    match result {
        Value::Double(n) => assert!((n - 6.0).abs() < 0.001),
        _ => panic!("Expected 6.0, got {:?}", result),
    }
}

/// Test: Greater than comparison
#[test]
fn test_e2e_greater_than() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("100 > 50;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Less than or equal
#[test]
fn test_e2e_less_than_or_equal() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("5 <= 5;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Greater than or equal
#[test]
fn test_e2e_greater_than_or_equal() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("10 >= 10;").expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Operator precedence (multiplication before addition)
#[test]
fn test_e2e_operator_precedence() {
    let mut runtime = Runtime::new(false);
    let result = runtime.execute_string("2 + 3 * 4;").expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 14), // 3*4=12, 12+2=14
        Value::Double(n) => assert_eq!(n, 14.0),
        _ => panic!("Expected 14, got {:?}", result),
    }
}
