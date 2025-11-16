//! Memory management for JavaScript runtime.
//!
//! This crate provides garbage collection and heap management including:
//! - Generational garbage collector (young + old generation)
//! - Hidden class system for property access optimization
//! - JavaScript object representation
//! - Write barriers for GC correctness
//!
//! # Overview
//!
//! - [`Heap`] - Main heap with generational GC
//! - [`HiddenClass`] - Hidden class for property optimization
//! - [`JSObject`] - JavaScript object with hidden class
//! - [`write_barrier`] - Write barrier for GC invariants
//! - [`Object`] - Low-level heap object
//!
//! # Example
//!
//! ```
//! use memory_manager::{Heap, HiddenClass, JSObject};
//! use core_types::Value;
//!
//! // Create a heap for allocation
//! let mut heap = Heap::new();
//!
//! // Create a hidden class and object
//! let class = Box::new(HiddenClass::new());
//! let class_ptr = Box::into_raw(class);
//! let mut obj = JSObject::new(class_ptr);
//!
//! // Set properties on the object
//! obj.set_property("x".to_string(), Value::Smi(10));
//! obj.set_property("y".to_string(), Value::Smi(20));
//!
//! assert_eq!(obj.get_property("x"), Some(Value::Smi(10)));
//! assert_eq!(obj.get_property("y"), Some(Value::Smi(20)));
//!
//! // Allocate memory from the heap
//! let ptr = heap.allocate(64);
//! assert!(!ptr.is_null());
//!
//! // Trigger garbage collection
//! heap.collect_garbage();
//!
//! // Clean up
//! unsafe { let _ = Box::from_raw(class_ptr); }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
// Allow unsafe code - necessary for raw pointer operations in GC
#![allow(unsafe_code)]

pub mod barriers;
pub mod heap;
pub mod hidden_class;
pub mod object;

// Re-export main types
pub use barriers::{clear_global_heap, init_global_heap, write_barrier};
pub use heap::{Arena, Heap, Object, ObjectHeader};
pub use hidden_class::{HiddenClass, PropertyDescriptor};
pub use object::JSObject;

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::Value;

    #[test]
    fn test_integration_heap_and_object() {
        let mut heap = Heap::new();

        // Allocate space for an object
        let ptr = heap.allocate(128);
        assert!(!ptr.is_null());

        // Create a JSObject
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let mut obj = JSObject::new(class_ptr);

        obj.set_property("value".to_string(), Value::Smi(42));
        assert_eq!(obj.get_property("value"), Some(Value::Smi(42)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_integration_hidden_class_transitions() {
        let class1 = HiddenClass::new();
        let class2 = class1.add_property("a".to_string());
        let class3 = class2.add_property("b".to_string());

        assert_eq!(class3.lookup_property("a"), Some(0));
        assert_eq!(class3.lookup_property("b"), Some(1));
        assert_eq!(class3.lookup_property("c"), None);
    }

    #[test]
    fn test_integration_write_barrier() {
        let mut heap = Heap::new();
        let heap_ptr = &mut heap as *mut Heap;

        unsafe {
            init_global_heap(heap_ptr);

            let obj_ptr = heap.allocate(64) as *mut Object;
            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            write_barrier(obj_ptr, slot, Value::Smi(100));
            assert_eq!(*slot, Value::Smi(100));

            clear_global_heap();
        }
    }

    #[test]
    fn test_heap_young_and_old_generation() {
        let heap = Heap::new();

        // Verify both generations have size > 0
        assert!(heap.young_generation_size() > 0);
        assert!(heap.old_generation_size() > 0);

        // Old gen should be larger than young gen
        assert!(heap.old_generation_size() > heap.young_generation_size());
    }
}
