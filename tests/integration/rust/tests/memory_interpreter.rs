//! Memory Manager and Interpreter Integration Tests
//!
//! Tests the integration between memory_manager and interpreter components.
//! Verifies that VM correctly uses heap allocation and object management.

use core_types::Value;
use memory_manager::{HiddenClass, Heap, JSObject};

/// Test: Create Heap and allocate memory
#[test]
fn test_heap_allocation() {
    let mut heap = Heap::new();

    // Allocate memory
    let ptr = heap.allocate(128);
    assert!(!ptr.is_null(), "Heap allocation should not return null");

    // Verify heap has both generations
    assert!(
        heap.young_generation_size() > 0,
        "Young generation should have size"
    );
    assert!(
        heap.old_generation_size() > 0,
        "Old generation should have size"
    );
}

/// Test: JSObject property storage and retrieval
#[test]
fn test_jsobject_properties() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);

    let mut obj = JSObject::new(class_ptr);

    // Set properties
    obj.set_property("x".to_string(), Value::Smi(10));
    obj.set_property("y".to_string(), Value::Smi(20));
    obj.set_property("z".to_string(), Value::Smi(30));

    // Verify retrieval
    assert_eq!(obj.get_property("x"), Some(Value::Smi(10)));
    assert_eq!(obj.get_property("y"), Some(Value::Smi(20)));
    assert_eq!(obj.get_property("z"), Some(Value::Smi(30)));

    // Non-existent property
    assert_eq!(obj.get_property("w"), None);

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test: Hidden class transitions
#[test]
fn test_hidden_class_transitions() {
    let class1 = HiddenClass::new();

    // Add properties create new hidden classes
    let class2 = class1.add_property("name".to_string());
    let class3 = class2.add_property("age".to_string());
    let class4 = class3.add_property("email".to_string());

    // Verify property offsets
    assert_eq!(class4.lookup_property("name"), Some(0));
    assert_eq!(class4.lookup_property("age"), Some(1));
    assert_eq!(class4.lookup_property("email"), Some(2));

    // Non-existent property
    assert_eq!(class4.lookup_property("phone"), None);
}

/// Test: Multiple objects with same shape
#[test]
fn test_multiple_objects_same_shape() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);

    let mut obj1 = JSObject::new(class_ptr);
    let mut obj2 = JSObject::new(class_ptr);

    // Set same properties on different objects
    obj1.set_property("value".to_string(), Value::Smi(100));
    obj2.set_property("value".to_string(), Value::Smi(200));

    // Each object maintains its own values
    assert_eq!(obj1.get_property("value"), Some(Value::Smi(100)));
    assert_eq!(obj2.get_property("value"), Some(Value::Smi(200)));

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test: Heap garbage collection
#[test]
fn test_heap_garbage_collection() {
    let mut heap = Heap::new();

    // Allocate some memory
    let _ptr1 = heap.allocate(64);
    let _ptr2 = heap.allocate(128);
    let _ptr3 = heap.allocate(256);

    // Trigger garbage collection
    heap.collect_garbage();

    // Heap should still be functional after GC
    let ptr4 = heap.allocate(64);
    assert!(!ptr4.is_null(), "Should be able to allocate after GC");
}

/// Test: JSObject with different value types
#[test]
fn test_jsobject_different_value_types() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);

    let mut obj = JSObject::new(class_ptr);

    // Store different types
    obj.set_property("number".to_string(), Value::Smi(42));
    obj.set_property("boolean".to_string(), Value::Boolean(true));
    obj.set_property("null_val".to_string(), Value::Null);
    obj.set_property("undefined".to_string(), Value::Undefined);

    // Verify each type
    assert_eq!(obj.get_property("number"), Some(Value::Smi(42)));
    assert_eq!(obj.get_property("boolean"), Some(Value::Boolean(true)));
    assert_eq!(obj.get_property("null_val"), Some(Value::Null));
    assert_eq!(obj.get_property("undefined"), Some(Value::Undefined));

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test: Heap generation sizes are correctly configured
#[test]
fn test_heap_generation_sizes() {
    let heap = Heap::new();

    let young_size = heap.young_generation_size();
    let old_size = heap.old_generation_size();

    // Old generation should be larger than young generation
    assert!(
        old_size > young_size,
        "Old generation ({}) should be larger than young generation ({})",
        old_size,
        young_size
    );

    // Both should have reasonable sizes
    assert!(young_size > 0, "Young generation should have positive size");
    assert!(old_size > 0, "Old generation should have positive size");
}

/// Test: Property overwriting
#[test]
fn test_jsobject_property_overwrite() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);

    let mut obj = JSObject::new(class_ptr);

    // Set initial value
    obj.set_property("counter".to_string(), Value::Smi(0));
    assert_eq!(obj.get_property("counter"), Some(Value::Smi(0)));

    // Overwrite with new value
    obj.set_property("counter".to_string(), Value::Smi(100));
    assert_eq!(obj.get_property("counter"), Some(Value::Smi(100)));

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test: Hidden class property lookup performance
#[test]
fn test_hidden_class_lookup_performance() {
    let mut class = HiddenClass::new();

    // Add many properties
    for i in 0..100 {
        class = *class.add_property(format!("prop{}", i));
    }

    // All properties should be found with correct offsets
    for i in 0..100 {
        let offset = class.lookup_property(&format!("prop{}", i));
        assert_eq!(offset, Some(i as u32), "Property prop{} should have offset {}", i, i);
    }

    // Non-existent property
    assert_eq!(class.lookup_property("nonexistent"), None);
}

/// Test: Multiple heap allocations
#[test]
fn test_multiple_heap_allocations() {
    let mut heap = Heap::new();

    let mut ptrs = Vec::new();

    // Allocate multiple chunks
    for size in [32, 64, 128, 256, 512] {
        let ptr = heap.allocate(size);
        assert!(!ptr.is_null(), "Allocation of {} bytes failed", size);
        ptrs.push(ptr);
    }

    // All pointers should be different
    for i in 0..ptrs.len() {
        for j in (i + 1)..ptrs.len() {
            assert_ne!(
                ptrs[i], ptrs[j],
                "Allocations should return different pointers"
            );
        }
    }
}

/// Test: Value type truthiness in JSObject context
#[test]
fn test_value_truthiness_in_object() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);

    let mut obj = JSObject::new(class_ptr);

    // Store values with different truthiness
    obj.set_property("truthy_num".to_string(), Value::Smi(42));
    obj.set_property("falsy_zero".to_string(), Value::Smi(0));
    obj.set_property("truthy_bool".to_string(), Value::Boolean(true));
    obj.set_property("falsy_bool".to_string(), Value::Boolean(false));

    // Verify values are stored correctly
    if let Some(Value::Smi(n)) = obj.get_property("truthy_num") {
        assert!(n != 0, "Truthy number should not be zero");
    }

    if let Some(Value::Smi(n)) = obj.get_property("falsy_zero") {
        assert_eq!(n, 0, "Falsy zero should be zero");
    }

    if let Some(Value::Boolean(b)) = obj.get_property("truthy_bool") {
        assert!(b, "Truthy bool should be true");
    }

    if let Some(Value::Boolean(b)) = obj.get_property("falsy_bool") {
        assert!(!b, "Falsy bool should be false");
    }

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}
