//! Reflect object implementation per ES2024
//!
//! The Reflect object provides methods for interceptable JavaScript operations.
//! All methods are static and match the proxy handler traps.

use crate::proxy::PropertyDescriptor;
use crate::value::{JsError, JsResult, JsValue};

/// Reflect object - provides static methods for object operations
pub struct ReflectObject;

impl ReflectObject {
    /// Reflect.get(target, propertyKey [, receiver])
    ///
    /// Gets the value of a property on an object.
    pub fn get(target: &JsValue, key: &str, _receiver: Option<&JsValue>) -> JsResult<JsValue> {
        Self::validate_object(target, "Reflect.get")?;

        match target {
            JsValue::Object(obj) => {
                let borrowed = obj.borrow();
                Ok(borrowed
                    .properties
                    .get(key)
                    .cloned()
                    .unwrap_or(JsValue::undefined()))
            }
            JsValue::Array(arr) => {
                if key == "length" {
                    Ok(JsValue::number(arr.borrow().elements.len() as f64))
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(arr
                        .borrow()
                        .elements
                        .get(index)
                        .cloned()
                        .unwrap_or(JsValue::undefined()))
                } else {
                    Ok(JsValue::undefined())
                }
            }
            _ => Ok(JsValue::undefined()),
        }
    }

    /// Reflect.set(target, propertyKey, value [, receiver])
    ///
    /// Sets the value of a property on an object.
    pub fn set(
        target: &JsValue,
        key: &str,
        value: JsValue,
        _receiver: Option<&JsValue>,
    ) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.set")?;

        match target {
            JsValue::Object(obj) => {
                // Check if object is extensible (if adding new property)
                let is_new = !obj.borrow().properties.contains_key(key);
                if is_new && !obj.borrow().extensible.unwrap_or(true) {
                    return Ok(false);
                }
                obj.borrow_mut().properties.insert(key.to_string(), value);
                Ok(true)
            }
            JsValue::Array(arr) => {
                if let Ok(index) = key.parse::<usize>() {
                    let mut borrowed = arr.borrow_mut();
                    if index >= borrowed.elements.len() {
                        borrowed.elements.resize(index + 1, JsValue::undefined());
                    }
                    borrowed.elements[index] = value;
                    Ok(true)
                } else if key == "length" {
                    if let Some(len) = value.as_number() {
                        let new_len = len as usize;
                        arr.borrow_mut().elements.resize(new_len, JsValue::undefined());
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    /// Reflect.has(target, propertyKey)
    ///
    /// Returns a Boolean indicating whether the target has the property.
    pub fn has(target: &JsValue, key: &str) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.has")?;

        match target {
            JsValue::Object(obj) => {
                // Check own properties first
                if obj.borrow().properties.contains_key(key) {
                    return Ok(true);
                }
                // Check prototype chain
                let proto = obj.borrow().prototype.clone();
                if let Some(p) = proto {
                    Self::has(&p, key)
                } else {
                    Ok(false)
                }
            }
            JsValue::Array(arr) => {
                if key == "length" {
                    Ok(true)
                } else if let Ok(index) = key.parse::<usize>() {
                    Ok(index < arr.borrow().elements.len())
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    /// Reflect.deleteProperty(target, propertyKey)
    ///
    /// Deletes a property from an object.
    pub fn delete_property(target: &JsValue, key: &str) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.deleteProperty")?;

        match target {
            JsValue::Object(obj) => {
                obj.borrow_mut().properties.remove(key);
                Ok(true)
            }
            JsValue::Array(arr) => {
                if let Ok(index) = key.parse::<usize>() {
                    let mut borrowed = arr.borrow_mut();
                    if index < borrowed.elements.len() {
                        borrowed.elements[index] = JsValue::undefined();
                    }
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    /// Reflect.ownKeys(target)
    ///
    /// Returns an array of the target object's own property keys.
    pub fn own_keys(target: &JsValue) -> JsResult<Vec<String>> {
        Self::validate_object(target, "Reflect.ownKeys")?;

        match target {
            JsValue::Object(obj) => {
                let keys: Vec<String> = obj.borrow().properties.keys().cloned().collect();
                Ok(keys)
            }
            JsValue::Array(arr) => {
                let len = arr.borrow().elements.len();
                let mut keys: Vec<String> = (0..len).map(|i| i.to_string()).collect();
                keys.push("length".to_string());
                Ok(keys)
            }
            _ => Ok(vec![]),
        }
    }

    /// Reflect.getPrototypeOf(target)
    ///
    /// Returns the prototype of the target object.
    pub fn get_prototype_of(target: &JsValue) -> JsResult<Option<JsValue>> {
        Self::validate_object(target, "Reflect.getPrototypeOf")?;

        match target {
            JsValue::Object(obj) => Ok(obj.borrow().prototype.as_ref().map(|p| (**p).clone())),
            _ => Ok(None),
        }
    }

    /// Reflect.setPrototypeOf(target, prototype)
    ///
    /// Sets the prototype of the target object.
    pub fn set_prototype_of(target: &JsValue, proto: Option<&JsValue>) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.setPrototypeOf")?;

        match target {
            JsValue::Object(obj) => {
                obj.borrow_mut().prototype = proto.map(|p| Box::new(p.clone()));
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Reflect.isExtensible(target)
    ///
    /// Returns a Boolean indicating whether the target is extensible.
    pub fn is_extensible(target: &JsValue) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.isExtensible")?;

        match target {
            JsValue::Object(obj) => Ok(obj.borrow().extensible.unwrap_or(true)),
            _ => Ok(false),
        }
    }

    /// Reflect.preventExtensions(target)
    ///
    /// Prevents new properties from ever being added to the target object.
    pub fn prevent_extensions(target: &JsValue) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.preventExtensions")?;

        match target {
            JsValue::Object(obj) => {
                obj.borrow_mut().extensible = Some(false);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Reflect.defineProperty(target, propertyKey, attributes)
    ///
    /// Defines a new property directly on an object.
    pub fn define_property(
        target: &JsValue,
        key: &str,
        descriptor: &PropertyDescriptor,
    ) -> JsResult<bool> {
        Self::validate_object(target, "Reflect.defineProperty")?;

        match target {
            JsValue::Object(obj) => {
                // Check if object is extensible (if adding new property)
                let is_new = !obj.borrow().properties.contains_key(key);
                if is_new && !obj.borrow().extensible.unwrap_or(true) {
                    return Ok(false);
                }

                if let Some(value) = &descriptor.value {
                    obj.borrow_mut()
                        .properties
                        .insert(key.to_string(), value.clone());
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Reflect.getOwnPropertyDescriptor(target, propertyKey)
    ///
    /// Returns a property descriptor for the given property.
    pub fn get_own_property_descriptor(
        target: &JsValue,
        key: &str,
    ) -> JsResult<Option<PropertyDescriptor>> {
        Self::validate_object(target, "Reflect.getOwnPropertyDescriptor")?;

        match target {
            JsValue::Object(obj) => {
                if let Some(value) = obj.borrow().properties.get(key) {
                    Ok(Some(PropertyDescriptor {
                        value: Some(value.clone()),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                        get: None,
                        set: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            JsValue::Array(arr) => {
                if key == "length" {
                    Ok(Some(PropertyDescriptor {
                        value: Some(JsValue::number(arr.borrow().elements.len() as f64)),
                        writable: Some(true),
                        enumerable: Some(false),
                        configurable: Some(false),
                        get: None,
                        set: None,
                    }))
                } else if let Ok(index) = key.parse::<usize>() {
                    if let Some(value) = arr.borrow().elements.get(index) {
                        Ok(Some(PropertyDescriptor {
                            value: Some(value.clone()),
                            writable: Some(true),
                            enumerable: Some(true),
                            configurable: Some(true),
                            get: None,
                            set: None,
                        }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Reflect.apply(target, thisArgument, argumentsList)
    ///
    /// Calls a target function with arguments as specified.
    pub fn apply(func: &JsValue, this_arg: &JsValue, args: &[JsValue]) -> JsResult<JsValue> {
        match func {
            JsValue::Function(f) => (f.borrow().func)(this_arg.clone(), args.to_vec()),
            _ => Err(JsError::type_error(
                "Reflect.apply called on non-function",
            )),
        }
    }

    /// Reflect.construct(target, argumentsList [, newTarget])
    ///
    /// Calls a constructor function with arguments.
    pub fn construct(
        target: &JsValue,
        args: &[JsValue],
        _new_target: Option<&JsValue>,
    ) -> JsResult<JsValue> {
        match target {
            JsValue::Constructor(c) => (c.borrow().func)(args.to_vec()),
            _ => Err(JsError::type_error(
                "Reflect.construct called on non-constructor",
            )),
        }
    }

    /// Helper: Validate that target is an object
    fn validate_object(target: &JsValue, method: &str) -> JsResult<()> {
        match target {
            JsValue::Object(_)
            | JsValue::Array(_)
            | JsValue::Function(_)
            | JsValue::Constructor(_)
            | JsValue::Proxy(_) => Ok(()),
            _ => Err(JsError::type_error(format!(
                "{} called on non-object",
                method
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflect_get_existing_property() {
        let obj = JsValue::object();
        obj.set("key", JsValue::number(42.0));

        let result = ReflectObject::get(&obj, "key", None).unwrap();
        assert_eq!(result.as_number(), Some(42.0));
    }

    #[test]
    fn test_reflect_get_missing_property() {
        let obj = JsValue::object();

        let result = ReflectObject::get(&obj, "missing", None).unwrap();
        assert!(result.is_undefined());
    }

    #[test]
    fn test_reflect_set_new_property() {
        let obj = JsValue::object();

        let success = ReflectObject::set(&obj, "new", JsValue::string("value"), None).unwrap();
        assert!(success);
        assert_eq!(
            obj.get("new").unwrap().as_string(),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_reflect_set_existing_property() {
        let obj = JsValue::object();
        obj.set("existing", JsValue::number(1.0));

        let success = ReflectObject::set(&obj, "existing", JsValue::number(2.0), None).unwrap();
        assert!(success);
        assert_eq!(obj.get("existing").unwrap().as_number(), Some(2.0));
    }

    #[test]
    fn test_reflect_has_own_property() {
        let obj = JsValue::object();
        obj.set("exists", JsValue::boolean(true));

        assert!(ReflectObject::has(&obj, "exists").unwrap());
        assert!(!ReflectObject::has(&obj, "missing").unwrap());
    }

    #[test]
    fn test_reflect_delete_property() {
        let obj = JsValue::object();
        obj.set("toDelete", JsValue::number(1.0));

        let success = ReflectObject::delete_property(&obj, "toDelete").unwrap();
        assert!(success);
        assert!(!obj.has_own("toDelete"));
    }

    #[test]
    fn test_reflect_own_keys() {
        let obj = JsValue::object();
        obj.set("a", JsValue::number(1.0));
        obj.set("b", JsValue::number(2.0));

        let keys = ReflectObject::own_keys(&obj).unwrap();
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_reflect_get_prototype_of() {
        let proto = JsValue::object();
        let obj = JsValue::object_with_proto(&proto);

        let result = ReflectObject::get_prototype_of(&obj).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_reflect_set_prototype_of() {
        let obj = JsValue::object();
        let new_proto = JsValue::object();

        let success = ReflectObject::set_prototype_of(&obj, Some(&new_proto)).unwrap();
        assert!(success);

        let proto = ReflectObject::get_prototype_of(&obj).unwrap();
        assert!(proto.is_some());
    }

    #[test]
    fn test_reflect_is_extensible() {
        let obj = JsValue::object();

        assert!(ReflectObject::is_extensible(&obj).unwrap());
    }

    #[test]
    fn test_reflect_prevent_extensions() {
        let obj = JsValue::object();

        let success = ReflectObject::prevent_extensions(&obj).unwrap();
        assert!(success);
        assert!(!ReflectObject::is_extensible(&obj).unwrap());
    }

    #[test]
    fn test_reflect_define_property() {
        let obj = JsValue::object();
        let desc = PropertyDescriptor {
            value: Some(JsValue::number(100.0)),
            writable: Some(true),
            enumerable: Some(true),
            configurable: Some(true),
            get: None,
            set: None,
        };

        let success = ReflectObject::define_property(&obj, "defined", &desc).unwrap();
        assert!(success);
        assert_eq!(obj.get("defined").unwrap().as_number(), Some(100.0));
    }

    #[test]
    fn test_reflect_get_own_property_descriptor() {
        let obj = JsValue::object();
        obj.set("prop", JsValue::string("test"));

        let desc = ReflectObject::get_own_property_descriptor(&obj, "prop").unwrap();
        assert!(desc.is_some());
        let desc = desc.unwrap();
        assert_eq!(
            desc.value.unwrap().as_string(),
            Some("test".to_string())
        );
        assert_eq!(desc.writable, Some(true));
    }

    #[test]
    fn test_reflect_get_own_property_descriptor_missing() {
        let obj = JsValue::object();

        let desc = ReflectObject::get_own_property_descriptor(&obj, "missing").unwrap();
        assert!(desc.is_none());
    }

    #[test]
    fn test_reflect_type_error_on_non_object() {
        let num = JsValue::number(42.0);

        assert!(ReflectObject::get(&num, "foo", None).is_err());
        assert!(ReflectObject::set(&num, "foo", JsValue::null(), None).is_err());
        assert!(ReflectObject::has(&num, "foo").is_err());
    }
}
