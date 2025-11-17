//! GC integration for JavaScript object allocation
//!
//! Provides heap-allocated JavaScript objects that integrate with
//! the memory_manager's garbage collector.

use core_types::Value;
use memory_manager::{Heap, HiddenClass};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// GC-managed JavaScript object
///
/// This structure represents a JavaScript object that is allocated
/// on the GC heap and supports property access with hidden class
/// optimization.
pub struct GCObject {
    /// Reference to the shared heap
    heap: Rc<RefCell<Heap>>,
    /// Property storage
    properties: HashMap<String, Value>,
    /// Prototype object (for prototype chain)
    prototype: Option<Box<GCObject>>,
    /// Hidden class for property layout optimization
    hidden_class: Option<Box<HiddenClass>>,
}

impl std::fmt::Debug for GCObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GCObject")
            .field("properties", &self.properties)
            .field("prototype", &self.prototype.as_ref().map(|_| "..."))
            .field("hidden_class", &self.hidden_class.as_ref().map(|_| "HiddenClass"))
            .finish()
    }
}

impl GCObject {
    /// Create a new empty GC-managed object
    ///
    /// # Arguments
    ///
    /// * `heap` - Shared reference to the GC heap
    pub fn new(heap: Rc<RefCell<Heap>>) -> Self {
        Self {
            heap,
            properties: HashMap::new(),
            prototype: None,
            hidden_class: Some(Box::new(HiddenClass::new())),
        }
    }

    /// Create a new GC-managed object with a prototype
    ///
    /// # Arguments
    ///
    /// * `heap` - Shared reference to the GC heap
    /// * `prototype` - The prototype object for this object
    pub fn with_prototype(heap: Rc<RefCell<Heap>>, prototype: GCObject) -> Self {
        Self {
            heap,
            properties: HashMap::new(),
            prototype: Some(Box::new(prototype)),
            hidden_class: Some(Box::new(HiddenClass::new())),
        }
    }

    /// Get a property value by name
    ///
    /// Traverses the prototype chain if the property is not found
    /// on the current object.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name to retrieve
    ///
    /// # Returns
    ///
    /// The property value, or `Value::Undefined` if not found
    pub fn get(&self, key: &str) -> Value {
        // Check own properties first (using hidden class if available)
        if let Some(ref class) = self.hidden_class {
            if class.lookup_property(key).is_some() {
                if let Some(value) = self.properties.get(key) {
                    return value.clone();
                }
            }
        } else if let Some(value) = self.properties.get(key) {
            return value.clone();
        }

        // Check prototype chain
        if let Some(ref proto) = self.prototype {
            return proto.get(key);
        }

        Value::Undefined
    }

    /// Set a property value
    ///
    /// Updates the hidden class if this is a new property.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name
    /// * `value` - The value to set
    pub fn set(&mut self, key: String, value: Value) {
        // Update hidden class for new property
        if !self.properties.contains_key(&key) {
            if let Some(ref class) = self.hidden_class {
                self.hidden_class = Some(class.add_property(key.clone()));
            }
        }
        self.properties.insert(key, value);
    }

    /// Check if the object has a property (including prototype chain)
    ///
    /// # Arguments
    ///
    /// * `key` - The property name to check
    ///
    /// # Returns
    ///
    /// `true` if the property exists, `false` otherwise
    pub fn has(&self, key: &str) -> bool {
        self.properties.contains_key(key)
            || self
                .prototype
                .as_ref()
                .map(|p| p.has(key))
                .unwrap_or(false)
    }

    /// Check if the object has an own property (not from prototype)
    ///
    /// # Arguments
    ///
    /// * `key` - The property name to check
    ///
    /// # Returns
    ///
    /// `true` if the property exists on this object directly
    pub fn has_own(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Get all own property keys
    ///
    /// # Returns
    ///
    /// A vector of property names
    pub fn keys(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }

    /// Get a reference to the prototype object
    pub fn prototype(&self) -> Option<&GCObject> {
        self.prototype.as_deref()
    }

    /// Set the prototype object
    ///
    /// # Arguments
    ///
    /// * `prototype` - The new prototype object
    pub fn set_prototype(&mut self, prototype: GCObject) {
        self.prototype = Some(Box::new(prototype));
    }

    /// Remove a property from the object
    ///
    /// # Arguments
    ///
    /// * `key` - The property name to remove
    ///
    /// # Returns
    ///
    /// `true` if the property was removed, `false` if it didn't exist
    pub fn delete(&mut self, key: &str) -> bool {
        self.properties.remove(key).is_some()
    }

    /// Get the number of own properties
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Get a reference to the hidden class
    pub fn hidden_class(&self) -> Option<&HiddenClass> {
        self.hidden_class.as_deref()
    }

    /// Get a reference to the heap
    pub fn heap(&self) -> &Rc<RefCell<Heap>> {
        &self.heap
    }
}

/// Heap wrapper for the VM
///
/// Provides a simplified interface for creating and managing
/// GC-allocated JavaScript objects.
pub struct VMHeap {
    /// Shared heap reference
    heap: Rc<RefCell<Heap>>,
}

impl VMHeap {
    /// Create a new VM heap with default configuration
    ///
    /// Uses 4MB young generation and promotion threshold of 3.
    pub fn new() -> Self {
        Self {
            heap: Rc::new(RefCell::new(Heap::new())),
        }
    }

    /// Create a new VM heap with custom configuration
    ///
    /// # Arguments
    ///
    /// * `young_gen_size` - Size of young generation in bytes
    /// * `promotion_threshold` - Number of GC cycles before promotion
    pub fn with_config(young_gen_size: usize, promotion_threshold: u8) -> Self {
        Self {
            heap: Rc::new(RefCell::new(Heap::with_config(
                young_gen_size,
                promotion_threshold,
            ))),
        }
    }

    /// Create a new empty GC-managed object
    pub fn create_object(&self) -> GCObject {
        GCObject::new(Rc::clone(&self.heap))
    }

    /// Create a new GC-managed object with a prototype
    ///
    /// # Arguments
    ///
    /// * `prototype` - The prototype object
    pub fn create_object_with_prototype(&self, prototype: GCObject) -> GCObject {
        GCObject::with_prototype(Rc::clone(&self.heap), prototype)
    }

    /// Trigger garbage collection
    ///
    /// Performs a young generation collection.
    pub fn collect_garbage(&self) {
        self.heap.borrow_mut().collect_garbage();
    }

    /// Trigger a full garbage collection
    ///
    /// Collects both young and old generations.
    pub fn full_gc(&self) {
        self.heap.borrow_mut().full_gc();
    }

    /// Get heap statistics
    ///
    /// # Returns
    ///
    /// Tuple of (young_generation_size, old_generation_size)
    pub fn stats(&self) -> (usize, usize) {
        let heap = self.heap.borrow();
        (heap.young_generation_size(), heap.old_generation_size())
    }

    /// Get detailed GC statistics
    pub fn gc_stats(&self) -> memory_manager::GcStats {
        self.heap.borrow().stats().clone()
    }

    /// Get total memory usage
    pub fn total_memory(&self) -> usize {
        self.heap.borrow().total_memory()
    }

    /// Get the number of young GC collections performed
    pub fn young_gc_count(&self) -> usize {
        self.heap.borrow().stats().young_gc_count
    }

    /// Get the number of full GC collections performed
    pub fn old_gc_count(&self) -> usize {
        self.heap.borrow().stats().old_gc_count
    }

    /// Reset GC statistics
    pub fn reset_stats(&self) {
        self.heap.borrow_mut().reset_stats();
    }

    /// Get shared heap reference (for advanced operations)
    pub fn heap_ref(&self) -> Rc<RefCell<Heap>> {
        Rc::clone(&self.heap)
    }
}

impl Default for VMHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for VMHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (young, old) = self.stats();
        f.debug_struct("VMHeap")
            .field("young_generation_size", &young)
            .field("old_generation_size", &old)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_object_new() {
        let heap = VMHeap::new();
        let obj = heap.create_object();
        assert_eq!(obj.property_count(), 0);
        assert!(obj.hidden_class().is_some());
        assert!(obj.prototype().is_none());
    }

    #[test]
    fn test_gc_object_set_and_get() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        obj.set("x".to_string(), Value::Smi(42));
        obj.set("y".to_string(), Value::Double(3.14));

        assert_eq!(obj.get("x"), Value::Smi(42));
        assert_eq!(obj.get("y"), Value::Double(3.14));
        assert_eq!(obj.get("z"), Value::Undefined);
    }

    #[test]
    fn test_gc_object_has_property() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        obj.set("x".to_string(), Value::Smi(1));

        assert!(obj.has("x"));
        assert!(!obj.has("y"));
        assert!(obj.has_own("x"));
        assert!(!obj.has_own("y"));
    }

    #[test]
    fn test_gc_object_keys() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        obj.set("a".to_string(), Value::Smi(1));
        obj.set("b".to_string(), Value::Smi(2));
        obj.set("c".to_string(), Value::Smi(3));

        let keys = obj.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"c".to_string()));
    }

    #[test]
    fn test_gc_object_delete() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        obj.set("x".to_string(), Value::Smi(42));
        assert!(obj.has("x"));

        let deleted = obj.delete("x");
        assert!(deleted);
        assert!(!obj.has("x"));

        let deleted_again = obj.delete("x");
        assert!(!deleted_again);
    }

    #[test]
    fn test_gc_object_prototype_chain() {
        let heap = VMHeap::new();
        let mut proto = heap.create_object();
        proto.set("inherited".to_string(), Value::Smi(100));

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
    fn test_gc_object_prototype_shadowing() {
        let heap = VMHeap::new();
        let mut proto = heap.create_object();
        proto.set("x".to_string(), Value::Smi(100));

        let mut obj = heap.create_object_with_prototype(proto);
        obj.set("x".to_string(), Value::Smi(42));

        // Should get own property, not inherited
        assert_eq!(obj.get("x"), Value::Smi(42));
    }

    #[test]
    fn test_gc_object_has_with_prototype() {
        let heap = VMHeap::new();
        let mut proto = heap.create_object();
        proto.set("inherited".to_string(), Value::Smi(100));

        let obj = heap.create_object_with_prototype(proto);

        assert!(obj.has("inherited"));
        assert!(!obj.has_own("inherited"));
    }

    #[test]
    fn test_gc_object_hidden_class_transitions() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        // Initial hidden class has no properties
        let initial_count = obj.hidden_class().unwrap().property_count();
        assert_eq!(initial_count, 0);

        // Adding a property creates a new hidden class
        obj.set("x".to_string(), Value::Smi(1));
        let count_after_x = obj.hidden_class().unwrap().property_count();
        assert_eq!(count_after_x, 1);

        obj.set("y".to_string(), Value::Smi(2));
        let count_after_y = obj.hidden_class().unwrap().property_count();
        assert_eq!(count_after_y, 2);

        // Updating existing property doesn't change hidden class
        obj.set("x".to_string(), Value::Smi(100));
        let count_after_update = obj.hidden_class().unwrap().property_count();
        assert_eq!(count_after_update, 2);
    }

    #[test]
    fn test_vm_heap_new() {
        let heap = VMHeap::new();
        let (young, old) = heap.stats();
        assert_eq!(young, 0);
        assert_eq!(old, 0);
    }

    #[test]
    fn test_vm_heap_with_config() {
        let heap = VMHeap::with_config(1024, 5);
        let (young, old) = heap.stats();
        assert_eq!(young, 0);
        assert_eq!(old, 0);
    }

    #[test]
    fn test_vm_heap_gc_stats() {
        let heap = VMHeap::new();
        let stats = heap.gc_stats();
        assert_eq!(stats.young_gc_count, 0);
        assert_eq!(stats.old_gc_count, 0);
    }

    #[test]
    fn test_vm_heap_collect_garbage() {
        let heap = VMHeap::new();
        heap.collect_garbage();
        assert_eq!(heap.young_gc_count(), 1);
    }

    #[test]
    fn test_vm_heap_full_gc() {
        let heap = VMHeap::new();
        heap.full_gc();
        assert_eq!(heap.young_gc_count(), 1);
        assert_eq!(heap.old_gc_count(), 1);
    }

    #[test]
    fn test_vm_heap_reset_stats() {
        let heap = VMHeap::new();
        heap.collect_garbage();
        heap.full_gc();

        assert!(heap.young_gc_count() > 0);
        assert!(heap.old_gc_count() > 0);

        heap.reset_stats();

        assert_eq!(heap.young_gc_count(), 0);
        assert_eq!(heap.old_gc_count(), 0);
    }

    #[test]
    fn test_vm_heap_total_memory() {
        let heap = VMHeap::new();
        assert_eq!(heap.total_memory(), 0);
    }

    #[test]
    fn test_vm_heap_default() {
        let heap = VMHeap::default();
        let (young, old) = heap.stats();
        assert_eq!(young, 0);
        assert_eq!(old, 0);
    }

    #[test]
    fn test_vm_heap_debug() {
        let heap = VMHeap::new();
        let debug_str = format!("{:?}", heap);
        assert!(debug_str.contains("VMHeap"));
        assert!(debug_str.contains("young_generation_size"));
        assert!(debug_str.contains("old_generation_size"));
    }

    #[test]
    fn test_gc_object_multiple_operations() {
        let heap = VMHeap::new();
        let mut obj = heap.create_object();

        // Set various types
        obj.set("number".to_string(), Value::Smi(42));
        obj.set("float".to_string(), Value::Double(3.14));
        obj.set("bool".to_string(), Value::Boolean(true));
        obj.set("null".to_string(), Value::Null);
        obj.set("undefined".to_string(), Value::Undefined);

        assert_eq!(obj.property_count(), 5);

        // Verify all values
        assert_eq!(obj.get("number"), Value::Smi(42));
        assert_eq!(obj.get("float"), Value::Double(3.14));
        assert_eq!(obj.get("bool"), Value::Boolean(true));
        assert_eq!(obj.get("null"), Value::Null);
        assert_eq!(obj.get("undefined"), Value::Undefined);

        // Delete one
        obj.delete("null");
        assert_eq!(obj.property_count(), 4);
        assert_eq!(obj.get("null"), Value::Undefined);
    }

    #[test]
    fn test_gc_object_shared_heap() {
        let heap = VMHeap::new();
        let obj1 = heap.create_object();
        let obj2 = heap.create_object();

        // Both objects should share the same heap reference
        assert!(Rc::ptr_eq(obj1.heap(), obj2.heap()));
    }

    #[test]
    fn test_gc_object_set_prototype() {
        let heap = VMHeap::new();
        let mut proto = heap.create_object();
        proto.set("proto_prop".to_string(), Value::Smi(1));

        let mut obj = heap.create_object();
        assert!(obj.prototype().is_none());

        obj.set_prototype(proto);
        assert!(obj.prototype().is_some());
        assert_eq!(obj.get("proto_prop"), Value::Smi(1));
    }
}
