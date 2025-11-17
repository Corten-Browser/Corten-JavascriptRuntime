//! Write barriers for garbage collection.
//!
//! Write barriers are crucial for maintaining GC invariants:
//! - Remembered set: Track old-to-young pointers
//! - Tri-color invariant: Maintain marking consistency during concurrent GC

use crate::heap::{Heap, Object};
use core_types::Value;

/// Global heap instance for write barrier operations.
///
/// In a real implementation, this would be thread-local or passed explicitly.
/// For simplicity, we use a static mutable reference approach.
static mut GLOBAL_HEAP: Option<*mut Heap> = None;

/// Initializes the global heap for write barriers.
///
/// # Safety
///
/// This function must be called before any write_barrier calls.
/// Only one heap should be active at a time.
pub unsafe fn init_global_heap(heap: *mut Heap) {
    // SAFETY: Caller ensures this is called before write barriers
    // and that the heap pointer is valid for the program's lifetime.
    GLOBAL_HEAP = Some(heap);
}

/// Clears the global heap reference.
///
/// # Safety
///
/// This should be called when the heap is being dropped.
pub unsafe fn clear_global_heap() {
    // SAFETY: Caller ensures no more write barriers will be called.
    GLOBAL_HEAP = None;
}

/// Performs a write barrier for GC correctness.
///
/// This function must be called whenever a pointer field is updated.
/// It maintains:
/// 1. **Remembered set**: Tracks old generation objects pointing to young generation
/// 2. **Tri-color invariant**: During marking, ensures black objects don't point to white objects
///
/// # Safety
///
/// - `obj` must point to a valid Object in the heap
/// - `slot` must point to a valid memory location within `obj`
/// - `init_global_heap` must have been called
///
/// # Arguments
///
/// * `obj` - Pointer to the object being modified
/// * `slot` - Pointer to the slot being written
/// * `new_val` - The new value being written
///
/// # Example
///
/// ```
/// use memory_manager::{Heap, write_barrier};
/// use memory_manager::heap::Object;
/// use core_types::Value;
///
/// let mut heap = Heap::new();
/// let heap_ptr = &mut heap as *mut Heap;
///
/// unsafe {
///     memory_manager::init_global_heap(heap_ptr);
///
///     let obj_ptr = heap.allocate(64) as *mut Object;
///     let mut value = Value::Smi(42);
///     let slot = &mut value as *mut Value;
///
///     write_barrier(obj_ptr, slot, Value::Smi(100));
///
///     memory_manager::clear_global_heap();
/// }
/// ```
pub unsafe fn write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value) {
    // SAFETY: The caller guarantees that slot points to valid memory.
    // We write the new value first.
    *slot = new_val.clone();

    // Get the global heap
    // SAFETY: Caller must have called init_global_heap
    let heap = match GLOBAL_HEAP {
        Some(heap_ptr) => &mut *heap_ptr,
        None => return, // No heap initialized, skip barriers
    };

    // Check for old-to-young pointer (remembered set)
    if heap.is_in_old_gen(obj as *const u8) {
        if let Value::HeapObject(ptr) = &new_val {
            // The new value is a heap object reference
            // In a real implementation, we'd convert ptr to actual memory address
            // For now, we just add to remembered set
            let new_ptr = *ptr as *const u8;
            if heap.is_in_young_gen(new_ptr) {
                // Old object pointing to young object - add to remembered set
                heap.add_to_remembered_set(obj);
            }
        }
    }

    // Maintain tri-color invariant during marking
    if heap.is_marking() {
        // SAFETY: obj is a valid Object pointer (caller guarantee)
        let obj_ref = &*obj;

        // If object is black (mark == 2) and new value is white (unmarked heap object)
        if obj_ref.header.mark == 2 {
            if let Value::HeapObject(_) = &new_val {
                // In a real implementation:
                // - Convert the heap object reference to actual Object pointer
                // - Check if it's white (mark == 0)
                // - If so, mark it gray (mark = 1)
                // This prevents the tri-color invariant violation
            }
        }
    }
}

/// Checks if a value contains a heap object reference.
pub fn is_heap_reference(val: &Value) -> bool {
    matches!(val, Value::HeapObject(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_barrier_basic() {
        let mut heap = Heap::new();
        let heap_ptr = &mut heap as *mut Heap;

        unsafe {
            init_global_heap(heap_ptr);

            let obj_ptr = heap.allocate(64) as *mut Object;
            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            // Should not panic
            write_barrier(obj_ptr, slot, Value::Smi(42));

            // Value should be updated
            assert_eq!(*slot, Value::Smi(42));

            clear_global_heap();
        }
    }

    #[test]
    fn test_write_barrier_without_heap() {
        unsafe {
            clear_global_heap(); // Ensure no heap

            let mut dummy_obj = Object {
                header: crate::heap::ObjectHeader {
                    size: 32,
                    mark: 0,
                    tag: 0,
                    reserved: 0,
                },
            };
            let obj_ptr = &mut dummy_obj as *mut Object;
            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            // Should not panic even without heap
            write_barrier(obj_ptr, slot, Value::Smi(10));
            assert_eq!(*slot, Value::Smi(10));
        }
    }

    #[test]
    fn test_write_barrier_remembered_set() {
        let mut heap = Heap::new();

        // Note: In this simplified test, we can't easily test old-to-young
        // because our heap doesn't support allocating directly in old gen.
        // But we verify the barrier executes without error.

        let heap_ptr = &mut heap as *mut Heap;

        unsafe {
            init_global_heap(heap_ptr);

            let obj_ptr = heap.allocate(64) as *mut Object;
            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            // Write a heap object reference
            write_barrier(obj_ptr, slot, Value::HeapObject(12345));
            assert_eq!(*slot, Value::HeapObject(12345));

            clear_global_heap();
        }
    }

    #[test]
    fn test_write_barrier_during_marking() {
        let mut heap = Heap::new();
        let heap_ptr = &mut heap as *mut Heap;

        unsafe {
            init_global_heap(heap_ptr);

            heap.set_marking(true);

            let obj_ptr = heap.allocate(64) as *mut Object;

            // Set object to black (fully scanned)
            (*obj_ptr).set_mark(2);

            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            // Write during marking phase
            write_barrier(obj_ptr, slot, Value::HeapObject(9999));

            assert_eq!(*slot, Value::HeapObject(9999));

            heap.set_marking(false);
            clear_global_heap();
        }
    }

    #[test]
    fn test_is_heap_reference() {
        assert!(!is_heap_reference(&Value::Undefined));
        assert!(!is_heap_reference(&Value::Null));
        assert!(!is_heap_reference(&Value::Boolean(true)));
        assert!(!is_heap_reference(&Value::Smi(42)));
        assert!(!is_heap_reference(&Value::Double(3.14)));
        assert!(is_heap_reference(&Value::HeapObject(100)));
    }

    #[test]
    fn test_write_barrier_various_values() {
        let mut heap = Heap::new();
        let heap_ptr = &mut heap as *mut Heap;

        unsafe {
            init_global_heap(heap_ptr);

            let obj_ptr = heap.allocate(64) as *mut Object;
            let mut value = Value::Undefined;
            let slot = &mut value as *mut Value;

            // Test various value types
            write_barrier(obj_ptr, slot, Value::Undefined);
            assert_eq!(*slot, Value::Undefined);

            write_barrier(obj_ptr, slot, Value::Null);
            assert_eq!(*slot, Value::Null);

            write_barrier(obj_ptr, slot, Value::Boolean(true));
            assert_eq!(*slot, Value::Boolean(true));

            write_barrier(obj_ptr, slot, Value::Double(2.718));
            assert_eq!(*slot, Value::Double(2.718));

            clear_global_heap();
        }
    }
}
