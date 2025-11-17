//! Exception handling tests for try/catch/finally and throw statements

use bytecode_system::{BytecodeChunk, Opcode, RegisterId};
use core_types::{ErrorKind, Value};
use interpreter::dispatch::Dispatcher;
use interpreter::ExecutionContext;

#[test]
fn test_try_catch_basic() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Simulate: try { throw 42; } catch (e) { result = e; } result
    // Index 0: PushTry -> catch at 5
    chunk.emit(Opcode::PushTry(5));
    // Index 1: LoadConstant (42)
    chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::LoadConstant(0));
    // Index 2: Throw
    chunk.emit(Opcode::Throw);
    // Index 3: PopTry (skipped)
    chunk.emit(Opcode::PopTry);
    // Index 4: Jump to after catch (skipped)
    chunk.emit(Opcode::Jump(7));
    // Index 5: Catch - store exception in reg 0
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    // Index 6: Load result
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Index 7: Return
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        Value::Double(n) => assert_eq!(n, 42.0),
        Value::Smi(n) => assert_eq!(n, 42),
        _ => panic!("Expected number, got {:?}", val),
    }
}

#[test]
fn test_uncaught_exception() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Simulate: throw 99;
    chunk.add_constant(bytecode_system::Value::Number(99.0));
    chunk.emit(Opcode::LoadConstant(0));
    chunk.emit(Opcode::Throw);
    chunk.emit(Opcode::Return);

    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::InternalError);
    assert!(err.message.contains("Uncaught exception"));
}

#[test]
fn test_pop_opcode() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Load value and pop it
    chunk.add_constant(bytecode_system::Value::Number(100.0));
    chunk.emit(Opcode::LoadConstant(0));
    chunk.emit(Opcode::Pop);
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Return);

    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_try_no_exception() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Simulate: try { result = 10; } catch (e) { result = 20; } result
    // Index 0: PushTry -> catch at 5
    chunk.emit(Opcode::PushTry(5));
    // Index 1: LoadConstant (10)
    chunk.add_constant(bytecode_system::Value::Number(10.0));
    chunk.emit(Opcode::LoadConstant(0));
    // Index 2: StoreLocal
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    // Index 3: PopTry
    chunk.emit(Opcode::PopTry);
    // Index 4: Jump over catch to return result
    chunk.emit(Opcode::Jump(8));
    // Index 5: Catch - Pop exception (not reached)
    chunk.emit(Opcode::Pop);
    // Index 6: LoadConstant (20)
    chunk.add_constant(bytecode_system::Value::Number(20.0));
    chunk.emit(Opcode::LoadConstant(1));
    // Index 7: StoreLocal
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    // Index 8: Load result
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Index 9: Return
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        Value::Double(n) => assert_eq!(n, 10.0),
        Value::Smi(n) => assert_eq!(n, 10),
        _ => panic!("Expected 10, got {:?}", val),
    }
}

#[test]
fn test_nested_try_catch() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Simulate nested try-catch - inner throws, outer catches
    // Index 0: Outer PushTry -> outer catch at 11
    chunk.emit(Opcode::PushTry(11));
    // Index 1: Inner PushTry -> inner catch at 6
    chunk.emit(Opcode::PushTry(6));
    // Index 2: Load 1
    chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(0));
    // Index 3: Throw 1
    chunk.emit(Opcode::Throw);
    // Index 4: PopTry (inner, skipped)
    chunk.emit(Opcode::PopTry);
    // Index 5: Jump (skipped)
    chunk.emit(Opcode::Jump(9));
    // Index 6: Inner catch - pop exception 1
    chunk.emit(Opcode::Pop);
    // Index 7: Load 2
    chunk.add_constant(bytecode_system::Value::Number(2.0));
    chunk.emit(Opcode::LoadConstant(1));
    // Index 8: Throw 2
    chunk.emit(Opcode::Throw);
    // Index 9: PopTry (outer, skipped)
    chunk.emit(Opcode::PopTry);
    // Index 10: Jump (skipped)
    chunk.emit(Opcode::Jump(13));
    // Index 11: Outer catch - store exception
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    // Index 12: Load result
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Index 13: Return
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        Value::Double(n) => assert_eq!(n, 2.0),
        Value::Smi(n) => assert_eq!(n, 2),
        _ => panic!("Expected 2, got {:?}", val),
    }
}

#[test]
fn test_try_finally_no_exception() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Simulate: let x = 0; try { x = 1; } finally { x = 2; } x
    // Initialize x = 0
    chunk.add_constant(bytecode_system::Value::Number(0.0));
    chunk.emit(Opcode::LoadConstant(0));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Try block - no PushTry since no catch, just run finally after
    chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(1));
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // x = 1

    // Finally block (always runs)
    chunk.add_constant(bytecode_system::Value::Number(2.0));
    chunk.emit(Opcode::LoadConstant(2));
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // x = 2

    // Return x
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        Value::Double(n) => assert_eq!(n, 2.0),
        Value::Smi(n) => assert_eq!(n, 2),
        _ => panic!("Expected 2, got {:?}", val),
    }
}

#[test]
fn test_throw_with_boolean() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // throw true; caught in catch
    chunk.emit(Opcode::PushTry(3));
    chunk.emit(Opcode::LoadTrue);
    chunk.emit(Opcode::Throw);
    // Catch block
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_throw_with_undefined() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // throw undefined; caught in catch
    chunk.emit(Opcode::PushTry(3));
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Throw);
    // Catch block
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Return);

    chunk.register_count = 1;
    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_stack_unwinding_on_throw() {
    let mut dispatcher = Dispatcher::new();
    let mut chunk = BytecodeChunk::new();

    // Test that stack is properly unwound when throwing
    // Push some values, then throw
    chunk.emit(Opcode::PushTry(6));
    chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(0));
    chunk.add_constant(bytecode_system::Value::Number(2.0));
    chunk.emit(Opcode::LoadConstant(1));
    chunk.add_constant(bytecode_system::Value::Number(3.0));
    chunk.emit(Opcode::LoadConstant(2));
    // Now stack has [1, 2, 3], throw 42
    chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::LoadConstant(3));
    chunk.emit(Opcode::Throw);
    // Catch block - exception should be on stack
    chunk.emit(Opcode::Return);

    let mut ctx = ExecutionContext::new(chunk);
    let functions = vec![];

    let result = dispatcher.execute(&mut ctx, &functions);
    assert!(result.is_ok());
    let val = result.unwrap();
    match val {
        Value::Double(n) => assert_eq!(n, 42.0),
        Value::Smi(n) => assert_eq!(n, 42),
        _ => panic!("Expected 42, got {:?}", val),
    }
}
