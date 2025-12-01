//! JavaScript value representation using tagged pointers.
//!
//! This module provides the core `Value` enum that represents all possible
//! JavaScript values using a tagged pointer scheme for efficiency.

use num_bigint::BigInt;
use num_traits::Zero;
use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// Represents any JavaScript value.
///
/// This enum uses a tagged representation for efficient value handling.
/// Primitive values are stored inline, while objects are referenced by ID.
///
/// # Tagged Pointer Representation
///
/// In a full implementation, this would use actual tagged pointers:
/// - Bit 0 = 1: Small integer (Smi)
/// - Bit 0 = 0: Heap object pointer
///
/// For safety and portability, this implementation uses an enum with
/// explicit variants.
///
/// # Examples
///
/// ```
/// use core_types::Value;
///
/// let undefined = Value::Undefined;
/// let number = Value::Smi(42);
/// let float = Value::Double(3.14);
///
/// assert!(!undefined.is_truthy());
/// assert!(number.is_truthy());
/// assert_eq!(number.type_of(), "number");
/// ```
#[derive(Clone)]
pub enum Value {
    /// JavaScript undefined value
    Undefined,
    /// JavaScript null value
    Null,
    /// JavaScript boolean (true or false)
    Boolean(bool),
    /// Small integer (fits in 32 bits, tagged representation)
    Smi(i32),
    /// Heap-allocated object (referenced by ID for safety)
    HeapObject(usize),
    /// IEEE 754 double-precision floating point
    Double(f64),
    /// JavaScript string value
    String(std::string::String),
    /// Native object (console, Math, etc.)
    NativeObject(Rc<RefCell<dyn Any>>),
    /// Native function reference by name
    NativeFunction(std::string::String),
    /// JavaScript BigInt (arbitrary precision integer)
    BigInt(BigInt),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "Undefined"),
            Value::Null => write!(f, "Null"),
            Value::Boolean(b) => f.debug_tuple("Boolean").field(b).finish(),
            Value::Smi(n) => f.debug_tuple("Smi").field(n).finish(),
            Value::HeapObject(id) => f.debug_tuple("HeapObject").field(id).finish(),
            Value::Double(n) => f.debug_tuple("Double").field(n).finish(),
            Value::String(s) => f.debug_tuple("String").field(s).finish(),
            Value::NativeObject(_) => write!(f, "NativeObject(...)"),
            Value::NativeFunction(name) => f.debug_tuple("NativeFunction").field(name).finish(),
            Value::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Smi(a), Value::Smi(b)) => a == b,
            (Value::HeapObject(a), Value::HeapObject(b)) => a == b,
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::NativeObject(a), Value::NativeObject(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => a == b,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    /// Returns whether this value is truthy in JavaScript semantics.
    ///
    /// In JavaScript, the following values are falsy:
    /// - undefined
    /// - null
    /// - false
    /// - 0 (including -0)
    /// - NaN
    /// - "" (empty string, not applicable here)
    ///
    /// All other values are truthy, including all objects.
    ///
    /// # Examples
    ///
    /// ```
    /// use core_types::Value;
    ///
    /// assert!(!Value::Undefined.is_truthy());
    /// assert!(!Value::Null.is_truthy());
    /// assert!(!Value::Boolean(false).is_truthy());
    /// assert!(!Value::Smi(0).is_truthy());
    /// assert!(!Value::Double(f64::NAN).is_truthy());
    ///
    /// assert!(Value::Boolean(true).is_truthy());
    /// assert!(Value::Smi(42).is_truthy());
    /// assert!(Value::HeapObject(0).is_truthy());
    /// ```
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Smi(n) => *n != 0,
            Value::Double(n) => !n.is_nan() && *n != 0.0,
            Value::HeapObject(_) => true, // All objects are truthy
            Value::String(s) => !s.is_empty(), // Empty string is falsy
            Value::NativeObject(_) => true, // Native objects are truthy
            Value::NativeFunction(_) => true, // Functions are truthy
            Value::BigInt(n) => !n.is_zero(), // 0n is falsy
        }
    }

    /// Returns the JavaScript typeof result for this value.
    ///
    /// This follows JavaScript's `typeof` operator behavior:
    /// - undefined → "undefined"
    /// - null → "object" (historical quirk)
    /// - boolean → "boolean"
    /// - number (Smi or Double) → "number"
    /// - object → "object"
    ///
    /// Note: Functions would return "function", but this simplified
    /// implementation treats all heap objects as generic objects.
    ///
    /// # Examples
    ///
    /// ```
    /// use core_types::Value;
    ///
    /// assert_eq!(Value::Undefined.type_of(), "undefined");
    /// assert_eq!(Value::Null.type_of(), "object");
    /// assert_eq!(Value::Boolean(true).type_of(), "boolean");
    /// assert_eq!(Value::Smi(42).type_of(), "number");
    /// ```
    pub fn type_of(&self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "object".to_string(), // JavaScript quirk
            Value::Boolean(_) => "boolean".to_string(),
            Value::Smi(_) => "number".to_string(),
            Value::Double(_) => "number".to_string(),
            Value::HeapObject(_) => "object".to_string(),
            Value::String(_) => "string".to_string(),
            Value::NativeObject(_) => "object".to_string(),
            Value::NativeFunction(_) => "function".to_string(),
            Value::BigInt(_) => "bigint".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_variants() {
        let _undef = Value::Undefined;
        let _null = Value::Null;
        let _bool = Value::Boolean(true);
        let _smi = Value::Smi(42);
        let _heap = Value::HeapObject(0);
        let _double = Value::Double(3.14);
    }

    #[test]
    fn test_is_truthy_basic() {
        assert!(!Value::Undefined.is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
    }

    #[test]
    fn test_to_string_basic() {
        assert_eq!(Value::Undefined.to_string(), "undefined");
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_type_of_basic() {
        assert_eq!(Value::Undefined.type_of(), "undefined");
        assert_eq!(Value::Null.type_of(), "object");
    }
}

/// Implementation of Display trait for JavaScript string conversion.
///
/// This follows JavaScript's `String()` conversion rules:
/// - undefined → "undefined"
/// - null → "null"
/// - boolean → "true" or "false"
/// - number → decimal representation
/// - object → "[object Object]" (simplified)
///
/// # Examples
///
/// ```
/// use core_types::Value;
///
/// assert_eq!(Value::Undefined.to_string(), "undefined");
/// assert_eq!(Value::Null.to_string(), "null");
/// assert_eq!(Value::Boolean(true).to_string(), "true");
/// assert_eq!(Value::Smi(42).to_string(), "42");
/// ```
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Smi(n) => write!(f, "{}", n),
            Value::Double(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if n.is_infinite() {
                    if n.is_sign_positive() {
                        write!(f, "Infinity")
                    } else {
                        write!(f, "-Infinity")
                    }
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    // Integer-valued doubles display without decimal point
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::HeapObject(_) => write!(f, "[object Object]"),
            Value::String(s) => write!(f, "{}", s),
            Value::NativeObject(_) => write!(f, "[object Object]"),
            Value::NativeFunction(name) => write!(f, "function {}() {{ [native code] }}", name),
            Value::BigInt(n) => write!(f, "{}n", n),
        }
    }
}
