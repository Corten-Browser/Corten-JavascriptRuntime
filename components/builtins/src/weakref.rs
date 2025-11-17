//! WeakRef and FinalizationRegistry implementations
//!
//! ES2024 compliant WeakRef and FinalizationRegistry for weak reference management
//! and cleanup callbacks when objects are garbage collected.

use crate::value::{JsError, JsResult, JsValue, ObjectData};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

/// WeakRef internal data
///
/// Holds a weak reference to a JavaScript object that allows the target
/// to be garbage collected when there are no strong references.
#[derive(Debug, Clone)]
pub struct WeakRefData {
    /// Weak reference to the target object's internal data
    target: WeakObjectRef,
}

/// Wrapper for different types of weak object references
#[derive(Debug, Clone)]
pub enum WeakObjectRef {
    /// Weak reference to an Object
    Object(Weak<RefCell<ObjectData>>),
    /// Weak reference to an Array
    Array(Weak<RefCell<crate::value::ArrayData>>),
    /// Weak reference to a Map
    Map(Weak<RefCell<crate::value::MapData>>),
    /// Weak reference to a Set
    Set(Weak<RefCell<crate::value::SetData>>),
    /// Weak reference to a Function
    Function(Weak<RefCell<crate::value::FunctionData>>),
    /// Weak reference to a Constructor
    Constructor(Weak<RefCell<crate::value::ConstructorData>>),
    /// Weak reference to a WeakMap
    WeakMap(Weak<RefCell<crate::value::WeakMapData>>),
    /// Weak reference to a WeakSet
    WeakSet(Weak<RefCell<crate::value::WeakSetData>>),
    /// Weak reference to an Error
    Error(Weak<RefCell<crate::error::JsErrorObject>>),
    /// Weak reference to a RegExp
    RegExp(Weak<RefCell<crate::regexp::RegExpObject>>),
}

impl WeakObjectRef {
    /// Check if the weak reference is still alive
    pub fn is_alive(&self) -> bool {
        match self {
            WeakObjectRef::Object(weak) => weak.strong_count() > 0,
            WeakObjectRef::Array(weak) => weak.strong_count() > 0,
            WeakObjectRef::Map(weak) => weak.strong_count() > 0,
            WeakObjectRef::Set(weak) => weak.strong_count() > 0,
            WeakObjectRef::Function(weak) => weak.strong_count() > 0,
            WeakObjectRef::Constructor(weak) => weak.strong_count() > 0,
            WeakObjectRef::WeakMap(weak) => weak.strong_count() > 0,
            WeakObjectRef::WeakSet(weak) => weak.strong_count() > 0,
            WeakObjectRef::Error(weak) => weak.strong_count() > 0,
            WeakObjectRef::RegExp(weak) => weak.strong_count() > 0,
        }
    }

    /// Upgrade the weak reference to a strong reference (returns the JsValue if alive)
    pub fn upgrade(&self) -> Option<JsValue> {
        match self {
            WeakObjectRef::Object(weak) => weak.upgrade().map(JsValue::Object),
            WeakObjectRef::Array(weak) => weak.upgrade().map(JsValue::Array),
            WeakObjectRef::Map(weak) => weak.upgrade().map(JsValue::Map),
            WeakObjectRef::Set(weak) => weak.upgrade().map(JsValue::Set),
            WeakObjectRef::Function(weak) => weak.upgrade().map(JsValue::Function),
            WeakObjectRef::Constructor(weak) => weak.upgrade().map(JsValue::Constructor),
            WeakObjectRef::WeakMap(weak) => weak.upgrade().map(JsValue::WeakMap),
            WeakObjectRef::WeakSet(weak) => weak.upgrade().map(JsValue::WeakSet),
            WeakObjectRef::Error(weak) => weak.upgrade().map(JsValue::Error),
            WeakObjectRef::RegExp(weak) => weak.upgrade().map(JsValue::RegExp),
        }
    }
}

/// WeakRef built-in object
///
/// Implements ES2024 WeakRef with:
/// - Weak reference to target object (target can be GC'd)
/// - deref() method to retrieve target or undefined
/// - Only accepts objects (not primitives)
pub struct WeakRefObject;

impl WeakRefObject {
    /// Create a new WeakRef holding a weak reference to the target
    ///
    /// Returns TypeError if target is not an object (primitives not allowed).
    pub fn new(target: &JsValue) -> JsResult<JsValue> {
        let weak_ref = Self::create_weak_ref(target)?;

        Ok(JsValue::WeakRef(Rc::new(RefCell::new(WeakRefData {
            target: weak_ref,
        }))))
    }

    /// Create a weak reference from a JsValue
    fn create_weak_ref(target: &JsValue) -> JsResult<WeakObjectRef> {
        match target {
            JsValue::Object(rc) => Ok(WeakObjectRef::Object(Rc::downgrade(rc))),
            JsValue::Array(rc) => Ok(WeakObjectRef::Array(Rc::downgrade(rc))),
            JsValue::Map(rc) => Ok(WeakObjectRef::Map(Rc::downgrade(rc))),
            JsValue::Set(rc) => Ok(WeakObjectRef::Set(Rc::downgrade(rc))),
            JsValue::Function(rc) => Ok(WeakObjectRef::Function(Rc::downgrade(rc))),
            JsValue::Constructor(rc) => Ok(WeakObjectRef::Constructor(Rc::downgrade(rc))),
            JsValue::WeakMap(rc) => Ok(WeakObjectRef::WeakMap(Rc::downgrade(rc))),
            JsValue::WeakSet(rc) => Ok(WeakObjectRef::WeakSet(Rc::downgrade(rc))),
            JsValue::Error(rc) => Ok(WeakObjectRef::Error(Rc::downgrade(rc))),
            JsValue::RegExp(rc) => Ok(WeakObjectRef::RegExp(Rc::downgrade(rc))),
            // Primitives and non-weak-referenceable types
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_)
            | JsValue::BigInt(_)
            | JsValue::Proxy(_)
            | JsValue::Generator(_)
            | JsValue::AsyncGenerator(_) => Err(JsError::type_error(
                "WeakRef constructor requires an object as target",
            )),
            // WeakRef and FinalizationRegistry cannot be targets of themselves
            JsValue::WeakRef(_) | JsValue::FinalizationRegistry(_) => Err(JsError::type_error(
                "WeakRef constructor requires an object as target",
            )),
        }
    }

    /// Dereference the WeakRef to get the target object
    ///
    /// Returns the target object if it's still alive, or undefined if it has been collected.
    pub fn deref(weak_ref: &JsValue) -> JsResult<JsValue> {
        if let JsValue::WeakRef(data) = weak_ref {
            let weak_ref_data = data.borrow();
            Ok(weak_ref_data.target.upgrade().unwrap_or(JsValue::Undefined))
        } else {
            Err(JsError::type_error("deref called on non-WeakRef"))
        }
    }

    /// Check if the WeakRef's target is still alive
    ///
    /// This is not part of the ES spec but useful for testing.
    pub fn is_alive(weak_ref: &JsValue) -> JsResult<bool> {
        if let JsValue::WeakRef(data) = weak_ref {
            let weak_ref_data = data.borrow();
            Ok(weak_ref_data.target.is_alive())
        } else {
            Err(JsError::type_error("is_alive called on non-WeakRef"))
        }
    }
}

/// FinalizationRegistry cleanup entry
#[derive(Debug, Clone)]
struct CleanupEntry {
    /// Weak reference to the target being watched
    target: WeakObjectRef,
    /// Value to pass to cleanup callback when target is collected
    held_value: JsValue,
    /// Optional unregister token (object identity)
    unregister_token: Option<usize>,
}

/// FinalizationRegistry internal data
///
/// Manages cleanup callbacks for objects when they are garbage collected.
#[derive(Debug)]
pub struct FinalizationRegistryData {
    /// The cleanup callback function
    cleanup_callback: Rc<RefCell<crate::value::FunctionData>>,
    /// Registry of targets being watched
    entries: Vec<CleanupEntry>,
    /// Queue of held values for cleanup (from collected targets)
    cleanup_queue: Arc<Mutex<Vec<JsValue>>>,
}

impl Clone for FinalizationRegistryData {
    fn clone(&self) -> Self {
        FinalizationRegistryData {
            cleanup_callback: self.cleanup_callback.clone(),
            entries: self.entries.clone(),
            cleanup_queue: self.cleanup_queue.clone(),
        }
    }
}

/// FinalizationRegistry built-in object
///
/// Implements ES2024 FinalizationRegistry with:
/// - Cleanup callback invoked when registered objects are collected
/// - register(target, heldValue, unregisterToken?) for watching objects
/// - unregister(unregisterToken) to remove registrations
/// - Thread-safe cleanup queue
pub struct FinalizationRegistryObject;

impl FinalizationRegistryObject {
    /// Create a new FinalizationRegistry with the given cleanup callback
    ///
    /// The callback will be invoked with the held value when a registered target is collected.
    pub fn new(cleanup_callback: JsValue) -> JsResult<JsValue> {
        let callback = match cleanup_callback {
            JsValue::Function(func) => func,
            _ => {
                return Err(JsError::type_error(
                    "FinalizationRegistry constructor requires a callback function",
                ))
            }
        };

        Ok(JsValue::FinalizationRegistry(Rc::new(RefCell::new(
            FinalizationRegistryData {
                cleanup_callback: callback,
                entries: Vec::new(),
                cleanup_queue: Arc::new(Mutex::new(Vec::new())),
            },
        ))))
    }

    /// Register a target object with the registry
    ///
    /// When target is garbage collected, the cleanup callback will be called with held_value.
    /// Optional unregister_token allows later removal of the registration.
    ///
    /// Returns undefined.
    /// Returns TypeError if target or unregister_token (if provided) is not an object.
    pub fn register(
        registry: &JsValue,
        target: &JsValue,
        held_value: JsValue,
        unregister_token: Option<&JsValue>,
    ) -> JsResult<JsValue> {
        if let JsValue::FinalizationRegistry(data) = registry {
            // Create weak reference to target
            let weak_ref = WeakRefObject::create_weak_ref(target)?;

            // Get unregister token identity if provided
            let token_id = if let Some(token) = unregister_token {
                let id = token.object_identity().ok_or_else(|| {
                    JsError::type_error(
                        "FinalizationRegistry.register: unregisterToken must be an object",
                    )
                })?;
                Some(id)
            } else {
                None
            };

            // Held value must not be the target itself
            if target.object_identity() == held_value.object_identity()
                && target.object_identity().is_some()
            {
                return Err(JsError::type_error(
                    "FinalizationRegistry.register: target and heldValue must not be the same",
                ));
            }

            let mut registry_data = data.borrow_mut();
            registry_data.entries.push(CleanupEntry {
                target: weak_ref,
                held_value,
                unregister_token: token_id,
            });

            Ok(JsValue::Undefined)
        } else {
            Err(JsError::type_error("register called on non-FinalizationRegistry"))
        }
    }

    /// Unregister all entries with the given unregister token
    ///
    /// Returns true if any entries were removed, false otherwise.
    /// Returns TypeError if unregister_token is not an object.
    pub fn unregister(registry: &JsValue, unregister_token: &JsValue) -> JsResult<bool> {
        if let JsValue::FinalizationRegistry(data) = registry {
            let token_id = unregister_token.object_identity().ok_or_else(|| {
                JsError::type_error("FinalizationRegistry.unregister: token must be an object")
            })?;

            let mut registry_data = data.borrow_mut();
            let original_len = registry_data.entries.len();
            registry_data
                .entries
                .retain(|entry| entry.unregister_token != Some(token_id));
            let removed = registry_data.entries.len() < original_len;

            Ok(removed)
        } else {
            Err(JsError::type_error("unregister called on non-FinalizationRegistry"))
        }
    }

    /// Check for collected targets and queue their cleanup callbacks
    ///
    /// This should be called periodically by the runtime/GC to process
    /// targets that have been garbage collected.
    pub fn cleanup_some(registry: &JsValue) -> JsResult<()> {
        if let JsValue::FinalizationRegistry(data) = registry {
            let mut registry_data = data.borrow_mut();
            let cleanup_queue = registry_data.cleanup_queue.clone();

            // Find entries where target has been collected
            let mut collected_values = Vec::new();
            registry_data.entries.retain(|entry| {
                if !entry.target.is_alive() {
                    collected_values.push(entry.held_value.clone());
                    false // Remove this entry
                } else {
                    true // Keep this entry
                }
            });

            // Add to cleanup queue
            if !collected_values.is_empty() {
                let mut queue = cleanup_queue.lock().unwrap();
                queue.extend(collected_values);
            }

            Ok(())
        } else {
            Err(JsError::type_error("cleanupSome called on non-FinalizationRegistry"))
        }
    }

    /// Execute pending cleanup callbacks
    ///
    /// This processes the cleanup queue and invokes the callback for each held value.
    /// Should be called by the runtime event loop.
    pub fn run_cleanup_callbacks(registry: &JsValue) -> JsResult<Vec<JsValue>> {
        if let JsValue::FinalizationRegistry(data) = registry {
            let registry_data = data.borrow();
            let cleanup_queue = registry_data.cleanup_queue.clone();
            let callback = registry_data.cleanup_callback.clone();
            drop(registry_data);

            // Get queued values
            let values: Vec<JsValue> = {
                let mut queue = cleanup_queue.lock().unwrap();
                std::mem::take(&mut *queue)
            };

            // Execute callbacks
            let mut results = Vec::new();
            for held_value in values {
                let callback_data = callback.borrow();
                let result = (callback_data.func)(JsValue::Undefined, vec![held_value])?;
                results.push(result);
            }

            Ok(results)
        } else {
            Err(JsError::type_error(
                "runCleanupCallbacks called on non-FinalizationRegistry",
            ))
        }
    }

    /// Get the number of registered entries (for testing)
    pub fn entry_count(registry: &JsValue) -> JsResult<usize> {
        if let JsValue::FinalizationRegistry(data) = registry {
            Ok(data.borrow().entries.len())
        } else {
            Err(JsError::type_error("entryCount called on non-FinalizationRegistry"))
        }
    }

    /// Get the number of items in the cleanup queue (for testing)
    pub fn queue_size(registry: &JsValue) -> JsResult<usize> {
        if let JsValue::FinalizationRegistry(data) = registry {
            let registry_data = data.borrow();
            let queue = registry_data.cleanup_queue.lock().unwrap();
            Ok(queue.len())
        } else {
            Err(JsError::type_error("queueSize called on non-FinalizationRegistry"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod weakref_tests {
        use super::*;

        #[test]
        fn test_weakref_creation_with_object() {
            let obj = JsValue::object();
            let weak_ref = WeakRefObject::new(&obj).unwrap();
            assert!(weak_ref.is_weak_ref());
        }

        #[test]
        fn test_weakref_creation_with_array() {
            let arr = JsValue::array();
            let weak_ref = WeakRefObject::new(&arr).unwrap();
            assert!(weak_ref.is_weak_ref());
        }

        #[test]
        fn test_weakref_creation_with_function() {
            let func = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let weak_ref = WeakRefObject::new(&func).unwrap();
            assert!(weak_ref.is_weak_ref());
        }

        #[test]
        fn test_weakref_rejects_primitives() {
            // Number
            let result = WeakRefObject::new(&JsValue::number(42.0));
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("object as target"));

            // String
            let result = WeakRefObject::new(&JsValue::string("test"));
            assert!(result.is_err());

            // Boolean
            let result = WeakRefObject::new(&JsValue::boolean(true));
            assert!(result.is_err());

            // Null
            let result = WeakRefObject::new(&JsValue::null());
            assert!(result.is_err());

            // Undefined
            let result = WeakRefObject::new(&JsValue::undefined());
            assert!(result.is_err());
        }

        #[test]
        fn test_weakref_deref_returns_target() {
            let obj = JsValue::object();
            obj.set("test", JsValue::number(42.0));

            let weak_ref = WeakRefObject::new(&obj).unwrap();
            let derefed = WeakRefObject::deref(&weak_ref).unwrap();

            assert!(derefed.is_object());
            assert_eq!(derefed.get("test").unwrap().as_number().unwrap(), 42.0);
        }

        #[test]
        fn test_weakref_is_alive() {
            let obj = JsValue::object();
            let weak_ref = WeakRefObject::new(&obj).unwrap();

            assert!(WeakRefObject::is_alive(&weak_ref).unwrap());
        }

        #[test]
        fn test_weakref_returns_undefined_when_collected() {
            // Simulate GC by letting the strong reference go out of scope
            let weak_ref = {
                let obj = JsValue::object();
                obj.set("id", JsValue::number(1.0));
                WeakRefObject::new(&obj).unwrap()
            };
            // obj is now out of scope and should be "collected"

            let derefed = WeakRefObject::deref(&weak_ref).unwrap();
            assert!(derefed.is_undefined());
            assert!(!WeakRefObject::is_alive(&weak_ref).unwrap());
        }

        #[test]
        fn test_weakref_with_map() {
            let map = JsValue::map();
            let weak_ref = WeakRefObject::new(&map).unwrap();
            assert!(WeakRefObject::deref(&weak_ref).unwrap().is_map());
        }

        #[test]
        fn test_weakref_with_set() {
            let set = JsValue::set_collection();
            let weak_ref = WeakRefObject::new(&set).unwrap();
            assert!(WeakRefObject::deref(&weak_ref).unwrap().is_set());
        }

        #[test]
        fn test_weakref_multiple_refs_same_target() {
            let obj = JsValue::object();
            let weak_ref1 = WeakRefObject::new(&obj).unwrap();
            let weak_ref2 = WeakRefObject::new(&obj).unwrap();

            // Both should be alive while obj exists
            assert!(WeakRefObject::is_alive(&weak_ref1).unwrap());
            assert!(WeakRefObject::is_alive(&weak_ref2).unwrap());
        }

        #[test]
        fn test_weakref_deref_on_non_weakref_errors() {
            let obj = JsValue::object();
            let result = WeakRefObject::deref(&obj);
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("non-WeakRef"));
        }
    }

    mod finalization_registry_tests {
        use super::*;
        use std::cell::RefCell;
        use std::rc::Rc;

        #[test]
        fn test_finalization_registry_creation() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();
            assert!(registry.is_finalization_registry());
        }

        #[test]
        fn test_finalization_registry_rejects_non_function() {
            let result = FinalizationRegistryObject::new(JsValue::object());
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("callback function"));
        }

        #[test]
        fn test_finalization_registry_register_target() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target = JsValue::object();
            let held_value = JsValue::string("cleanup data");

            let result =
                FinalizationRegistryObject::register(&registry, &target, held_value, None).unwrap();
            assert!(result.is_undefined());
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 1);
        }

        #[test]
        fn test_finalization_registry_register_rejects_primitive_target() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let result = FinalizationRegistryObject::register(
                &registry,
                &JsValue::number(42.0),
                JsValue::string("data"),
                None,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_finalization_registry_unregister_with_token() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target = JsValue::object();
            let token = JsValue::object();
            let held_value = JsValue::string("data");

            FinalizationRegistryObject::register(&registry, &target, held_value, Some(&token))
                .unwrap();
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 1);

            // Unregister with token
            let removed = FinalizationRegistryObject::unregister(&registry, &token).unwrap();
            assert!(removed);
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 0);
        }

        #[test]
        fn test_finalization_registry_unregister_returns_false_if_not_found() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let token = JsValue::object();
            let removed = FinalizationRegistryObject::unregister(&registry, &token).unwrap();
            assert!(!removed);
        }

        #[test]
        fn test_finalization_registry_unregister_rejects_primitive_token() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let result =
                FinalizationRegistryObject::unregister(&registry, &JsValue::string("token"));
            assert!(result.is_err());
        }

        #[test]
        fn test_finalization_registry_multiple_registrations_same_target() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target = JsValue::object();

            // Register same target multiple times with different held values
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("data1"),
                None,
            )
            .unwrap();
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("data2"),
                None,
            )
            .unwrap();
            FinalizationRegistryObject::register(
                &registry,
                &target,
                JsValue::string("data3"),
                None,
            )
            .unwrap();

            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 3);
        }

        #[test]
        fn test_finalization_registry_cleanup_callback_invoked() {
            let cleanup_values = Rc::new(RefCell::new(Vec::new()));
            let cleanup_values_clone = cleanup_values.clone();

            let callback = JsValue::function(move |_, args| {
                if !args.is_empty() {
                    cleanup_values_clone.borrow_mut().push(args[0].clone());
                }
                Ok(JsValue::undefined())
            });
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            // Register a target that will be collected
            {
                let target = JsValue::object();
                FinalizationRegistryObject::register(
                    &registry,
                    &target,
                    JsValue::string("cleanup me"),
                    None,
                )
                .unwrap();
            }
            // target is now out of scope

            // Check for collected targets
            FinalizationRegistryObject::cleanup_some(&registry).unwrap();

            // Verify entry was removed
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 0);

            // Verify held value is in cleanup queue
            assert_eq!(FinalizationRegistryObject::queue_size(&registry).unwrap(), 1);

            // Run cleanup callbacks
            let results = FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();
            assert_eq!(results.len(), 1);

            // Verify callback was called with held value
            let values = cleanup_values.borrow();
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].as_string().unwrap(), "cleanup me");
        }

        #[test]
        fn test_finalization_registry_cleanup_with_multiple_collected() {
            let cleanup_count = Rc::new(RefCell::new(0));
            let cleanup_count_clone = cleanup_count.clone();

            let callback = JsValue::function(move |_, _| {
                *cleanup_count_clone.borrow_mut() += 1;
                Ok(JsValue::undefined())
            });
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            // Register multiple targets that will be collected
            for i in 0..5 {
                let target = JsValue::object();
                FinalizationRegistryObject::register(
                    &registry,
                    &target,
                    JsValue::number(i as f64),
                    None,
                )
                .unwrap();
            }

            // All targets are now out of scope
            FinalizationRegistryObject::cleanup_some(&registry).unwrap();
            FinalizationRegistryObject::run_cleanup_callbacks(&registry).unwrap();

            assert_eq!(*cleanup_count.borrow(), 5);
        }

        #[test]
        fn test_finalization_registry_unregister_removes_multiple_entries_same_token() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target1 = JsValue::object();
            let target2 = JsValue::object();
            let token = JsValue::object();

            // Register multiple targets with same token
            FinalizationRegistryObject::register(&registry, &target1, JsValue::number(1.0), Some(&token))
                .unwrap();
            FinalizationRegistryObject::register(&registry, &target2, JsValue::number(2.0), Some(&token))
                .unwrap();

            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 2);

            // Unregister should remove both
            FinalizationRegistryObject::unregister(&registry, &token).unwrap();
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 0);
        }

        #[test]
        fn test_finalization_registry_target_and_held_value_must_differ() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target = JsValue::object();

            // Same target as held value should error
            let result = FinalizationRegistryObject::register(
                &registry,
                &target,
                target.clone(),
                None,
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().message.contains("must not be the same"));
        }

        #[test]
        fn test_finalization_registry_thread_safe_queue() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            // Register targets
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

            // Queue should be accessible
            assert_eq!(FinalizationRegistryObject::queue_size(&registry).unwrap(), 0);
            FinalizationRegistryObject::cleanup_some(&registry).unwrap();
            assert_eq!(FinalizationRegistryObject::queue_size(&registry).unwrap(), 1);
        }

        #[test]
        fn test_finalization_registry_live_target_not_cleaned() {
            let callback = JsValue::function(|_, _| Ok(JsValue::undefined()));
            let registry = FinalizationRegistryObject::new(callback).unwrap();

            let target = JsValue::object();
            FinalizationRegistryObject::register(&registry, &target, JsValue::string("data"), None)
                .unwrap();

            // Target is still alive
            FinalizationRegistryObject::cleanup_some(&registry).unwrap();

            // Entry should still be there
            assert_eq!(FinalizationRegistryObject::entry_count(&registry).unwrap(), 1);

            // Queue should be empty
            assert_eq!(FinalizationRegistryObject::queue_size(&registry).unwrap(), 0);
        }
    }
}
