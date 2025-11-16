//! Unit tests for Value enum
//!
//! Following TDD: These tests are written FIRST before implementation.

use core_types::Value;

#[cfg(test)]
mod value_creation_tests {
    use super::*;

    #[test]
    fn test_value_undefined() {
        let val = Value::Undefined;
        assert!(matches!(val, Value::Undefined));
    }

    #[test]
    fn test_value_null() {
        let val = Value::Null;
        assert!(matches!(val, Value::Null));
    }

    #[test]
    fn test_value_boolean_true() {
        let val = Value::Boolean(true);
        assert!(matches!(val, Value::Boolean(true)));
    }

    #[test]
    fn test_value_boolean_false() {
        let val = Value::Boolean(false);
        assert!(matches!(val, Value::Boolean(false)));
    }

    #[test]
    fn test_value_smi_positive() {
        let val = Value::Smi(42);
        assert!(matches!(val, Value::Smi(42)));
    }

    #[test]
    fn test_value_smi_negative() {
        let val = Value::Smi(-100);
        assert!(matches!(val, Value::Smi(-100)));
    }

    #[test]
    fn test_value_smi_zero() {
        let val = Value::Smi(0);
        assert!(matches!(val, Value::Smi(0)));
    }

    #[test]
    fn test_value_smi_max() {
        let val = Value::Smi(i32::MAX);
        assert!(matches!(val, Value::Smi(n) if n == i32::MAX));
    }

    #[test]
    fn test_value_smi_min() {
        let val = Value::Smi(i32::MIN);
        assert!(matches!(val, Value::Smi(n) if n == i32::MIN));
    }

    #[test]
    fn test_value_double_positive() {
        let val = Value::Double(3.14);
        assert!(matches!(val, Value::Double(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_value_double_negative() {
        let val = Value::Double(-2.71);
        assert!(matches!(val, Value::Double(n) if (n - (-2.71)).abs() < f64::EPSILON));
    }

    #[test]
    fn test_value_double_zero() {
        let val = Value::Double(0.0);
        assert!(matches!(val, Value::Double(n) if n == 0.0));
    }

    #[test]
    fn test_value_double_infinity() {
        let val = Value::Double(f64::INFINITY);
        assert!(matches!(val, Value::Double(n) if n.is_infinite() && n.is_sign_positive()));
    }

    #[test]
    fn test_value_double_neg_infinity() {
        let val = Value::Double(f64::NEG_INFINITY);
        assert!(matches!(val, Value::Double(n) if n.is_infinite() && n.is_sign_negative()));
    }

    #[test]
    fn test_value_double_nan() {
        let val = Value::Double(f64::NAN);
        assert!(matches!(val, Value::Double(n) if n.is_nan()));
    }

    #[test]
    fn test_value_heap_object() {
        let val = Value::HeapObject(42); // Using usize as object ID
        assert!(matches!(val, Value::HeapObject(42)));
    }

    #[test]
    fn test_value_heap_object_zero() {
        let val = Value::HeapObject(0);
        assert!(matches!(val, Value::HeapObject(0)));
    }
}

#[cfg(test)]
mod value_is_truthy_tests {
    use super::*;

    #[test]
    fn test_undefined_is_falsy() {
        assert!(!Value::Undefined.is_truthy());
    }

    #[test]
    fn test_null_is_falsy() {
        assert!(!Value::Null.is_truthy());
    }

    #[test]
    fn test_boolean_true_is_truthy() {
        assert!(Value::Boolean(true).is_truthy());
    }

    #[test]
    fn test_boolean_false_is_falsy() {
        assert!(!Value::Boolean(false).is_truthy());
    }

    #[test]
    fn test_smi_zero_is_falsy() {
        assert!(!Value::Smi(0).is_truthy());
    }

    #[test]
    fn test_smi_positive_is_truthy() {
        assert!(Value::Smi(1).is_truthy());
        assert!(Value::Smi(42).is_truthy());
        assert!(Value::Smi(i32::MAX).is_truthy());
    }

    #[test]
    fn test_smi_negative_is_truthy() {
        assert!(Value::Smi(-1).is_truthy());
        assert!(Value::Smi(-100).is_truthy());
        assert!(Value::Smi(i32::MIN).is_truthy());
    }

    #[test]
    fn test_double_zero_is_falsy() {
        assert!(!Value::Double(0.0).is_truthy());
    }

    #[test]
    fn test_double_negative_zero_is_falsy() {
        assert!(!Value::Double(-0.0).is_truthy());
    }

    #[test]
    fn test_double_nan_is_falsy() {
        assert!(!Value::Double(f64::NAN).is_truthy());
    }

    #[test]
    fn test_double_positive_is_truthy() {
        assert!(Value::Double(0.1).is_truthy());
        assert!(Value::Double(3.14).is_truthy());
        assert!(Value::Double(f64::INFINITY).is_truthy());
    }

    #[test]
    fn test_double_negative_is_truthy() {
        assert!(Value::Double(-0.1).is_truthy());
        assert!(Value::Double(-3.14).is_truthy());
        assert!(Value::Double(f64::NEG_INFINITY).is_truthy());
    }

    #[test]
    fn test_heap_object_is_truthy() {
        // All objects are truthy in JavaScript
        assert!(Value::HeapObject(0).is_truthy());
        assert!(Value::HeapObject(1).is_truthy());
        assert!(Value::HeapObject(usize::MAX).is_truthy());
    }
}

#[cfg(test)]
mod value_to_string_tests {
    use super::*;

    #[test]
    fn test_undefined_to_string() {
        assert_eq!(Value::Undefined.to_string(), "undefined");
    }

    #[test]
    fn test_null_to_string() {
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_boolean_true_to_string() {
        assert_eq!(Value::Boolean(true).to_string(), "true");
    }

    #[test]
    fn test_boolean_false_to_string() {
        assert_eq!(Value::Boolean(false).to_string(), "false");
    }

    #[test]
    fn test_smi_positive_to_string() {
        assert_eq!(Value::Smi(42).to_string(), "42");
    }

    #[test]
    fn test_smi_negative_to_string() {
        assert_eq!(Value::Smi(-100).to_string(), "-100");
    }

    #[test]
    fn test_smi_zero_to_string() {
        assert_eq!(Value::Smi(0).to_string(), "0");
    }

    #[test]
    fn test_double_integer_to_string() {
        assert_eq!(Value::Double(42.0).to_string(), "42");
    }

    #[test]
    fn test_double_decimal_to_string() {
        let s = Value::Double(3.14).to_string();
        assert!(s.contains("3.14"));
    }

    #[test]
    fn test_double_infinity_to_string() {
        assert_eq!(Value::Double(f64::INFINITY).to_string(), "Infinity");
    }

    #[test]
    fn test_double_neg_infinity_to_string() {
        assert_eq!(Value::Double(f64::NEG_INFINITY).to_string(), "-Infinity");
    }

    #[test]
    fn test_double_nan_to_string() {
        assert_eq!(Value::Double(f64::NAN).to_string(), "NaN");
    }

    #[test]
    fn test_heap_object_to_string() {
        // Objects typically return "[object Object]" or similar
        let s = Value::HeapObject(123).to_string();
        assert!(s.contains("object") || s.contains("Object"));
    }
}

#[cfg(test)]
mod value_type_of_tests {
    use super::*;

    #[test]
    fn test_undefined_type_of() {
        assert_eq!(Value::Undefined.type_of(), "undefined");
    }

    #[test]
    fn test_null_type_of() {
        // In JavaScript, typeof null === "object" (historical bug)
        assert_eq!(Value::Null.type_of(), "object");
    }

    #[test]
    fn test_boolean_type_of() {
        assert_eq!(Value::Boolean(true).type_of(), "boolean");
        assert_eq!(Value::Boolean(false).type_of(), "boolean");
    }

    #[test]
    fn test_smi_type_of() {
        assert_eq!(Value::Smi(0).type_of(), "number");
        assert_eq!(Value::Smi(42).type_of(), "number");
        assert_eq!(Value::Smi(-100).type_of(), "number");
    }

    #[test]
    fn test_double_type_of() {
        assert_eq!(Value::Double(3.14).type_of(), "number");
        assert_eq!(Value::Double(f64::NAN).type_of(), "number");
        assert_eq!(Value::Double(f64::INFINITY).type_of(), "number");
    }

    #[test]
    fn test_heap_object_type_of() {
        // Generic heap objects are "object" type
        assert_eq!(Value::HeapObject(0).type_of(), "object");
    }
}

#[cfg(test)]
mod value_clone_and_debug_tests {
    use super::*;

    #[test]
    fn test_value_clone() {
        let val1 = Value::Smi(42);
        let val2 = val1.clone();
        assert!(matches!(val2, Value::Smi(42)));
    }

    #[test]
    fn test_value_debug() {
        let val = Value::Boolean(true);
        let debug_str = format!("{:?}", val);
        assert!(debug_str.contains("Boolean") || debug_str.contains("true"));
    }

    #[test]
    fn test_value_partial_eq() {
        assert_eq!(Value::Undefined, Value::Undefined);
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_eq!(Value::Smi(42), Value::Smi(42));
        assert_ne!(Value::Smi(1), Value::Smi(2));
    }
}
