//! Hidden class implementation for property access optimization
//!
//! Hidden classes (also known as "shapes" or "maps") track the layout
//! of JavaScript objects, enabling fast property access through inline caches.

use std::collections::HashMap;

/// Hidden class for tracking object property layout
pub struct HiddenClass {
    /// Property name to offset mapping
    properties: HashMap<String, u32>,
    /// Next available offset for new properties
    next_offset: u32,
}

impl HiddenClass {
    /// Create a new empty hidden class
    pub fn new() -> Self {
        HiddenClass {
            properties: HashMap::new(),
            next_offset: 0,
        }
    }

    /// Add a property and return a new hidden class with the property
    ///
    /// Hidden classes are immutable - adding a property creates a new class
    pub fn add_property(&self, name: String) -> Box<HiddenClass> {
        let mut new_properties = self.properties.clone();
        let offset = self.next_offset;
        new_properties.insert(name, offset);

        Box::new(HiddenClass {
            properties: new_properties,
            next_offset: self.next_offset + 1,
        })
    }

    /// Look up the offset of a property by name
    ///
    /// Returns None if the property doesn't exist in this class
    pub fn lookup_property(&self, name: &str) -> Option<u32> {
        self.properties.get(name).copied()
    }

    /// Returns the number of properties in this hidden class
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }
}

impl Default for HiddenClass {
    fn default() -> Self {
        Self::new()
    }
}
