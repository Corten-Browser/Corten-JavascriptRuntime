//! Contract tests for WeakRef and FinalizationRegistry
//!
//! Tests ES2024 compliance for WeakRef and FinalizationRegistry implementations.

use builtins::{FinalizationRegistryObject, JsValue, WeakRefObject};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(test)]
mod weakref_contract_tests {
    use super::*;

    #[test]
    fn test_weakref_constructor_accepts_objects() {
        // WeakRef should accept any object type
        let obj = JsValue::object();
        assert!(WeakRefObject::new(&obj).is_ok());

        let arr = JsValue::array();
        assert!(WeakRefObject::new(&arr).is_ok());

        let map = JsValue::map();
        assert!(WeakRefObject::new(&map).is_ok());

        let set = JsValue::set_collection();
        assert!(WeakRefObject::new(&set).is_ok());

        let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
        assert!(WeakRefObject::new(&func).is_ok());
    }

    #[test]
    fn test_weakref_constructor_rejects_primitives() {
        // WeakRef should reject all primitive types
        assert!(WeakRefObject::new(&JsValue::undefined()).is_err());
        assert!(WeakRefObject::new(&JsValue::null()).is_err());
        assert!(WeakRefObject::new(&JsValue::boolean(true)).is_err());
        assert!(WeakRefObject::new(&JsValue::boolean(false)).is_err());
        assert!(WeakRefObject::new(&JsValue::number(42.0)).is_err());
        assert!(WeakRefObject::new(&JsValue::number(f64::NAN)).is_err());
        assert!(WeakRefObject::new(&JsValue::string("test")).is_err());
    }

    #[test]
    fn test_weakref_deref_returns_target_when_alive() {
        let obj = JsValue::object();
        obj.set("value", JsValue::number(100.0));

        let weak_ref = WeakRefObject::new(&obj).unwrap();
        let derefed = WeakRefObject::deref(&weak_ref).unwrap();

        assert!(derefed.is_object());
        assert_eq!(derefed.get("value").unwrap().as_number().unwrap(), 100.0);
    }

    #[test]
    fn test_weakref_deref_returns_undefined_when_collected() {
        let weak_ref = {
            let obj = JsValue::object();
            obj.set("temp", JsValue::string("will be collected"));
            WeakRefObject::new(&obj).unwrap()
        };

        // Target is out of scope and should be collected
        let derefed = WeakRefObject::deref(&weak_ref).unwrap();
        assert!(derefed.is_undefined());
    }

    #[test]
    fn test_weakref_type_is_object() {
        let obj = JsValue::object();
        let weak_ref = WeakRefObject::new(&obj).unwrap();

        assert_eq!(weak_ref.type_of(), "object");
        assert!(weak_ref.is_weak_ref());
    }

    #[test]
    fn test_weakref_to_string() {
        let obj = JsValue::object();
        let weak_ref = WeakRefObject::new(&obj).unwrap();

        assert_eq!(weak_ref.to_js_string(), "[object WeakRef]");
    }

    #[test]
    fn test_weakref_identity() {
        let obj = JsValue::object();
        let weak_ref1 = WeakRefObject::new(&obj).unwrap();
        let weak_ref2 = WeakRefObject::new(&obj).unwrap();

        // Two different WeakRef instances pointing to same target
        assert!(!weak_ref1.equals(&weak_ref2));
        assert!(weak_ref1.object_identity() != weak_ref2.object_identity());
    }

    #[test]
    fn test_weakref_same_instance_equals() {
        let obj = JsValue::object();
        let weak_ref = WeakRefObject::new(&obj).unwrap();
        let cloned = weak_ref.clone();

        // Same instance should be equal
        assert!(weak_ref.equals(&cloned));
    }

    #[test]
    fn test_weakref_multiple_refs_all_alive() {
        let obj = JsValue::object();
        let ref1 = WeakRefObject::new(&obj).unwrap();
        let ref2 = WeakRefObject::new(&obj).unwrap();
        let ref3 = WeakRefObject::new(&obj).unwrap();

        // All refs should be alive while target exists
        assert!(WeakRefObject::is_alive(&ref1).unwrap());
        assert!(WeakRefObject::is_alive(&ref2).unwrap());
        assert!(WeakRefObject::is_alive(&ref3).unwrap());

        // All should return same object
        assert!(WeakRefObject::deref(&ref1).unwrap().is_object());
        assert!(WeakRefObject::deref(&ref2).unwrap().is_object());
        assert!(WeakRefObject::deref(&ref3).unwrap().is_object());
    }

    #[test]
    fn test_weakref_with_nested_object() {
        let inner = JsValue::object();
        inner.set("inner_value", JsValue::number(42.0));

        let outer = JsValue::object();
        outer.set("nested", inner);

        let weak_ref = WeakRefObject::new(&outer).unwrap();
        let derefed = WeakRefObject::deref(&weak_ref).unwrap();

        let nested = derefed.get("nested").unwrap();
        assert!(nested.is_object());
        assert_eq!(
            nested.get("inner_value").unwrap().as_number().unwrap(),
            42.0
        );
    }

    #[test]
    fn test_weakref_error_on_deref_non_weakref() {
        let obj = JsValue::object();
        let result = WeakRefObject::deref(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_weakref_error_on_is_alive_non_weakref() {
        let arr = JsValue::array();
        let result = WeakRefObject::is_alive(&arr);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod finalization_registry_contract_tests {
    use super::*;

    #[test]
    fn test_finalization_registry_constructor_accepts_function() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        assert!(registry.is_finalization_registry());
    }

    #[test]
    fn test_finalization_registry_constructor_rejects_non_function() {
        assert!(FinalizationRegistryObject::new(JsValue::object()).is_err());
        assert!(FinalizationRegistryObject::new(JsValue::array()).is_err());
        assert!(FinalizationRegistryObject::new(JsValue::number(42.0)).is_err());
        assert!(FinalizationRegistryObject::new(JsValue::string("test")).is_err());
        assert!(FinalizationRegistryObject::new(JsValue::null()).is_err());
        assert!(FinalizationRegistryObject::new(JsValue::undefined()).is_err());
    }

    #[test]
    fn test_finalization_registry_register_returns_undefined() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();

        let result = FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::string("held"),
            None,
        )
        .unwrap();

        assert!(result.is_undefined());
    }

    #[test]
    fn test_finalization_registry_register_rejects_primitive_target() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        // All primitives should be rejected
        assert!(FinalizationRegistryObject::register(
            &registry,
            &JsValue::undefined(),
            JsValue::number(1.0),
            None
        )
        .is_err());

        assert!(FinalizationRegistryObject::register(
            &registry,
            &JsValue::null(),
            JsValue::number(1.0),
            None
        )
        .is_err());

        assert!(FinalizationRegistryObject::register(
            &registry,
            &JsValue::boolean(true),
            JsValue::number(1.0),
            None
        )
        .is_err());

        assert!(FinalizationRegistryObject::register(
            &registry,
            &JsValue::number(42.0),
            JsValue::number(1.0),
            None
        )
        .is_err());

        assert!(FinalizationRegistryObject::register(
            &registry,
            &JsValue::string("target"),
            JsValue::number(1.0),
            None
        )
        .is_err());
    }

    #[test]
    fn test_finalization_registry_unregister_requires_object_token() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        // Primitive tokens should be rejected
        assert!(FinalizationRegistryObject::unregister(&registry, &JsValue::number(1.0)).is_err());
        assert!(
            FinalizationRegistryObject::unregister(&registry, &JsValue::string("token")).is_err()
        );
        assert!(FinalizationRegistryObject::unregister(&registry, &JsValue::null()).is_err());
    }

    #[test]
    fn test_finalization_registry_unregister_removes_registration() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();
        let token = JsValue::object();

        FinalizationRegistryObject::register(&registry, &target, JsValue::number(1.0), Some(&token))
            .unwrap();

        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            1
        );

        let removed = FinalizationRegistryObject::unregister(&registry, &token).unwrap();
        assert!(removed);
        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            0
        );
    }

    #[test]
    fn test_finalization_registry_unregister_returns_false_when_not_found() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let token = JsValue::object();

        let removed = FinalizationRegistryObject::unregister(&registry, &token).unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_finalization_registry_multiple_registrations_same_target() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();

        // ES2024 allows multiple registrations for same target
        FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::string("first"),
            None,
        )
        .unwrap();
        FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::string("second"),
            None,
        )
        .unwrap();
        FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::string("third"),
            None,
        )
        .unwrap();

        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            3
        );
    }

    #[test]
    fn test_finalization_registry_cleanup_callback_invocation() {
        let collected_values = Rc::new(RefCell::new(Vec::new()));
        let values_clone = collected_values.clone();

        let callback = JsValue::function(move |_, args| {
            if !args.is_empty() {
                values_clone.borrow_mut().push(args[0].clone());
            }
            Ok(JsValue::undefined())
        });
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        // Register and then let target be collected
        {
            let target = JsValue::object();
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("cleanup value"),
                None,
            )
            .unwrap();
        }

        // Process cleanup
        FinalizationRegistryObject::cleanup_some(&registry).unwrap();
        FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();

        // Verify callback was called with held value
        let values = collected_values.borrow();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_string().unwrap(), "cleanup value");
    }

    #[test]
    fn test_finalization_registry_type_is_object() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        assert_eq!(registry.type_of(), "object");
        assert!(registry.is_finalization_registry());
    }

    #[test]
    fn test_finalization_registry_to_string() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        assert_eq!(
            registry.to_js_string(),
            "[object FinalizationRegistry]"
        );
    }

    #[test]
    fn test_finalization_registry_held_value_can_be_any_type() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();

        // Held value can be any type
        FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::number(42.0),
            None,
        )
        .unwrap();

        let target2 = JsValue::object();
        FinalizationRegistryObject::register(&registry, &target2, JsValue::null(), None).unwrap();

        let target3 = JsValue::object();
        FinalizationRegistryObject::register(
            &registry,
            &target3,
            JsValue::array_from(vec![JsValue::number(1.0)]),
            None,
        )
        .unwrap();

        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            3
        );
    }

    #[test]
    fn test_finalization_registry_target_not_equal_to_held_value() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();

        // Target and held value must not be the same object
        let result = FinalizationRegistryObject::register(
            &registry,
            &target,
            target.clone(), // Same object
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_finalization_registry_live_target_not_cleaned() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let target = JsValue::object();

        FinalizationRegistryObject::register(
            &registry,
            &target,
            JsValue::string("should not clean"),
            None,
        )
        .unwrap();

        // Target is still alive
        FinalizationRegistryObject::cleanup_some(&registry).unwrap();

        // Entry should still exist
        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            1
        );

        // Queue should be empty (no cleanup needed)
        assert_eq!(
            FinalizationRegistryObject::queue_size(&registry).unwrap(),
            0
        );
    }

    #[test]
    fn test_finalization_registry_unregister_with_same_token_removes_all() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();
        let token = JsValue::object();

        let target1 = JsValue::object();
        let target2 = JsValue::object();
        let target3 = JsValue::object();

        // Register multiple targets with same token
        FinalizationRegistryObject::register(
            &registry,
            &target1,
            JsValue::number(1.0),
            Some(&token),
        )
        .unwrap();
        FinalizationRegistryObject::register(
            &registry,
            &target2,
            JsValue::number(2.0),
            Some(&token),
        )
        .unwrap();
        FinalizationRegistryObject::register(
            &registry,
            &target3,
            JsValue::number(3.0),
            Some(&token),
        )
        .unwrap();

        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            3
        );

        // Unregister should remove all entries with this token
        FinalizationRegistryObject::unregister(&registry, &token).unwrap();
        assert_eq!(
            FinalizationRegistryObject::entry_count(&registry).unwrap(),
            0
        );
    }

    #[test]
    fn test_finalization_registry_thread_safe_queue() {
        let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        // Queue operations should be thread-safe
        {
            let target = JsValue::object();
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("thread safe"),
                None,
            )
            .unwrap();
        }

        FinalizationRegistryObject::cleanup_some(&registry).unwrap();

        // Queue size should be 1
        assert_eq!(
            FinalizationRegistryObject::queue_size(&registry).unwrap(),
            1
        );

        // After processing, queue should be empty
        FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();
        assert_eq!(
            FinalizationRegistryObject::queue_size(&registry).unwrap(),
            0
        );
    }

    #[test]
    fn test_finalization_registry_callback_error_propagates() {
        let callback = JsValue::function(|_, _| {
            Err(builtins::JsError::new("callback error"))
        });
        let registry = FinalizationRegistryObject::new(callback).unwrap();

        {
            let target = JsValue::object();
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("will error"),
                None,
            )
            .unwrap();
        }

        FinalizationRegistryObject::cleanup_some(&registry).unwrap();

        // Callback error should propagate
        let result = FinalizationRegistryObject::run_cleanup_callbacks(&registry);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_weakref_and_finalization_registry_integration() {
        let cleanup_count = Rc::new(RefCell::new(0));
        let count_clone = cleanup_count.clone();

        let callback = JsValue::function(move |_, _| {
            *count_clone.borrow_mut() += 1;
            Ok(JsValue::undefined())
        });

        let registry = FinalizationRegistryObject::new(callback).unwrap();

        // Create object with WeakRef and register with FinalizationRegistry
        let weak_ref = {
            let obj = JsValue::object();
            obj.set("id", JsValue::number(1.0));

            FinalizationRegistryObject::register(
                &registry,
                &obj,
                JsValue::string("cleanup object 1"),
                None,
            )
            .unwrap();

            WeakRefObject::new(&obj).unwrap()
        };

        // Object is now out of scope
        // WeakRef should return undefined
        assert!(WeakRefObject::deref(&weak_ref).unwrap().is_undefined());

        // FinalizationRegistry should detect collection
        FinalizationRegistryObject::cleanup_some(&registry).unwrap();
        FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();

        // Cleanup should have been called
        assert_eq!(*cleanup_count.borrow(), 1);
    }

    #[test]
    fn test_weakref_keeps_object_alive_preventing_finalization() {
        let cleanup_count = Rc::new(RefCell::new(0));
        let count_clone = cleanup_count.clone();

        let callback = JsValue::function(move |_, _| {
            *count_clone.borrow_mut() += 1;
            Ok(JsValue::undefined())
        });

        let registry = FinalizationRegistryObject::new(callback).unwrap();

        let obj = JsValue::object();
        obj.set("id", JsValue::number(2.0));

        FinalizationRegistryObject::register(
            &registry,
            &obj,
            JsValue::string("cleanup object 2"),
            None,
        )
        .unwrap();

        let _weak_ref = WeakRefObject::new(&obj).unwrap();

        // Object is still in scope (we hold strong reference)
        FinalizationRegistryObject::cleanup_some(&registry).unwrap();
        FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();

        // Cleanup should NOT have been called (object still alive)
        assert_eq!(*cleanup_count.borrow(), 0);

        // WeakRef should still be able to deref
        assert!(WeakRefObject::is_alive(&_weak_ref).unwrap());
    }

    #[test]
    fn test_multiple_weakrefs_and_finalization_registries() {
        let registry1_count = Rc::new(RefCell::new(0));
        let registry2_count = Rc::new(RefCell::new(0));

        let count1_clone = registry1_count.clone();
        let count2_clone = registry2_count.clone();

        let callback1 = JsValue::function(move |_, _| {
            *count1_clone.borrow_mut() += 1;
            Ok(JsValue::undefined())
        });

        let callback2 = JsValue::function(move |_, _| {
            *count2_clone.borrow_mut() += 1;
            Ok(JsValue::undefined())
        });

        let registry1 = FinalizationRegistryObject::new(callback1).unwrap();
        let registry2 = FinalizationRegistryObject::new(callback2).unwrap();

        // Create objects and register with both registries
        let (weak_ref1, weak_ref2) = {
            let obj1 = JsValue::object();
            let obj2 = JsValue::object();

            FinalizationRegistryObject::register(
                &registry1,
                &obj1,
                JsValue::string("obj1"),
                None,
            )
            .unwrap();
            FinalizationRegistryObject::register(
                &registry2,
                &obj2,
                JsValue::string("obj2"),
                None,
            )
            .unwrap();

            (
                WeakRefObject::new(&obj1).unwrap(),
                WeakRefObject::new(&obj2).unwrap(),
            )
        };

        // Both objects should be collected
        assert!(WeakRefObject::deref(&weak_ref1).unwrap().is_undefined());
        assert!(WeakRefObject::deref(&weak_ref2).unwrap().is_undefined());

        // Both registries should have cleanup
        FinalizationRegistryObject::cleanup_some(&registry1).unwrap();
        FinalizationRegistryObject::cleanup_some(&registry2).unwrap();
        FinalizationRegistryObject::run_cleanup_callbacks(&registry1).unwrap();
        FinalizationRegistryObject::run_cleanup_callbacks(&registry2).unwrap();

        assert_eq!(*registry1_count.borrow(), 1);
        assert_eq!(*registry2_count.borrow(), 1);
    }
}
