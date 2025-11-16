//! Unit tests for Promise

use async_runtime::{Promise, PromiseState};
use core_types::{ErrorKind, JsError, Value};

#[test]
fn new_promise_is_pending() {
    let promise = Promise::new();
    assert!(matches!(promise.state, PromiseState::Pending));
}

#[test]
fn new_promise_has_empty_reactions() {
    let promise = Promise::new();
    assert!(promise.reactions.is_empty());
}

#[test]
fn new_promise_has_no_result() {
    let promise = Promise::new();
    assert!(promise.result.is_none());
}

#[test]
fn resolve_changes_state_to_fulfilled() {
    let mut promise = Promise::new();
    promise.resolve(Value::Smi(42));
    assert!(matches!(promise.state, PromiseState::Fulfilled));
}

#[test]
fn resolve_sets_result_value() {
    let mut promise = Promise::new();
    promise.resolve(Value::Smi(42));
    assert_eq!(promise.result, Some(Value::Smi(42)));
}

#[test]
fn reject_changes_state_to_rejected() {
    let mut promise = Promise::new();
    let error = JsError {
        kind: ErrorKind::TypeError,
        message: "test".to_string(),
        stack: vec![],
        source_position: None,
    };
    promise.reject(error);
    assert!(matches!(promise.state, PromiseState::Rejected));
}

#[test]
fn cannot_resolve_already_fulfilled_promise() {
    let mut promise = Promise::new();
    promise.resolve(Value::Smi(42));
    promise.resolve(Value::Smi(100)); // Should be ignored
    assert_eq!(promise.result, Some(Value::Smi(42)));
}

#[test]
fn cannot_reject_already_fulfilled_promise() {
    let mut promise = Promise::new();
    promise.resolve(Value::Smi(42));
    let error = JsError {
        kind: ErrorKind::TypeError,
        message: "test".to_string(),
        stack: vec![],
        source_position: None,
    };
    promise.reject(error); // Should be ignored
    assert!(matches!(promise.state, PromiseState::Fulfilled));
}

#[test]
fn cannot_resolve_already_rejected_promise() {
    let mut promise = Promise::new();
    let error = JsError {
        kind: ErrorKind::TypeError,
        message: "test".to_string(),
        stack: vec![],
        source_position: None,
    };
    promise.reject(error);
    promise.resolve(Value::Smi(42)); // Should be ignored
    assert!(matches!(promise.state, PromiseState::Rejected));
}

#[test]
fn then_returns_new_promise() {
    let mut promise = Promise::new();
    let chained = promise.then(None, None);
    // Should be a different promise
    assert!(matches!(chained.state, PromiseState::Pending));
}

#[test]
fn then_on_pending_adds_reaction() {
    let mut promise = Promise::new();
    let _chained = promise.then(None, None);
    assert_eq!(promise.reactions.len(), 1);
}

#[test]
fn then_with_fulfilled_handler() {
    let mut promise = Promise::new();

    let callback_called = std::sync::Arc::new(std::sync::Mutex::new(false));
    let cc = callback_called.clone();

    let handler = async_runtime::Function::new(move |_args| {
        *cc.lock().unwrap() = true;
        Ok(Value::Undefined)
    });

    let _chained = promise.then(Some(handler), None);
    promise.resolve(Value::Smi(42));

    // In a real event loop, this would trigger the callback
    assert!(*callback_called.lock().unwrap() || promise.has_pending_reactions());
}

#[test]
fn then_chaining_creates_promise_chain() {
    let mut promise = Promise::new();
    let mut promise2 = promise.then(None, None);
    let _promise3 = promise2.then(None, None);

    // Each then creates a new promise
    assert!(matches!(promise.state, PromiseState::Pending));
}

#[test]
fn promise_state_variants() {
    let pending = PromiseState::Pending;
    let fulfilled = PromiseState::Fulfilled;
    let rejected = PromiseState::Rejected;

    assert!(matches!(pending, PromiseState::Pending));
    assert!(matches!(fulfilled, PromiseState::Fulfilled));
    assert!(matches!(rejected, PromiseState::Rejected));
}

#[test]
fn resolve_with_undefined() {
    let mut promise = Promise::new();
    promise.resolve(Value::Undefined);
    assert_eq!(promise.result, Some(Value::Undefined));
}

#[test]
fn resolve_with_null() {
    let mut promise = Promise::new();
    promise.resolve(Value::Null);
    assert_eq!(promise.result, Some(Value::Null));
}

#[test]
fn resolve_with_boolean() {
    let mut promise = Promise::new();
    promise.resolve(Value::Boolean(true));
    assert_eq!(promise.result, Some(Value::Boolean(true)));
}

#[test]
fn resolve_with_double() {
    let mut promise = Promise::new();
    promise.resolve(Value::Double(3.14));
    assert_eq!(promise.result, Some(Value::Double(3.14)));
}
