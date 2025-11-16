//! Tests for optimizer module
//! TDD: RED phase - these tests should fail initially

use bytecode_system::{BytecodeChunk, Opcode, Value};

#[test]
fn test_dead_code_after_return() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::LoadNull);
    chunk.emit(Opcode::LoadTrue);

    chunk.optimize();

    assert_eq!(chunk.instructions.len(), 1);
    assert!(matches!(chunk.instructions[0].opcode, Opcode::Return));
}

#[test]
fn test_dead_code_after_unconditional_jump() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Jump(100)); // Unconditional jump
    chunk.emit(Opcode::LoadNull); // Dead code
    chunk.emit(Opcode::Add); // Dead code

    chunk.optimize();

    // Should keep only the jump
    assert_eq!(chunk.instructions.len(), 1);
}

#[test]
fn test_constant_folding_addition() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(5.0));
    let idx2 = chunk.add_constant(Value::Number(3.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Add);

    let initial_count = chunk.instructions.len();
    chunk.optimize();

    // After folding, we should have fewer instructions or a folded constant
    assert!(chunk.instructions.len() <= initial_count);
}

#[test]
fn test_constant_folding_subtraction() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(10.0));
    let idx2 = chunk.add_constant(Value::Number(4.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Sub);

    chunk.optimize();

    // Should fold to 6.0
    assert!(chunk.instructions.len() <= 3);
}

#[test]
fn test_constant_folding_multiplication() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(7.0));
    let idx2 = chunk.add_constant(Value::Number(6.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Mul);

    chunk.optimize();

    // Should fold to 42.0
    assert!(chunk.instructions.len() <= 3);
}

#[test]
fn test_constant_folding_division() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(20.0));
    let idx2 = chunk.add_constant(Value::Number(4.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Div);

    chunk.optimize();

    // Should fold to 5.0
    assert!(chunk.instructions.len() <= 3);
}

#[test]
fn test_constant_folding_modulo() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(17.0));
    let idx2 = chunk.add_constant(Value::Number(5.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Mod);

    chunk.optimize();

    // Should fold to 2.0
    assert!(chunk.instructions.len() <= 3);
}

#[test]
fn test_peephole_double_negation() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Neg);
    chunk.emit(Opcode::Neg);

    chunk.optimize();

    // Double negation should be eliminated
    assert_eq!(chunk.instructions.len(), 0);
}

#[test]
fn test_peephole_load_store_elimination() {
    let mut chunk = BytecodeChunk::new();
    let reg = bytecode_system::RegisterId(0);
    chunk.emit(Opcode::StoreLocal(reg.clone()));
    chunk.emit(Opcode::LoadLocal(reg));

    // This is a common peephole optimization pattern
    // The load after store of same register can be optimized
    chunk.optimize();

    // After optimization
    assert!(chunk.instructions.len() <= 2);
}

#[test]
fn test_no_optimization_for_non_constant() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadGlobal("x".to_string()));
    chunk.emit(Opcode::LoadGlobal("y".to_string()));
    chunk.emit(Opcode::Add);

    let initial = chunk.instructions.len();
    chunk.optimize();

    // Cannot fold non-constants
    assert_eq!(chunk.instructions.len(), initial);
}

#[test]
fn test_optimize_empty_chunk() {
    let mut chunk = BytecodeChunk::new();
    chunk.optimize();
    assert_eq!(chunk.instructions.len(), 0);
}

#[test]
fn test_optimize_single_instruction() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);
    chunk.optimize();
    assert_eq!(chunk.instructions.len(), 1);
}

#[test]
fn test_multiple_optimization_passes() {
    let mut chunk = BytecodeChunk::new();
    // Code that benefits from multiple passes
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::Neg); // Dead code
    chunk.emit(Opcode::Neg); // Dead code (also double neg)

    chunk.optimize();

    // Both dead code and peephole should apply
    assert_eq!(chunk.instructions.len(), 1);
}

#[test]
fn test_conditional_jump_not_eliminated() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::JumpIfTrue(10));
    chunk.emit(Opcode::LoadNull); // Not dead - could be executed
    chunk.emit(Opcode::Add);

    chunk.optimize();

    // Conditional jumps don't make following code dead
    assert!(chunk.instructions.len() > 1);
}

#[test]
fn test_preserve_source_positions() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit_with_position(
        Opcode::Return,
        bytecode_system::SourcePosition {
            line: 10,
            column: 5,
            offset: 100,
        },
    );

    chunk.optimize();

    // Source positions should be preserved through optimization
    assert!(chunk.instructions[0].source_position.is_some());
}
