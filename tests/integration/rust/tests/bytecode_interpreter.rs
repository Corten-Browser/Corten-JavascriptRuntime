//! Bytecode to Interpreter Integration Tests
//!
//! Tests the integration between bytecode_system and interpreter components.
//! Verifies that bytecode instructions are correctly executed by the VM.

use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
use core_types::Value;
use interpreter::VM;

/// Test: Execute LoadConstant and Return
#[test]
fn test_execute_load_constant_number() {
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(BcValue::Number(42.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute addition
#[test]
fn test_execute_addition() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(10.0));
    let idx2 = chunk.add_constant(BcValue::Number(32.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Add);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute subtraction
#[test]
fn test_execute_subtraction() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(50.0));
    let idx2 = chunk.add_constant(BcValue::Number(8.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Sub);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute multiplication
#[test]
fn test_execute_multiplication() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(6.0));
    let idx2 = chunk.add_constant(BcValue::Number(7.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Mul);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute division
#[test]
fn test_execute_division() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(84.0));
    let idx2 = chunk.add_constant(BcValue::Number(2.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Div);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute LoadTrue
#[test]
fn test_execute_load_true() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected true, got {:?}", result),
    }
}

/// Test: Execute LoadFalse
#[test]
fn test_execute_load_false() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadFalse);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(!b),
        _ => panic!("Expected false, got {:?}", result),
    }
}

/// Test: Execute LoadUndefined
#[test]
fn test_execute_load_undefined() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Execute LoadNull
#[test]
fn test_execute_load_null() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadNull);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    assert!(matches!(result, Value::Null));
}

/// Test: Execute comparison - LessThan
#[test]
fn test_execute_less_than_true() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(5.0));
    let idx2 = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LessThan);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected boolean, got {:?}", result),
    }
}

/// Test: Execute comparison - LessThan false
#[test]
fn test_execute_less_than_false() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(10.0));
    let idx2 = chunk.add_constant(BcValue::Number(5.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LessThan);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(!b),
        _ => panic!("Expected boolean, got {:?}", result),
    }
}

/// Test: Execute StrictEqual
#[test]
fn test_execute_strict_equal() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(BcValue::Number(42.0));
    let idx2 = chunk.add_constant(BcValue::Number(42.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::StrictEqual);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected boolean, got {:?}", result),
    }
}

/// Test: Execute global variable store and load
#[test]
fn test_execute_global_variable() {
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(BcValue::Number(100.0));

    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::StoreGlobal("myVar".to_string()));
    chunk.emit(Opcode::LoadGlobal("myVar".to_string()));
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 100),
        Value::Double(n) => assert_eq!(n, 100.0),
        _ => panic!("Expected number result, got {:?}", result),
    }
}

/// Test: Execute conditional jump - JumpIfFalse (taken)
#[test]
fn test_execute_jump_if_false_taken() {
    let mut chunk = BytecodeChunk::new();
    let idx_true = chunk.add_constant(BcValue::Number(100.0));
    let idx_false = chunk.add_constant(BcValue::Number(200.0));

    chunk.emit(Opcode::LoadFalse);
    chunk.emit(Opcode::JumpIfFalse(4)); // Jump to LoadConstant(idx_false)
    chunk.emit(Opcode::LoadConstant(idx_true));
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::LoadConstant(idx_false)); // Index 4
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 200),
        Value::Double(n) => assert_eq!(n, 200.0),
        _ => panic!("Expected 200, got {:?}", result),
    }
}

/// Test: Execute conditional jump - JumpIfFalse (not taken)
#[test]
fn test_execute_jump_if_false_not_taken() {
    let mut chunk = BytecodeChunk::new();
    let idx_true = chunk.add_constant(BcValue::Number(100.0));
    let idx_false = chunk.add_constant(BcValue::Number(200.0));

    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::JumpIfFalse(4)); // Jump to LoadConstant(idx_false)
    chunk.emit(Opcode::LoadConstant(idx_true));
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::LoadConstant(idx_false)); // Index 4
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 100),
        Value::Double(n) => assert_eq!(n, 100.0),
        _ => panic!("Expected 100, got {:?}", result),
    }
}

/// Test: Execute negation
#[test]
fn test_execute_negation() {
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(BcValue::Number(42.0));

    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Neg);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, -42),
        Value::Double(n) => assert_eq!(n, -42.0),
        _ => panic!("Expected -42, got {:?}", result),
    }
}

/// Test: Empty bytecode (just return undefined)
#[test]
fn test_execute_empty_bytecode() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    assert!(matches!(result, Value::Undefined));
}

/// Test: Complex arithmetic expression
#[test]
fn test_execute_complex_arithmetic() {
    // Calculate: (10 + 20) * 3 - 48 = 42
    let mut chunk = BytecodeChunk::new();
    let idx_10 = chunk.add_constant(BcValue::Number(10.0));
    let idx_20 = chunk.add_constant(BcValue::Number(20.0));
    let idx_3 = chunk.add_constant(BcValue::Number(3.0));
    let idx_48 = chunk.add_constant(BcValue::Number(48.0));

    chunk.emit(Opcode::LoadConstant(idx_10));
    chunk.emit(Opcode::LoadConstant(idx_20));
    chunk.emit(Opcode::Add); // 30
    chunk.emit(Opcode::LoadConstant(idx_3));
    chunk.emit(Opcode::Mul); // 90
    chunk.emit(Opcode::LoadConstant(idx_48));
    chunk.emit(Opcode::Sub); // 42
    chunk.emit(Opcode::Return);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("Execution failed");

    match result {
        Value::Smi(n) => assert_eq!(n, 42),
        Value::Double(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected 42, got {:?}", result),
    }
}
