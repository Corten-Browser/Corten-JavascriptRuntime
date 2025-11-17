//! Contract tests verifying the memory_manager API matches the contract specification.
//! These tests ensure all exported types and functions exist with correct signatures.

use core_types::Value;
use memory_manager::{write_barrier, Heap, HiddenClass, JSObject};

/// Test Heap contract: new() -> Self
#[test]
fn contract_heap_new() {
    let heap = Heap::new();
    // Should create a new heap
    let _ = heap;
}

/// Test Heap contract: allocate(size: usize) -> *mut u8
#[test]
fn contract_heap_allocate() {
    let mut heap = Heap::new();
    let ptr = heap.allocate(64);
    assert!(!ptr.is_null());
}

/// Test Heap contract: collect_garbage() -> ()
#[test]
fn contract_heap_collect_garbage() {
    let mut heap = Heap::new();
    heap.collect_garbage();
    // Should complete without error
}

/// Test Heap contract: young_generation_size() -> usize
#[test]
fn contract_heap_young_generation_size() {
    let heap = Heap::new();
    // Size returns used space - starts at 0 for empty heap
    let size = heap.young_generation_size();
    assert_eq!(size, 0);
    // Capacity should be > 0
    let capacity = heap.young_generation_capacity();
    assert!(capacity > 0);
}

/// Test Heap contract: old_generation_size() -> usize
#[test]
fn contract_heap_old_generation_size() {
    let heap = Heap::new();
    // Old generation returns total reserved memory (may be 0 initially)
    let size = heap.old_generation_size();
    // Just verify it doesn't panic and returns a valid value
    let _ = size;
}

/// Test HiddenClass contract: new() -> Self
#[test]
fn contract_hidden_class_new() {
    let class = HiddenClass::new();
    let _ = class;
}

/// Test HiddenClass contract: add_property(name: String) -> Box<HiddenClass>
#[test]
fn contract_hidden_class_add_property() {
    let class = HiddenClass::new();
    let new_class = class.add_property("foo".to_string());
    let _ = new_class;
}

/// Test HiddenClass contract: lookup_property(name: &str) -> Option<u32>
#[test]
fn contract_hidden_class_lookup_property() {
    let class = HiddenClass::new();
    let new_class = class.add_property("foo".to_string());
    let offset = new_class.lookup_property("foo");
    assert!(offset.is_some());
}

/// Test JSObject contract: new(class: *const HiddenClass) -> Self
#[test]
fn contract_jsobject_new() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);
    let obj = JSObject::new(class_ptr);
    let _ = obj;
    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test JSObject contract: get_property(name: &str) -> Option<Value>
#[test]
fn contract_jsobject_get_property() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);
    let obj = JSObject::new(class_ptr);
    let value = obj.get_property("nonexistent");
    assert!(value.is_none());
    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test JSObject contract: set_property(name: String, value: Value) -> ()
#[test]
fn contract_jsobject_set_property() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);
    let mut obj = JSObject::new(class_ptr);
    obj.set_property("foo".to_string(), Value::Smi(42));
    let value = obj.get_property("foo");
    assert_eq!(value, Some(Value::Smi(42)));
    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test JSObject has required fields
#[test]
fn contract_jsobject_fields() {
    let class = Box::new(HiddenClass::new());
    let class_ptr = Box::into_raw(class);
    let obj = JSObject::new(class_ptr);

    // Verify fields exist and are accessible
    let _class: *const HiddenClass = obj.class;
    let _properties: &Vec<Value> = &obj.properties;
    let _elements: &Vec<Value> = &obj.elements;

    // Clean up
    unsafe {
        let _ = Box::from_raw(class_ptr);
    }
}

/// Test write_barrier function exists with correct signature
#[test]
fn contract_write_barrier_exists() {
    // The function signature is:
    // pub unsafe fn write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value)
    // We just verify it compiles with the correct signature
    let _fn_ptr: unsafe fn(*mut memory_manager::Object, *mut Value, Value) = write_barrier;
}
