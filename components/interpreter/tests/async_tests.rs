//! Integration tests for Promise and async/await support
//!
//! Tests the integration between async_runtime and interpreter.

use bytecode_system::{BytecodeChunk, Opcode};
use core_types::Value;
use interpreter::promise_integration::{is_promise, PromiseConstructor, PromiseObject};
use interpreter::VM;

#[test]
fn test_promise_resolve_creates_fulfilled_promise() {
    let value = PromiseConstructor::resolve(Value::Smi(42));
    assert!(is_promise(&value));

    // Check the promise is fulfilled
    if let Value::NativeObject(obj) = value {
        let borrowed = obj.borrow();
        let promise_obj = borrowed.downcast_ref::<PromiseObject>().unwrap();
        assert!(matches!(
            promise_obj.state(),
            async_runtime::PromiseState::Fulfilled
        ));
        assert_eq!(promise_obj.value(), Some(&Value::Smi(42)));
    } else {
        panic!("Expected NativeObject");
    }
}

#[test]
fn test_promise_reject_creates_rejected_promise() {
    let error = core_types::JsError {
        kind: core_types::ErrorKind::TypeError,
        message: "test error".to_string(),
        stack: vec![],
        source_position: None,
    };
    let value = PromiseConstructor::reject(error);
    assert!(is_promise(&value));

    if let Value::NativeObject(obj) = value {
        let borrowed = obj.borrow();
        let promise_obj = borrowed.downcast_ref::<PromiseObject>().unwrap();
        assert!(matches!(
            promise_obj.state(),
            async_runtime::PromiseState::Rejected
        ));
        assert!(promise_obj.error().is_some());
    }
}

#[test]
fn test_promise_new_pending() {
    let value = PromiseConstructor::new_pending();
    assert!(is_promise(&value));

    if let Value::NativeObject(obj) = value {
        let borrowed = obj.borrow();
        let promise_obj = borrowed.downcast_ref::<PromiseObject>().unwrap();
        assert!(matches!(
            promise_obj.state(),
            async_runtime::PromiseState::Pending
        ));
    }
}

#[test]
fn test_await_resolved_promise_returns_value() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Create a resolved promise with value 5
    chunk.emit(Opcode::LoadGlobal("Promise".to_string()));
    chunk.emit(Opcode::LoadProperty("resolve".to_string()));
    // Push the value 5
    let idx = chunk.add_constant(bytecode_system::Value::Number(5.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Call(1));
    // Await the promise
    chunk.emit(Opcode::Await);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk).unwrap();
    // Should return the resolved value
    assert_eq!(result, Value::Smi(5));
}

#[test]
fn test_await_non_promise_returns_value() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Await a non-promise value (should just return it)
    let idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Await);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Value::Smi(42));
}

#[test]
fn test_promise_global_exists() {
    let vm = VM::new();
    let promise = vm.get_global("Promise");
    assert!(promise.is_some());
    assert!(matches!(promise.unwrap(), Value::NativeFunction(_)));
}

#[test]
fn test_promise_resolve_method() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Load Promise.resolve(100)
    chunk.emit(Opcode::LoadGlobal("Promise".to_string()));
    chunk.emit(Opcode::LoadProperty("resolve".to_string()));
    let idx = chunk.add_constant(bytecode_system::Value::Number(100.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Call(1));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk).unwrap();
    assert!(is_promise(&result));
}

#[test]
fn test_promise_reject_method() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Load Promise.reject("error")
    chunk.emit(Opcode::LoadGlobal("Promise".to_string()));
    chunk.emit(Opcode::LoadProperty("reject".to_string()));
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Call(1));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk).unwrap();
    assert!(is_promise(&result));

    // Verify it's rejected
    if let Value::NativeObject(obj) = result {
        let borrowed = obj.borrow();
        let promise_obj = borrowed.downcast_ref::<PromiseObject>().unwrap();
        assert!(matches!(
            promise_obj.state(),
            async_runtime::PromiseState::Rejected
        ));
    }
}

#[test]
fn test_await_rejected_promise_throws() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Create a rejected promise
    chunk.emit(Opcode::LoadGlobal("Promise".to_string()));
    chunk.emit(Opcode::LoadProperty("reject".to_string()));
    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Call(1));
    // Await should throw
    chunk.emit(Opcode::Await);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    // Should be an error
    assert!(result.is_err());
}

#[test]
fn test_create_async_function_opcode() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Create an async function (just test that opcode is handled)
    chunk.emit(Opcode::CreateAsyncFunction(0, vec![]));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk).unwrap();
    // Should return a HeapObject with async marker
    assert!(matches!(result, Value::HeapObject(_)));
}

#[test]
fn test_promise_object_do_resolve() {
    let mut promise_obj = PromiseObject::new();
    assert!(matches!(
        promise_obj.state(),
        async_runtime::PromiseState::Pending
    ));

    promise_obj.do_resolve(Value::Smi(99));
    assert!(matches!(
        promise_obj.state(),
        async_runtime::PromiseState::Fulfilled
    ));
    assert_eq!(promise_obj.value(), Some(&Value::Smi(99)));
}

#[test]
fn test_promise_object_do_reject() {
    let mut promise_obj = PromiseObject::new();
    let error = core_types::JsError {
        kind: core_types::ErrorKind::ReferenceError,
        message: "not defined".to_string(),
        stack: vec![],
        source_position: None,
    };

    promise_obj.do_reject(error);
    assert!(matches!(
        promise_obj.state(),
        async_runtime::PromiseState::Rejected
    ));
    assert!(promise_obj.error().is_some());
}
