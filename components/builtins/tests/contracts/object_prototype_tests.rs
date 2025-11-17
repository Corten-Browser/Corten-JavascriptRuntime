//! Contract tests for ObjectPrototype

use builtins::{ObjectPrototype, JsValue, JsResult};

#[test]
fn test_has_own_property_exists() {
    let obj = JsValue::object();
    obj.set("foo", JsValue::number(42.0));

    let result = ObjectPrototype::has_own_property(&obj, "foo");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::boolean(true));
}

#[test]
fn test_has_own_property_not_exists() {
    let obj = JsValue::object();

    let result = ObjectPrototype::has_own_property(&obj, "bar");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::boolean(false));
}

#[test]
fn test_is_prototype_of() {
    let proto = JsValue::object();
    let obj = JsValue::object_with_proto(&proto);

    let result = ObjectPrototype::is_prototype_of(&proto, &obj);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::boolean(true));
}

#[test]
fn test_to_string() {
    let obj = JsValue::object();

    let result = ObjectPrototype::to_string(&obj);
    assert!(result.is_ok());
    let s = result.unwrap().as_string().unwrap();
    assert!(s.contains("object"));
}

#[test]
fn test_value_of() {
    let obj = JsValue::object();

    let result = ObjectPrototype::value_of(&obj);
    assert!(result.is_ok());
    // valueOf returns the object itself by default
    assert!(result.unwrap().is_object());
}
