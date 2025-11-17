//! Object.prototype methods

use crate::value::{JsResult, JsValue};

/// Object.prototype methods
pub struct ObjectPrototype;

impl ObjectPrototype {
    /// Object.prototype.hasOwnProperty(prop)
    pub fn has_own_property(obj: &JsValue, prop: &str) -> JsResult<JsValue> {
        Ok(JsValue::boolean(obj.has_own(prop)))
    }

    /// Object.prototype.isPrototypeOf(obj)
    pub fn is_prototype_of(proto: &JsValue, obj: &JsValue) -> JsResult<JsValue> {
        // Check if proto is in the prototype chain of obj
        let mut current = obj.get_prototype();
        while let Some(p) = current {
            // Check if same object (by reference equality for objects)
            if std::ptr::eq(proto as *const JsValue, &p as *const JsValue) {
                return Ok(JsValue::boolean(true));
            }
            // For Rc-based objects, check pointer equality
            if let (JsValue::Object(a), JsValue::Object(b)) = (proto, &p) {
                if Rc::ptr_eq(a, b) {
                    return Ok(JsValue::boolean(true));
                }
            }
            current = p.get_prototype();
        }
        Ok(JsValue::boolean(false))
    }

    /// Object.prototype.toString()
    pub fn to_string(obj: &JsValue) -> JsResult<JsValue> {
        let type_tag = match obj {
            JsValue::Undefined => "Undefined",
            JsValue::Null => "Null",
            JsValue::Boolean(_) => "Boolean",
            JsValue::Number(_) => "Number",
            JsValue::String(_) => "String",
            JsValue::Symbol(_) => "Symbol",
            JsValue::Object(_) => "Object",
            JsValue::Array(_) => "Array",
            JsValue::Map(_) => "Map",
            JsValue::Set(_) => "Set",
            JsValue::Error(_) => "Error",
            JsValue::RegExp(_) => "RegExp",
            JsValue::Function(_) => "Function",
            JsValue::Constructor(_) => "Function",
            JsValue::Proxy(_) => "Object",
            JsValue::WeakMap(_) => "WeakMap",
            JsValue::WeakSet(_) => "WeakSet",
            JsValue::Generator(_) => "Generator",
            JsValue::AsyncGenerator(_) => "AsyncGenerator",
            JsValue::BigInt(_) => "BigInt",
            JsValue::WeakRef(_) => "WeakRef",
            JsValue::FinalizationRegistry(_) => "FinalizationRegistry",
        };
        Ok(JsValue::string(format!("[object {}]", type_tag)))
    }

    /// Object.prototype.valueOf()
    pub fn value_of(obj: &JsValue) -> JsResult<JsValue> {
        // By default, valueOf returns the object itself
        Ok(obj.clone())
    }
}

use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_own_property_true() {
        let obj = JsValue::object();
        obj.set("foo", JsValue::number(42.0));

        let result = ObjectPrototype::has_own_property(&obj, "foo").unwrap();
        assert_eq!(result, JsValue::boolean(true));
    }

    #[test]
    fn test_has_own_property_false() {
        let obj = JsValue::object();

        let result = ObjectPrototype::has_own_property(&obj, "bar").unwrap();
        assert_eq!(result, JsValue::boolean(false));
    }

    #[test]
    fn test_to_string_object() {
        let obj = JsValue::object();
        let result = ObjectPrototype::to_string(&obj).unwrap();
        assert_eq!(result.as_string().unwrap(), "[object Object]");
    }

    #[test]
    fn test_to_string_array() {
        let arr = JsValue::array();
        let result = ObjectPrototype::to_string(&arr).unwrap();
        assert_eq!(result.as_string().unwrap(), "[object Array]");
    }

    #[test]
    fn test_value_of() {
        let obj = JsValue::object();
        let result = ObjectPrototype::value_of(&obj).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn test_is_prototype_of() {
        let proto = JsValue::object();
        let obj = JsValue::object_with_proto(&proto);

        let result = ObjectPrototype::is_prototype_of(&proto, &obj).unwrap();
        assert_eq!(result, JsValue::boolean(true));
    }
}
