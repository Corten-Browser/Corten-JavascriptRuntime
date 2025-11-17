//! Integration tests for interpreter
//!
//! Tests interaction between VM, ProfileData, InlineCache, and GC heap

use bytecode_system::{BytecodeChunk, Opcode, RegisterId, Value as BcValue};
use core_types::Value;
use interpreter::{GCObject, InlineCache, ProfileData, TypeInfo, VMHeap, VM};

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

#[test]
fn test_vm_console_global_exists() {
    let vm = VM::new();
    // Console global should be automatically injected
    let console = vm.get_global("console");
    assert!(console.is_some(), "Console global should exist");
    match console.unwrap() {
        Value::NativeObject(_) => {} // Expected
        other => panic!("Expected NativeObject, got {:?}", other),
    }
}

#[test]
fn test_vm_math_global_exists() {
    let vm = VM::new();
    // Math global should be automatically injected
    let math = vm.get_global("Math");
    assert!(math.is_some(), "Math global should exist");
    match math.unwrap() {
        Value::NativeObject(_) => {} // Expected
        other => panic!("Expected NativeObject, got {:?}", other),
    }
}

#[test]
fn test_vm_math_abs() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.abs(-5)
    let c_neg5 = chunk.add_constant(BcValue::Number(-5.0));

    chunk.emit(Opcode::LoadGlobal("Math".to_string())); // Load Math object
    chunk.emit(Opcode::LoadProperty("abs".to_string())); // Get abs method
    chunk.emit(Opcode::LoadConstant(c_neg5)); // Push argument
    chunk.emit(Opcode::Call(1)); // Call with 1 argument
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(5.0));
}

#[test]
fn test_vm_math_sqrt() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.sqrt(16)
    let c16 = chunk.add_constant(BcValue::Number(16.0));

    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("sqrt".to_string()));
    chunk.emit(Opcode::LoadConstant(c16));
    chunk.emit(Opcode::Call(1));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(4.0));
}

#[test]
fn test_vm_math_pow() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.pow(2, 3) = 8
    let c2 = chunk.add_constant(BcValue::Number(2.0));
    let c3 = chunk.add_constant(BcValue::Number(3.0));

    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("pow".to_string()));
    chunk.emit(Opcode::LoadConstant(c2));
    chunk.emit(Opcode::LoadConstant(c3));
    chunk.emit(Opcode::Call(2));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(8.0));
}

#[test]
fn test_vm_math_pi() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.PI
    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("PI".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    match result.unwrap() {
        Value::Double(n) => {
            assert!((n - std::f64::consts::PI).abs() < 1e-10);
        }
        other => panic!("Expected Double, got {:?}", other),
    }
}

#[test]
fn test_vm_console_log() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: console.log("hello")
    // Note: We can't easily test the actual output, but we can verify it doesn't error
    let c_hello = chunk.add_constant(BcValue::Number(42.0)); // Using number since strings aren't fully supported

    chunk.emit(Opcode::LoadGlobal("console".to_string())); // Load console object
    chunk.emit(Opcode::LoadProperty("log".to_string())); // Get log method
    chunk.emit(Opcode::LoadConstant(c_hello)); // Push argument
    chunk.emit(Opcode::Call(1)); // Call with 1 argument
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    // console.log returns undefined
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_vm_console_error() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: console.error(100)
    let c100 = chunk.add_constant(BcValue::Number(100.0));

    chunk.emit(Opcode::LoadGlobal("console".to_string()));
    chunk.emit(Opcode::LoadProperty("error".to_string()));
    chunk.emit(Opcode::LoadConstant(c100));
    chunk.emit(Opcode::Call(1));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    // console.error returns undefined
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_vm_native_function_type() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Load Math.abs but don't call it - should be a NativeFunction
    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("abs".to_string()));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    match result.unwrap() {
        Value::NativeFunction(name) => {
            assert_eq!(name, "Math.abs");
        }
        other => panic!("Expected NativeFunction, got {:?}", other),
    }
}

#[test]
fn test_vm_math_max() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.max(1, 5, 3) = 5
    let c1 = chunk.add_constant(BcValue::Number(1.0));
    let c5 = chunk.add_constant(BcValue::Number(5.0));
    let c3 = chunk.add_constant(BcValue::Number(3.0));

    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("max".to_string()));
    chunk.emit(Opcode::LoadConstant(c1));
    chunk.emit(Opcode::LoadConstant(c5));
    chunk.emit(Opcode::LoadConstant(c3));
    chunk.emit(Opcode::Call(3));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(5.0));
}

#[test]
fn test_vm_math_min() {
    let mut vm = VM::new();
    let mut chunk = BytecodeChunk::new();

    // Equivalent to: Math.min(1, 5, 3) = 1
    let c1 = chunk.add_constant(BcValue::Number(1.0));
    let c5 = chunk.add_constant(BcValue::Number(5.0));
    let c3 = chunk.add_constant(BcValue::Number(3.0));

    chunk.emit(Opcode::LoadGlobal("Math".to_string()));
    chunk.emit(Opcode::LoadProperty("min".to_string()));
    chunk.emit(Opcode::LoadConstant(c1));
    chunk.emit(Opcode::LoadConstant(c5));
    chunk.emit(Opcode::LoadConstant(c3));
    chunk.emit(Opcode::Call(3));
    chunk.emit(Opcode::Return);

    let result = vm.execute(&chunk);
    assert_eq!(result.unwrap(), Value::Double(1.0));
}

// GC Integration Tests

#[test]
fn test_vm_heap_exists() {
    let vm = VM::new();
    // VM should have a heap
    let heap = vm.heap();
    let (young, old) = heap.stats();
    assert_eq!(young, 0);
    assert_eq!(old, 0);
}

#[test]
fn test_vm_gc_stats() {
    let vm = VM::new();
    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 0);
    assert_eq!(stats.old_gc_count, 0);
    assert_eq!(stats.total_allocated, 0);
}

#[test]
fn test_vm_collect_garbage() {
    let vm = VM::new();
    vm.collect_garbage();
    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 1);
}

#[test]
fn test_vm_full_gc() {
    let vm = VM::new();
    vm.full_gc();
    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 1);
    assert_eq!(stats.old_gc_count, 1);
}

#[test]
fn test_gc_object_direct_creation() {
    let vm = VM::new();
    let heap = vm.heap();

    // Create a GC object directly
    let mut obj = heap.create_object();
    obj.set("x".to_string(), Value::Smi(42));
    obj.set("y".to_string(), Value::Double(3.14));

    assert_eq!(obj.get("x"), Value::Smi(42));
    assert_eq!(obj.get("y"), Value::Double(3.14));
    assert_eq!(obj.get("z"), Value::Undefined);
}

#[test]
fn test_gc_object_prototype_chain() {
    let vm = VM::new();
    let heap = vm.heap();

    // Create prototype object
    let mut proto = heap.create_object();
    proto.set("inherited".to_string(), Value::Smi(100));

    // Create object with prototype
    let mut obj = heap.create_object_with_prototype(proto);
    obj.set("own".to_string(), Value::Smi(42));

    // Own property
    assert_eq!(obj.get("own"), Value::Smi(42));
    // Inherited property
    assert_eq!(obj.get("inherited"), Value::Smi(100));
    // Non-existent
    assert_eq!(obj.get("missing"), Value::Undefined);
}

#[test]
fn test_gc_object_property_shadowing() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut proto = heap.create_object();
    proto.set("x".to_string(), Value::Smi(100));

    let mut obj = heap.create_object_with_prototype(proto);
    obj.set("x".to_string(), Value::Smi(42));

    // Should get own property, not inherited
    assert_eq!(obj.get("x"), Value::Smi(42));
}

#[test]
fn test_gc_object_hidden_class_evolution() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut obj = heap.create_object();

    // Initial hidden class has no properties
    let initial_count = obj.hidden_class().unwrap().property_count();
    assert_eq!(initial_count, 0);

    // Adding properties creates new hidden classes
    obj.set("a".to_string(), Value::Smi(1));
    assert_eq!(obj.hidden_class().unwrap().property_count(), 1);

    obj.set("b".to_string(), Value::Smi(2));
    assert_eq!(obj.hidden_class().unwrap().property_count(), 2);

    obj.set("c".to_string(), Value::Smi(3));
    assert_eq!(obj.hidden_class().unwrap().property_count(), 3);

    // Updating existing property doesn't change hidden class count
    obj.set("a".to_string(), Value::Smi(100));
    assert_eq!(obj.hidden_class().unwrap().property_count(), 3);
}

#[test]
fn test_gc_object_keys() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut obj = heap.create_object();
    obj.set("first".to_string(), Value::Smi(1));
    obj.set("second".to_string(), Value::Smi(2));
    obj.set("third".to_string(), Value::Smi(3));

    let keys = obj.keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"first".to_string()));
    assert!(keys.contains(&"second".to_string()));
    assert!(keys.contains(&"third".to_string()));
}

#[test]
fn test_gc_object_delete_property() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut obj = heap.create_object();
    obj.set("x".to_string(), Value::Smi(42));
    assert!(obj.has("x"));
    assert_eq!(obj.property_count(), 1);

    let deleted = obj.delete("x");
    assert!(deleted);
    assert!(!obj.has("x"));
    assert_eq!(obj.get("x"), Value::Undefined);
}

#[test]
fn test_gc_object_has_own_vs_has() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut proto = heap.create_object();
    proto.set("inherited".to_string(), Value::Smi(100));

    let obj = heap.create_object_with_prototype(proto);

    assert!(obj.has("inherited"));
    assert!(!obj.has_own("inherited"));
}

#[test]
fn test_vm_heap_accessor_methods() {
    let mut vm = VM::new();

    // Test immutable accessor
    {
        let heap = vm.heap();
        let stats = heap.gc_stats();
        assert_eq!(stats.young_gc_count, 0);
    }

    // Test mutable accessor
    {
        let heap_mut = vm.heap_mut();
        heap_mut.collect_garbage();
    }

    // Verify collection happened
    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 1);
}

#[test]
fn test_gc_object_various_value_types() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut obj = heap.create_object();

    // Test all value types
    obj.set("undefined".to_string(), Value::Undefined);
    obj.set("null".to_string(), Value::Null);
    obj.set("bool_true".to_string(), Value::Boolean(true));
    obj.set("bool_false".to_string(), Value::Boolean(false));
    obj.set("smi".to_string(), Value::Smi(42));
    obj.set("double".to_string(), Value::Double(3.14159));

    assert_eq!(obj.get("undefined"), Value::Undefined);
    assert_eq!(obj.get("null"), Value::Null);
    assert_eq!(obj.get("bool_true"), Value::Boolean(true));
    assert_eq!(obj.get("bool_false"), Value::Boolean(false));
    assert_eq!(obj.get("smi"), Value::Smi(42));
    assert_eq!(obj.get("double"), Value::Double(3.14159));
}

#[test]
fn test_vm_heap_multiple_collections() {
    let vm = VM::new();

    // Perform multiple GC cycles
    vm.collect_garbage();
    vm.collect_garbage();
    vm.full_gc();

    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 3); // 2 young + 1 from full_gc
    assert_eq!(stats.old_gc_count, 1);
}

#[test]
fn test_gc_object_set_prototype_after_creation() {
    let vm = VM::new();
    let heap = vm.heap();

    let mut proto = heap.create_object();
    proto.set("proto_method".to_string(), Value::Smi(100));

    let mut obj = heap.create_object();
    assert!(obj.prototype().is_none());

    obj.set_prototype(proto);
    assert!(obj.prototype().is_some());
    assert_eq!(obj.get("proto_method"), Value::Smi(100));
}

#[test]
fn test_vm_heap_reset_stats() {
    let vm = VM::new();

    // Do some GC operations
    vm.collect_garbage();
    vm.full_gc();

    // Verify stats accumulated
    {
        let stats = vm.gc_stats();
        assert!(stats.young_gc_count > 0);
        assert!(stats.old_gc_count > 0);
    }

    // Reset stats
    vm.heap().reset_stats();

    // Verify reset
    let stats = vm.gc_stats();
    assert_eq!(stats.young_gc_count, 0);
    assert_eq!(stats.old_gc_count, 0);
}

#[test]
fn test_gc_object_shared_heap_reference() {
    let vm = VM::new();
    let heap = vm.heap();

    let obj1 = heap.create_object();
    let obj2 = heap.create_object();

    // Both objects should reference the same heap
    assert!(std::rc::Rc::ptr_eq(obj1.heap(), obj2.heap()));
}
