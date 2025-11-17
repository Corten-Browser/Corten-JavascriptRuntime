//! Map and Set collection implementations
//!
//! ES2024 compliant Map and Set built-in objects with SameValueZero semantics
//! and insertion order preservation.

use crate::value::{JsError, JsResult, JsValue};

/// Map built-in object
///
/// Implements ES2024 Map with:
/// - SameValueZero key comparison (NaN === NaN, -0 === +0)
/// - Insertion order preservation
/// - Support for any value type as keys
pub struct MapObject;

impl MapObject {
    /// Create a new empty Map
    pub fn new() -> JsValue {
        JsValue::map()
    }

    /// Create a Map from key-value pairs
    pub fn from_entries(entries: Vec<(JsValue, JsValue)>) -> JsValue {
        let map = Self::new();
        for (key, value) in entries {
            Self::set(&map, key, value);
        }
        map
    }

    /// Get the size of the Map
    pub fn size(map: &JsValue) -> usize {
        if let JsValue::Map(data) = map {
            data.borrow().entries.len()
        } else {
            0
        }
    }

    /// Set a key-value pair in the Map
    ///
    /// Returns the Map for chaining
    pub fn set(map: &JsValue, key: JsValue, value: JsValue) -> JsValue {
        if let JsValue::Map(data) = map {
            let mut map_data = data.borrow_mut();

            // Check if key already exists using SameValueZero
            if let Some(index) = map_data
                .entries
                .iter()
                .position(|(k, _)| k.same_value_zero(&key))
            {
                // Update existing entry (preserves insertion order)
                map_data.entries[index].1 = value;
            } else {
                // Add new entry
                map_data.entries.push((key, value));
            }
        }
        map.clone()
    }

    /// Get a value from the Map
    ///
    /// Returns None if key not found (JavaScript returns undefined)
    pub fn get(map: &JsValue, key: &JsValue) -> Option<JsValue> {
        if let JsValue::Map(data) = map {
            let map_data = data.borrow();
            map_data
                .entries
                .iter()
                .find(|(k, _)| k.same_value_zero(key))
                .map(|(_, v)| v.clone())
        } else {
            None
        }
    }

    /// Check if the Map has a key
    pub fn has(map: &JsValue, key: &JsValue) -> bool {
        if let JsValue::Map(data) = map {
            let map_data = data.borrow();
            map_data
                .entries
                .iter()
                .any(|(k, _)| k.same_value_zero(key))
        } else {
            false
        }
    }

    /// Delete a key from the Map
    ///
    /// Returns true if the key was found and deleted, false otherwise
    pub fn delete(map: &JsValue, key: &JsValue) -> bool {
        if let JsValue::Map(data) = map {
            let mut map_data = data.borrow_mut();
            if let Some(index) = map_data
                .entries
                .iter()
                .position(|(k, _)| k.same_value_zero(key))
            {
                map_data.entries.remove(index);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Clear all entries from the Map
    pub fn clear(map: &JsValue) {
        if let JsValue::Map(data) = map {
            data.borrow_mut().entries.clear();
        }
    }

    /// Get all keys in insertion order
    pub fn keys(map: &JsValue) -> Vec<JsValue> {
        if let JsValue::Map(data) = map {
            data.borrow()
                .entries
                .iter()
                .map(|(k, _)| k.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all values in insertion order
    pub fn values(map: &JsValue) -> Vec<JsValue> {
        if let JsValue::Map(data) = map {
            data.borrow()
                .entries
                .iter()
                .map(|(_, v)| v.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all entries as (key, value) tuples in insertion order
    pub fn entries(map: &JsValue) -> Vec<(JsValue, JsValue)> {
        if let JsValue::Map(data) = map {
            data.borrow().entries.clone()
        } else {
            Vec::new()
        }
    }

    /// Execute a callback for each entry
    pub fn for_each<F>(map: &JsValue, mut callback: F) -> JsResult<()>
    where
        F: FnMut(&JsValue, &JsValue) -> JsResult<()>,
    {
        if let JsValue::Map(data) = map {
            let entries = data.borrow().entries.clone();
            for (key, value) in entries {
                callback(&key, &value)?;
            }
            Ok(())
        } else {
            Err(JsError::type_error("forEach called on non-Map"))
        }
    }

    /// Create an iterator over the Map entries
    pub fn iter(map: &JsValue) -> MapIterator {
        MapIterator::new(map)
    }
}

/// Iterator for Map entries
pub struct MapIterator {
    entries: Vec<(JsValue, JsValue)>,
    index: usize,
}

impl MapIterator {
    fn new(map: &JsValue) -> Self {
        let entries = MapObject::entries(map);
        MapIterator { entries, index: 0 }
    }
}

impl Iterator for MapIterator {
    type Item = (JsValue, JsValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let entry = self.entries[self.index].clone();
            self.index += 1;
            Some(entry)
        } else {
            None
        }
    }
}

/// Set built-in object
///
/// Implements ES2024 Set with:
/// - SameValueZero value comparison (NaN === NaN, -0 === +0)
/// - Insertion order preservation
/// - Support for any value type
pub struct SetObject;

impl SetObject {
    /// Create a new empty Set
    pub fn new() -> JsValue {
        JsValue::set_collection()
    }

    /// Create a Set from values (automatically deduplicates)
    pub fn from_values(values: Vec<JsValue>) -> JsValue {
        let set = Self::new();
        for value in values {
            Self::add(&set, value);
        }
        set
    }

    /// Get the size of the Set
    pub fn size(set: &JsValue) -> usize {
        if let JsValue::Set(data) = set {
            data.borrow().values.len()
        } else {
            0
        }
    }

    /// Add a value to the Set
    ///
    /// Returns the Set for chaining
    pub fn add(set: &JsValue, value: JsValue) -> JsValue {
        if let JsValue::Set(data) = set {
            let mut set_data = data.borrow_mut();

            // Check if value already exists using SameValueZero
            if !set_data
                .values
                .iter()
                .any(|v| v.same_value_zero(&value))
            {
                set_data.values.push(value);
            }
        }
        set.clone()
    }

    /// Check if the Set has a value
    pub fn has(set: &JsValue, value: &JsValue) -> bool {
        if let JsValue::Set(data) = set {
            let set_data = data.borrow();
            set_data
                .values
                .iter()
                .any(|v| v.same_value_zero(value))
        } else {
            false
        }
    }

    /// Delete a value from the Set
    ///
    /// Returns true if the value was found and deleted, false otherwise
    pub fn delete(set: &JsValue, value: &JsValue) -> bool {
        if let JsValue::Set(data) = set {
            let mut set_data = data.borrow_mut();
            if let Some(index) = set_data
                .values
                .iter()
                .position(|v| v.same_value_zero(value))
            {
                set_data.values.remove(index);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Clear all values from the Set
    pub fn clear(set: &JsValue) {
        if let JsValue::Set(data) = set {
            data.borrow_mut().values.clear();
        }
    }

    /// Get all values in insertion order
    ///
    /// This is the same as keys() for Set (for consistency with JS API)
    pub fn values(set: &JsValue) -> Vec<JsValue> {
        if let JsValue::Set(data) = set {
            data.borrow().values.clone()
        } else {
            Vec::new()
        }
    }

    /// Get all keys in insertion order
    ///
    /// For Set, keys() is an alias for values()
    pub fn keys(set: &JsValue) -> Vec<JsValue> {
        Self::values(set)
    }

    /// Get all entries as (value, value) tuples
    ///
    /// For Set, entries are [value, value] pairs for consistency with Map
    pub fn entries(set: &JsValue) -> Vec<(JsValue, JsValue)> {
        if let JsValue::Set(data) = set {
            data.borrow()
                .values
                .iter()
                .map(|v| (v.clone(), v.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Execute a callback for each value
    pub fn for_each<F>(set: &JsValue, mut callback: F) -> JsResult<()>
    where
        F: FnMut(&JsValue) -> JsResult<()>,
    {
        if let JsValue::Set(data) = set {
            let values = data.borrow().values.clone();
            for value in values {
                callback(&value)?;
            }
            Ok(())
        } else {
            Err(JsError::type_error("forEach called on non-Set"))
        }
    }

    /// Create an iterator over the Set values
    pub fn iter(set: &JsValue) -> SetIterator {
        SetIterator::new(set)
    }
}

/// Iterator for Set values
pub struct SetIterator {
    values: Vec<JsValue>,
    index: usize,
}

impl SetIterator {
    fn new(set: &JsValue) -> Self {
        let values = SetObject::values(set);
        SetIterator { values, index: 0 }
    }
}

impl Iterator for SetIterator {
    type Item = JsValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.values.len() {
            let value = self.values[self.index].clone();
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// WeakMap built-in object
///
/// Implements ES2024 WeakMap with:
/// - Only objects as keys (not primitives)
/// - Weak references to keys (keys can be garbage collected)
/// - No size property (because of weak references)
/// - No iteration methods (keys are weakly held)
pub struct WeakMapObject;

impl WeakMapObject {
    /// Create a new empty WeakMap
    pub fn new() -> JsValue {
        JsValue::weak_map()
    }

    /// Create a WeakMap from key-value pairs
    ///
    /// Returns an error if any key is not an object.
    pub fn from_entries(entries: Vec<(JsValue, JsValue)>) -> JsResult<JsValue> {
        let weak_map = Self::new();
        for (key, value) in entries {
            Self::set(&weak_map, key, value)?;
        }
        Ok(weak_map)
    }

    /// Set a key-value pair in the WeakMap
    ///
    /// Returns the WeakMap for chaining.
    /// Returns TypeError if key is not an object.
    pub fn set(weak_map: &JsValue, key: JsValue, value: JsValue) -> JsResult<JsValue> {
        // Key must be an object
        let key_id = key
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used as weak map key"))?;

        if let JsValue::WeakMap(data) = weak_map {
            let mut weak_map_data = data.borrow_mut();
            weak_map_data.entries.insert(key_id, value);
        }
        Ok(weak_map.clone())
    }

    /// Get a value from the WeakMap
    ///
    /// Returns None if key not found (JavaScript returns undefined).
    /// Returns TypeError if key is not an object.
    pub fn get(weak_map: &JsValue, key: &JsValue) -> JsResult<Option<JsValue>> {
        // Key must be an object
        let key_id = key
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used as weak map key"))?;

        if let JsValue::WeakMap(data) = weak_map {
            let weak_map_data = data.borrow();
            Ok(weak_map_data.entries.get(&key_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Check if the WeakMap has a key
    ///
    /// Returns TypeError if key is not an object.
    pub fn has(weak_map: &JsValue, key: &JsValue) -> JsResult<bool> {
        // Key must be an object
        let key_id = key
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used as weak map key"))?;

        if let JsValue::WeakMap(data) = weak_map {
            let weak_map_data = data.borrow();
            Ok(weak_map_data.entries.contains_key(&key_id))
        } else {
            Ok(false)
        }
    }

    /// Delete a key from the WeakMap
    ///
    /// Returns true if the key was found and deleted, false otherwise.
    /// Returns TypeError if key is not an object.
    pub fn delete(weak_map: &JsValue, key: &JsValue) -> JsResult<bool> {
        // Key must be an object
        let key_id = key
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used as weak map key"))?;

        if let JsValue::WeakMap(data) = weak_map {
            let mut weak_map_data = data.borrow_mut();
            Ok(weak_map_data.entries.remove(&key_id).is_some())
        } else {
            Ok(false)
        }
    }
}

/// WeakSet built-in object
///
/// Implements ES2024 WeakSet with:
/// - Only objects as values (not primitives)
/// - Weak references to values (values can be garbage collected)
/// - No size property (because of weak references)
/// - No iteration methods (values are weakly held)
pub struct WeakSetObject;

impl WeakSetObject {
    /// Create a new empty WeakSet
    pub fn new() -> JsValue {
        JsValue::weak_set()
    }

    /// Create a WeakSet from values
    ///
    /// Returns an error if any value is not an object.
    pub fn from_values(values: Vec<JsValue>) -> JsResult<JsValue> {
        let weak_set = Self::new();
        for value in values {
            Self::add(&weak_set, value)?;
        }
        Ok(weak_set)
    }

    /// Add a value to the WeakSet
    ///
    /// Returns the WeakSet for chaining.
    /// Returns TypeError if value is not an object.
    pub fn add(weak_set: &JsValue, value: JsValue) -> JsResult<JsValue> {
        // Value must be an object
        let value_id = value
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used in weak set"))?;

        if let JsValue::WeakSet(data) = weak_set {
            let mut weak_set_data = data.borrow_mut();
            weak_set_data.values.insert(value_id, ());
        }
        Ok(weak_set.clone())
    }

    /// Check if the WeakSet has a value
    ///
    /// Returns TypeError if value is not an object.
    pub fn has(weak_set: &JsValue, value: &JsValue) -> JsResult<bool> {
        // Value must be an object
        let value_id = value
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used in weak set"))?;

        if let JsValue::WeakSet(data) = weak_set {
            let weak_set_data = data.borrow();
            Ok(weak_set_data.values.contains_key(&value_id))
        } else {
            Ok(false)
        }
    }

    /// Delete a value from the WeakSet
    ///
    /// Returns true if the value was found and deleted, false otherwise.
    /// Returns TypeError if value is not an object.
    pub fn delete(weak_set: &JsValue, value: &JsValue) -> JsResult<bool> {
        // Value must be an object
        let value_id = value
            .object_identity()
            .ok_or_else(|| JsError::type_error("Invalid value used in weak set"))?;

        if let JsValue::WeakSet(data) = weak_set {
            let mut weak_set_data = data.borrow_mut();
            Ok(weak_set_data.values.remove(&value_id).is_some())
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Map unit tests
    mod map_tests {
        use super::*;

        #[test]
        fn test_map_new() {
            let map = MapObject::new();
            assert!(map.is_map());
            assert_eq!(MapObject::size(&map), 0);
        }

        #[test]
        fn test_map_set_get() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("key"), JsValue::number(42.0));
            let value = MapObject::get(&map, &JsValue::string("key")).unwrap();
            assert_eq!(value.as_number().unwrap(), 42.0);
        }

        #[test]
        fn test_map_has() {
            let map = MapObject::new();
            assert!(!MapObject::has(&map, &JsValue::string("key")));
            MapObject::set(&map, JsValue::string("key"), JsValue::number(1.0));
            assert!(MapObject::has(&map, &JsValue::string("key")));
        }

        #[test]
        fn test_map_delete() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("key"), JsValue::number(1.0));
            assert!(MapObject::delete(&map, &JsValue::string("key")));
            assert!(!MapObject::has(&map, &JsValue::string("key")));
            assert!(!MapObject::delete(&map, &JsValue::string("key")));
        }

        #[test]
        fn test_map_clear() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
            MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));
            MapObject::clear(&map);
            assert_eq!(MapObject::size(&map), 0);
        }

        #[test]
        fn test_map_insertion_order() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("c"), JsValue::number(3.0));
            MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
            MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

            let keys = MapObject::keys(&map);
            let key_strings: Vec<String> =
                keys.iter().map(|k| k.as_string().unwrap()).collect();
            assert_eq!(key_strings, vec!["c", "a", "b"]);
        }

        #[test]
        fn test_map_nan_key() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::number(f64::NAN), JsValue::string("nan"));

            // SameValueZero: NaN === NaN
            assert!(MapObject::has(&map, &JsValue::number(f64::NAN)));
            let value = MapObject::get(&map, &JsValue::number(f64::NAN)).unwrap();
            assert_eq!(value.as_string().unwrap(), "nan");
        }

        #[test]
        fn test_map_negative_zero_key() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::number(-0.0), JsValue::string("zero"));

            // SameValueZero: -0 === +0
            assert!(MapObject::has(&map, &JsValue::number(0.0)));
            let value = MapObject::get(&map, &JsValue::number(0.0)).unwrap();
            assert_eq!(value.as_string().unwrap(), "zero");
        }

        #[test]
        fn test_map_update_preserves_order() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
            MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));
            MapObject::set(&map, JsValue::string("a"), JsValue::number(3.0)); // Update

            let keys = MapObject::keys(&map);
            let key_strings: Vec<String> =
                keys.iter().map(|k| k.as_string().unwrap()).collect();
            assert_eq!(key_strings, vec!["a", "b"]); // Order preserved
            assert_eq!(MapObject::size(&map), 2); // Size unchanged
        }

        #[test]
        fn test_map_from_entries() {
            let entries = vec![
                (JsValue::string("a"), JsValue::number(1.0)),
                (JsValue::string("b"), JsValue::number(2.0)),
            ];
            let map = MapObject::from_entries(entries);
            assert_eq!(MapObject::size(&map), 2);
            assert!(MapObject::has(&map, &JsValue::string("a")));
            assert!(MapObject::has(&map, &JsValue::string("b")));
        }

        #[test]
        fn test_map_entries() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
            MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

            let entries = MapObject::entries(&map);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0.as_string().unwrap(), "a");
            assert_eq!(entries[0].1.as_number().unwrap(), 1.0);
        }

        #[test]
        fn test_map_for_each() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
            MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

            let mut sum = 0.0;
            MapObject::for_each(&map, |_k, v| {
                sum += v.as_number().unwrap();
                Ok(())
            })
            .unwrap();
            assert_eq!(sum, 3.0);
        }

        #[test]
        fn test_map_iter() {
            let map = MapObject::new();
            MapObject::set(&map, JsValue::number(1.0), JsValue::string("one"));
            MapObject::set(&map, JsValue::number(2.0), JsValue::string("two"));

            let mut iter = MapObject::iter(&map);
            let (k1, v1) = iter.next().unwrap();
            assert_eq!(k1.as_number().unwrap(), 1.0);
            assert_eq!(v1.as_string().unwrap(), "one");

            let (k2, v2) = iter.next().unwrap();
            assert_eq!(k2.as_number().unwrap(), 2.0);
            assert_eq!(v2.as_string().unwrap(), "two");

            assert!(iter.next().is_none());
        }
    }

    // Set unit tests
    mod set_tests {
        use super::*;

        #[test]
        fn test_set_new() {
            let set = SetObject::new();
            assert!(set.is_set());
            assert_eq!(SetObject::size(&set), 0);
        }

        #[test]
        fn test_set_add_has() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(42.0));
            assert!(SetObject::has(&set, &JsValue::number(42.0)));
            assert!(!SetObject::has(&set, &JsValue::number(43.0)));
        }

        #[test]
        fn test_set_delete() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            assert!(SetObject::delete(&set, &JsValue::number(1.0)));
            assert!(!SetObject::has(&set, &JsValue::number(1.0)));
            assert!(!SetObject::delete(&set, &JsValue::number(1.0)));
        }

        #[test]
        fn test_set_clear() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));
            SetObject::clear(&set);
            assert_eq!(SetObject::size(&set), 0);
        }

        #[test]
        fn test_set_insertion_order() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(3.0));
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));

            let values = SetObject::values(&set);
            let nums: Vec<f64> = values.iter().map(|v| v.as_number().unwrap()).collect();
            assert_eq!(nums, vec![3.0, 1.0, 2.0]);
        }

        #[test]
        fn test_set_deduplication() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(1.0)); // Duplicate
            SetObject::add(&set, JsValue::number(2.0));
            assert_eq!(SetObject::size(&set), 2);
        }

        #[test]
        fn test_set_nan_value() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(f64::NAN));

            // SameValueZero: NaN === NaN
            assert!(SetObject::has(&set, &JsValue::number(f64::NAN)));

            // Adding NaN again should not increase size
            SetObject::add(&set, JsValue::number(f64::NAN));
            assert_eq!(SetObject::size(&set), 1);
        }

        #[test]
        fn test_set_negative_zero() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(-0.0));

            // SameValueZero: -0 === +0
            assert!(SetObject::has(&set, &JsValue::number(0.0)));
            SetObject::add(&set, JsValue::number(0.0));
            assert_eq!(SetObject::size(&set), 1);
        }

        #[test]
        fn test_set_from_values() {
            let values = vec![
                JsValue::number(1.0),
                JsValue::number(2.0),
                JsValue::number(1.0), // Duplicate
            ];
            let set = SetObject::from_values(values);
            assert_eq!(SetObject::size(&set), 2);
        }

        #[test]
        fn test_set_keys_equals_values() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));

            let keys = SetObject::keys(&set);
            let values = SetObject::values(&set);
            assert_eq!(keys.len(), values.len());
        }

        #[test]
        fn test_set_entries() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));

            let entries = SetObject::entries(&set);
            assert_eq!(entries.len(), 2);
            // For Set, entries are [value, value] pairs
            assert!(entries[0].0.equals(&entries[0].1));
            assert!(entries[1].0.equals(&entries[1].1));
        }

        #[test]
        fn test_set_for_each() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));
            SetObject::add(&set, JsValue::number(3.0));

            let mut sum = 0.0;
            SetObject::for_each(&set, |v| {
                sum += v.as_number().unwrap();
                Ok(())
            })
            .unwrap();
            assert_eq!(sum, 6.0);
        }

        #[test]
        fn test_set_iter() {
            let set = SetObject::new();
            SetObject::add(&set, JsValue::number(1.0));
            SetObject::add(&set, JsValue::number(2.0));

            let mut iter = SetObject::iter(&set);
            assert_eq!(iter.next().unwrap().as_number().unwrap(), 1.0);
            assert_eq!(iter.next().unwrap().as_number().unwrap(), 2.0);
            assert!(iter.next().is_none());
        }
    }

    // WeakMap unit tests
    mod weak_map_tests {
        use super::*;

        #[test]
        fn test_weak_map_new() {
            let weak_map = WeakMapObject::new();
            assert!(weak_map.is_weak_map());
        }

        #[test]
        fn test_weak_map_set_get() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();
            key.set("id", JsValue::number(1.0));

            WeakMapObject::set(&weak_map, key.clone(), JsValue::string("value"))
                .unwrap();
            let value = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
            assert_eq!(value.as_string().unwrap(), "value");
        }

        #[test]
        fn test_weak_map_has() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();
            assert!(!WeakMapObject::has(&weak_map, &key).unwrap());

            WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).unwrap();
            assert!(WeakMapObject::has(&weak_map, &key).unwrap());
        }

        #[test]
        fn test_weak_map_delete() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();
            WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).unwrap();
            assert!(WeakMapObject::delete(&weak_map, &key).unwrap());
            assert!(!WeakMapObject::has(&weak_map, &key).unwrap());
            assert!(!WeakMapObject::delete(&weak_map, &key).unwrap());
        }

        #[test]
        fn test_weak_map_rejects_primitive_key() {
            let weak_map = WeakMapObject::new();

            // Number key should fail
            let result = WeakMapObject::set(&weak_map, JsValue::number(42.0), JsValue::string("value"));
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Invalid value used as weak map key"));

            // String key should fail
            let result = WeakMapObject::set(&weak_map, JsValue::string("key"), JsValue::string("value"));
            assert!(result.is_err());

            // Boolean key should fail
            let result = WeakMapObject::set(&weak_map, JsValue::boolean(true), JsValue::string("value"));
            assert!(result.is_err());

            // Null key should fail
            let result = WeakMapObject::set(&weak_map, JsValue::null(), JsValue::string("value"));
            assert!(result.is_err());

            // Undefined key should fail
            let result = WeakMapObject::set(&weak_map, JsValue::undefined(), JsValue::string("value"));
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_map_accepts_object_keys() {
            let weak_map = WeakMapObject::new();

            // Object key should work
            let obj_key = JsValue::object();
            assert!(WeakMapObject::set(&weak_map, obj_key.clone(), JsValue::number(1.0)).is_ok());

            // Array key should work
            let arr_key = JsValue::array();
            assert!(WeakMapObject::set(&weak_map, arr_key.clone(), JsValue::number(2.0)).is_ok());

            // Function key should work
            let func_key = JsValue::function(|_, _| Ok(JsValue::undefined()));
            assert!(WeakMapObject::set(&weak_map, func_key.clone(), JsValue::number(3.0)).is_ok());

            // Map key should work
            let map_key = JsValue::map();
            assert!(WeakMapObject::set(&weak_map, map_key.clone(), JsValue::number(4.0)).is_ok());

            // Set key should work
            let set_key = JsValue::set_collection();
            assert!(WeakMapObject::set(&weak_map, set_key.clone(), JsValue::number(5.0)).is_ok());

            // All keys should be retrievable
            assert!(WeakMapObject::has(&weak_map, &obj_key).unwrap());
            assert!(WeakMapObject::has(&weak_map, &arr_key).unwrap());
            assert!(WeakMapObject::has(&weak_map, &func_key).unwrap());
            assert!(WeakMapObject::has(&weak_map, &map_key).unwrap());
            assert!(WeakMapObject::has(&weak_map, &set_key).unwrap());
        }

        #[test]
        fn test_weak_map_object_identity() {
            let weak_map = WeakMapObject::new();
            let obj1 = JsValue::object();
            obj1.set("id", JsValue::number(1.0));

            WeakMapObject::set(&weak_map, obj1.clone(), JsValue::string("obj1")).unwrap();

            // Same reference should match
            assert!(WeakMapObject::has(&weak_map, &obj1).unwrap());

            // Different object with same properties should NOT match
            let obj2 = JsValue::object();
            obj2.set("id", JsValue::number(1.0));
            assert!(!WeakMapObject::has(&weak_map, &obj2).unwrap());
        }

        #[test]
        fn test_weak_map_update_value() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();

            WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).unwrap();
            let value = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
            assert_eq!(value.as_number().unwrap(), 1.0);

            // Update value for same key
            WeakMapObject::set(&weak_map, key.clone(), JsValue::number(2.0)).unwrap();
            let value = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
            assert_eq!(value.as_number().unwrap(), 2.0);
        }

        #[test]
        fn test_weak_map_get_missing_returns_none() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();
            let result = WeakMapObject::get(&weak_map, &key).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn test_weak_map_from_entries() {
            let key1 = JsValue::object();
            let key2 = JsValue::array();
            let entries = vec![
                (key1.clone(), JsValue::number(1.0)),
                (key2.clone(), JsValue::number(2.0)),
            ];
            let weak_map = WeakMapObject::from_entries(entries).unwrap();

            assert!(WeakMapObject::has(&weak_map, &key1).unwrap());
            assert!(WeakMapObject::has(&weak_map, &key2).unwrap());
            assert_eq!(
                WeakMapObject::get(&weak_map, &key1).unwrap().unwrap().as_number().unwrap(),
                1.0
            );
            assert_eq!(
                WeakMapObject::get(&weak_map, &key2).unwrap().unwrap().as_number().unwrap(),
                2.0
            );
        }

        #[test]
        fn test_weak_map_from_entries_rejects_primitive() {
            let entries = vec![
                (JsValue::string("invalid"), JsValue::number(1.0)),
            ];
            let result = WeakMapObject::from_entries(entries);
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_map_set_returns_weak_map() {
            let weak_map = WeakMapObject::new();
            let key = JsValue::object();
            let result = WeakMapObject::set(&weak_map, key, JsValue::number(1.0)).unwrap();
            assert!(result.is_weak_map());
        }

        #[test]
        fn test_weak_map_multiple_keys() {
            let weak_map = WeakMapObject::new();
            let keys: Vec<JsValue> = (0..10).map(|_| JsValue::object()).collect();

            for (i, key) in keys.iter().enumerate() {
                WeakMapObject::set(&weak_map, key.clone(), JsValue::number(i as f64)).unwrap();
            }

            for (i, key) in keys.iter().enumerate() {
                let value = WeakMapObject::get(&weak_map, key).unwrap().unwrap();
                assert_eq!(value.as_number().unwrap(), i as f64);
            }
        }

        #[test]
        fn test_weak_map_has_rejects_primitive() {
            let weak_map = WeakMapObject::new();
            let result = WeakMapObject::has(&weak_map, &JsValue::number(42.0));
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_map_delete_rejects_primitive() {
            let weak_map = WeakMapObject::new();
            let result = WeakMapObject::delete(&weak_map, &JsValue::string("key"));
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_map_get_rejects_primitive() {
            let weak_map = WeakMapObject::new();
            let result = WeakMapObject::get(&weak_map, &JsValue::boolean(true));
            assert!(result.is_err());
        }
    }

    // WeakSet unit tests
    mod weak_set_tests {
        use super::*;

        #[test]
        fn test_weak_set_new() {
            let weak_set = WeakSetObject::new();
            assert!(weak_set.is_weak_set());
        }

        #[test]
        fn test_weak_set_add_has() {
            let weak_set = WeakSetObject::new();
            let value = JsValue::object();

            WeakSetObject::add(&weak_set, value.clone()).unwrap();
            assert!(WeakSetObject::has(&weak_set, &value).unwrap());
        }

        #[test]
        fn test_weak_set_delete() {
            let weak_set = WeakSetObject::new();
            let value = JsValue::object();

            WeakSetObject::add(&weak_set, value.clone()).unwrap();
            assert!(WeakSetObject::delete(&weak_set, &value).unwrap());
            assert!(!WeakSetObject::has(&weak_set, &value).unwrap());
            assert!(!WeakSetObject::delete(&weak_set, &value).unwrap());
        }

        #[test]
        fn test_weak_set_rejects_primitive_value() {
            let weak_set = WeakSetObject::new();

            // Number value should fail
            let result = WeakSetObject::add(&weak_set, JsValue::number(42.0));
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("Invalid value used in weak set"));

            // String value should fail
            let result = WeakSetObject::add(&weak_set, JsValue::string("value"));
            assert!(result.is_err());

            // Boolean value should fail
            let result = WeakSetObject::add(&weak_set, JsValue::boolean(true));
            assert!(result.is_err());

            // Null value should fail
            let result = WeakSetObject::add(&weak_set, JsValue::null());
            assert!(result.is_err());

            // Undefined value should fail
            let result = WeakSetObject::add(&weak_set, JsValue::undefined());
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_set_accepts_object_values() {
            let weak_set = WeakSetObject::new();

            // Object should work
            let obj = JsValue::object();
            assert!(WeakSetObject::add(&weak_set, obj.clone()).is_ok());

            // Array should work
            let arr = JsValue::array();
            assert!(WeakSetObject::add(&weak_set, arr.clone()).is_ok());

            // Function should work
            let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
            assert!(WeakSetObject::add(&weak_set, func.clone()).is_ok());

            // Map should work
            let map = JsValue::map();
            assert!(WeakSetObject::add(&weak_set, map.clone()).is_ok());

            // Set should work
            let set = JsValue::set_collection();
            assert!(WeakSetObject::add(&weak_set, set.clone()).is_ok());

            // All values should be present
            assert!(WeakSetObject::has(&weak_set, &obj).unwrap());
            assert!(WeakSetObject::has(&weak_set, &arr).unwrap());
            assert!(WeakSetObject::has(&weak_set, &func).unwrap());
            assert!(WeakSetObject::has(&weak_set, &map).unwrap());
            assert!(WeakSetObject::has(&weak_set, &set).unwrap());
        }

        #[test]
        fn test_weak_set_object_identity() {
            let weak_set = WeakSetObject::new();
            let obj1 = JsValue::object();
            obj1.set("id", JsValue::number(1.0));

            WeakSetObject::add(&weak_set, obj1.clone()).unwrap();

            // Same reference should match
            assert!(WeakSetObject::has(&weak_set, &obj1).unwrap());

            // Different object with same properties should NOT match
            let obj2 = JsValue::object();
            obj2.set("id", JsValue::number(1.0));
            assert!(!WeakSetObject::has(&weak_set, &obj2).unwrap());
        }

        #[test]
        fn test_weak_set_deduplication() {
            let weak_set = WeakSetObject::new();
            let obj = JsValue::object();

            // Adding same object twice should not create duplicates
            WeakSetObject::add(&weak_set, obj.clone()).unwrap();
            WeakSetObject::add(&weak_set, obj.clone()).unwrap();

            // Can only check by has/delete since no size property
            assert!(WeakSetObject::has(&weak_set, &obj).unwrap());
            assert!(WeakSetObject::delete(&weak_set, &obj).unwrap());
            assert!(!WeakSetObject::has(&weak_set, &obj).unwrap());
        }

        #[test]
        fn test_weak_set_from_values() {
            let obj1 = JsValue::object();
            let obj2 = JsValue::array();
            let values = vec![obj1.clone(), obj2.clone()];
            let weak_set = WeakSetObject::from_values(values).unwrap();

            assert!(WeakSetObject::has(&weak_set, &obj1).unwrap());
            assert!(WeakSetObject::has(&weak_set, &obj2).unwrap());
        }

        #[test]
        fn test_weak_set_from_values_rejects_primitive() {
            let values = vec![JsValue::string("invalid")];
            let result = WeakSetObject::from_values(values);
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_set_add_returns_weak_set() {
            let weak_set = WeakSetObject::new();
            let obj = JsValue::object();
            let result = WeakSetObject::add(&weak_set, obj).unwrap();
            assert!(result.is_weak_set());
        }

        #[test]
        fn test_weak_set_multiple_values() {
            let weak_set = WeakSetObject::new();
            let values: Vec<JsValue> = (0..10).map(|_| JsValue::object()).collect();

            for value in &values {
                WeakSetObject::add(&weak_set, value.clone()).unwrap();
            }

            for value in &values {
                assert!(WeakSetObject::has(&weak_set, value).unwrap());
            }
        }

        #[test]
        fn test_weak_set_has_rejects_primitive() {
            let weak_set = WeakSetObject::new();
            let result = WeakSetObject::has(&weak_set, &JsValue::number(42.0));
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_set_delete_rejects_primitive() {
            let weak_set = WeakSetObject::new();
            let result = WeakSetObject::delete(&weak_set, &JsValue::string("value"));
            assert!(result.is_err());
        }

        #[test]
        fn test_weak_set_chaining() {
            let weak_set = WeakSetObject::new();
            let obj1 = JsValue::object();
            let obj2 = JsValue::array();

            // Chaining: weak_set.add(obj1).add(obj2)
            let ws = WeakSetObject::add(&weak_set, obj1.clone()).unwrap();
            let _ws = WeakSetObject::add(&ws, obj2.clone()).unwrap();

            assert!(WeakSetObject::has(&weak_set, &obj1).unwrap());
            assert!(WeakSetObject::has(&weak_set, &obj2).unwrap());
        }
    }
}
