//! Contract tests for WeakMap and WeakSet collections
//!
//! These tests verify ES2024 compliance for WeakMap and WeakSet built-in objects.

use builtins::{JsValue, WeakMapObject, WeakSetObject};

mod weak_map_tests {
    use super::*;

    #[test]
    fn test_weak_map_creation_empty() {
        let weak_map = WeakMapObject::new();
        assert!(weak_map.is_weak_map());
    }

    #[test]
    fn test_weak_map_creation_from_iterable() {
        let key1 = JsValue::object();
        let key2 = JsValue::array();
        let entries = vec![
            (key1.clone(), JsValue::number(1.0)),
            (key2.clone(), JsValue::number(2.0)),
        ];
        let weak_map = WeakMapObject::from_entries(entries).unwrap();
        assert!(WeakMapObject::has(&weak_map, &key1).unwrap());
        assert!(WeakMapObject::has(&weak_map, &key2).unwrap());
    }

    #[test]
    fn test_weak_map_set_and_get() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();
        WeakMapObject::set(&weak_map, key.clone(), JsValue::number(42.0)).unwrap();
        let value = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert_eq!(value.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_weak_map_has() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();
        WeakMapObject::set(&weak_map, key.clone(), JsValue::number(42.0)).unwrap();
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());

        let other_key = JsValue::object();
        assert!(!WeakMapObject::has(&weak_map, &other_key).unwrap());
    }

    #[test]
    fn test_weak_map_delete() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();
        WeakMapObject::set(&weak_map, key.clone(), JsValue::number(42.0)).unwrap();
        assert!(WeakMapObject::delete(&weak_map, &key).unwrap());
        assert!(!WeakMapObject::has(&weak_map, &key).unwrap());
        assert!(!WeakMapObject::delete(&weak_map, &key).unwrap());
    }

    // Key type restrictions - only objects allowed
    #[test]
    fn test_weak_map_rejects_number_key() {
        let weak_map = WeakMapObject::new();
        let result = WeakMapObject::set(&weak_map, JsValue::number(42.0), JsValue::number(1.0));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Invalid value used as weak map key"));
    }

    #[test]
    fn test_weak_map_rejects_string_key() {
        let weak_map = WeakMapObject::new();
        let result = WeakMapObject::set(&weak_map, JsValue::string("key"), JsValue::number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_map_rejects_boolean_key() {
        let weak_map = WeakMapObject::new();
        let result = WeakMapObject::set(&weak_map, JsValue::boolean(true), JsValue::number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_map_rejects_null_key() {
        let weak_map = WeakMapObject::new();
        let result = WeakMapObject::set(&weak_map, JsValue::null(), JsValue::number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_map_rejects_undefined_key() {
        let weak_map = WeakMapObject::new();
        let result = WeakMapObject::set(&weak_map, JsValue::undefined(), JsValue::number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_map_rejects_symbol_key() {
        let weak_map = WeakMapObject::new();
        let sym = builtins::SymbolConstructor::new(Some("test".to_string()));
        let result = WeakMapObject::set(&weak_map, JsValue::symbol(sym), JsValue::number(1.0));
        assert!(result.is_err());
    }

    // Object key types that should work
    #[test]
    fn test_weak_map_accepts_object_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_array_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::array();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_function_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::function(|_, _| Ok(JsValue::undefined()));
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_map_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::map();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_set_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::set_collection();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_weak_map_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::weak_map();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    #[test]
    fn test_weak_map_accepts_weak_set_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::weak_set();
        assert!(WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).is_ok());
        assert!(WeakMapObject::has(&weak_map, &key).unwrap());
    }

    // Object identity semantics
    #[test]
    fn test_weak_map_uses_object_identity() {
        let weak_map = WeakMapObject::new();
        let obj1 = JsValue::object();
        obj1.set("id", JsValue::number(1.0));

        WeakMapObject::set(&weak_map, obj1.clone(), JsValue::string("obj1")).unwrap();

        // Same object reference should match
        assert!(WeakMapObject::has(&weak_map, &obj1).unwrap());

        // Different object with same properties should NOT match
        let obj2 = JsValue::object();
        obj2.set("id", JsValue::number(1.0));
        assert!(!WeakMapObject::has(&weak_map, &obj2).unwrap());
    }

    #[test]
    fn test_weak_map_update_existing_key() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();

        WeakMapObject::set(&weak_map, key.clone(), JsValue::number(1.0)).unwrap();
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
    fn test_weak_map_set_returns_weak_map() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();
        let result = WeakMapObject::set(&weak_map, key, JsValue::number(1.0)).unwrap();
        // set() should return the weak_map for chaining
        assert!(result.is_weak_map());
    }

    #[test]
    fn test_weak_map_chaining() {
        let weak_map = WeakMapObject::new();
        let key1 = JsValue::object();
        let key2 = JsValue::array();

        // Chaining: weak_map.set(key1, 1).set(key2, 2)
        let wm = WeakMapObject::set(&weak_map, key1.clone(), JsValue::number(1.0)).unwrap();
        let _wm = WeakMapObject::set(&wm, key2.clone(), JsValue::number(2.0)).unwrap();

        assert!(WeakMapObject::has(&weak_map, &key1).unwrap());
        assert!(WeakMapObject::has(&weak_map, &key2).unwrap());
    }

    // Error cases for operations with primitives
    #[test]
    fn test_weak_map_has_rejects_primitive() {
        let weak_map = WeakMapObject::new();
        assert!(WeakMapObject::has(&weak_map, &JsValue::number(42.0)).is_err());
        assert!(WeakMapObject::has(&weak_map, &JsValue::string("key")).is_err());
        assert!(WeakMapObject::has(&weak_map, &JsValue::boolean(true)).is_err());
        assert!(WeakMapObject::has(&weak_map, &JsValue::null()).is_err());
        assert!(WeakMapObject::has(&weak_map, &JsValue::undefined()).is_err());
    }

    #[test]
    fn test_weak_map_get_rejects_primitive() {
        let weak_map = WeakMapObject::new();
        assert!(WeakMapObject::get(&weak_map, &JsValue::number(42.0)).is_err());
        assert!(WeakMapObject::get(&weak_map, &JsValue::string("key")).is_err());
        assert!(WeakMapObject::get(&weak_map, &JsValue::boolean(true)).is_err());
        assert!(WeakMapObject::get(&weak_map, &JsValue::null()).is_err());
        assert!(WeakMapObject::get(&weak_map, &JsValue::undefined()).is_err());
    }

    #[test]
    fn test_weak_map_delete_rejects_primitive() {
        let weak_map = WeakMapObject::new();
        assert!(WeakMapObject::delete(&weak_map, &JsValue::number(42.0)).is_err());
        assert!(WeakMapObject::delete(&weak_map, &JsValue::string("key")).is_err());
        assert!(WeakMapObject::delete(&weak_map, &JsValue::boolean(true)).is_err());
        assert!(WeakMapObject::delete(&weak_map, &JsValue::null()).is_err());
        assert!(WeakMapObject::delete(&weak_map, &JsValue::undefined()).is_err());
    }

    // Value can be any type (primitives allowed as values, just not keys)
    #[test]
    fn test_weak_map_value_can_be_primitive() {
        let weak_map = WeakMapObject::new();
        let key = JsValue::object();

        // Number value
        WeakMapObject::set(&weak_map, key.clone(), JsValue::number(42.0)).unwrap();
        let val = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert_eq!(val.as_number().unwrap(), 42.0);

        // String value
        WeakMapObject::set(&weak_map, key.clone(), JsValue::string("test")).unwrap();
        let val = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert_eq!(val.as_string().unwrap(), "test");

        // Boolean value
        WeakMapObject::set(&weak_map, key.clone(), JsValue::boolean(true)).unwrap();
        let val = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert_eq!(val.as_boolean().unwrap(), true);

        // Null value
        WeakMapObject::set(&weak_map, key.clone(), JsValue::null()).unwrap();
        let val = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert!(val.is_null());

        // Undefined value
        WeakMapObject::set(&weak_map, key.clone(), JsValue::undefined()).unwrap();
        let val = WeakMapObject::get(&weak_map, &key).unwrap().unwrap();
        assert!(val.is_undefined());
    }

    #[test]
    fn test_weak_map_to_string() {
        let weak_map = WeakMapObject::new();
        assert_eq!(weak_map.to_js_string(), "[object WeakMap]");
    }

    #[test]
    fn test_weak_map_typeof() {
        let weak_map = WeakMapObject::new();
        assert_eq!(weak_map.type_of(), "object");
    }
}

mod weak_set_tests {
    use super::*;

    #[test]
    fn test_weak_set_creation_empty() {
        let weak_set = WeakSetObject::new();
        assert!(weak_set.is_weak_set());
    }

    #[test]
    fn test_weak_set_creation_from_iterable() {
        let obj1 = JsValue::object();
        let obj2 = JsValue::array();
        let values = vec![obj1.clone(), obj2.clone()];
        let weak_set = WeakSetObject::from_values(values).unwrap();
        assert!(WeakSetObject::has(&weak_set, &obj1).unwrap());
        assert!(WeakSetObject::has(&weak_set, &obj2).unwrap());
    }

    #[test]
    fn test_weak_set_add_and_has() {
        let weak_set = WeakSetObject::new();
        let obj = JsValue::object();
        WeakSetObject::add(&weak_set, obj.clone()).unwrap();
        assert!(WeakSetObject::has(&weak_set, &obj).unwrap());

        let other_obj = JsValue::object();
        assert!(!WeakSetObject::has(&weak_set, &other_obj).unwrap());
    }

    #[test]
    fn test_weak_set_add_returns_weak_set() {
        let weak_set = WeakSetObject::new();
        let obj = JsValue::object();
        let result = WeakSetObject::add(&weak_set, obj).unwrap();
        // add() should return the weak_set for chaining
        assert!(result.is_weak_set());
    }

    #[test]
    fn test_weak_set_delete() {
        let weak_set = WeakSetObject::new();
        let obj = JsValue::object();
        WeakSetObject::add(&weak_set, obj.clone()).unwrap();
        assert!(WeakSetObject::delete(&weak_set, &obj).unwrap());
        assert!(!WeakSetObject::has(&weak_set, &obj).unwrap());
        assert!(!WeakSetObject::delete(&weak_set, &obj).unwrap());
    }

    // Value type restrictions - only objects allowed
    #[test]
    fn test_weak_set_rejects_number_value() {
        let weak_set = WeakSetObject::new();
        let result = WeakSetObject::add(&weak_set, JsValue::number(42.0));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Invalid value used in weak set"));
    }

    #[test]
    fn test_weak_set_rejects_string_value() {
        let weak_set = WeakSetObject::new();
        let result = WeakSetObject::add(&weak_set, JsValue::string("value"));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_set_rejects_boolean_value() {
        let weak_set = WeakSetObject::new();
        let result = WeakSetObject::add(&weak_set, JsValue::boolean(true));
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_set_rejects_null_value() {
        let weak_set = WeakSetObject::new();
        let result = WeakSetObject::add(&weak_set, JsValue::null());
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_set_rejects_undefined_value() {
        let weak_set = WeakSetObject::new();
        let result = WeakSetObject::add(&weak_set, JsValue::undefined());
        assert!(result.is_err());
    }

    #[test]
    fn test_weak_set_rejects_symbol_value() {
        let weak_set = WeakSetObject::new();
        let sym = builtins::SymbolConstructor::new(Some("test".to_string()));
        let result = WeakSetObject::add(&weak_set, JsValue::symbol(sym));
        assert!(result.is_err());
    }

    // Object value types that should work
    #[test]
    fn test_weak_set_accepts_object_value() {
        let weak_set = WeakSetObject::new();
        let obj = JsValue::object();
        assert!(WeakSetObject::add(&weak_set, obj.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &obj).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_array_value() {
        let weak_set = WeakSetObject::new();
        let arr = JsValue::array();
        assert!(WeakSetObject::add(&weak_set, arr.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &arr).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_function_value() {
        let weak_set = WeakSetObject::new();
        let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
        assert!(WeakSetObject::add(&weak_set, func.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &func).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_map_value() {
        let weak_set = WeakSetObject::new();
        let map = JsValue::map();
        assert!(WeakSetObject::add(&weak_set, map.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &map).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_set_value() {
        let weak_set = WeakSetObject::new();
        let set = JsValue::set_collection();
        assert!(WeakSetObject::add(&weak_set, set.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &set).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_weak_map_value() {
        let weak_set = WeakSetObject::new();
        let wm = JsValue::weak_map();
        assert!(WeakSetObject::add(&weak_set, wm.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &wm).unwrap());
    }

    #[test]
    fn test_weak_set_accepts_weak_set_value() {
        let weak_set = WeakSetObject::new();
        let ws = JsValue::weak_set();
        assert!(WeakSetObject::add(&weak_set, ws.clone()).is_ok());
        assert!(WeakSetObject::has(&weak_set, &ws).unwrap());
    }

    // Object identity semantics
    #[test]
    fn test_weak_set_uses_object_identity() {
        let weak_set = WeakSetObject::new();
        let obj1 = JsValue::object();
        obj1.set("id", JsValue::number(1.0));

        WeakSetObject::add(&weak_set, obj1.clone()).unwrap();

        // Same object reference should match
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

        // Verify only one entry by deleting
        assert!(WeakSetObject::has(&weak_set, &obj).unwrap());
        assert!(WeakSetObject::delete(&weak_set, &obj).unwrap());
        assert!(!WeakSetObject::has(&weak_set, &obj).unwrap());
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

    // Error cases for operations with primitives
    #[test]
    fn test_weak_set_has_rejects_primitive() {
        let weak_set = WeakSetObject::new();
        assert!(WeakSetObject::has(&weak_set, &JsValue::number(42.0)).is_err());
        assert!(WeakSetObject::has(&weak_set, &JsValue::string("value")).is_err());
        assert!(WeakSetObject::has(&weak_set, &JsValue::boolean(true)).is_err());
        assert!(WeakSetObject::has(&weak_set, &JsValue::null()).is_err());
        assert!(WeakSetObject::has(&weak_set, &JsValue::undefined()).is_err());
    }

    #[test]
    fn test_weak_set_delete_rejects_primitive() {
        let weak_set = WeakSetObject::new();
        assert!(WeakSetObject::delete(&weak_set, &JsValue::number(42.0)).is_err());
        assert!(WeakSetObject::delete(&weak_set, &JsValue::string("value")).is_err());
        assert!(WeakSetObject::delete(&weak_set, &JsValue::boolean(true)).is_err());
        assert!(WeakSetObject::delete(&weak_set, &JsValue::null()).is_err());
        assert!(WeakSetObject::delete(&weak_set, &JsValue::undefined()).is_err());
    }

    #[test]
    fn test_weak_set_to_string() {
        let weak_set = WeakSetObject::new();
        assert_eq!(weak_set.to_js_string(), "[object WeakSet]");
    }

    #[test]
    fn test_weak_set_typeof() {
        let weak_set = WeakSetObject::new();
        assert_eq!(weak_set.type_of(), "object");
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_weak_map_with_various_object_keys() {
        let weak_map = WeakMapObject::new();

        let obj = JsValue::object();
        let arr = JsValue::array();
        let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let map = JsValue::map();
        let set = JsValue::set_collection();

        WeakMapObject::set(&weak_map, obj.clone(), JsValue::string("object")).unwrap();
        WeakMapObject::set(&weak_map, arr.clone(), JsValue::string("array")).unwrap();
        WeakMapObject::set(&weak_map, func.clone(), JsValue::string("function")).unwrap();
        WeakMapObject::set(&weak_map, map.clone(), JsValue::string("map")).unwrap();
        WeakMapObject::set(&weak_map, set.clone(), JsValue::string("set")).unwrap();

        assert_eq!(
            WeakMapObject::get(&weak_map, &obj)
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "object"
        );
        assert_eq!(
            WeakMapObject::get(&weak_map, &arr)
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "array"
        );
        assert_eq!(
            WeakMapObject::get(&weak_map, &func)
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "function"
        );
        assert_eq!(
            WeakMapObject::get(&weak_map, &map)
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "map"
        );
        assert_eq!(
            WeakMapObject::get(&weak_map, &set)
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "set"
        );
    }

    #[test]
    fn test_weak_set_with_various_object_values() {
        let weak_set = WeakSetObject::new();

        let obj = JsValue::object();
        let arr = JsValue::array();
        let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let map = JsValue::map();
        let set = JsValue::set_collection();

        WeakSetObject::add(&weak_set, obj.clone()).unwrap();
        WeakSetObject::add(&weak_set, arr.clone()).unwrap();
        WeakSetObject::add(&weak_set, func.clone()).unwrap();
        WeakSetObject::add(&weak_set, map.clone()).unwrap();
        WeakSetObject::add(&weak_set, set.clone()).unwrap();

        assert!(WeakSetObject::has(&weak_set, &obj).unwrap());
        assert!(WeakSetObject::has(&weak_set, &arr).unwrap());
        assert!(WeakSetObject::has(&weak_set, &func).unwrap());
        assert!(WeakSetObject::has(&weak_set, &map).unwrap());
        assert!(WeakSetObject::has(&weak_set, &set).unwrap());
    }

    #[test]
    fn test_weak_map_and_weak_set_nesting() {
        // WeakMap can use WeakSet as key
        let weak_map = WeakMapObject::new();
        let weak_set = WeakSetObject::new();

        WeakMapObject::set(&weak_map, weak_set.clone(), JsValue::string("nested")).unwrap();
        assert!(WeakMapObject::has(&weak_map, &weak_set).unwrap());

        // WeakSet can contain WeakMap
        let weak_set2 = WeakSetObject::new();
        let weak_map2 = WeakMapObject::new();

        WeakSetObject::add(&weak_set2, weak_map2.clone()).unwrap();
        assert!(WeakSetObject::has(&weak_set2, &weak_map2).unwrap());
    }

    #[test]
    fn test_weak_map_delete_all_keys() {
        let weak_map = WeakMapObject::new();
        let keys: Vec<JsValue> = (0..5).map(|_| JsValue::object()).collect();

        for (i, key) in keys.iter().enumerate() {
            WeakMapObject::set(&weak_map, key.clone(), JsValue::number(i as f64)).unwrap();
        }

        // Delete all keys
        for key in &keys {
            assert!(WeakMapObject::delete(&weak_map, key).unwrap());
        }

        // Verify all deleted
        for key in &keys {
            assert!(!WeakMapObject::has(&weak_map, key).unwrap());
        }
    }

    #[test]
    fn test_weak_set_delete_all_values() {
        let weak_set = WeakSetObject::new();
        let values: Vec<JsValue> = (0..5).map(|_| JsValue::object()).collect();

        for value in &values {
            WeakSetObject::add(&weak_set, value.clone()).unwrap();
        }

        // Delete all values
        for value in &values {
            assert!(WeakSetObject::delete(&weak_set, value).unwrap());
        }

        // Verify all deleted
        for value in &values {
            assert!(!WeakSetObject::has(&weak_set, value).unwrap());
        }
    }
}
