//! Promise integration between async_runtime and interpreter
//!
//! This module provides the bridge between builtins Promise and runtime Value types.

use async_runtime::{Promise, PromiseState};
use core_types::{JsError, Value};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// A JavaScript Promise object that wraps async_runtime::Promise
#[derive(Debug)]
pub struct PromiseObject {
    /// The underlying Promise from async_runtime
    pub promise: Promise,
}

impl PromiseObject {
    /// Create a new pending Promise
    pub fn new() -> Self {
        Self {
            promise: Promise::new(),
        }
    }

    /// Create a resolved Promise with the given value
    pub fn resolve(value: Value) -> Self {
        let mut promise = Promise::new();
        promise.resolve(value);
        Self { promise }
    }

    /// Create a rejected Promise with the given error
    pub fn reject(error: JsError) -> Self {
        let mut promise = Promise::new();
        promise.reject(error);
        Self { promise }
    }

    /// Get the current state of the Promise
    pub fn state(&self) -> &PromiseState {
        &self.promise.state
    }

    /// Get the resolved value (if fulfilled)
    pub fn value(&self) -> Option<&Value> {
        self.promise.result.as_ref()
    }

    /// Get the rejection error (if rejected)
    pub fn error(&self) -> Option<&JsError> {
        self.promise.error.as_ref()
    }

    /// Resolve this Promise with a value
    pub fn do_resolve(&mut self, value: Value) {
        self.promise.resolve(value);
    }

    /// Reject this Promise with an error
    pub fn do_reject(&mut self, error: JsError) {
        self.promise.reject(error);
    }
}

impl Default for PromiseObject {
    fn default() -> Self {
        Self::new()
    }
}

/// Promise constructor object for global scope
#[derive(Debug)]
pub struct PromiseConstructor;

impl PromiseConstructor {
    /// Create a new resolved Promise
    pub fn resolve(value: Value) -> Value {
        let promise_obj = PromiseObject::resolve(value);
        Value::NativeObject(Rc::new(RefCell::new(promise_obj)) as Rc<RefCell<dyn Any>>)
    }

    /// Create a new rejected Promise
    pub fn reject(error: JsError) -> Value {
        let promise_obj = PromiseObject::reject(error);
        Value::NativeObject(Rc::new(RefCell::new(promise_obj)) as Rc<RefCell<dyn Any>>)
    }

    /// Create a new pending Promise
    pub fn new_pending() -> Value {
        let promise_obj = PromiseObject::new();
        Value::NativeObject(Rc::new(RefCell::new(promise_obj)) as Rc<RefCell<dyn Any>>)
    }
}

/// Convert a Value to an Rc<RefCell<dyn Any>> if it's a Promise
/// Note: The returned Rc is the actual object, caller should use downcast_ref to access PromiseObject
pub fn as_promise_any(value: &Value) -> Option<Rc<RefCell<dyn Any>>> {
    match value {
        Value::NativeObject(obj) => {
            // Try to downcast to PromiseObject
            let borrowed = obj.borrow();
            if borrowed.is::<PromiseObject>() {
                drop(borrowed);
                // Return the Rc<RefCell<dyn Any>> directly
                Some(Rc::clone(obj))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a Value is a Promise
pub fn is_promise(value: &Value) -> bool {
    match value {
        Value::NativeObject(obj) => obj.borrow().is::<PromiseObject>(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::ErrorKind;

    #[test]
    fn test_promise_object_new() {
        let promise = PromiseObject::new();
        assert!(matches!(promise.state(), PromiseState::Pending));
        assert!(promise.value().is_none());
        assert!(promise.error().is_none());
    }

    #[test]
    fn test_promise_object_resolve() {
        let promise = PromiseObject::resolve(Value::Smi(42));
        assert!(matches!(promise.state(), PromiseState::Fulfilled));
        assert_eq!(promise.value(), Some(&Value::Smi(42)));
    }

    #[test]
    fn test_promise_object_reject() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "test error".to_string(),
            stack: vec![],
            source_position: None,
        };
        let promise = PromiseObject::reject(error);
        assert!(matches!(promise.state(), PromiseState::Rejected));
        assert!(promise.error().is_some());
    }

    #[test]
    fn test_promise_constructor_resolve() {
        let value = PromiseConstructor::resolve(Value::Smi(100));
        assert!(is_promise(&value));
    }

    #[test]
    fn test_promise_constructor_new_pending() {
        let value = PromiseConstructor::new_pending();
        assert!(is_promise(&value));
    }

    #[test]
    fn test_is_promise() {
        let promise = PromiseConstructor::new_pending();
        assert!(is_promise(&promise));

        let not_promise = Value::Smi(42);
        assert!(!is_promise(&not_promise));
    }
}
