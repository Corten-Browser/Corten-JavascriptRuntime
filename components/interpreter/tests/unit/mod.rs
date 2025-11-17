//! Unit tests for interpreter components

use bytecode_system::{BytecodeChunk, Opcode, RegisterId, Value as BcValue};
use core_types::Value;
use interpreter::{
    BranchOutcome, CallFrame, ExecutionContext, InlineCache, ProfileData, TypeInfo, VM,
};

// ============================================================================
// VM Tests
// ============================================================================

#[test]
fn test_vm_creation() {
    let vm = VM::new();
    // VM should be initialized with empty global object
    assert!(vm.get_global("undefined").is_none());
}

#[test]
fn test_vm_global_variables() {
    let mut vm = VM::new();

    vm.set_global("x".to_string(), Value::Smi(10));
    vm.set_global("y".to_string(), Value::Double(3.14));
    vm.set_global("flag".to_string(), Value::Boolean(true));

    assert_eq!(vm.get_global("x"), Some(Value::Smi(10)));
    assert_eq!(vm.get_global("y"), Some(Value::Double(3.14)));
    assert_eq!(vm.get_global("flag"), Some(Value::Boolean(true)));
}

#[test]
fn test_vm_global_overwrite() {
    let mut vm = VM::new();

    vm.set_global("x".to_string(), Value::Smi(10));
    vm.set_global("x".to_string(), Value::Smi(20));

    assert_eq!(vm.get_global("x"), Some(Value::Smi(20)));
}

#[test]
fn test_vm_execute_load_constant() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx = chunk.add_constant(BcValue::Number(3.14));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(3.14));
}

#[test]
fn test_vm_execute_load_undefined() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_vm_execute_load_null() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    chunk.emit(Opcode::LoadNull);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_vm_execute_load_true() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_execute_load_false() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    chunk.emit(Opcode::LoadFalse);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_vm_execute_add_smi() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(10.0));
    let b = chunk.add_constant(BcValue::Number(20.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Add);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(30));
}

#[test]
fn test_vm_execute_add_double() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(1.5));
    let b = chunk.add_constant(BcValue::Number(2.5));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Add);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(4.0));
}

#[test]
fn test_vm_execute_sub() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(30.0));
    let b = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Sub);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(20));
}

#[test]
fn test_vm_execute_mul() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(6.0));
    let b = chunk.add_constant(BcValue::Number(7.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Mul);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(42));
}

#[test]
fn test_vm_execute_div() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(100.0));
    let b = chunk.add_constant(BcValue::Number(4.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Div);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(25.0));
}

#[test]
fn test_vm_execute_mod() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(10.0));
    let b = chunk.add_constant(BcValue::Number(3.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Mod);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(1));
}

#[test]
fn test_vm_execute_neg() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(42.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::Neg);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(-42));
}

#[test]
fn test_vm_execute_equal() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(10.0));
    let b = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::Equal);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_execute_strict_equal() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(10.0));
    let b = chunk.add_constant(BcValue::Number(10.5));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::StrictEqual);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    // Strict equality should be false for different values
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_vm_execute_less_than() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(5.0));
    let b = chunk.add_constant(BcValue::Number(10.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::LessThan);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_execute_greater_than() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let a = chunk.add_constant(BcValue::Number(10.0));
    let b = chunk.add_constant(BcValue::Number(5.0));

    chunk.emit(Opcode::LoadConstant(a));
    chunk.emit(Opcode::LoadConstant(b));
    chunk.emit(Opcode::GreaterThan);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_vm_execute_jump() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let val1 = chunk.add_constant(BcValue::Number(1.0));
    let val2 = chunk.add_constant(BcValue::Number(2.0));

    chunk.emit(Opcode::Jump(2)); // Jump over LoadConstant(val1)
    chunk.emit(Opcode::LoadConstant(val1)); // This should be skipped
    chunk.emit(Opcode::LoadConstant(val2));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(2));
}

#[test]
fn test_vm_execute_jump_if_true() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let val = chunk.add_constant(BcValue::Number(42.0));

    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::JumpIfTrue(3)); // Jump over next instruction
    chunk.emit(Opcode::LoadUndefined); // Should be skipped
    chunk.emit(Opcode::LoadConstant(val));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(42));
}

#[test]
fn test_vm_execute_jump_if_false() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let val = chunk.add_constant(BcValue::Number(42.0));

    chunk.emit(Opcode::LoadFalse);
    chunk.emit(Opcode::JumpIfFalse(3)); // Jump over next instruction
    chunk.emit(Opcode::LoadUndefined); // Should be skipped
    chunk.emit(Opcode::LoadConstant(val));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(42));
}

#[test]
fn test_vm_execute_load_global() {
    let mut vm = VM::new();
    vm.set_global("myVar".to_string(), Value::Smi(999));

    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadGlobal("myVar".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(999));
}

#[test]
fn test_vm_execute_store_global() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let val = chunk.add_constant(BcValue::Number(123.0));
    chunk.emit(Opcode::LoadConstant(val));
    chunk.emit(Opcode::StoreGlobal("newVar".to_string()));
    chunk.emit(Opcode::LoadGlobal("newVar".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(123));
}

#[test]
fn test_vm_execute_load_local() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 5;

    let val = chunk.add_constant(BcValue::Number(77.0));
    chunk.emit(Opcode::LoadConstant(val));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(77));
}

#[test]
fn test_vm_execute_store_local() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 3;

    let val1 = chunk.add_constant(BcValue::Number(11.0));
    let val2 = chunk.add_constant(BcValue::Number(22.0));

    chunk.emit(Opcode::LoadConstant(val1));
    chunk.emit(Opcode::StoreLocal(RegisterId(1)));
    chunk.emit(Opcode::LoadConstant(val2));
    chunk.emit(Opcode::StoreLocal(RegisterId(2)));
    chunk.emit(Opcode::LoadLocal(RegisterId(2)));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Smi(22));
}

// ============================================================================
// ExecutionContext Tests
// ============================================================================

#[test]
fn test_execution_context_creation() {
    let chunk = BytecodeChunk::new();
    let ctx = ExecutionContext {
        registers: vec![Value::Undefined; 5],
        instruction_pointer: 0,
        bytecode: chunk,
    };

    assert_eq!(ctx.registers.len(), 5);
    assert_eq!(ctx.instruction_pointer, 0);
}

#[test]
fn test_execution_context_registers() {
    let chunk = BytecodeChunk::new();
    let mut ctx = ExecutionContext {
        registers: vec![Value::Undefined; 3],
        instruction_pointer: 0,
        bytecode: chunk,
    };

    ctx.registers[0] = Value::Smi(100);
    ctx.registers[1] = Value::Boolean(true);

    assert_eq!(ctx.registers[0], Value::Smi(100));
    assert_eq!(ctx.registers[1], Value::Boolean(true));
    assert_eq!(ctx.registers[2], Value::Undefined);
}

#[test]
fn test_execution_context_instruction_pointer() {
    let chunk = BytecodeChunk::new();
    let mut ctx = ExecutionContext {
        registers: vec![],
        instruction_pointer: 0,
        bytecode: chunk,
    };

    ctx.instruction_pointer = 10;
    assert_eq!(ctx.instruction_pointer, 10);

    ctx.instruction_pointer += 5;
    assert_eq!(ctx.instruction_pointer, 15);
}

// ============================================================================
// InlineCache Tests
// ============================================================================

#[test]
fn test_inline_cache_uninitialized() {
    let cache = InlineCache::Uninitialized;

    assert!(cache.lookup(0).is_none());
    assert!(cache.lookup(100).is_none());
}

#[test]
fn test_inline_cache_monomorphic_hit() {
    let cache = InlineCache::Monomorphic {
        shape: 42,
        offset: 7,
    };

    assert_eq!(cache.lookup(42), Some(7));
}

#[test]
fn test_inline_cache_monomorphic_miss() {
    let cache = InlineCache::Monomorphic {
        shape: 42,
        offset: 7,
    };

    assert!(cache.lookup(99).is_none());
}

#[test]
fn test_inline_cache_polymorphic() {
    let mut entries = arrayvec::ArrayVec::new();
    entries.push((10, 0));
    entries.push((20, 1));
    entries.push((30, 2));

    let cache = InlineCache::Polymorphic { entries };

    assert_eq!(cache.lookup(10), Some(0));
    assert_eq!(cache.lookup(20), Some(1));
    assert_eq!(cache.lookup(30), Some(2));
    assert!(cache.lookup(40).is_none());
}

#[test]
fn test_inline_cache_megamorphic() {
    let cache = InlineCache::Megamorphic;

    // Megamorphic always misses (fallback to hash table)
    assert!(cache.lookup(0).is_none());
    assert!(cache.lookup(999).is_none());
}

#[test]
fn test_inline_cache_update_uninitialized_to_mono() {
    let mut cache = InlineCache::Uninitialized;

    cache.update(50, 3);

    assert_eq!(cache.lookup(50), Some(3));
}

#[test]
fn test_inline_cache_update_mono_to_poly() {
    let mut cache = InlineCache::Monomorphic {
        shape: 10,
        offset: 0,
    };

    cache.update(20, 1);

    // Should now be polymorphic with both entries
    assert_eq!(cache.lookup(10), Some(0));
    assert_eq!(cache.lookup(20), Some(1));
}

#[test]
fn test_inline_cache_update_poly_to_mega() {
    let mut entries = arrayvec::ArrayVec::new();
    entries.push((1, 0));
    entries.push((2, 1));
    entries.push((3, 2));
    entries.push((4, 3));

    let mut cache = InlineCache::Polymorphic { entries };

    // Adding 5th entry should transition to megamorphic
    cache.update(5, 4);

    // After transition to megamorphic, all lookups miss
    match cache {
        InlineCache::Megamorphic => (),
        _ => panic!("Expected megamorphic state"),
    }
}

// ============================================================================
// ProfileData Tests
// ============================================================================

#[test]
fn test_profile_data_creation() {
    let profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    assert_eq!(profile.execution_count, 0);
    assert!(profile.type_feedback.is_empty());
    assert!(profile.branch_outcomes.is_empty());
}

#[test]
fn test_profile_data_record_execution() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    profile.record_execution();
    assert_eq!(profile.execution_count, 1);

    profile.record_execution();
    assert_eq!(profile.execution_count, 2);

    for _ in 0..100 {
        profile.record_execution();
    }
    assert_eq!(profile.execution_count, 102);
}

#[test]
fn test_profile_data_record_type() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    profile.record_type(TypeInfo::Number);
    profile.record_type(TypeInfo::Boolean);
    profile.record_type(TypeInfo::Object);

    assert_eq!(profile.type_feedback.len(), 3);
    assert_eq!(profile.type_feedback[0], TypeInfo::Number);
    assert_eq!(profile.type_feedback[1], TypeInfo::Boolean);
    assert_eq!(profile.type_feedback[2], TypeInfo::Object);
}

#[test]
fn test_profile_data_baseline_threshold() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    // Below threshold
    profile.execution_count = 499;
    assert!(!profile.should_compile_baseline());

    // At threshold
    profile.execution_count = 500;
    assert!(profile.should_compile_baseline());

    // Above threshold
    profile.execution_count = 1000;
    assert!(profile.should_compile_baseline());
}

#[test]
fn test_profile_data_optimized_threshold() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    // Below threshold
    profile.execution_count = 9999;
    assert!(!profile.should_compile_optimized());

    // At threshold
    profile.execution_count = 10000;
    assert!(profile.should_compile_optimized());

    // Above threshold
    profile.execution_count = 50000;
    assert!(profile.should_compile_optimized());
}

#[test]
fn test_type_info_variants() {
    let _number = TypeInfo::Number;
    let _boolean = TypeInfo::Boolean;
    let _string = TypeInfo::String;
    let _object = TypeInfo::Object;
    let _undefined = TypeInfo::Undefined;
    let _null = TypeInfo::Null;
}

#[test]
fn test_branch_outcome_variants() {
    let _taken = BranchOutcome::Taken;
    let _not_taken = BranchOutcome::NotTaken;
}

// ============================================================================
// CallFrame Tests
// ============================================================================

#[test]
fn test_call_frame_creation() {
    let frame = CallFrame {
        return_address: 10,
        base_register: 0,
        function_id: 1,
    };

    assert_eq!(frame.return_address, 10);
    assert_eq!(frame.base_register, 0);
    assert_eq!(frame.function_id, 1);
}
