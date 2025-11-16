//! Contract tests for jit_compiler
//!
//! These tests verify that the public API matches the contract in contracts/jit_compiler.yaml

use bytecode_system::{BytecodeChunk, Opcode};
use core_types::{JsError, Value};
use interpreter::{ExecutionContext, ProfileData};
use jit_compiler::{BaselineJIT, CompiledCode, Deoptimizer, OSREntry, OptimizingJIT};

#[test]
fn baseline_jit_has_new_constructor() {
    let _jit = BaselineJIT::new();
}

#[test]
fn baseline_jit_compile_takes_chunk_returns_result() {
    let mut jit = BaselineJIT::new();
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadConstant(0));
    chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::Return);

    let result: Result<CompiledCode, JsError> = jit.compile(&chunk);
    assert!(result.is_ok());
}

#[test]
fn optimizing_jit_has_new_constructor() {
    let _jit = OptimizingJIT::new();
}

#[test]
fn optimizing_jit_compile_takes_chunk_and_profile_returns_result() {
    let mut jit = OptimizingJIT::new();
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::LoadConstant(0));
    chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::Return);
    let profile = ProfileData::new();

    let result: Result<CompiledCode, JsError> = jit.compile(&chunk, &profile);
    assert!(result.is_ok());
}

#[test]
fn compiled_code_has_required_fields() {
    let mut jit = BaselineJIT::new();
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);

    let code = jit.compile(&chunk).unwrap();

    // Test field existence (contract requirement)
    let _code_ptr: *const u8 = code.code;
    let _size: usize = code.size;
    let _entry: *const () = code.entry_point;
    let _osr: &Vec<OSREntry> = &code.osr_entries;
}

#[test]
fn compiled_code_execute_returns_result_value() {
    let mut jit = BaselineJIT::new();
    let mut chunk = BytecodeChunk::new();
    let idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Return);

    let code = jit.compile(&chunk).unwrap();
    let result: Result<Value, JsError> = code.execute();
    assert!(result.is_ok());
}

#[test]
fn osr_entry_has_required_fields() {
    let mut jit = BaselineJIT::new();
    let mut chunk = BytecodeChunk::new();
    // Create a loop to generate OSR entries
    chunk.emit(Opcode::LoadConstant(0));
    chunk.add_constant(bytecode_system::Value::Number(0.0));
    chunk.emit(Opcode::Jump(0)); // Simple loop back

    let code = jit.compile(&chunk).unwrap();

    if let Some(entry) = code.osr_entries.first() {
        let _bytecode_offset: usize = entry.bytecode_offset;
        let _native_offset: usize = entry.native_offset;
        let _frame_mapping = &entry.frame_mapping;
    }
}

#[test]
fn osr_entry_enter_at_takes_context_returns_result() {
    let osr_entry = OSREntry::new(0, 0);
    let chunk = BytecodeChunk::new();
    let context = ExecutionContext::new(chunk);

    let result: Result<(), JsError> = osr_entry.enter_at(&context);
    // OSR entry might fail if not properly set up, that's OK for contract test
    let _ = result;
}

#[test]
fn deoptimizer_deoptimize_returns_execution_context() {
    let mut jit = BaselineJIT::new();
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::Return);

    let code = jit.compile(&chunk).unwrap();
    let deopt = Deoptimizer::new();

    let _ctx: ExecutionContext = deopt.deoptimize(&code);
}
