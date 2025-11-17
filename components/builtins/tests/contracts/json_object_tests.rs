//! Contract tests for JSONObject

use builtins::{JSONObject, JsValue};

#[test]
fn test_json_parse_number() {
    let result = JSONObject::parse("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_number().unwrap(), 42.0);
}

#[test]
fn test_json_parse_string() {
    let result = JSONObject::parse(r#""hello""#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_string().unwrap(), "hello");
}

#[test]
fn test_json_parse_boolean() {
    let result = JSONObject::parse("true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_boolean().unwrap(), true);
}

#[test]
fn test_json_parse_null() {
    let result = JSONObject::parse("null");
    assert!(result.is_ok());
    assert!(result.unwrap().is_null());
}

#[test]
fn test_json_parse_array() {
    let result = JSONObject::parse("[1, 2, 3]");
    assert!(result.is_ok());
    assert!(result.unwrap().is_array());
}

#[test]
fn test_json_parse_object() {
    let result = JSONObject::parse(r#"{"key": "value"}"#);
    assert!(result.is_ok());
    assert!(result.unwrap().is_object());
}

#[test]
fn test_json_parse_invalid() {
    let result = JSONObject::parse("invalid json");
    assert!(result.is_err());
}

#[test]
fn test_json_stringify_number() {
    let val = JsValue::number(42.0);
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_json_stringify_string() {
    let val = JsValue::string("hello");
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#""hello""#);
}

#[test]
fn test_json_stringify_boolean() {
    let val = JsValue::boolean(true);
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_json_stringify_null() {
    let val = JsValue::null();
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_json_stringify_array() {
    let val = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[1,2]");
}

#[test]
fn test_json_stringify_undefined() {
    let val = JsValue::undefined();
    let result = JSONObject::stringify(&val);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "undefined");
}
