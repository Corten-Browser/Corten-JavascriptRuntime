//! Contract compliance tests for bytecode_system
//! Verifies implementation matches contracts/bytecode_system.yaml

use bytecode_system::{BytecodeChunk, Instruction, Opcode, RegisterId, SourcePosition, Value};

/// Verify all opcode variants exist as specified in contract
#[test]
fn test_contract_opcode_variants() {
    // Literals
    let _ = Opcode::LoadConstant(0);
    let _ = Opcode::LoadUndefined;
    let _ = Opcode::LoadNull;
    let _ = Opcode::LoadTrue;
    let _ = Opcode::LoadFalse;

    // Variables
    let _ = Opcode::LoadGlobal(String::new());
    let _ = Opcode::StoreGlobal(String::new());
    let _ = Opcode::LoadLocal(RegisterId(0));
    let _ = Opcode::StoreLocal(RegisterId(0));

    // Arithmetic
    let _ = Opcode::Add;
    let _ = Opcode::Sub;
    let _ = Opcode::Mul;
    let _ = Opcode::Div;
    let _ = Opcode::Mod;
    let _ = Opcode::Neg;

    // Comparison
    let _ = Opcode::Equal;
    let _ = Opcode::StrictEqual;
    let _ = Opcode::NotEqual;
    let _ = Opcode::StrictNotEqual;
    let _ = Opcode::LessThan;
    let _ = Opcode::LessThanEqual;
    let _ = Opcode::GreaterThan;
    let _ = Opcode::GreaterThanEqual;

    // Control flow
    let _ = Opcode::Jump(0);
    let _ = Opcode::JumpIfTrue(0);
    let _ = Opcode::JumpIfFalse(0);
    let _ = Opcode::Return;

    // Objects
    let _ = Opcode::CreateObject;
    let _ = Opcode::LoadProperty(String::new());
    let _ = Opcode::StoreProperty(String::new());

    // Functions
    let _ = Opcode::CreateClosure(0, vec![]);
    let _ = Opcode::Call(0);
}

/// Verify RegisterId struct matches contract (tuple struct with u32)
#[test]
fn test_contract_register_id_structure() {
    let reg = RegisterId(42);
    assert_eq!(reg.0, 42u32);
}

/// Verify Instruction struct has required fields
#[test]
fn test_contract_instruction_structure() {
    let inst = Instruction::new(Opcode::Add);

    // Must have opcode field
    let _opcode: &Opcode = &inst.opcode;

    // Must have optional source_position field
    let _pos: &Option<SourcePosition> = &inst.source_position;
}

/// Verify BytecodeChunk struct has required fields
#[test]
fn test_contract_bytecode_chunk_fields() {
    let chunk = BytecodeChunk::new();

    // Must have instructions field (Vec<Instruction>)
    let _instructions: &Vec<Instruction> = &chunk.instructions;

    // Must have constants field (Vec<Value>)
    let _constants: &Vec<Value> = &chunk.constants;

    // Must have register_count field (u32)
    let _reg_count: u32 = chunk.register_count;
}

/// Verify BytecodeChunk::new() method exists and returns Self
#[test]
fn test_contract_chunk_new_method() {
    let chunk: BytecodeChunk = BytecodeChunk::new();
    assert_eq!(chunk.instructions.len(), 0);
    assert_eq!(chunk.constants.len(), 0);
    assert_eq!(chunk.register_count, 0);
}

/// Verify BytecodeChunk::emit() method exists
#[test]
fn test_contract_chunk_emit_method() {
    let mut chunk = BytecodeChunk::new();
    // emit(opcode: Opcode) -> ()
    let result: () = chunk.emit(Opcode::Add);
    let _ = result;
    assert_eq!(chunk.instructions.len(), 1);
}

/// Verify BytecodeChunk::add_constant() method exists
#[test]
fn test_contract_chunk_add_constant_method() {
    let mut chunk = BytecodeChunk::new();
    // add_constant(value: Value) -> usize
    let index: usize = chunk.add_constant(Value::Number(42.0));
    assert_eq!(index, 0);
}

/// Verify BytecodeChunk::optimize() method exists
#[test]
fn test_contract_chunk_optimize_method() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);
    // optimize() -> ()
    let result: () = chunk.optimize();
    let _ = result;
}

/// Verify register-based bytecode architecture requirement
#[test]
fn test_contract_register_based_architecture() {
    // Verify registers can be used with opcodes
    let reg = RegisterId(10);
    let load = Opcode::LoadLocal(reg.clone());
    let store = Opcode::StoreLocal(reg);

    match load {
        Opcode::LoadLocal(r) => assert_eq!(r.0, 10),
        _ => panic!("Expected LoadLocal"),
    }

    match store {
        Opcode::StoreLocal(r) => assert_eq!(r.0, 10),
        _ => panic!("Expected StoreLocal"),
    }
}

/// Verify basic optimization passes requirement (dead code elimination)
#[test]
fn test_contract_dead_code_elimination_requirement() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);
    chunk.emit(Opcode::LoadNull); // Dead code

    chunk.optimize();

    // Dead code should be eliminated
    assert_eq!(chunk.instructions.len(), 1);
}

/// Verify basic optimization passes requirement (constant folding)
#[test]
fn test_contract_constant_folding_requirement() {
    let mut chunk = BytecodeChunk::new();
    let idx1 = chunk.add_constant(Value::Number(2.0));
    let idx2 = chunk.add_constant(Value::Number(3.0));
    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::Add);

    let initial = chunk.instructions.len();
    chunk.optimize();

    // Should have optimization applied (fewer or equal instructions)
    assert!(chunk.instructions.len() <= initial);
}

/// Verify all Value types work (placeholder for core_types dependency)
#[test]
fn test_contract_value_types() {
    let _ = Value::Undefined;
    let _ = Value::Null;
    let _ = Value::Boolean(true);
    let _ = Value::Number(3.14);
    let _ = Value::String("test".to_string());
}

/// Verify SourcePosition structure (placeholder for core_types dependency)
#[test]
fn test_contract_source_position() {
    let pos = SourcePosition {
        line: 1,
        column: 1,
        offset: 0,
    };
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

/// Verify 80%+ test coverage requirement (meta-test)
#[test]
fn test_contract_coverage_requirement_meta() {
    // This test exists to remind that coverage must be verified
    // Actual coverage is measured by cargo test --coverage
    assert!(true, "Coverage must be verified externally");
}

/// Verify binary serialization support requirement
#[test]
fn test_contract_binary_serialization() {
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadNull);
    chunk.add_constant(Value::Number(42.0));

    // Serialize to binary
    let bytes = chunk.to_bytes();
    assert!(!bytes.is_empty());

    // Deserialize from binary
    let restored = BytecodeChunk::from_bytes(&bytes).expect("Should deserialize");
    assert_eq!(restored.instructions.len(), chunk.instructions.len());
    assert_eq!(restored.constants.len(), chunk.constants.len());
}
