//! Tests for Instruction struct
//! TDD: RED phase - these tests should fail initially

use bytecode_system::{Instruction, Opcode, SourcePosition};

#[test]
fn test_instruction_creation() {
    let inst = Instruction::new(Opcode::Add);
    assert!(matches!(inst.opcode, Opcode::Add));
    assert!(inst.source_position.is_none());
}

#[test]
fn test_instruction_with_source_position() {
    let pos = SourcePosition {
        line: 10,
        column: 5,
        offset: 100,
    };
    let inst = Instruction::with_position(Opcode::Sub, pos);
    assert!(matches!(inst.opcode, Opcode::Sub));
    match inst.source_position {
        Some(sp) => {
            assert_eq!(sp.line, 10);
            assert_eq!(sp.column, 5);
            assert_eq!(sp.offset, 100);
        }
        None => panic!("Expected source position"),
    }
}

#[test]
fn test_instruction_opcode_access() {
    let inst = Instruction::new(Opcode::LoadConstant(42));
    match inst.opcode {
        Opcode::LoadConstant(idx) => assert_eq!(idx, 42),
        _ => panic!("Expected LoadConstant"),
    }
}

#[test]
fn test_instruction_clone() {
    let inst1 = Instruction::new(Opcode::Mul);
    let inst2 = inst1.clone();
    assert!(matches!(inst2.opcode, Opcode::Mul));
}

#[test]
fn test_instruction_debug() {
    let inst = Instruction::new(Opcode::Div);
    let debug_str = format!("{:?}", inst);
    assert!(debug_str.contains("Div"));
}

#[test]
fn test_source_position_creation() {
    let pos = SourcePosition {
        line: 1,
        column: 1,
        offset: 0,
    };
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_source_position_clone() {
    let pos1 = SourcePosition {
        line: 20,
        column: 15,
        offset: 500,
    };
    let pos2 = pos1.clone();
    assert_eq!(pos1.line, pos2.line);
    assert_eq!(pos1.column, pos2.column);
    assert_eq!(pos1.offset, pos2.offset);
}

#[test]
fn test_source_position_debug() {
    let pos = SourcePosition {
        line: 42,
        column: 10,
        offset: 1000,
    };
    let debug_str = format!("{:?}", pos);
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("10"));
}

#[test]
fn test_instruction_equality() {
    let inst1 = Instruction::new(Opcode::Return);
    let inst2 = Instruction::new(Opcode::Return);
    assert_eq!(inst1.opcode, inst2.opcode);
}
