//! JavaScript object representation with hidden class optimization.

use crate::hidden_class::HiddenClass;
use core_types::Value;

/// A JavaScript object with hidden class optimization.
///
/// JSObject uses a hidden class to efficiently store and retrieve properties.
/// Properties are stored in a dense vector using offsets from the hidden class.
///
/// # Safety
///
/// The `class` pointer must point to a valid `HiddenClass` that outlives this object.
///
/// # Example
///
/// ```
/// use memory_manager::{HiddenClass, JSObject};
/// use core_types::Value;
///
/// let class = Box::new(HiddenClass::new());
/// let class_ptr = Box::into_raw(class);
/// let mut obj = JSObject::new(class_ptr);
///
/// obj.set_property("x".to_string(), Value::Smi(10));
/// assert_eq!(obj.get_property("x"), Some(Value::Smi(10)));
///
/// // Clean up
/// unsafe { let _ = Box::from_raw(class_ptr); }
/// ```
#[derive(Debug)]
pub struct JSObject {
    /// Pointer to the hidden class describing this object's shape
    pub class: *const HiddenClass,
    /// Dense storage for named properties (indexed by hidden class offsets)
    pub properties: Vec<Value>,
    /// Dense storage for array elements (integer indices)
    pub elements: Vec<Value>,
    /// The current hidden class (owned, for property transitions)
    hidden_class: Box<HiddenClass>,
}

impl JSObject {
    /// Creates a new JavaScript object with the given hidden class.
    ///
    /// # Arguments
    ///
    /// * `class` - Pointer to the hidden class for this object
    ///
    /// # Safety Note
    ///
    /// While this function accepts a raw pointer, it is designed to be used
    /// safely. The caller should ensure that `class` points to a valid
    /// `HiddenClass` that will outlive this object. If `class` is null,
    /// the object will be created with an empty property list.
    ///
    /// The raw pointer is only dereferenced once during construction to
    /// determine the initial property count.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(class: *const HiddenClass) -> Self {
        // SAFETY: We check for null before dereferencing. The caller is
        // responsible for providing a valid pointer if not null.
        let initial_size = unsafe {
            if class.is_null() {
                0
            } else {
                (*class).properties.len()
            }
        };

        JSObject {
            class,
            properties: vec![Value::Undefined; initial_size],
            elements: Vec::new(),
            hidden_class: Box::new(HiddenClass::new()),
        }
    }

    /// Gets a property value by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name to look up
    ///
    /// # Returns
    ///
    /// `Some(Value)` if the property exists, `None` otherwise.
    pub fn get_property(&self, name: &str) -> Option<Value> {
        // First check our own hidden class
        if let Some(offset) = self.hidden_class.lookup_property(name) {
            return self.properties.get(offset as usize).cloned();
        }

        // Then check the original class pointer
        // SAFETY: We trust the class pointer to be valid if not null
        if !self.class.is_null() {
            let offset = unsafe { (*self.class).lookup_property(name) };
            if let Some(idx) = offset {
                return self.properties.get(idx as usize).cloned();
            }
        }

        None
    }

    /// Sets a property value by name.
    ///
    /// If the property doesn't exist, it creates a transition to a new hidden class
    /// and adds the property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name
    /// * `value` - The value to set
    pub fn set_property(&mut self, name: String, value: Value) {
        // Check if property already exists in our hidden class
        if let Some(offset) = self.hidden_class.lookup_property(&name) {
            // Property exists, just update the value
            if (offset as usize) < self.properties.len() {
                self.properties[offset as usize] = value;
            } else {
                // Extend properties vector if needed
                self.properties
                    .resize(offset as usize + 1, Value::Undefined);
                self.properties[offset as usize] = value;
            }
            return;
        }

        // Check original class pointer
        if !self.class.is_null() {
            // SAFETY: We trust the class pointer to be valid
            let offset = unsafe { (*self.class).lookup_property(&name) };
            if let Some(idx) = offset {
                if (idx as usize) < self.properties.len() {
                    self.properties[idx as usize] = value;
                } else {
                    self.properties.resize(idx as usize + 1, Value::Undefined);
                    self.properties[idx as usize] = value;
                }
                return;
            }
        }

        // Property doesn't exist, transition to new hidden class
        let new_class = self.hidden_class.add_property(name);
        let new_offset = new_class.properties.len() - 1;

        // Update our hidden class
        self.hidden_class = new_class;

        // Ensure properties vector is large enough
        if new_offset >= self.properties.len() {
            self.properties.resize(new_offset + 1, Value::Undefined);
        }
        self.properties[new_offset] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object_with_empty_class() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let obj = JSObject::new(class_ptr);

        assert_eq!(obj.class, class_ptr);
        assert!(obj.properties.is_empty());
        assert!(obj.elements.is_empty());

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_new_object_with_null_class() {
        let obj = JSObject::new(std::ptr::null());
        assert!(obj.properties.is_empty());
    }

    #[test]
    fn test_set_and_get_single_property() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let mut obj = JSObject::new(class_ptr);

        obj.set_property("name".to_string(), Value::Smi(42));
        assert_eq!(obj.get_property("name"), Some(Value::Smi(42)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_set_and_get_multiple_properties() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let mut obj = JSObject::new(class_ptr);

        obj.set_property("x".to_string(), Value::Smi(10));
        obj.set_property("y".to_string(), Value::Smi(20));
        obj.set_property("z".to_string(), Value::Double(3.14));

        assert_eq!(obj.get_property("x"), Some(Value::Smi(10)));
        assert_eq!(obj.get_property("y"), Some(Value::Smi(20)));
        assert_eq!(obj.get_property("z"), Some(Value::Double(3.14)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_update_existing_property() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let mut obj = JSObject::new(class_ptr);

        obj.set_property("value".to_string(), Value::Smi(1));
        assert_eq!(obj.get_property("value"), Some(Value::Smi(1)));

        obj.set_property("value".to_string(), Value::Smi(2));
        assert_eq!(obj.get_property("value"), Some(Value::Smi(2)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_get_nonexistent_property() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let obj = JSObject::new(class_ptr);

        assert_eq!(obj.get_property("nonexistent"), None);

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_different_value_types() {
        let class = Box::new(HiddenClass::new());
        let class_ptr = Box::into_raw(class);
        let mut obj = JSObject::new(class_ptr);

        obj.set_property("undefined".to_string(), Value::Undefined);
        obj.set_property("null".to_string(), Value::Null);
        obj.set_property("bool".to_string(), Value::Boolean(true));
        obj.set_property("int".to_string(), Value::Smi(100));
        obj.set_property("float".to_string(), Value::Double(1.5));

        assert_eq!(obj.get_property("undefined"), Some(Value::Undefined));
        assert_eq!(obj.get_property("null"), Some(Value::Null));
        assert_eq!(obj.get_property("bool"), Some(Value::Boolean(true)));
        assert_eq!(obj.get_property("int"), Some(Value::Smi(100)));
        assert_eq!(obj.get_property("float"), Some(Value::Double(1.5)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }

    #[test]
    fn test_object_with_predefined_class() {
        let class = HiddenClass::new();
        let class_with_x = class.add_property("x".to_string());
        let class_ptr = Box::into_raw(class_with_x);

        let mut obj = JSObject::new(class_ptr);

        // Object should have space for x
        obj.set_property("x".to_string(), Value::Smi(50));
        assert_eq!(obj.get_property("x"), Some(Value::Smi(50)));

        // Clean up
        unsafe {
            let _ = Box::from_raw(class_ptr);
        }
    }
}
