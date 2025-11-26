//! Test262 harness built-in objects and functions
//!
//! This module provides native implementations of the $262 object and assert functions
//! required by the Test262 conformance test suite.

use crate::value::{JsError, JsResult, JsValue};

/// Test262 $262 object native implementation
pub struct Test262Object;

impl Test262Object {
    /// Create a new realm (isolated global environment)
    ///
    /// This is a placeholder implementation that returns an empty object.
    /// Full implementation would require creating a new VM instance.
    pub fn create_realm() -> JsValue {
        JsValue::object()
    }

    /// Evaluate JavaScript code in the current realm
    ///
    /// This is a placeholder that should delegate to the interpreter's eval.
    pub fn eval_script(_code: &str) -> JsResult<JsValue> {
        // TODO: Integrate with interpreter's eval function
        Err(JsError::type_error("evalScript not yet implemented"))
    }

    /// Trigger garbage collection
    ///
    /// This is a placeholder. Full implementation would trigger the GC.
    pub fn gc() {
        // TODO: Integrate with memory_manager GC
        // For now, this is a no-op
    }

    /// Detach an ArrayBuffer, making it unusable
    ///
    /// This is a placeholder. Full implementation would detach the buffer.
    pub fn detach_array_buffer(_buffer: &JsValue) -> JsResult<()> {
        // TODO: Integrate with typed_arrays module
        Err(JsError::type_error("detachArrayBuffer not yet implemented"))
    }

    /// Get the global object
    pub fn global() -> JsValue {
        // TODO: Return actual global object from VM
        JsValue::object()
    }
}

/// Test262 assert functions
pub struct Assert;

impl Assert {
    /// Basic assertion - throws if condition is false
    pub fn assert(condition: bool, message: Option<&str>) -> JsResult<()> {
        if !condition {
            let msg = message.unwrap_or("Assertion failed");
            Err(JsError::new(msg))
        } else {
            Ok(())
        }
    }

    /// Assert two values are strictly equal (===)
    pub fn same_value(actual: &JsValue, expected: &JsValue, message: Option<&str>) -> JsResult<()> {
        if !actual.strict_equals(expected) {
            let msg = format!(
                "Expected {:?} but got {:?}{}",
                expected,
                actual,
                message.map(|m| format!(": {}", m)).unwrap_or_default()
            );
            Err(JsError::new(msg))
        } else {
            Ok(())
        }
    }

    /// Assert two values are NOT strictly equal (!==)
    pub fn not_same_value(actual: &JsValue, unexpected: &JsValue, message: Option<&str>) -> JsResult<()> {
        if actual.strict_equals(unexpected) {
            let msg = format!(
                "Value should not be {:?}{}",
                unexpected,
                message.map(|m| format!(": {}", m)).unwrap_or_default()
            );
            Err(JsError::new(msg))
        } else {
            Ok(())
        }
    }

    /// Assert that a function throws an exception
    pub fn throws<F>(f: F, message: Option<&str>) -> JsResult<()>
    where
        F: FnOnce() -> JsResult<JsValue>,
    {
        match f() {
            Ok(_) => {
                let msg = format!(
                    "Expected exception but none was thrown{}",
                    message.map(|m| format!(": {}", m)).unwrap_or_default()
                );
                Err(JsError::new(msg))
            }
            Err(_e) => {
                // TODO: Check if error type matches expected type
                // For now, any error passes
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_passes_when_true() {
        assert!(Assert::assert(true, None).is_ok());
    }

    #[test]
    fn test_assert_fails_when_false() {
        assert!(Assert::assert(false, None).is_err());
    }

    #[test]
    fn test_same_value_passes_for_equal() {
        let a = JsValue::number(42.0);
        let b = JsValue::number(42.0);
        assert!(Assert::same_value(&a, &b, None).is_ok());
    }

    #[test]
    fn test_same_value_fails_for_different() {
        let a = JsValue::number(42.0);
        let b = JsValue::number(43.0);
        assert!(Assert::same_value(&a, &b, None).is_err());
    }

    #[test]
    fn test_not_same_value_passes_for_different() {
        let a = JsValue::number(42.0);
        let b = JsValue::number(43.0);
        assert!(Assert::not_same_value(&a, &b, None).is_ok());
    }

    #[test]
    fn test_not_same_value_fails_for_equal() {
        let a = JsValue::number(42.0);
        let b = JsValue::number(42.0);
        assert!(Assert::not_same_value(&a, &b, None).is_err());
    }

    #[test]
    fn test_create_realm_returns_object() {
        let realm = Test262Object::create_realm();
        assert!(realm.is_object());
    }
}
