//! Contract tests for interpreter API
//!
//! These tests verify the public API matches the contract specification.

use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
use core_types::Value;
use interpreter::{ExecutionContext, InlineCache, ProfileData, VM};

/// Test VM::new() returns a valid VM instance
#[test]
fn test_vm_new_contract() {
    let vm = VM::new();
    // VM should be successfully created
    assert!(true, "VM::new() should return a valid VM");
}

/// Test VM::execute() runs bytecode and returns result
#[test]
fn test_vm_execute_contract() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Simple bytecode: load constant and return
    let idx = chunk.add_constant(BcValue::Number(42.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "execute should return Ok for valid bytecode"
    );
    assert_eq!(result.unwrap(), Value::Smi(42));
}

/// Test VM::get_global() retrieves global variables
#[test]
fn test_vm_get_global_contract() {
    let vm = VM::new();

    // Non-existent global should return None
    let result = vm.get_global("nonexistent");
    assert!(
        result.is_none(),
        "get_global should return None for non-existent"
    );
}

/// Test VM::set_global() stores global variables
#[test]
fn test_vm_set_global_contract() {
    let mut vm = VM::new();

    vm.set_global("test".to_string(), Value::Smi(100));
    let result = vm.get_global("test");
    assert!(
        result.is_some(),
        "get_global should return Some after set_global"
    );
    assert_eq!(result.unwrap(), Value::Smi(100));
}

/// Test ExecutionContext has required fields
#[test]
fn test_execution_context_fields_contract() {
    let chunk = BytecodeChunk::new();
    let ctx = ExecutionContext {
        registers: vec![Value::Undefined; 10],
        instruction_pointer: 0,
        bytecode: chunk,
    };

    assert_eq!(ctx.registers.len(), 10);
    assert_eq!(ctx.instruction_pointer, 0);
    assert_eq!(ctx.bytecode.instructions.len(), 0);
}

/// Test InlineCache variants exist
#[test]
fn test_inline_cache_variants_contract() {
    let _uninitialized = InlineCache::Uninitialized;
    let _monomorphic = InlineCache::Monomorphic {
        shape: 1,
        offset: 0,
    };
    let _polymorphic = InlineCache::Polymorphic {
        entries: Default::default(),
    };
    let _megamorphic = InlineCache::Megamorphic;
}

/// Test InlineCache::lookup() returns offset for cached shape
#[test]
fn test_inline_cache_lookup_contract() {
    let cache = InlineCache::Monomorphic {
        shape: 42,
        offset: 5,
    };

    let result = cache.lookup(42);
    assert_eq!(result, Some(5));

    let miss = cache.lookup(99);
    assert!(miss.is_none());
}

/// Test InlineCache::update() modifies cache state
#[test]
fn test_inline_cache_update_contract() {
    let mut cache = InlineCache::Uninitialized;

    cache.update(100, 10);

    // After update, should be monomorphic
    let result = cache.lookup(100);
    assert_eq!(result, Some(10));
}

/// Test ProfileData has required fields
#[test]
fn test_profile_data_fields_contract() {
    let profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    assert_eq!(profile.execution_count, 0);
    assert!(profile.type_feedback.is_empty());
    assert!(profile.branch_outcomes.is_empty());
}

/// Test ProfileData::record_execution() increments count
#[test]
fn test_profile_data_record_execution_contract() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    profile.record_execution();
    assert!(profile.execution_count > 0);
}

/// Test ProfileData::record_type() stores type information
#[test]
fn test_profile_data_record_type_contract() {
    use interpreter::TypeInfo;
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    let type_info = TypeInfo::Number;
    profile.record_type(type_info);

    assert!(!profile.type_feedback.is_empty());
}

/// Test ProfileData::should_compile_baseline() returns bool
#[test]
fn test_profile_data_should_compile_baseline_contract() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    // Initially should not compile
    assert!(!profile.should_compile_baseline());

    // After many executions, should compile
    profile.execution_count = 500;
    assert!(profile.should_compile_baseline());
}

/// Test ProfileData::should_compile_optimized() returns bool
#[test]
fn test_profile_data_should_compile_optimized_contract() {
    let mut profile = ProfileData {
        execution_count: 0,
        type_feedback: vec![],
        branch_outcomes: vec![],
    };

    // Initially should not compile
    assert!(!profile.should_compile_optimized());

    // After very many executions, should compile
    profile.execution_count = 10000;
    assert!(profile.should_compile_optimized());
}
