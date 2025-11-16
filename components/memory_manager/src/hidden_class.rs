//! Hidden class system for optimizing JavaScript object property access.
//!
//! Hidden classes enable fast property access by tracking object shape
//! and using offset-based lookups instead of hash table lookups.

use std::collections::HashMap;

/// A property descriptor for a hidden class.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDescriptor {
    /// Name of the property
    pub name: String,
    /// Offset in the properties vector
    pub offset: u32,
}

/// Hidden class for JavaScript objects.
///
/// Objects with the same properties in the same order share a hidden class,
/// enabling fast property access through offset-based lookups.
///
/// # Example
///
/// ```
/// use memory_manager::HiddenClass;
///
/// let empty_class = HiddenClass::new();
/// let with_x = empty_class.add_property("x".to_string());
/// let with_xy = with_x.add_property("y".to_string());
///
/// assert_eq!(with_xy.lookup_property("x"), Some(0));
/// assert_eq!(with_xy.lookup_property("y"), Some(1));
/// ```
#[derive(Debug)]
pub struct HiddenClass {
    /// Properties in this class with their offsets
    pub properties: Vec<PropertyDescriptor>,
    /// Transitions to other hidden classes when properties are added
    pub transitions: HashMap<String, Box<HiddenClass>>,
    /// Prototype object reference (not used in minimal implementation)
    pub prototype: Option<usize>,
}

impl HiddenClass {
    /// Creates a new empty hidden class.
    ///
    /// # Returns
    ///
    /// A hidden class with no properties.
    pub fn new() -> Self {
        HiddenClass {
            properties: Vec::new(),
            transitions: HashMap::new(),
            prototype: None,
        }
    }

    /// Adds a property to the hidden class, creating a new hidden class.
    ///
    /// This method implements the transition mechanism. When a property is added
    /// to an object, we transition to a new hidden class that includes that property.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the property to add
    ///
    /// # Returns
    ///
    /// A new hidden class with the added property.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_manager::HiddenClass;
    ///
    /// let class1 = HiddenClass::new();
    /// let class2 = class1.add_property("name".to_string());
    /// assert_eq!(class2.lookup_property("name"), Some(0));
    /// ```
    pub fn add_property(&self, name: String) -> Box<HiddenClass> {
        // Check if we already have a transition for this property
        if let Some(existing) = self.transitions.get(&name) {
            // Note: In a real implementation, we'd return a shared reference
            // For simplicity, we create a new one with the same shape
            return Box::new(HiddenClass {
                properties: existing.properties.clone(),
                transitions: HashMap::new(),
                prototype: existing.prototype,
            });
        }

        // Create new hidden class with the added property
        let offset = self.properties.len() as u32;
        let mut new_properties = self.properties.clone();
        new_properties.push(PropertyDescriptor {
            name: name.clone(),
            offset,
        });

        Box::new(HiddenClass {
            properties: new_properties,
            transitions: HashMap::new(),
            prototype: self.prototype,
        })
    }

    /// Looks up a property by name and returns its offset.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the property to look up
    ///
    /// # Returns
    ///
    /// `Some(offset)` if the property exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_manager::HiddenClass;
    ///
    /// let class = HiddenClass::new().add_property("x".to_string());
    /// assert_eq!(class.lookup_property("x"), Some(0));
    /// assert_eq!(class.lookup_property("y"), None);
    /// ```
    pub fn lookup_property(&self, name: &str) -> Option<u32> {
        self.properties
            .iter()
            .find(|prop| prop.name == name)
            .map(|prop| prop.offset)
    }
}

impl Default for HiddenClass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hidden_class() {
        let class = HiddenClass::new();
        assert!(class.properties.is_empty());
        assert!(class.transitions.is_empty());
        assert!(class.prototype.is_none());
    }

    #[test]
    fn test_add_single_property() {
        let class = HiddenClass::new();
        let new_class = class.add_property("foo".to_string());

        assert_eq!(new_class.properties.len(), 1);
        assert_eq!(new_class.properties[0].name, "foo");
        assert_eq!(new_class.properties[0].offset, 0);
    }

    #[test]
    fn test_add_multiple_properties() {
        let class = HiddenClass::new();
        let class1 = class.add_property("x".to_string());
        let class2 = class1.add_property("y".to_string());
        let class3 = class2.add_property("z".to_string());

        assert_eq!(class3.properties.len(), 3);
        assert_eq!(class3.lookup_property("x"), Some(0));
        assert_eq!(class3.lookup_property("y"), Some(1));
        assert_eq!(class3.lookup_property("z"), Some(2));
    }

    #[test]
    fn test_lookup_nonexistent_property() {
        let class = HiddenClass::new();
        assert_eq!(class.lookup_property("nonexistent"), None);

        let with_prop = class.add_property("exists".to_string());
        assert_eq!(with_prop.lookup_property("nonexistent"), None);
    }

    #[test]
    fn test_hidden_class_preserves_prototype() {
        let mut class = HiddenClass::new();
        class.prototype = Some(42);
        let new_class = class.add_property("x".to_string());
        assert_eq!(new_class.prototype, Some(42));
    }

    #[test]
    fn test_default_implementation() {
        let class = HiddenClass::default();
        assert!(class.properties.is_empty());
    }
}
