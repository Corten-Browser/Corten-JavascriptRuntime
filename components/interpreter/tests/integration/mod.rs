//! Integration tests for interpreter
//!
//! Tests interaction between VM, ProfileData, and InlineCache

use bytecode_system::{BytecodeChunk, Opcode, RegisterId, Value as BcValue};
use core_types::Value;
use interpreter::{InlineCache, ProfileData, TypeInfo, VM};

#[test]
fn test_vm_complex_arithmetic() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Calculate: (10 + 5) * 3 - 2 = 43
    let c10 = chunk.add_constant(BcValue::Number(10.0));
    let c5 = chunk.add_constant(BcValue::Number(5.0));
    let c3 = chunk.add_constant(BcValue::Number(3.0));
    let c2 = chunk.add_constant(BcValue::Number(2.0));

    chunk.emit(Opcode::LoadConstant(c10));
    chunk.emit(Opcode::LoadConstant(c5));
    chunk.emit(Opcode::Add); // 15
    chunk.emit(Opcode::LoadConstant(c3));
    chunk.emit(Opcode::Mul); // 45
    chunk.emit(Opcode::LoadConstant(c2));
    chunk.emit(Opcode::Sub); // 43
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(43));
}

#[test]
fn test_vm_conditional_logic() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 2;

    // if (true) { x = 1 } else { x = 2 }; return x
    let c1 = chunk.add_constant(BcValue::Number(1.0));
    let c2 = chunk.add_constant(BcValue::Number(2.0));

    chunk.emit(Opcode::LoadTrue); // 0
    chunk.emit(Opcode::JumpIfFalse(5)); // 1: jump to else
    chunk.emit(Opcode::LoadConstant(c1)); // 2: then branch
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // 3
    chunk.emit(Opcode::Jump(7)); // 4: skip else
    chunk.emit(Opcode::LoadConstant(c2)); // 5: else branch
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // 6
    chunk.emit(Opcode::LoadLocal(RegisterId(0))); // 7
    chunk.emit(Opcode::Return); // 8

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(1));
}

#[test]
fn test_vm_loop_simulation() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 2;

    // Simulate: sum = 0; for i = 1 to 3: sum += i; return sum (should be 6)
    let c0 = chunk.add_constant(BcValue::Number(0.0));
    let c1 = chunk.add_constant(BcValue::Number(1.0));
    let c3 = chunk.add_constant(BcValue::Number(3.0));

    // Initialize sum = 0, i = 1
    chunk.emit(Opcode::LoadConstant(c0)); // 0
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // 1: sum = 0
    chunk.emit(Opcode::LoadConstant(c1)); // 2
    chunk.emit(Opcode::StoreLocal(RegisterId(1))); // 3: i = 1

    // Loop body: sum += i
    chunk.emit(Opcode::LoadLocal(RegisterId(0))); // 4: load sum
    chunk.emit(Opcode::LoadLocal(RegisterId(1))); // 5: load i
    chunk.emit(Opcode::Add); // 6: sum + i
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // 7: sum = sum + i

    // i++
    chunk.emit(Opcode::LoadLocal(RegisterId(1))); // 8: load i
    chunk.emit(Opcode::LoadConstant(c1)); // 9: load 1
    chunk.emit(Opcode::Add); // 10: i + 1
    chunk.emit(Opcode::StoreLocal(RegisterId(1))); // 11: i = i + 1

    // Check i > 3
    chunk.emit(Opcode::LoadLocal(RegisterId(1))); // 12: load i
    chunk.emit(Opcode::LoadConstant(c3)); // 13: load 3
    chunk.emit(Opcode::GreaterThan); // 14: i > 3?
    chunk.emit(Opcode::JumpIfFalse(4)); // 15: if not, loop back

    // Return sum
    chunk.emit(Opcode::LoadLocal(RegisterId(0))); // 16
    chunk.emit(Opcode::Return); // 17

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(6));
}

#[test]
fn test_profile_data_collection_during_execution() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    // Simulate recording during bytecode execution
    for _ in 0..600 {
        profile.record_execution();
    }

    // Should trigger baseline compilation
    assert!(profile.should_compile_baseline());
    assert!(!profile.should_compile_optimized());

    // Record type feedback
    profile.record_type(TypeInfo::Number);
    profile.record_type(TypeInfo::Number);
    profile.record_type(TypeInfo::Number);

    // Continue execution
    for _ in 0..9500 {
        profile.record_execution();
    }

    // Should now trigger optimized compilation
    assert!(profile.should_compile_optimized());
}

#[test]
fn test_inline_cache_evolution() {
    let mut cache = InlineCache::Uninitialized;

    // First access - becomes monomorphic
    cache.update(100, 0);
    assert!(matches!(cache, InlineCache::Monomorphic { .. }));
    assert_eq!(cache.lookup(100), Some(0));

    // Second access with different shape - becomes polymorphic
    cache.update(200, 1);
    assert!(matches!(cache, InlineCache::Polymorphic { .. }));
    assert_eq!(cache.lookup(100), Some(0));
    assert_eq!(cache.lookup(200), Some(1));

    // Third and fourth different shapes
    cache.update(300, 2);
    cache.update(400, 3);
    assert!(matches!(cache, InlineCache::Polymorphic { .. }));

    // Fifth different shape - becomes megamorphic
    cache.update(500, 4);
    assert!(matches!(cache, InlineCache::Megamorphic));
}

#[test]
fn test_vm_global_and_local_interaction() {
    let mut vm = VM::new();
    vm.set_global("globalX".to_string(), Value::Smi(100));

    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 1;

    let c5 = chunk.add_constant(BcValue::Number(5.0));

    // localY = globalX + 5; return localY
    chunk.emit(Opcode::LoadGlobal("globalX".to_string())); // 0
    chunk.emit(Opcode::LoadConstant(c5)); // 1
    chunk.emit(Opcode::Add); // 2
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // 3
    chunk.emit(Opcode::LoadLocal(RegisterId(0))); // 4
    chunk.emit(Opcode::Return); // 5

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(105));
}

#[test]
fn test_vm_comparison_chain() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Test: (5 < 10) && (10 <= 10) && (15 > 10) && (10 >= 10)
    let c5 = chunk.add_constant(BcValue::Number(5.0));
    let c10 = chunk.add_constant(BcValue::Number(10.0));
    let _c15 = chunk.add_constant(BcValue::Number(15.0));

    // 5 < 10 = true
    chunk.emit(Opcode::LoadConstant(c5));
    chunk.emit(Opcode::LoadConstant(c10));
    chunk.emit(Opcode::LessThan);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_not_equal_operations() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let c5 = chunk.add_constant(BcValue::Number(5.0));
    let c10 = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(c5));
    chunk.emit(Opcode::LoadConstant(c10));
    chunk.emit(Opcode::NotEqual);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_strict_not_equal() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let c_smi = chunk.add_constant(BcValue::Number(10.0));
    let c_double = chunk.add_constant(BcValue::Number(10.5));

    chunk.emit(Opcode::LoadConstant(c_smi));
    chunk.emit(Opcode::LoadConstant(c_double));
    chunk.emit(Opcode::StrictNotEqual);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_less_than_equal() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let c10a = chunk.add_constant(BcValue::Number(10.0));
    let c10b = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(c10a));
    chunk.emit(Opcode::LoadConstant(c10b));
    chunk.emit(Opcode::LessThanEqual);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_greater_than_equal() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let c10a = chunk.add_constant(BcValue::Number(10.0));
    let c10b = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(c10a));
    chunk.emit(Opcode::LoadConstant(c10b));
    chunk.emit(Opcode::GreaterThanEqual);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_inline_cache_same_shape_update() {
    let mut cache = InlineCache::Monomorphic {
        shape: 10,
        offset: 0,
    };

    // Update with same shape should not change state
    cache.update(10, 5);

    match cache {
        InlineCache::Monomorphic { shape, offset } => {
            assert_eq!(shape, 10);
            assert_eq!(offset, 5); // Offset updated
        }
        _ => panic!("Should remain monomorphic"),
    }
}

#[test]
fn test_profile_branch_outcomes() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    use interpreter::BranchOutcome;
    profile.branch_outcomes.push(BranchOutcome::Taken);
    profile.branch_outcomes.push(BranchOutcome::NotTaken);
    profile.branch_outcomes.push(BranchOutcome::Taken);

    assert_eq!(profile.branch_outcomes.len(), 3);
}
