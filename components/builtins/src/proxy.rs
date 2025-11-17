//! Proxy object implementation per ES2024
//!
//! Proxy enables you to create a proxy for another object, which can intercept
//! and redefine fundamental operations for that object.

use crate::reflect::ReflectObject;
use crate::value::{JsError, JsResult, JsValue};
use std::cell::RefCell;
use std::rc::Rc;

/// Property descriptor for defineProperty and getOwnPropertyDescriptor
pub struct PropertyDescriptor {
    /// The value of the property
    pub value: Option<JsValue>,
    /// Whether the property value can be changed
    pub writable: Option<bool>,
    /// Whether the property shows up in for...in loops
    pub enumerable: Option<bool>,
    /// Whether the property can be deleted or attributes changed
    pub configurable: Option<bool>,
    /// Getter function
    pub get: Option<Box<dyn Fn() -> JsResult<JsValue>>>,
    /// Setter function
    pub set: Option<Box<dyn Fn(JsValue) -> JsResult<()>>>,
}

impl std::fmt::Debug for PropertyDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyDescriptor")
            .field("value", &self.value)
            .field("writable", &self.writable)
            .field("enumerable", &self.enumerable)
            .field("configurable", &self.configurable)
            .field("get", &self.get.as_ref().map(|_| "<function>"))
            .field("set", &self.set.as_ref().map(|_| "<function>"))
            .finish()
    }
}

impl Default for PropertyDescriptor {
    fn default() -> Self {
        PropertyDescriptor {
            value: None,
            writable: None,
            enumerable: None,
            configurable: None,
            get: None,
            set: None,
        }
    }
}

impl PropertyDescriptor {
    /// Check if this is a data descriptor
    pub fn is_data_descriptor(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }

    /// Check if this is an accessor descriptor
    pub fn is_accessor_descriptor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }
}

/// Type alias for get trap function
pub type GetTrap = Box<dyn Fn(&JsValue, &str, Option<&JsValue>) -> JsResult<JsValue>>;
/// Type alias for set trap function
pub type SetTrap = Box<dyn Fn(&JsValue, &str, JsValue, Option<&JsValue>) -> JsResult<bool>>;
/// Type alias for has trap function
pub type HasTrap = Box<dyn Fn(&JsValue, &str) -> JsResult<bool>>;
/// Type alias for deleteProperty trap function
pub type DeletePropertyTrap = Box<dyn Fn(&JsValue, &str) -> JsResult<bool>>;
/// Type alias for ownKeys trap function
pub type OwnKeysTrap = Box<dyn Fn(&JsValue) -> JsResult<Vec<String>>>;
/// Type alias for getPrototypeOf trap function
pub type GetPrototypeOfTrap = Box<dyn Fn(&JsValue) -> JsResult<Option<JsValue>>>;
/// Type alias for setPrototypeOf trap function
pub type SetPrototypeOfTrap = Box<dyn Fn(&JsValue, Option<&JsValue>) -> JsResult<bool>>;
/// Type alias for isExtensible trap function
pub type IsExtensibleTrap = Box<dyn Fn(&JsValue) -> JsResult<bool>>;
/// Type alias for preventExtensions trap function
pub type PreventExtensionsTrap = Box<dyn Fn(&JsValue) -> JsResult<bool>>;
/// Type alias for getOwnPropertyDescriptor trap function
pub type GetOwnPropertyDescriptorTrap =
    Box<dyn Fn(&JsValue, &str) -> JsResult<Option<PropertyDescriptor>>>;
/// Type alias for defineProperty trap function
pub type DefinePropertyTrap =
    Box<dyn Fn(&JsValue, &str, &PropertyDescriptor) -> JsResult<bool>>;
/// Type alias for apply trap function
pub type ApplyTrap = Box<dyn Fn(&JsValue, &JsValue, &[JsValue]) -> JsResult<JsValue>>;
/// Type alias for construct trap function
pub type ConstructTrap =
    Box<dyn Fn(&JsValue, &[JsValue], Option<&JsValue>) -> JsResult<JsValue>>;

/// Handler object containing trap functions
pub struct ProxyHandler {
    /// get(target, property, receiver)
    pub get: Option<GetTrap>,
    /// set(target, property, value, receiver)
    pub set: Option<SetTrap>,
    /// has(target, property)
    pub has: Option<HasTrap>,
    /// deleteProperty(target, property)
    pub delete_property: Option<DeletePropertyTrap>,
    /// ownKeys(target)
    pub own_keys: Option<OwnKeysTrap>,
    /// getPrototypeOf(target)
    pub get_prototype_of: Option<GetPrototypeOfTrap>,
    /// setPrototypeOf(target, prototype)
    pub set_prototype_of: Option<SetPrototypeOfTrap>,
    /// isExtensible(target)
    pub is_extensible: Option<IsExtensibleTrap>,
    /// preventExtensions(target)
    pub prevent_extensions: Option<PreventExtensionsTrap>,
    /// getOwnPropertyDescriptor(target, property)
    pub get_own_property_descriptor: Option<GetOwnPropertyDescriptorTrap>,
    /// defineProperty(target, property, descriptor)
    pub define_property: Option<DefinePropertyTrap>,
    /// apply(target, thisArg, argumentsList)
    pub apply: Option<ApplyTrap>,
    /// construct(target, argumentsList, newTarget)
    pub construct: Option<ConstructTrap>,
}

impl Default for ProxyHandler {
    fn default() -> Self {
        ProxyHandler {
            get: None,
            set: None,
            has: None,
            delete_property: None,
            own_keys: None,
            get_prototype_of: None,
            set_prototype_of: None,
            is_extensible: None,
            prevent_extensions: None,
            get_own_property_descriptor: None,
            define_property: None,
            apply: None,
            construct: None,
        }
    }
}

/// Internal proxy data
struct ProxyData {
    target: JsValue,
    handler: ProxyHandler,
    revoked: bool,
}

/// Proxy object
#[derive(Clone)]
pub struct ProxyObject {
    data: Rc<RefCell<ProxyData>>,
}

impl ProxyObject {
    /// Create a new Proxy object
    pub fn new(target: JsValue, handler: ProxyHandler) -> JsResult<Self> {
        Ok(ProxyObject {
            data: Rc::new(RefCell::new(ProxyData {
                target,
                handler,
                revoked: false,
            })),
        })
    }

    /// Create a revocable proxy
    pub fn revocable(
        target: JsValue,
        handler: ProxyHandler,
    ) -> JsResult<(Self, Box<dyn Fn()>)> {
        let proxy = ProxyObject::new(target, handler)?;
        let proxy_data = proxy.data.clone();

        let revoke = Box::new(move || {
            proxy_data.borrow_mut().revoked = true;
        });

        Ok((proxy, revoke))
    }

    /// Check if proxy has been revoked
    pub fn is_revoked(&self) -> bool {
        self.data.borrow().revoked
    }

    /// Helper to check if revoked and return error
    fn check_revoked(&self, operation: &str) -> JsResult<()> {
        if self.is_revoked() {
            Err(JsError::type_error(format!(
                "Cannot perform '{}' on a proxy that has been revoked",
                operation
            )))
        } else {
            Ok(())
        }
    }

    /// Get the target object
    pub fn target(&self) -> JsValue {
        self.data.borrow().target.clone()
    }

    /// [[Get]] internal method
    pub fn get(&self, key: &str) -> JsResult<JsValue> {
        self.check_revoked("get")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.get {
            trap(&data.target, key, None)
        } else {
            ReflectObject::get(&data.target, key, None)
        }
    }

    /// [[Set]] internal method
    pub fn set(&self, key: &str, value: JsValue) -> JsResult<bool> {
        self.check_revoked("set")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.set {
            trap(&data.target, key, value, None)
        } else {
            ReflectObject::set(&data.target, key, value, None)
        }
    }

    /// [[Has]] internal method
    pub fn has(&self, key: &str) -> JsResult<bool> {
        self.check_revoked("has")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.has {
            trap(&data.target, key)
        } else {
            ReflectObject::has(&data.target, key)
        }
    }

    /// [[Delete]] internal method
    pub fn delete_property(&self, key: &str) -> JsResult<bool> {
        self.check_revoked("deleteProperty")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.delete_property {
            trap(&data.target, key)
        } else {
            ReflectObject::delete_property(&data.target, key)
        }
    }

    /// [[OwnPropertyKeys]] internal method
    pub fn own_keys(&self) -> JsResult<Vec<String>> {
        self.check_revoked("ownKeys")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.own_keys {
            trap(&data.target)
        } else {
            ReflectObject::own_keys(&data.target)
        }
    }

    /// [[GetPrototypeOf]] internal method
    pub fn get_prototype_of(&self) -> JsResult<Option<JsValue>> {
        self.check_revoked("getPrototypeOf")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.get_prototype_of {
            trap(&data.target)
        } else {
            ReflectObject::get_prototype_of(&data.target)
        }
    }

    /// [[SetPrototypeOf]] internal method
    pub fn set_prototype_of(&self, proto: Option<&JsValue>) -> JsResult<bool> {
        self.check_revoked("setPrototypeOf")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.set_prototype_of {
            trap(&data.target, proto)
        } else {
            ReflectObject::set_prototype_of(&data.target, proto)
        }
    }

    /// [[IsExtensible]] internal method
    pub fn is_extensible(&self) -> JsResult<bool> {
        self.check_revoked("isExtensible")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.is_extensible {
            trap(&data.target)
        } else {
            ReflectObject::is_extensible(&data.target)
        }
    }

    /// [[PreventExtensions]] internal method
    pub fn prevent_extensions(&self) -> JsResult<bool> {
        self.check_revoked("preventExtensions")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.prevent_extensions {
            trap(&data.target)
        } else {
            ReflectObject::prevent_extensions(&data.target)
        }
    }

    /// [[GetOwnProperty]] internal method
    pub fn get_own_property_descriptor(
        &self,
        key: &str,
    ) -> JsResult<Option<PropertyDescriptor>> {
        self.check_revoked("getOwnPropertyDescriptor")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.get_own_property_descriptor {
            trap(&data.target, key)
        } else {
            ReflectObject::get_own_property_descriptor(&data.target, key)
        }
    }

    /// [[DefineOwnProperty]] internal method
    pub fn define_property(
        &self,
        key: &str,
        descriptor: &PropertyDescriptor,
    ) -> JsResult<bool> {
        self.check_revoked("defineProperty")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.define_property {
            trap(&data.target, key, descriptor)
        } else {
            ReflectObject::define_property(&data.target, key, descriptor)
        }
    }

    /// [[Call]] internal method (for function proxies)
    pub fn apply(&self, this_arg: &JsValue, args: &[JsValue]) -> JsResult<JsValue> {
        self.check_revoked("apply")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.apply {
            trap(&data.target, this_arg, args)
        } else {
            ReflectObject::apply(&data.target, this_arg, args)
        }
    }

    /// [[Construct]] internal method (for constructor proxies)
    pub fn construct(
        &self,
        args: &[JsValue],
        new_target: Option<&JsValue>,
    ) -> JsResult<JsValue> {
        self.check_revoked("construct")?;

        let data = self.data.borrow();
        if let Some(ref trap) = data.handler.construct {
            trap(&data.target, args, new_target)
        } else {
            ReflectObject::construct(&data.target, args, new_target)
        }
    }
}

impl std::fmt::Debug for ProxyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyObject")
            .field("revoked", &self.is_revoked())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_creation() {
        let target = JsValue::object();
        let handler = ProxyHandler::default();

        let proxy = ProxyObject::new(target, handler).unwrap();
        assert!(!proxy.is_revoked());
    }

    #[test]
    fn test_proxy_get_passthrough() {
        let target = JsValue::object();
        target.set("value", JsValue::number(42.0));

        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target, handler).unwrap();

        let result = proxy.get("value").unwrap();
        assert_eq!(result.as_number(), Some(42.0));
    }

    #[test]
    fn test_proxy_set_passthrough() {
        let target = JsValue::object();

        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target.clone(), handler).unwrap();

        proxy.set("key", JsValue::string("test")).unwrap();
        assert_eq!(
            target.get("key").unwrap().as_string(),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_proxy_get_trap() {
        let target = JsValue::object();
        target.set("original", JsValue::number(10.0));

        let mut handler = ProxyHandler::default();
        handler.get = Some(Box::new(|_target, key, _receiver| {
            Ok(JsValue::string(format!("trapped_{}", key)))
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();
        let result = proxy.get("original").unwrap();

        assert_eq!(
            result.as_string(),
            Some("trapped_original".to_string())
        );
    }

    #[test]
    fn test_proxy_set_trap() {
        let target = JsValue::object();

        let mut handler = ProxyHandler::default();
        handler.set = Some(Box::new(|target, key, value, _receiver| {
            // Modify value before setting
            if let Some(n) = value.as_number() {
                target.set(key, JsValue::number(n * 2.0));
            }
            Ok(true)
        }));

        let proxy = ProxyObject::new(target.clone(), handler).unwrap();
        proxy.set("value", JsValue::number(5.0)).unwrap();

        assert_eq!(target.get("value").unwrap().as_number(), Some(10.0));
    }

    #[test]
    fn test_proxy_has_trap() {
        let target = JsValue::object();
        target.set("visible", JsValue::boolean(true));
        target.set("hidden", JsValue::boolean(true));

        let mut handler = ProxyHandler::default();
        handler.has = Some(Box::new(|_target, key| {
            Ok(key != "hidden")
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();

        assert!(proxy.has("visible").unwrap());
        assert!(!proxy.has("hidden").unwrap());
    }

    #[test]
    fn test_proxy_revocable() {
        let target = JsValue::object();
        target.set("data", JsValue::number(100.0));

        let handler = ProxyHandler::default();
        let (proxy, revoke) = ProxyObject::revocable(target, handler).unwrap();

        // Before revocation
        assert!(!proxy.is_revoked());
        assert_eq!(proxy.get("data").unwrap().as_number(), Some(100.0));

        // Revoke
        revoke();

        // After revocation
        assert!(proxy.is_revoked());
        assert!(proxy.get("data").is_err());
    }

    #[test]
    fn test_proxy_all_operations_fail_after_revoke() {
        let target = JsValue::object();
        let handler = ProxyHandler::default();
        let (proxy, revoke) = ProxyObject::revocable(target, handler).unwrap();

        revoke();

        assert!(proxy.get("any").is_err());
        assert!(proxy.set("any", JsValue::null()).is_err());
        assert!(proxy.has("any").is_err());
        assert!(proxy.delete_property("any").is_err());
        assert!(proxy.own_keys().is_err());
        assert!(proxy.get_prototype_of().is_err());
        assert!(proxy.set_prototype_of(None).is_err());
        assert!(proxy.is_extensible().is_err());
        assert!(proxy.prevent_extensions().is_err());
    }

    #[test]
    fn test_proxy_delete_property() {
        let target = JsValue::object();
        target.set("prop", JsValue::number(1.0));

        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target.clone(), handler).unwrap();

        proxy.delete_property("prop").unwrap();
        assert!(!target.has_own("prop"));
    }

    #[test]
    fn test_proxy_own_keys() {
        let target = JsValue::object();
        target.set("a", JsValue::number(1.0));
        target.set("b", JsValue::number(2.0));

        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target, handler).unwrap();

        let keys = proxy.own_keys().unwrap();
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn test_property_descriptor_data() {
        let desc = PropertyDescriptor {
            value: Some(JsValue::number(42.0)),
            writable: Some(true),
            enumerable: Some(true),
            configurable: Some(true),
            get: None,
            set: None,
        };

        assert!(desc.is_data_descriptor());
        assert!(!desc.is_accessor_descriptor());
    }

    #[test]
    fn test_property_descriptor_accessor() {
        let desc = PropertyDescriptor {
            value: None,
            writable: None,
            enumerable: Some(true),
            configurable: Some(true),
            get: Some(Box::new(|| Ok(JsValue::number(1.0)))),
            set: None,
        };

        assert!(!desc.is_data_descriptor());
        assert!(desc.is_accessor_descriptor());
    }

    #[test]
    fn test_proxy_is_extensible() {
        let target = JsValue::object();
        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target, handler).unwrap();

        assert!(proxy.is_extensible().unwrap());
    }

    #[test]
    fn test_proxy_prevent_extensions() {
        let target = JsValue::object();
        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target, handler).unwrap();

        proxy.prevent_extensions().unwrap();
        assert!(!proxy.is_extensible().unwrap());
    }
}
