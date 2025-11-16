//! Contract tests for NumberPrototype

use builtins::{NumberPrototype, JsValue};

#[test]
fn test_number_to_string() {
    let result = NumberPrototype::to_string(42.0, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_number_to_string_radix() {
    let result = NumberPrototype::to_string(255.0, Some(16));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ff");
}

#[test]
fn test_number_to_fixed() {
    let result = NumberPrototype::to_fixed(3.14159, 2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3.14");
}

#[test]
fn test_number_to_precision() {
    let result = NumberPrototype::to_precision(123.456, 4);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "123.5");
}

#[test]
fn test_number_value_of() {
    let val = JsValue::number(42.0);
    let result = NumberPrototype::value_of(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42.0);
}
