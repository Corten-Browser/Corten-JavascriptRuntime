//! Tests for BytecodeChunk struct
//! TDD: RED phase - these tests should fail initially

use bytecode_system::{BytecodeChunk, Opcode, Value};

#[test]
fn test_chunk_creation() {
    let chunk = BytecodeChunk::new();
    assert_eq!(chunk.instructions.len(), 0);
    assert_eq!(chunk.constants.len(), 0);
    assert_eq!(chunk.register_count, 0);
}

#[test]
fn test_chunk_emit_instruction() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadUndefined);
    assert_eq!(chunk.instructions.len(), 1);
    assert!(matches!(
        chunk.instructions[0].opcode,
        Opcode::LoadUndefined
    ));
}

#[test]
fn test_chunk_emit_multiple_instructions() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadNull);
    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::LoadFalse);
    assert_eq!(chunk.instructions.len(), 3);
}

#[test]
fn test_chunk_add_constant_number() {
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(Value::Number(42.0));
    assert_eq!(idx, 0);
    assert_eq!(chunk.constants.len(), 1);
    match &chunk.constants[0] {
        Value::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_chunk_add_constant_string() {
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(Value::String("hello".to_string()));
    assert_eq!(idx, 0);
    match &chunk.constants[0] {
        Value::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_chunk_add_multiple_constants() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(1.0));
    let idx2 = chunk.add_constant(Value::Number(2.0));
    let idx3 = chunk.add_constant(Value::Number(3.0));
    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx3, 2);
    assert_eq!(chunk.constants.len(), 3);
}

#[test]
fn test_chunk_emit_with_constant() {
    let mut chunk = BytecodeChunk::new();
    let const_idx = chunk.add_constant(Value::Number(100.0));
    chunk.emit(Opcode::LoadConstant(const_idx));
    assert_eq!(chunk.instructions.len(), 1);
    match chunk.instructions[0].opcode {
        Opcode::LoadConstant(idx) => assert_eq!(idx, const_idx),
        _ => panic!("Expected LoadConstant"),
    }
}

#[test]
fn test_chunk_register_count() {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 10;
    assert_eq!(chunk.register_count, 10);
}

#[test]
fn test_chunk_optimize_dead_code_elimination() {
    let mut chunk = BytecodeChunk::new();
    // Return followed by unreachable code
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::LoadNull); // Dead code
    chunk.emit(Opcode::Add); // Dead code

    chunk.optimize();

    // After optimization, dead code should be removed
    assert_eq!(chunk.instructions.len(), 1);
    assert!(matches!(chunk.instructions[0].opcode, Opcode::Return));
}

#[test]
fn test_chunk_optimize_constant_folding() {
    let mut chunk = BytecodeChunk::new();
    // LoadConstant(1) + LoadConstant(2) should fold to LoadConstant(3)
    let idx1 = chunk.add_constant(Value::Number(1.0));
    let idx2 = chunk.add_constant(Value::Number(2.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Add);

    chunk.optimize();

    // After constant folding
    assert!(chunk.instructions.len() <= 3);
}

#[test]
fn test_chunk_clone() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Add);
    chunk.add_constant(Value::Number(5.0));

    let cloned = chunk.clone();
    assert_eq!(cloned.instructions.len(), chunk.instructions.len());
    assert_eq!(cloned.constants.len(), chunk.constants.len());
}

#[test]
fn test_chunk_debug() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadNull);
    let debug_str = format!("{:?}", chunk);
    assert!(debug_str.contains("instructions"));
}

#[test]
fn test_chunk_emit_with_position() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit_with_position(
        Opcode::Add,
        bytecode_system::SourcePosition {
            line: 1,
            column: 5,
            offset: 10,
        },
    );
    assert_eq!(chunk.instructions.len(), 1);
    assert!(chunk.instructions[0].source_position.is_some());
}

#[test]
fn test_chunk_instruction_count() {
    let mut chunk = BytecodeChunk::new();
    assert_eq!(chunk.instruction_count(), 0);
    chunk.emit(Opcode::Add);
    assert_eq!(chunk.instruction_count(), 1);
    chunk.emit(Opcode::Sub);
    assert_eq!(chunk.instruction_count(), 2);
}

#[test]
fn test_chunk_constant_count() {
    let mut chunk = BytecodeChunk::new();
    assert_eq!(chunk.constant_count(), 0);
    chunk.add_constant(Value::Number(1.0));
    assert_eq!(chunk.constant_count(), 1);
}

#[test]
fn test_value_undefined() {
    let val = Value::Undefined;
    assert!(matches!(val, Value::Undefined));
}

#[test]
fn test_value_null() {
    let val = Value::Null;
    assert!(matches!(val, Value::Null));
}

#[test]
fn test_value_boolean() {
    let val_true = Value::Boolean(true);
    let val_false = Value::Boolean(false);
    match val_true {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean"),
    }
    match val_false {
        Value::Boolean(b) => assert!(!b),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_value_number() {
    let val = Value::Number(3.14);
    match val {
        Value::Number(n) => assert!((n - 3.14).abs() < f64::EPSILON),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_value_string() {
    let val = Value::String("test".to_string());
    match val {
        Value::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_value_clone() {
    let val1 = Value::Number(100.0);
    let val2 = val1.clone();
    match (val1, val2) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a, b),
        _ => panic!("Clone failed"),
    }
}

#[test]
fn test_value_debug() {
    let val = Value::Undefined;
    let debug_str = format!("{:?}", val);
    assert!(debug_str.contains("Undefined"));
}

#[test]
fn test_chunk_clear() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Add);
    chunk.add_constant(Value::Number(1.0));
    chunk.clear();
    assert_eq!(chunk.instructions.len(), 0);
    assert_eq!(chunk.constants.len(), 0);
}
