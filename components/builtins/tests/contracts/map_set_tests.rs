//! Contract tests for Map and Set collections
//!
//! These tests verify ES2024 compliance for Map and Set built-in objects.

use builtins::{JsValue, MapObject, SetObject};

mod map_tests {
    use super::*;

    #[test]
    fn test_map_creation_empty() {
        let map = MapObject::new();
        assert_eq!(MapObject::size(&map), 0);
    }

    #[test]
    fn test_map_creation_from_iterable() {
        let entries = vec![
            (JsValue::string("a"), JsValue::number(1.0)),
            (JsValue::string("b"), JsValue::number(2.0)),
        ];
        let map = MapObject::from_entries(entries);
        assert_eq!(MapObject::size(&map), 2);
    }

    #[test]
    fn test_map_set_and_get() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("key"), JsValue::number(42.0));
        let value = MapObject::get(&map, &JsValue::string("key")).unwrap();
        assert_eq!(value.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_map_has() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("key"), JsValue::number(42.0));
        assert!(MapObject::has(&map, &JsValue::string("key")));
        assert!(!MapObject::has(&map, &JsValue::string("missing")));
    }

    #[test]
    fn test_map_delete() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("key"), JsValue::number(42.0));
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
    fn test_map_size() {
        let map = MapObject::new();
        assert_eq!(MapObject::size(&map), 0);
        MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        assert_eq!(MapObject::size(&map), 1);
        MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));
        assert_eq!(MapObject::size(&map), 2);
        MapObject::set(&map, JsValue::string("a"), JsValue::number(3.0)); // Update existing
        assert_eq!(MapObject::size(&map), 2); // Size unchanged
    }

    #[test]
    fn test_map_preserves_insertion_order() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("c"), JsValue::number(3.0));
        MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

        let keys = MapObject::keys(&map);
        let key_strings: Vec<String> = keys.iter().map(|k| k.as_string().unwrap()).collect();
        assert_eq!(key_strings, vec!["c", "a", "b"]);
    }

    #[test]
    fn test_map_values() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));
        MapObject::set(&map, JsValue::string("c"), JsValue::number(3.0));

        let values = MapObject::values(&map);
        let nums: Vec<f64> = values.iter().map(|v| v.as_number().unwrap()).collect();
        assert_eq!(nums, vec![1.0, 2.0, 3.0]);
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
        assert_eq!(entries[1].0.as_string().unwrap(), "b");
        assert_eq!(entries[1].1.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_map_for_each() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

        let mut sum = 0.0;
        MapObject::for_each(&map, |_key, value| {
            sum += value.as_number().unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(sum, 3.0);
    }

    // Edge cases for SameValueZero semantics
    #[test]
    fn test_map_nan_key() {
        let map = MapObject::new();
        let nan_key = JsValue::number(f64::NAN);
        MapObject::set(&map, nan_key.clone(), JsValue::string("NaN value"));

        // NaN should be treated as equal to NaN in Map (SameValueZero)
        assert!(MapObject::has(&map, &JsValue::number(f64::NAN)));
        let value = MapObject::get(&map, &JsValue::number(f64::NAN)).unwrap();
        assert_eq!(value.as_string().unwrap(), "NaN value");
    }

    #[test]
    fn test_map_negative_zero_key() {
        let map = MapObject::new();
        let neg_zero = JsValue::number(-0.0);
        MapObject::set(&map, neg_zero, JsValue::string("negative zero"));

        // -0 and +0 should be treated as the same key (SameValueZero)
        assert!(MapObject::has(&map, &JsValue::number(0.0)));
        let value = MapObject::get(&map, &JsValue::number(0.0)).unwrap();
        assert_eq!(value.as_string().unwrap(), "negative zero");
    }

    #[test]
    fn test_map_object_key() {
        let map = MapObject::new();
        let obj_key = JsValue::object();
        obj_key.set("id", JsValue::number(1.0));

        MapObject::set(&map, obj_key.clone(), JsValue::string("object value"));

        // Same object reference should match
        assert!(MapObject::has(&map, &obj_key));

        // Different object with same properties should NOT match
        let other_obj = JsValue::object();
        other_obj.set("id", JsValue::number(1.0));
        assert!(!MapObject::has(&map, &other_obj));
    }

    #[test]
    fn test_map_undefined_and_null_keys() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::undefined(), JsValue::string("undefined"));
        MapObject::set(&map, JsValue::null(), JsValue::string("null"));

        assert!(MapObject::has(&map, &JsValue::undefined()));
        assert!(MapObject::has(&map, &JsValue::null()));
        assert_eq!(MapObject::size(&map), 2);
    }

    #[test]
    fn test_map_update_existing_key() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("key"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::string("key"), JsValue::number(2.0));

        assert_eq!(MapObject::size(&map), 1);
        let value = MapObject::get(&map, &JsValue::string("key")).unwrap();
        assert_eq!(value.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_map_get_missing_returns_undefined() {
        let map = MapObject::new();
        let result = MapObject::get(&map, &JsValue::string("missing"));
        assert!(result.is_none());
    }

    #[test]
    fn test_map_iterator() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));

        let mut iter = MapObject::iter(&map);
        let (k1, v1) = iter.next().unwrap();
        assert_eq!(k1.as_string().unwrap(), "a");
        assert_eq!(v1.as_number().unwrap(), 1.0);

        let (k2, v2) = iter.next().unwrap();
        assert_eq!(k2.as_string().unwrap(), "b");
        assert_eq!(v2.as_number().unwrap(), 2.0);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_map_set_returns_map() {
        let map = MapObject::new();
        let result = MapObject::set(&map, JsValue::string("key"), JsValue::number(1.0));
        // set() should return the map for chaining
        assert!(result.is_map());
    }
}

mod set_tests {
    use super::*;

    #[test]
    fn test_set_creation_empty() {
        let set = SetObject::new();
        assert_eq!(SetObject::size(&set), 0);
    }

    #[test]
    fn test_set_creation_from_iterable() {
        let values = vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ];
        let set = SetObject::from_values(values);
        assert_eq!(SetObject::size(&set), 3);
    }

    #[test]
    fn test_set_add_and_has() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(42.0));
        assert!(SetObject::has(&set, &JsValue::number(42.0)));
        assert!(!SetObject::has(&set, &JsValue::number(43.0)));
    }

    #[test]
    fn test_set_add_returns_set() {
        let set = SetObject::new();
        let result = SetObject::add(&set, JsValue::number(1.0));
        // add() should return the set for chaining
        assert!(result.is_set());
    }

    #[test]
    fn test_set_delete() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(42.0));
        assert!(SetObject::delete(&set, &JsValue::number(42.0)));
        assert!(!SetObject::has(&set, &JsValue::number(42.0)));
        assert!(!SetObject::delete(&set, &JsValue::number(42.0)));
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
    fn test_set_size() {
        let set = SetObject::new();
        assert_eq!(SetObject::size(&set), 0);
        SetObject::add(&set, JsValue::number(1.0));
        assert_eq!(SetObject::size(&set), 1);
        SetObject::add(&set, JsValue::number(2.0));
        assert_eq!(SetObject::size(&set), 2);
        SetObject::add(&set, JsValue::number(1.0)); // Duplicate
        assert_eq!(SetObject::size(&set), 2); // Size unchanged
    }

    #[test]
    fn test_set_preserves_insertion_order() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(3.0));
        SetObject::add(&set, JsValue::number(1.0));
        SetObject::add(&set, JsValue::number(2.0));

        let values = SetObject::values(&set);
        let nums: Vec<f64> = values.iter().map(|v| v.as_number().unwrap()).collect();
        assert_eq!(nums, vec![3.0, 1.0, 2.0]);
    }

    #[test]
    fn test_set_keys_is_alias_for_values() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(1.0));
        SetObject::add(&set, JsValue::number(2.0));

        let keys = SetObject::keys(&set);
        let values = SetObject::values(&set);

        assert_eq!(keys.len(), values.len());
        for i in 0..keys.len() {
            assert!(keys[i].equals(&values[i]));
        }
    }

    #[test]
    fn test_set_entries() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(1.0));
        SetObject::add(&set, JsValue::number(2.0));

        let entries = SetObject::entries(&set);
        // Set entries are [value, value] pairs
        assert_eq!(entries.len(), 2);
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
        SetObject::for_each(&set, |value| {
            sum += value.as_number().unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(sum, 6.0);
    }

    // Edge cases for SameValueZero semantics
    #[test]
    fn test_set_nan_value() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(f64::NAN));

        // NaN should be treated as equal to NaN in Set (SameValueZero)
        assert!(SetObject::has(&set, &JsValue::number(f64::NAN)));

        // Adding NaN again should not increase size
        SetObject::add(&set, JsValue::number(f64::NAN));
        assert_eq!(SetObject::size(&set), 1);
    }

    #[test]
    fn test_set_negative_zero() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(-0.0));

        // -0 and +0 should be treated as the same value (SameValueZero)
        assert!(SetObject::has(&set, &JsValue::number(0.0)));

        // Adding +0 should not increase size
        SetObject::add(&set, JsValue::number(0.0));
        assert_eq!(SetObject::size(&set), 1);
    }

    #[test]
    fn test_set_object_value() {
        let set = SetObject::new();
        let obj = JsValue::object();
        obj.set("id", JsValue::number(1.0));

        SetObject::add(&set, obj.clone());

        // Same object reference should match
        assert!(SetObject::has(&set, &obj));

        // Different object with same properties should NOT match
        let other_obj = JsValue::object();
        other_obj.set("id", JsValue::number(1.0));
        assert!(!SetObject::has(&set, &other_obj));
    }

    #[test]
    fn test_set_undefined_and_null() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::undefined());
        SetObject::add(&set, JsValue::null());

        assert!(SetObject::has(&set, &JsValue::undefined()));
        assert!(SetObject::has(&set, &JsValue::null()));
        assert_eq!(SetObject::size(&set), 2);
    }

    #[test]
    fn test_set_iterator() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::number(1.0));
        SetObject::add(&set, JsValue::number(2.0));

        let mut iter = SetObject::iter(&set);
        let v1 = iter.next().unwrap();
        assert_eq!(v1.as_number().unwrap(), 1.0);

        let v2 = iter.next().unwrap();
        assert_eq!(v2.as_number().unwrap(), 2.0);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_set_deduplication() {
        let values = vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(1.0), // Duplicate
            JsValue::number(3.0),
            JsValue::number(2.0), // Duplicate
        ];
        let set = SetObject::from_values(values);
        assert_eq!(SetObject::size(&set), 3);
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_map_with_mixed_key_types() {
        let map = MapObject::new();
        MapObject::set(&map, JsValue::string("str"), JsValue::number(1.0));
        MapObject::set(&map, JsValue::number(42.0), JsValue::number(2.0));
        MapObject::set(&map, JsValue::boolean(true), JsValue::number(3.0));
        MapObject::set(&map, JsValue::null(), JsValue::number(4.0));
        MapObject::set(&map, JsValue::undefined(), JsValue::number(5.0));

        assert_eq!(MapObject::size(&map), 5);
        assert!(MapObject::has(&map, &JsValue::string("str")));
        assert!(MapObject::has(&map, &JsValue::number(42.0)));
        assert!(MapObject::has(&map, &JsValue::boolean(true)));
        assert!(MapObject::has(&map, &JsValue::null()));
        assert!(MapObject::has(&map, &JsValue::undefined()));
    }

    #[test]
    fn test_set_with_mixed_value_types() {
        let set = SetObject::new();
        SetObject::add(&set, JsValue::string("str"));
        SetObject::add(&set, JsValue::number(42.0));
        SetObject::add(&set, JsValue::boolean(true));
        SetObject::add(&set, JsValue::null());
        SetObject::add(&set, JsValue::undefined());

        assert_eq!(SetObject::size(&set), 5);
        assert!(SetObject::has(&set, &JsValue::string("str")));
        assert!(SetObject::has(&set, &JsValue::number(42.0)));
        assert!(SetObject::has(&set, &JsValue::boolean(true)));
        assert!(SetObject::has(&set, &JsValue::null()));
        assert!(SetObject::has(&set, &JsValue::undefined()));
    }

    #[test]
    fn test_map_chaining() {
        let map = MapObject::new();
        // Chaining: map.set("a", 1).set("b", 2).set("c", 3)
        let map = MapObject::set(&map, JsValue::string("a"), JsValue::number(1.0));
        let map = MapObject::set(&map, JsValue::string("b"), JsValue::number(2.0));
        let _map = MapObject::set(&map, JsValue::string("c"), JsValue::number(3.0));

        assert_eq!(MapObject::size(&map), 3);
    }

    #[test]
    fn test_set_chaining() {
        let set = SetObject::new();
        // Chaining: set.add(1).add(2).add(3)
        let set = SetObject::add(&set, JsValue::number(1.0));
        let set = SetObject::add(&set, JsValue::number(2.0));
        let _set = SetObject::add(&set, JsValue::number(3.0));

        assert_eq!(SetObject::size(&set), 3);
    }
}
