//! JavaScript object representation
//!
//! Provides the core JSObject type with hidden class-based property storage.

use core_types::Value;

use crate::HiddenClass;

/// JavaScript object with hidden class-based property storage
pub struct JSObject {
    /// Pointer to the hidden class describing this object's layout
    pub class: *const HiddenClass,
    /// Property values, indexed by hidden class offsets
    pub properties: Vec<Value>,
    /// Array elements (for array-like objects)
    pub elements: Vec<Value>,
    /// Local property storage for properties not in hidden class
    /// This is used when the object has its own properties separate from the class
    local_properties: std::collections::HashMap<String, Value>,
}

impl JSObject {
    /// Create a new object with the given hidden class
    ///
    /// # Safety
    /// The hidden class pointer must remain valid for the object's lifetime
    pub fn new(class: *const HiddenClass) -> Self {
        JSObject {
            class,
            properties: Vec::new(),
            elements: Vec::new(),
            local_properties: std::collections::HashMap::new(),
        }
    }

    /// Get a property value by name
    pub fn get_property(&self, name: &str) -> Option<Value> {
        // First check local properties
        if let Some(value) = self.local_properties.get(name) {
            return Some(value.clone());
        }

        // Then check hidden class properties
        if !self.class.is_null() {
            // SAFETY: Caller must ensure class pointer is valid
            unsafe {
                if let Some(offset) = (*self.class).lookup_property(name) {
                    return self.properties.get(offset as usize).cloned();
                }
            }
        }

        None
    }

    /// Set a property value by name
    ///
    /// May transition to a new hidden class if adding a new property
    pub fn set_property(&mut self, name: String, value: Value) {
        // Check if property exists in hidden class
        if !self.class.is_null() {
            // SAFETY: Caller must ensure class pointer is valid
            unsafe {
                if let Some(offset) = (*self.class).lookup_property(&name) {
                    // Extend properties vector if needed
                    let offset = offset as usize;
                    if self.properties.len() <= offset {
                        self.properties.resize(offset + 1, Value::Undefined);
                    }
                    self.properties[offset] = value;
                    return;
                }
            }
        }

        // Store in local properties (simple implementation)
        // A more advanced implementation would transition to a new hidden class
        self.local_properties.insert(name, value);
    }
}
