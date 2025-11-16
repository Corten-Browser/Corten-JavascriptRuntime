//! JavaScript value representation for builtins
//!
//! This module provides a high-level JavaScript value type for use with built-in methods.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Error type for JavaScript operations
#[derive(Debug, Clone, PartialEq)]
pub struct JsError {
    /// The error message
    pub message: String,
}

impl JsError {
    /// Create a new generic error
    pub fn new(message: impl Into<String>) -> Self {
        JsError {
            message: message.into(),
        }
    }

    /// Create a TypeError
    pub fn type_error(message: impl Into<String>) -> Self {
        JsError::new(format!("TypeError: {}", message.into()))
    }

    /// Create a SyntaxError
    pub fn syntax_error(message: impl Into<String>) -> Self {
        JsError::new(format!("SyntaxError: {}", message.into()))
    }

    /// Create a RangeError
    pub fn range_error(message: impl Into<String>) -> Self {
        JsError::new(format!("RangeError: {}", message.into()))
    }
}

/// Result type for JavaScript operations
pub type JsResult<T> = Result<T, JsError>;

/// Internal object data
#[derive(Debug, Clone)]
pub struct ObjectData {
    /// Object properties map
    pub properties: HashMap<String, JsValue>,
    /// Optional prototype reference
    pub prototype: Option<Box<JsValue>>,
}

/// Internal array data
#[derive(Debug, Clone)]
pub struct ArrayData {
    /// Array elements
    pub elements: Vec<JsValue>,
}

/// JavaScript value representation
#[derive(Debug, Clone)]
pub enum JsValue {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number (IEEE 754 double)
    Number(f64),
    /// String value
    String(String),
    /// Object with properties
    Object(Rc<RefCell<ObjectData>>),
    /// Array
    Array(Rc<RefCell<ArrayData>>),
}

impl JsValue {
    /// Create undefined value
    pub fn undefined() -> Self {
        JsValue::Undefined
    }

    /// Create null value
    pub fn null() -> Self {
        JsValue::Null
    }

    /// Create boolean value
    pub fn boolean(v: bool) -> Self {
        JsValue::Boolean(v)
    }

    /// Create number value
    pub fn number(v: f64) -> Self {
        JsValue::Number(v)
    }

    /// Create string value
    pub fn string(s: impl Into<String>) -> Self {
        JsValue::String(s.into())
    }

    /// Create empty object
    pub fn object() -> Self {
        JsValue::Object(Rc::new(RefCell::new(ObjectData {
            properties: HashMap::new(),
            prototype: None,
        })))
    }

    /// Create object with prototype
    pub fn object_with_proto(proto: &JsValue) -> Self {
        JsValue::Object(Rc::new(RefCell::new(ObjectData {
            properties: HashMap::new(),
            prototype: Some(Box::new(proto.clone())),
        })))
    }

    /// Create empty array
    pub fn array() -> Self {
        JsValue::Array(Rc::new(RefCell::new(ArrayData {
            elements: Vec::new(),
        })))
    }

    /// Create array from values
    pub fn array_from(values: Vec<JsValue>) -> Self {
        JsValue::Array(Rc::new(RefCell::new(ArrayData { elements: values })))
    }

    /// Check if value is undefined
    pub fn is_undefined(&self) -> bool {
        matches!(self, JsValue::Undefined)
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsValue::Null)
    }

    /// Check if value is boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, JsValue::Boolean(_))
    }

    /// Check if value is number
    pub fn is_number(&self) -> bool {
        matches!(self, JsValue::Number(_))
    }

    /// Check if value is string
    pub fn is_string(&self) -> bool {
        matches!(self, JsValue::String(_))
    }

    /// Check if value is object
    pub fn is_object(&self) -> bool {
        matches!(self, JsValue::Object(_))
    }

    /// Check if value is array
    pub fn is_array(&self) -> bool {
        matches!(self, JsValue::Array(_))
    }

    /// Get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            JsValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<String> {
        match self {
            JsValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get array length
    pub fn array_length(&self) -> usize {
        match self {
            JsValue::Array(arr) => arr.borrow().elements.len(),
            _ => 0,
        }
    }

    /// Set object property
    pub fn set(&self, key: &str, value: JsValue) {
        if let JsValue::Object(obj) = self {
            obj.borrow_mut().properties.insert(key.to_string(), value);
        }
    }

    /// Get object property
    pub fn get(&self, key: &str) -> Option<JsValue> {
        match self {
            JsValue::Object(obj) => obj.borrow().properties.get(key).cloned(),
            _ => None,
        }
    }

    /// Check if object has own property
    pub fn has_own(&self, key: &str) -> bool {
        match self {
            JsValue::Object(obj) => obj.borrow().properties.contains_key(key),
            _ => false,
        }
    }

    /// Get object prototype
    pub fn get_prototype(&self) -> Option<JsValue> {
        match self {
            JsValue::Object(obj) => obj.borrow().prototype.as_ref().map(|p| (**p).clone()),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_js_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    }
                } else if *n == n.trunc() && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let elements: Vec<String> = arr
                    .borrow()
                    .elements
                    .iter()
                    .map(|e| e.to_js_string())
                    .collect();
                elements.join(",")
            }
        }
    }

    /// Check equality (loose comparison)
    pub fn equals(&self, other: &JsValue) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => {
                if a.is_nan() && b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (JsValue::String(a), JsValue::String(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined() {
        let v = JsValue::undefined();
        assert!(v.is_undefined());
        assert_eq!(v.to_js_string(), "undefined");
    }

    #[test]
    fn test_null() {
        let v = JsValue::null();
        assert!(v.is_null());
        assert_eq!(v.to_js_string(), "null");
    }

    #[test]
    fn test_boolean() {
        let v = JsValue::boolean(true);
        assert!(v.is_boolean());
        assert_eq!(v.as_boolean(), Some(true));
        assert_eq!(v.to_js_string(), "true");
    }

    #[test]
    fn test_number() {
        let v = JsValue::number(42.0);
        assert!(v.is_number());
        assert_eq!(v.as_number(), Some(42.0));
        assert_eq!(v.to_js_string(), "42");
    }

    #[test]
    fn test_string() {
        let v = JsValue::string("hello");
        assert!(v.is_string());
        assert_eq!(v.as_string(), Some("hello".to_string()));
        assert_eq!(v.to_js_string(), "hello");
    }

    #[test]
    fn test_object() {
        let obj = JsValue::object();
        assert!(obj.is_object());
        obj.set("foo", JsValue::number(42.0));
        assert_eq!(obj.get("foo"), Some(JsValue::number(42.0)));
        assert!(obj.has_own("foo"));
        assert!(!obj.has_own("bar"));
    }

    #[test]
    fn test_array() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        assert!(arr.is_array());
        assert_eq!(arr.array_length(), 2);
    }
}
