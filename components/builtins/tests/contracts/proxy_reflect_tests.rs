//! Contract tests for Proxy and Reflect per ES2024
//!
//! These tests verify that Proxy intercepts all operations and Reflect provides
//! corresponding static methods for object operations.

use builtins::{JsValue, ProxyHandler, ProxyObject, ReflectObject};
use builtins::proxy::PropertyDescriptor;

mod reflect_tests {
    use super::*;

    #[test]
    fn test_reflect_get_basic() {
        let obj = JsValue::object();
        obj.set("foo", JsValue::number(42.0));

        let result = ReflectObject::get(&obj, "foo", None).unwrap();
        assert_eq!(result.as_number(), Some(42.0));
    }

    #[test]
    fn test_reflect_get_missing_property() {
        let obj = JsValue::object();

        let result = ReflectObject::get(&obj, "missing", None).unwrap();
        assert!(result.is_undefined());
    }

    #[test]
    fn test_reflect_set_basic() {
        let obj = JsValue::object();

        let result = ReflectObject::set(&obj, "bar", JsValue::string("hello"), None).unwrap();
        assert!(result);
        assert_eq!(obj.get("bar").unwrap().as_string(), Some("hello".to_string()));
    }

    #[test]
    fn test_reflect_has_true() {
        let obj = JsValue::object();
        obj.set("exists", JsValue::boolean(true));

        let result = ReflectObject::has(&obj, "exists").unwrap();
        assert!(result);
    }

    #[test]
    fn test_reflect_has_false() {
        let obj = JsValue::object();

        let result = ReflectObject::has(&obj, "missing").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_reflect_delete_property_success() {
        let obj = JsValue::object();
        obj.set("toDelete", JsValue::number(1.0));

        let result = ReflectObject::delete_property(&obj, "toDelete").unwrap();
        assert!(result);
        assert!(!obj.has_own("toDelete"));
    }

    #[test]
    fn test_reflect_own_keys() {
        let obj = JsValue::object();
        obj.set("a", JsValue::number(1.0));
        obj.set("b", JsValue::number(2.0));
        obj.set("c", JsValue::number(3.0));

        let keys = ReflectObject::own_keys(&obj).unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"c".to_string()));
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
        new_proto.set("inherited", JsValue::number(100.0));

        let result = ReflectObject::set_prototype_of(&obj, Some(&new_proto)).unwrap();
        assert!(result);

        let proto = ReflectObject::get_prototype_of(&obj).unwrap();
        assert!(proto.is_some());
    }

    #[test]
    fn test_reflect_is_extensible() {
        let obj = JsValue::object();

        let result = ReflectObject::is_extensible(&obj).unwrap();
        assert!(result);
    }

    #[test]
    fn test_reflect_prevent_extensions() {
        let obj = JsValue::object();

        let result = ReflectObject::prevent_extensions(&obj).unwrap();
        assert!(result);

        let extensible = ReflectObject::is_extensible(&obj).unwrap();
        assert!(!extensible);
    }

    #[test]
    fn test_reflect_define_property() {
        let obj = JsValue::object();
        let descriptor = PropertyDescriptor {
            value: Some(JsValue::number(42.0)),
            writable: Some(true),
            enumerable: Some(true),
            configurable: Some(true),
            get: None,
            set: None,
        };

        let result = ReflectObject::define_property(&obj, "defined", &descriptor).unwrap();
        assert!(result);
        assert_eq!(obj.get("defined").unwrap().as_number(), Some(42.0));
    }

    #[test]
    fn test_reflect_get_own_property_descriptor() {
        let obj = JsValue::object();
        obj.set("prop", JsValue::string("value"));

        let desc = ReflectObject::get_own_property_descriptor(&obj, "prop").unwrap();
        assert!(desc.is_some());
        let desc = desc.unwrap();
        assert_eq!(desc.value.unwrap().as_string(), Some("value".to_string()));
    }

    #[test]
    fn test_reflect_type_error_on_non_object() {
        let non_obj = JsValue::number(42.0);

        let result = ReflectObject::get(&non_obj, "foo", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reflect_apply() {
        let func = JsValue::function(|_this, args| {
            let sum: f64 = args.iter()
                .filter_map(|v| v.as_number())
                .sum();
            Ok(JsValue::number(sum))
        });

        let result = ReflectObject::apply(
            &func,
            &JsValue::undefined(),
            &[JsValue::number(1.0), JsValue::number(2.0), JsValue::number(3.0)]
        ).unwrap();

        assert_eq!(result.as_number(), Some(6.0));
    }

    #[test]
    fn test_reflect_construct() {
        let constructor = JsValue::constructor(|args| {
            let obj = JsValue::object();
            if let Some(name) = args.first() {
                obj.set("name", name.clone());
            }
            Ok(obj)
        });

        let result = ReflectObject::construct(
            &constructor,
            &[JsValue::string("test")],
            None
        ).unwrap();

        assert!(result.is_object());
        assert_eq!(result.get("name").unwrap().as_string(), Some("test".to_string()));
    }
}

mod proxy_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_proxy_creation() {
        let target = JsValue::object();
        target.set("value", JsValue::number(42.0));

        let handler = ProxyHandler::default();
        let proxy = ProxyObject::new(target.clone(), handler).unwrap();

        assert!(!proxy.is_revoked());
    }

    #[test]
    fn test_proxy_get_trap() {
        let target = JsValue::object();
        target.set("original", JsValue::number(10.0));

        let mut handler = ProxyHandler::default();
        handler.get = Some(Box::new(|_target, key, _receiver| {
            Ok(JsValue::string(format!("intercepted_{}", key)))
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();
        let result = proxy.get("original").unwrap();

        assert_eq!(result.as_string(), Some("intercepted_original".to_string()));
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
    fn test_proxy_set_trap() {
        let target = JsValue::object();
        let set_called = Rc::new(RefCell::new(false));
        let set_called_clone = set_called.clone();

        let mut handler = ProxyHandler::default();
        handler.set = Some(Box::new(move |target, key, value, _receiver| {
            *set_called_clone.borrow_mut() = true;
            if let Some(n) = value.as_number() {
                target.set(key, JsValue::number(n * 2.0));
            }
            Ok(true)
        }));

        let proxy = ProxyObject::new(target.clone(), handler).unwrap();
        let result = proxy.set("count", JsValue::number(5.0)).unwrap();

        assert!(result);
        assert!(*set_called.borrow());
        assert_eq!(target.get("count").unwrap().as_number(), Some(10.0));
    }

    #[test]
    fn test_proxy_has_trap() {
        let target = JsValue::object();
        target.set("secret", JsValue::boolean(true));

        let mut handler = ProxyHandler::default();
        handler.has = Some(Box::new(|_target, key| {
            if key == "secret" {
                Ok(false)
            } else {
                Ok(true)
            }
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();

        assert!(!proxy.has("secret").unwrap());
        assert!(proxy.has("anything").unwrap());
    }

    #[test]
    fn test_proxy_delete_property_trap() {
        let target = JsValue::object();
        target.set("protected", JsValue::number(1.0));
        target.set("deletable", JsValue::number(2.0));

        let mut handler = ProxyHandler::default();
        handler.delete_property = Some(Box::new(|target, key| {
            if key == "protected" {
                Ok(false)
            } else {
                ReflectObject::delete_property(target, key)
            }
        }));

        let proxy = ProxyObject::new(target.clone(), handler).unwrap();

        assert!(!proxy.delete_property("protected").unwrap());
        assert!(target.has_own("protected"));

        assert!(proxy.delete_property("deletable").unwrap());
        assert!(!target.has_own("deletable"));
    }

    #[test]
    fn test_proxy_own_keys_trap() {
        let target = JsValue::object();
        target.set("a", JsValue::number(1.0));
        target.set("_private", JsValue::number(2.0));
        target.set("b", JsValue::number(3.0));

        let mut handler = ProxyHandler::default();
        handler.own_keys = Some(Box::new(|_target| {
            Ok(vec!["a".to_string(), "b".to_string()])
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();
        let keys = proxy.own_keys().unwrap();

        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(!keys.contains(&"_private".to_string()));
    }

    #[test]
    fn test_proxy_revocable() {
        let target = JsValue::object();
        target.set("value", JsValue::number(42.0));

        let handler = ProxyHandler::default();
        let (proxy, revoke) = ProxyObject::revocable(target, handler).unwrap();

        assert!(!proxy.is_revoked());
        let result = proxy.get("value").unwrap();
        assert_eq!(result.as_number(), Some(42.0));

        revoke();

        assert!(proxy.is_revoked());
        let err = proxy.get("value");
        assert!(err.is_err());
    }

    #[test]
    fn test_proxy_revoked_operations_fail() {
        let target = JsValue::object();
        let handler = ProxyHandler::default();
        let (proxy, revoke) = ProxyObject::revocable(target, handler).unwrap();

        revoke();

        assert!(proxy.get("any").is_err());
        assert!(proxy.set("any", JsValue::null()).is_err());
        assert!(proxy.has("any").is_err());
        assert!(proxy.delete_property("any").is_err());
        assert!(proxy.own_keys().is_err());
    }

    #[test]
    fn test_proxy_apply_trap() {
        let target = JsValue::function(|_this, _args| {
            Ok(JsValue::string("original"))
        });

        let mut handler = ProxyHandler::default();
        handler.apply = Some(Box::new(|_target, _this_arg, _args| {
            Ok(JsValue::string("intercepted"))
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();
        let result = proxy.apply(&JsValue::undefined(), &[]).unwrap();

        assert_eq!(result.as_string(), Some("intercepted".to_string()));
    }

    #[test]
    fn test_proxy_construct_trap() {
        let target = JsValue::constructor(|_args| {
            let obj = JsValue::object();
            obj.set("type", JsValue::string("original"));
            Ok(obj)
        });

        let mut handler = ProxyHandler::default();
        handler.construct = Some(Box::new(|_target, _args, _new_target| {
            let obj = JsValue::object();
            obj.set("type", JsValue::string("intercepted"));
            Ok(obj)
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();
        let result = proxy.construct(&[], None).unwrap();

        assert_eq!(result.get("type").unwrap().as_string(), Some("intercepted".to_string()));
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_reflect_and_proxy_interop() {
        let target = JsValue::object();
        target.set("count", JsValue::number(0.0));

        let mut handler = ProxyHandler::default();
        handler.set = Some(Box::new(|target, key, value, receiver| {
            ReflectObject::set(target, key, value, receiver)
        }));

        let proxy = ProxyObject::new(target.clone(), handler).unwrap();
        proxy.set("count", JsValue::number(5.0)).unwrap();

        assert_eq!(target.get("count").unwrap().as_number(), Some(5.0));
    }

    #[test]
    fn test_property_descriptor_complete() {
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
            get: Some(Box::new(|| Ok(JsValue::number(100.0)))),
            set: None,
        };

        assert!(!desc.is_data_descriptor());
        assert!(desc.is_accessor_descriptor());
    }

    #[test]
    fn test_logging_proxy() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let target = JsValue::object();
        target.set("data", JsValue::string("secret"));

        let log = Rc::new(RefCell::new(Vec::<String>::new()));
        let log_clone = log.clone();

        let mut handler = ProxyHandler::default();
        handler.get = Some(Box::new(move |target, key, _receiver| {
            log_clone.borrow_mut().push(format!("GET {}", key));
            ReflectObject::get(target, key, None)
        }));

        let proxy = ProxyObject::new(target, handler).unwrap();

        let _ = proxy.get("data");
        let _ = proxy.get("missing");

        let logs = log.borrow();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "GET data");
        assert_eq!(logs[1], "GET missing");
    }

    #[test]
    fn test_reflect_all_methods_exist() {
        let obj = JsValue::object();
        obj.set("test", JsValue::number(1.0));

        assert!(ReflectObject::get(&obj, "test", None).is_ok());
        assert!(ReflectObject::set(&obj, "test", JsValue::number(2.0), None).is_ok());
        assert!(ReflectObject::has(&obj, "test").is_ok());
        assert!(ReflectObject::delete_property(&obj, "test").is_ok());
        assert!(ReflectObject::own_keys(&obj).is_ok());
        assert!(ReflectObject::get_prototype_of(&obj).is_ok());
        assert!(ReflectObject::set_prototype_of(&obj, None).is_ok());
        assert!(ReflectObject::is_extensible(&obj).is_ok());
        assert!(ReflectObject::prevent_extensions(&obj).is_ok());

        let desc = PropertyDescriptor::default();
        assert!(ReflectObject::define_property(&obj, "new", &desc).is_ok());
        assert!(ReflectObject::get_own_property_descriptor(&obj, "new").is_ok());
    }
}
