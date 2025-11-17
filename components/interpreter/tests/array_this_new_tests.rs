//! Tests for array indexing, this binding, and new operator
//!
//! These tests verify the fixes for:
//! 1. Array[index] returns correct element
//! 2. Method calls bind `this` properly
//! 3. Constructor calls (new operator) work correctly

use bytecode_system::{BytecodeChunk, Opcode, RegisterId};
use core_types::Value;
use interpreter::VM;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_array_creation_and_indexing() {
    // Test: let a = [1, 2, 3]; a[1] should return 2
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Push array elements onto stack
    let idx1 = chunk.add_constant(bytecode_system::Value::Number(1.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(2.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(3.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    // Create array with 3 elements
    chunk.emit(Opcode::CreateArray(3));
    // Store array in register 0
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Load array from register
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Push index 1
    let index_val = chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    // Get element at index 1
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Array indexing should succeed");
    let value = result.unwrap();
    // Element at index 1 should be 2
    assert_eq!(value, Value::Smi(2), "a[1] should be 2, got {:?}", value);
}

#[test]
fn test_array_indexing_first_element() {
    // Test: let a = [10, 20, 30]; a[0] should return 10
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx1 = chunk.add_constant(bytecode_system::Value::Number(10.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(20.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(30.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    chunk.emit(Opcode::CreateArray(3));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    let index_val = chunk.add_constant(bytecode_system::Value::Number(0.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Smi(10), "a[0] should be 10");
}

#[test]
fn test_array_indexing_last_element() {
    // Test: let a = [10, 20, 30]; a[2] should return 30
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx1 = chunk.add_constant(bytecode_system::Value::Number(10.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(20.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(30.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    chunk.emit(Opcode::CreateArray(3));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    let index_val = chunk.add_constant(bytecode_system::Value::Number(2.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Smi(30), "a[2] should be 30");
}

#[test]
fn test_array_set_index() {
    // Test: let a = [1, 2, 3]; a[1] = 100; a[1] should return 100
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx1 = chunk.add_constant(bytecode_system::Value::Number(1.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(2.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(3.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    chunk.emit(Opcode::CreateArray(3));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Set a[1] = 100
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    let index_val = chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    let new_val = chunk.add_constant(bytecode_system::Value::Number(100.0));
    chunk.emit(Opcode::LoadConstant(new_val));
    chunk.emit(Opcode::SetIndex);

    // Get a[1]
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadConstant(index_val));
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Smi(100), "a[1] should be 100 after assignment");
}

#[test]
fn test_string_indexing() {
    // Test: "hello"[1] should return "e"
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let str_idx = chunk.add_constant(bytecode_system::Value::String("hello".to_string()));
    chunk.emit(Opcode::LoadConstant(str_idx));

    let index_val = chunk.add_constant(bytecode_system::Value::Number(1.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("e".to_string()),
        "hello[1] should be 'e'"
    );
}

#[test]
fn test_method_call_this_binding() {
    // Test: let obj = {x: 5, get: function() { return this.x; }}; obj.get() should return 5
    // This simulates the this binding through CallMethod opcode
    let mut vm = VM::new();

    // Create the method function
    // Function accesses `this` via LoadGlobal (as parser generates)
    let mut method_chunk = BytecodeChunk::new();
    // Load `this` from global (parser emits LoadGlobal("this") for ThisExpression)
    method_chunk.emit(Opcode::LoadGlobal("this".to_string()));
    // Load property 'x' from this
    method_chunk.emit(Opcode::LoadProperty("x".to_string()));
    method_chunk.emit(Opcode::Return);

    let method_idx = vm.register_function(method_chunk);

    // Main program
    let mut chunk = BytecodeChunk::new();

    // Create object {x: 5}
    chunk.emit(Opcode::CreateObject);
    chunk.emit(Opcode::Dup); // Keep reference to object
    let x_val = chunk.add_constant(bytecode_system::Value::Number(5.0));
    chunk.emit(Opcode::LoadConstant(x_val));
    chunk.emit(Opcode::StoreProperty("x".to_string()));

    // Dup object for storing method
    chunk.emit(Opcode::Dup);

    // Create closure for method (no captured variables)
    chunk.emit(Opcode::CreateClosure(method_idx, vec![]));
    chunk.emit(Opcode::StoreProperty("get".to_string()));

    // Store object in register 0
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Call obj.get()
    // Push receiver (obj)
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Duplicate for loading property
    chunk.emit(Opcode::Dup);
    // Load method
    chunk.emit(Opcode::LoadProperty("get".to_string()));
    // Call method with 0 arguments (receiver already on stack below method)
    chunk.emit(Opcode::CallMethod(0));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Method call should succeed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        Value::Smi(5),
        "obj.get() should return 5 (this.x)"
    );
}

#[test]
fn test_method_call_with_arguments() {
    // Test: let obj = {x: 10, add: function(y) { return this.x + y; }}; obj.add(5) should return 15
    let mut vm = VM::new();

    // Create the add method
    // this is accessed via LoadGlobal, y is in register 0
    let mut method_chunk = BytecodeChunk::new();
    // Load this.x (parser emits LoadGlobal("this") for ThisExpression)
    method_chunk.emit(Opcode::LoadGlobal("this".to_string()));
    method_chunk.emit(Opcode::LoadProperty("x".to_string()));
    // Load y (first parameter is in register 0)
    method_chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Add
    method_chunk.emit(Opcode::Add);
    method_chunk.emit(Opcode::Return);

    let method_idx = vm.register_function(method_chunk);

    let mut chunk = BytecodeChunk::new();

    // Create object {x: 10}
    chunk.emit(Opcode::CreateObject);
    chunk.emit(Opcode::Dup);
    let x_val = chunk.add_constant(bytecode_system::Value::Number(10.0));
    chunk.emit(Opcode::LoadConstant(x_val));
    chunk.emit(Opcode::StoreProperty("x".to_string()));

    chunk.emit(Opcode::Dup);
    chunk.emit(Opcode::CreateClosure(method_idx, vec![]));
    chunk.emit(Opcode::StoreProperty("add".to_string()));

    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Call obj.add(5)
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Dup);
    chunk.emit(Opcode::LoadProperty("add".to_string()));
    let arg_val = chunk.add_constant(bytecode_system::Value::Number(5.0));
    chunk.emit(Opcode::LoadConstant(arg_val));
    chunk.emit(Opcode::CallMethod(1));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Method call with args should succeed");
    assert_eq!(result.unwrap(), Value::Smi(15), "obj.add(5) should return 15");
}

#[test]
fn test_constructor_call_new_operator() {
    // Test: class Foo { constructor() { this.x = 1; } }; let f = new Foo(); f.x should return 1
    let mut vm = VM::new();

    // Create constructor function
    // this is accessed via LoadGlobal (as parser generates)
    let mut constructor_chunk = BytecodeChunk::new();
    // this.x = 1 (parser emits LoadGlobal("this") for ThisExpression)
    constructor_chunk.emit(Opcode::LoadGlobal("this".to_string()));
    let one_val = constructor_chunk.add_constant(bytecode_system::Value::Number(1.0));
    constructor_chunk.emit(Opcode::LoadConstant(one_val));
    constructor_chunk.emit(Opcode::StoreProperty("x".to_string()));
    // Return undefined (constructor returns this implicitly)
    constructor_chunk.emit(Opcode::LoadUndefined);
    constructor_chunk.emit(Opcode::Return);

    let constructor_idx = vm.register_function(constructor_chunk);

    let mut chunk = BytecodeChunk::new();

    // Load constructor (no arguments, so just push constructor)
    chunk.emit(Opcode::CreateClosure(constructor_idx, vec![]));
    // Call with new
    chunk.emit(Opcode::CallNew(0));
    // Store instance in register 0
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    // Load f.x
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadProperty("x".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Constructor call should succeed: {:?}", result);
    assert_eq!(
        result.unwrap(),
        Value::Smi(1),
        "new Foo().x should be 1"
    );
}

#[test]
fn test_constructor_with_arguments() {
    // Test: class Bar { constructor(val) { this.value = val; } }; let b = new Bar(42); b.value should return 42
    let mut vm = VM::new();

    // Create constructor: this via LoadGlobal, val in reg 0
    let mut constructor_chunk = BytecodeChunk::new();
    // this.value = val (parser emits LoadGlobal("this") for ThisExpression)
    constructor_chunk.emit(Opcode::LoadGlobal("this".to_string()));
    // val is the first parameter, so it's in register 0
    constructor_chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    constructor_chunk.emit(Opcode::StoreProperty("value".to_string()));
    constructor_chunk.emit(Opcode::LoadUndefined);
    constructor_chunk.emit(Opcode::Return);

    let constructor_idx = vm.register_function(constructor_chunk);

    let mut chunk = BytecodeChunk::new();

    // Stack order matches parser: push arguments first, then constructor
    let arg_val = chunk.add_constant(bytecode_system::Value::Number(42.0));
    chunk.emit(Opcode::LoadConstant(arg_val)); // Push argument first
    chunk.emit(Opcode::CreateClosure(constructor_idx, vec![])); // Then constructor
    chunk.emit(Opcode::CallNew(1));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));

    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadProperty("value".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Constructor with args should succeed");
    assert_eq!(result.unwrap(), Value::Smi(42), "new Bar(42).value should be 42");
}

#[test]
fn test_constructor_returning_object() {
    // Test: Constructor that returns an object should use that object
    let mut vm = VM::new();

    // Create constructor that returns an object
    let mut constructor_chunk = BytecodeChunk::new();
    constructor_chunk.emit(Opcode::CreateObject);
    constructor_chunk.emit(Opcode::Dup);
    let val = constructor_chunk.add_constant(bytecode_system::Value::Number(999.0));
    constructor_chunk.emit(Opcode::LoadConstant(val));
    constructor_chunk.emit(Opcode::StoreProperty("special".to_string()));
    constructor_chunk.emit(Opcode::Return);

    let constructor_idx = vm.register_function(constructor_chunk);

    let mut chunk = BytecodeChunk::new();

    chunk.emit(Opcode::CreateClosure(constructor_idx, vec![]));
    chunk.emit(Opcode::CallNew(0));
    chunk.emit(Opcode::LoadProperty("special".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::Smi(999),
        "Constructor returning object should use that object"
    );
}

#[test]
fn test_array_length_property() {
    // Test: [1, 2, 3].length should be 3
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx1 = chunk.add_constant(bytecode_system::Value::Number(1.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(2.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(3.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    chunk.emit(Opcode::CreateArray(3));
    chunk.emit(Opcode::LoadProperty("length".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Smi(3), "Array length should be 3");
}

#[test]
fn test_out_of_bounds_array_access() {
    // Test: [1, 2, 3][10] should return undefined
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    let idx1 = chunk.add_constant(bytecode_system::Value::Number(1.0));
    let idx2 = chunk.add_constant(bytecode_system::Value::Number(2.0));
    let idx3 = chunk.add_constant(bytecode_system::Value::Number(3.0));

    chunk.emit(Opcode::LoadConstant(idx1));
    chunk.emit(Opcode::LoadConstant(idx2));
    chunk.emit(Opcode::LoadConstant(idx3));
    chunk.emit(Opcode::CreateArray(3));

    let index_val = chunk.add_constant(bytecode_system::Value::Number(10.0));
    chunk.emit(Opcode::LoadConstant(index_val));
    chunk.emit(Opcode::GetIndex);
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::Undefined,
        "Out of bounds access should return undefined"
    );
}

#[test]
fn test_parser_generated_class_with_nested_function() {
    // Test: Simulates parser output for:
    // class Foo { constructor(x) { this.x = x; } }
    // let f = new Foo(5);
    // console.log(f.x);  // Should print 5
    //
    // This test verifies that the interpreter correctly handles
    // the bytecode that the parser generates for class definitions.
    let mut vm = VM::new();

    // Create constructor bytecode (as parser would generate)
    // Parser allocates parameters starting from register 0
    // Parser emits LoadGlobal("this") for ThisExpression
    let mut constructor_chunk = BytecodeChunk::new();
    // this.x = x
    // 1. Load this (parser emits LoadGlobal("this"))
    constructor_chunk.emit(Opcode::LoadGlobal("this".to_string()));
    // 2. Load x (first parameter is in register 0)
    constructor_chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // 3. Store as property
    constructor_chunk.emit(Opcode::StoreProperty("x".to_string()));
    // Return undefined (constructor returns this implicitly)
    constructor_chunk.emit(Opcode::LoadUndefined);
    constructor_chunk.emit(Opcode::Return);

    // Main chunk with nested constructor function
    let mut chunk = BytecodeChunk::new();
    // Add nested constructor function (index 0)
    chunk.add_nested_function(constructor_chunk);

    // Class definition: class Foo { constructor(x) { this.x = x; } }
    // Parser creates closure and stores as global
    chunk.emit(Opcode::CreateClosure(0, vec![])); // Create closure from nested function 0
    chunk.emit(Opcode::StoreGlobal("Foo".to_string())); // Store as global "Foo"

    // Instantiation: let f = new Foo(5);
    let five_val = chunk.add_constant(bytecode_system::Value::Number(5.0));
    chunk.emit(Opcode::LoadConstant(five_val)); // Push argument 5
    chunk.emit(Opcode::LoadGlobal("Foo".to_string())); // Load constructor
    chunk.emit(Opcode::CallNew(1)); // Call constructor with 1 argument
    chunk.emit(Opcode::StoreLocal(RegisterId(0))); // Store instance in register 0

    // Access: f.x
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadProperty("x".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "Parser-generated class should work: {:?}",
        result
    );
    assert_eq!(
        result.unwrap(),
        Value::Smi(5),
        "new Foo(5).x should be 5"
    );
}

#[test]
fn test_nested_this_binding() {
    // Test: Ensure `this` binding is properly saved/restored during nested constructor calls
    // class Outer { constructor() { this.inner = new Inner(); this.value = 1; } }
    // class Inner { constructor() { this.value = 2; } }
    let mut vm = VM::new();

    // Inner constructor
    let mut inner_constructor = BytecodeChunk::new();
    inner_constructor.emit(Opcode::LoadGlobal("this".to_string()));
    let two_val = inner_constructor.add_constant(bytecode_system::Value::Number(2.0));
    inner_constructor.emit(Opcode::LoadConstant(two_val));
    inner_constructor.emit(Opcode::StoreProperty("value".to_string()));
    inner_constructor.emit(Opcode::LoadUndefined);
    inner_constructor.emit(Opcode::Return);
    let inner_idx = vm.register_function(inner_constructor);

    // Outer constructor
    let mut outer_constructor = BytecodeChunk::new();
    // this.inner = new Inner()
    outer_constructor.emit(Opcode::LoadGlobal("this".to_string()));
    outer_constructor.emit(Opcode::CreateClosure(inner_idx, vec![]));
    outer_constructor.emit(Opcode::CallNew(0));
    outer_constructor.emit(Opcode::StoreProperty("inner".to_string()));
    // this.value = 1
    outer_constructor.emit(Opcode::LoadGlobal("this".to_string()));
    let one_val = outer_constructor.add_constant(bytecode_system::Value::Number(1.0));
    outer_constructor.emit(Opcode::LoadConstant(one_val));
    outer_constructor.emit(Opcode::StoreProperty("value".to_string()));
    outer_constructor.emit(Opcode::LoadUndefined);
    outer_constructor.emit(Opcode::Return);
    let outer_idx = vm.register_function(outer_constructor);

    // Main chunk
    let mut chunk = BytecodeChunk::new();
    chunk.emit(Opcode::CreateClosure(outer_idx, vec![]));
    chunk.emit(Opcode::CallNew(0));
    chunk.emit(Opcode::StoreLocal(RegisterId(0)));
    // Check outer.value == 1
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::LoadProperty("value".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Nested constructors should work: {:?}", result);
    assert_eq!(
        result.unwrap(),
        Value::Smi(1),
        "outer.value should be 1 (this binding restored after inner constructor)"
    );
}
