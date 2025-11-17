//! Tests for function call execution
//!
//! Tests cover:
//! - Simple function calls with arguments
//! - Return values
//! - Nested function calls
//! - Recursion (factorial)
//! - Immediately Invoked Function Expressions (IIFE)
//! - Closures

use bytecode_system::{BytecodeChunk, Opcode, RegisterId, Value as BcValue};
use core_types::Value;
use interpreter::VM;

/// Helper to create a simple add function bytecode
/// function add(a, b) { return a + b; }
fn create_add_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 2; // Two parameters: a, b

    // Load first argument (register 0)
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Load second argument (register 1)
    chunk.emit(Opcode::LoadLocal(RegisterId(1)));
    // Add them
    chunk.emit(Opcode::Add);
    // Return result
    chunk.emit(Opcode::Return);

    chunk
}

/// Helper to create factorial function bytecode
/// function factorial(n) {
///   if (n <= 1) return 1;
///   return n * factorial(n - 1);
/// }
fn create_factorial_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 2; // n and temp

    // Add constants
    let one_idx = chunk.add_constant(BcValue::Number(1.0));

    // Instruction 0: Load n (register 0)
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Instruction 1: Load 1
    chunk.emit(Opcode::LoadConstant(one_idx));
    // Instruction 2: Compare n <= 1
    chunk.emit(Opcode::LessThanEqual);
    // Instruction 3: Jump if false to recursive case (instruction 6)
    chunk.emit(Opcode::JumpIfFalse(6));

    // Base case: return 1
    // Instruction 4: Load 1
    chunk.emit(Opcode::LoadConstant(one_idx));
    // Instruction 5: Return
    chunk.emit(Opcode::Return);

    // Recursive case: n * factorial(n - 1)
    // Instruction 6: Load n
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Instruction 7: Load n again for argument
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Instruction 8: Load 1
    chunk.emit(Opcode::LoadConstant(one_idx));
    // Instruction 9: n - 1
    chunk.emit(Opcode::Sub);
    // Instruction 10: Load factorial function (will be global "factorial")
    chunk.emit(Opcode::LoadGlobal("factorial".to_string()));
    // Instruction 11: Call with 1 argument
    chunk.emit(Opcode::Call(1));
    // Instruction 12: Multiply n * result
    chunk.emit(Opcode::Mul);
    // Instruction 13: Return
    chunk.emit(Opcode::Return);

    chunk
}

/// Create a doubling function for IIFE test
/// function(x) { return x * 2; }
fn create_double_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 1; // One parameter: x

    let two_idx = chunk.add_constant(BcValue::Number(2.0));

    // Load x (register 0)
    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    // Load 2
    chunk.emit(Opcode::LoadConstant(two_idx));
    // Multiply
    chunk.emit(Opcode::Mul);
    // Return
    chunk.emit(Opcode::Return);

    chunk
}

/// Create an identity function
/// function(x) { return x; }
fn create_identity_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 1;

    chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    chunk.emit(Opcode::Return);

    chunk
}

/// Create a function that returns undefined
/// function() { }
fn create_void_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 0;

    chunk.emit(Opcode::LoadUndefined);
    chunk.emit(Opcode::Return);

    chunk
}

/// Create a function that returns a constant
/// function() { return 42; }
fn create_constant_function() -> BytecodeChunk {
    let mut chunk = BytecodeChunk::new();
    chunk.register_count = 0;

    let idx = chunk.add_constant(BcValue::Number(42.0));
    chunk.emit(Opcode::LoadConstant(idx));
    chunk.emit(Opcode::Return);

    chunk
}

#[test]
fn test_simple_function_call() {
    let mut vm = VM::new();

    // Register the add function
    let add_fn = create_add_function();
    let fn_idx = vm.register_function(add_fn);

    // Create main bytecode that calls add(2, 3)
    let mut main = BytecodeChunk::new();
    main.register_count = 0;

    // Push arguments onto stack
    let two_idx = main.add_constant(BcValue::Number(2.0));
    let three_idx = main.add_constant(BcValue::Number(3.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // push function first
    main.emit(Opcode::LoadConstant(two_idx)); // arg 1: 2
    main.emit(Opcode::LoadConstant(three_idx)); // arg 2: 3
    main.emit(Opcode::Call(2)); // call with 2 args
    main.emit(Opcode::Return); // return result

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(5)); // 2 + 3 = 5
}

#[test]
fn test_function_with_no_arguments() {
    let mut vm = VM::new();

    let void_fn = create_void_function();
    let fn_idx = vm.register_function(void_fn);

    let mut main = BytecodeChunk::new();
    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::Call(0));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Undefined);
}

#[test]
fn test_function_returns_constant() {
    let mut vm = VM::new();

    let const_fn = create_constant_function();
    let fn_idx = vm.register_function(const_fn);

    let mut main = BytecodeChunk::new();
    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::Call(0));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(42));
}

#[test]
fn test_nested_function_calls() {
    let mut vm = VM::new();

    // Register add function
    let add_fn = create_add_function();
    let add_idx = vm.register_function(add_fn);

    // Main: add(add(1, 2), add(3, 4))
    // Should return (1+2) + (3+4) = 3 + 7 = 10
    let mut main = BytecodeChunk::new();

    let one = main.add_constant(BcValue::Number(1.0));
    let two = main.add_constant(BcValue::Number(2.0));
    let three = main.add_constant(BcValue::Number(3.0));
    let four = main.add_constant(BcValue::Number(4.0));

    // First inner call: add(1, 2)
    main.emit(Opcode::CreateClosure(add_idx, vec![]));
    main.emit(Opcode::LoadConstant(one));
    main.emit(Opcode::LoadConstant(two));
    main.emit(Opcode::Call(2));

    // Second inner call: add(3, 4)
    main.emit(Opcode::CreateClosure(add_idx, vec![]));
    main.emit(Opcode::LoadConstant(three));
    main.emit(Opcode::LoadConstant(four));
    main.emit(Opcode::Call(2));

    // Outer call: add(result1, result2)
    main.emit(Opcode::CreateClosure(add_idx, vec![]));
    main.emit(Opcode::Call(2));

    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(10));
}

#[test]
fn test_iife_immediate_invocation() {
    let mut vm = VM::new();

    // (function(x) { return x * 2; })(10)
    let double_fn = create_double_function();
    let fn_idx = vm.register_function(double_fn);

    let mut main = BytecodeChunk::new();
    let ten_idx = main.add_constant(BcValue::Number(10.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // create function first
    main.emit(Opcode::LoadConstant(ten_idx)); // arg: 10
    main.emit(Opcode::Call(1)); // call immediately
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(20)); // 10 * 2 = 20
}

#[test]
fn test_factorial_base_case() {
    let mut vm = VM::new();

    let factorial_fn = create_factorial_function();
    let fn_idx = vm.register_function(factorial_fn);

    // Store factorial in global for recursive calls
    vm.set_global("factorial".to_string(), Value::HeapObject(fn_idx));

    let mut main = BytecodeChunk::new();
    let one_idx = main.add_constant(BcValue::Number(1.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // load factorial first
    main.emit(Opcode::LoadConstant(one_idx)); // arg: 1
    main.emit(Opcode::Call(1));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(1)); // factorial(1) = 1
}

#[test]
fn test_factorial_recursive() {
    let mut vm = VM::new();

    let factorial_fn = create_factorial_function();
    let fn_idx = vm.register_function(factorial_fn);

    // Store factorial in global for recursive calls
    vm.set_global("factorial".to_string(), Value::HeapObject(fn_idx));

    let mut main = BytecodeChunk::new();
    let five_idx = main.add_constant(BcValue::Number(5.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main.emit(Opcode::LoadConstant(five_idx)); // arg: 5
    main.emit(Opcode::Call(1));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(120)); // factorial(5) = 120
}

#[test]
fn test_identity_function() {
    let mut vm = VM::new();

    let id_fn = create_identity_function();
    let fn_idx = vm.register_function(id_fn);

    let mut main = BytecodeChunk::new();
    let val_idx = main.add_constant(BcValue::Number(999.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main.emit(Opcode::LoadConstant(val_idx));
    main.emit(Opcode::Call(1));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(999));
}

#[test]
fn test_call_with_extra_arguments() {
    // JavaScript ignores extra arguments
    let mut vm = VM::new();

    let id_fn = create_identity_function(); // expects 1 arg
    let fn_idx = vm.register_function(id_fn);

    let mut main = BytecodeChunk::new();
    let one_idx = main.add_constant(BcValue::Number(1.0));
    let two_idx = main.add_constant(BcValue::Number(2.0));
    let three_idx = main.add_constant(BcValue::Number(3.0));

    // Call with 3 arguments, function only uses first
    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main.emit(Opcode::LoadConstant(one_idx));
    main.emit(Opcode::LoadConstant(two_idx));
    main.emit(Opcode::LoadConstant(three_idx));
    main.emit(Opcode::Call(3));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(1)); // Should use first argument
}

#[test]
fn test_call_with_missing_arguments() {
    // JavaScript fills missing args with undefined
    let mut vm = VM::new();

    let add_fn = create_add_function(); // expects 2 args
    let fn_idx = vm.register_function(add_fn);

    let mut main = BytecodeChunk::new();
    let one_idx = main.add_constant(BcValue::Number(1.0));

    // Call with 1 argument, function expects 2
    main.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main.emit(Opcode::LoadConstant(one_idx));
    main.emit(Opcode::Call(1));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    // 1 + undefined = NaN (or implementation-specific)
    match result {
        Value::Double(n) => assert!(n.is_nan()),
        _ => panic!("Expected NaN for 1 + undefined"),
    }
}

#[test]
fn test_call_non_function_returns_error() {
    let mut vm = VM::new();

    let mut main = BytecodeChunk::new();
    let num_idx = main.add_constant(BcValue::Number(42.0));

    // Try to call a number
    main.emit(Opcode::LoadConstant(num_idx));
    main.emit(Opcode::Call(0));
    main.emit(Opcode::Return);

    let result = vm.execute(&main);
    // Should return a TypeError or similar
    assert!(result.is_err());
}

#[test]
fn test_deeply_nested_calls() {
    let mut vm = VM::new();

    let id_fn = create_identity_function();
    let fn_idx = vm.register_function(id_fn);

    // identity(identity(identity(42)))
    let mut main = BytecodeChunk::new();
    let val_idx = main.add_constant(BcValue::Number(42.0));

    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::LoadConstant(val_idx));
    main.emit(Opcode::Call(1));
    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::Call(1));
    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::Call(1));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(42));
}

#[test]
fn test_multiple_return_paths() {
    let mut vm = VM::new();

    // Function with if/else returning different values
    let mut fn_chunk = BytecodeChunk::new();
    fn_chunk.register_count = 1; // one param

    let five = fn_chunk.add_constant(BcValue::Number(5.0));
    let ten = fn_chunk.add_constant(BcValue::Number(10.0));
    let twenty = fn_chunk.add_constant(BcValue::Number(20.0));

    // if (x < 5) return 10; else return 20;
    fn_chunk.emit(Opcode::LoadLocal(RegisterId(0)));
    fn_chunk.emit(Opcode::LoadConstant(five));
    fn_chunk.emit(Opcode::LessThan);
    fn_chunk.emit(Opcode::JumpIfFalse(6));
    fn_chunk.emit(Opcode::LoadConstant(ten));
    fn_chunk.emit(Opcode::Return);
    fn_chunk.emit(Opcode::LoadConstant(twenty)); // instruction 6
    fn_chunk.emit(Opcode::Return);

    let fn_idx = vm.register_function(fn_chunk);

    // Test with x = 3 (should return 10)
    let mut main1 = BytecodeChunk::new();
    let three_idx = main1.add_constant(BcValue::Number(3.0));
    main1.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main1.emit(Opcode::LoadConstant(three_idx));
    main1.emit(Opcode::Call(1));
    main1.emit(Opcode::Return);

    let result1 = vm.execute(&main1).unwrap();
    assert_eq!(result1, Value::Smi(10));

    // Test with x = 7 (should return 20)
    let mut main2 = BytecodeChunk::new();
    let seven_idx = main2.add_constant(BcValue::Number(7.0));
    main2.emit(Opcode::CreateClosure(fn_idx, vec![])); // function first
    main2.emit(Opcode::LoadConstant(seven_idx));
    main2.emit(Opcode::Call(1));
    main2.emit(Opcode::Return);

    let result2 = vm.execute(&main2).unwrap();
    assert_eq!(result2, Value::Smi(20));
}

#[test]
fn test_function_stored_in_variable() {
    let mut vm = VM::new();

    let add_fn = create_add_function();
    let fn_idx = vm.register_function(add_fn);

    // Store function in global, then call it
    // var myAdd = add;
    // myAdd(1, 2);
    let mut main = BytecodeChunk::new();

    let one_idx = main.add_constant(BcValue::Number(1.0));
    let two_idx = main.add_constant(BcValue::Number(2.0));

    // Create closure and store in global
    main.emit(Opcode::CreateClosure(fn_idx, vec![]));
    main.emit(Opcode::StoreGlobal("myAdd".to_string()));

    // Load from global and call
    main.emit(Opcode::LoadConstant(one_idx));
    main.emit(Opcode::LoadConstant(two_idx));
    main.emit(Opcode::LoadGlobal("myAdd".to_string()));
    main.emit(Opcode::Call(2));
    main.emit(Opcode::Return);

    let result = vm.execute(&main).unwrap();
    assert_eq!(result, Value::Smi(3)); // 1 + 2 = 3
}
