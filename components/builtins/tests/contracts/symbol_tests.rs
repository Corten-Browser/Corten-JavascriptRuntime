//! Contract tests for Symbol primitive type
//!
//! Tests cover:
//! - Symbol creation and uniqueness
//! - Symbol.for() global registry
//! - Symbol.keyFor() reverse lookup
//! - Well-known symbols
//! - Symbol properties (description, toString, valueOf)
//! - Symbol as object property keys
//! - Type coercion restrictions

use builtins::symbol::SymbolConstructor;
use builtins::{JsValue, ObjectPrototype};

#[cfg(test)]
mod symbol_creation_tests {
    use super::*;

    #[test]
    fn symbol_without_description_is_unique() {
        let sym1 = SymbolConstructor::new(None);
        let sym2 = SymbolConstructor::new(None);

        assert_ne!(sym1.id(), sym2.id());
    }

    #[test]
    fn symbol_with_same_description_still_unique() {
        let sym1 = SymbolConstructor::new(Some("test".to_string()));
        let sym2 = SymbolConstructor::new(Some("test".to_string()));

        assert_ne!(sym1.id(), sym2.id());
    }

    #[test]
    fn symbol_preserves_description() {
        let sym = SymbolConstructor::new(Some("my description".to_string()));
        assert_eq!(sym.description(), Some("my description"));
    }

    #[test]
    fn symbol_without_description_returns_none() {
        let sym = SymbolConstructor::new(None);
        assert_eq!(sym.description(), None);
    }

    #[test]
    fn symbol_to_string_with_description() {
        let sym = SymbolConstructor::new(Some("foo".to_string()));
        assert_eq!(sym.to_string(), "Symbol(foo)");
    }

    #[test]
    fn symbol_to_string_without_description() {
        let sym = SymbolConstructor::new(None);
        assert_eq!(sym.to_string(), "Symbol()");
    }

    #[test]
    fn symbol_value_of_returns_self() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let id = sym.id();
        let value_of = sym.value_of();
        assert_eq!(value_of.id(), id);
    }

    #[test]
    fn symbol_with_empty_string_description() {
        let sym = SymbolConstructor::new(Some("".to_string()));
        assert_eq!(sym.description(), Some(""));
        assert_eq!(sym.to_string(), "Symbol()");
    }
}

#[cfg(test)]
mod symbol_for_registry_tests {
    use super::*;

    #[test]
    fn symbol_for_returns_same_symbol_for_same_key() {
        let sym1 = SymbolConstructor::for_key("shared");
        let sym2 = SymbolConstructor::for_key("shared");

        assert_eq!(sym1.id(), sym2.id());
    }

    #[test]
    fn symbol_for_different_keys_return_different_symbols() {
        let sym1 = SymbolConstructor::for_key("key1");
        let sym2 = SymbolConstructor::for_key("key2");

        assert_ne!(sym1.id(), sym2.id());
    }

    #[test]
    fn symbol_for_sets_description_to_key() {
        let sym = SymbolConstructor::for_key("mykey");
        assert_eq!(sym.description(), Some("mykey"));
    }

    #[test]
    fn symbol_for_different_from_regular_symbol() {
        let regular = SymbolConstructor::new(Some("test".to_string()));
        let registered = SymbolConstructor::for_key("test");

        // Even with same description, regular symbol is not in registry
        assert_ne!(regular.id(), registered.id());
    }

    #[test]
    fn symbol_key_for_registered_symbol() {
        let sym = SymbolConstructor::for_key("mykey");
        let key = SymbolConstructor::key_for(&sym);

        assert_eq!(key, Some("mykey".to_string()));
    }

    #[test]
    fn symbol_key_for_unregistered_symbol_returns_none() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let key = SymbolConstructor::key_for(&sym);

        assert_eq!(key, None);
    }

    #[test]
    fn symbol_key_for_well_known_symbol_returns_none() {
        let iterator_sym = SymbolConstructor::iterator();
        let key = SymbolConstructor::key_for(&iterator_sym);

        assert_eq!(key, None);
    }

    #[test]
    fn symbol_for_with_empty_key() {
        let sym = SymbolConstructor::for_key("");
        assert_eq!(sym.description(), Some(""));
        assert_eq!(SymbolConstructor::key_for(&sym), Some("".to_string()));
    }
}

#[cfg(test)]
mod well_known_symbols_tests {
    use super::*;

    #[test]
    fn symbol_iterator_is_unique() {
        let sym = SymbolConstructor::iterator();
        assert_eq!(sym.description(), Some("Symbol.iterator"));
    }

    #[test]
    fn symbol_iterator_returns_same_instance() {
        let sym1 = SymbolConstructor::iterator();
        let sym2 = SymbolConstructor::iterator();
        assert_eq!(sym1.id(), sym2.id());
    }

    #[test]
    fn symbol_async_iterator_exists() {
        let sym = SymbolConstructor::async_iterator();
        assert_eq!(sym.description(), Some("Symbol.asyncIterator"));
    }

    #[test]
    fn symbol_to_string_tag_exists() {
        let sym = SymbolConstructor::to_string_tag();
        assert_eq!(sym.description(), Some("Symbol.toStringTag"));
    }

    #[test]
    fn symbol_has_instance_exists() {
        let sym = SymbolConstructor::has_instance();
        assert_eq!(sym.description(), Some("Symbol.hasInstance"));
    }

    #[test]
    fn symbol_species_exists() {
        let sym = SymbolConstructor::species();
        assert_eq!(sym.description(), Some("Symbol.species"));
    }

    #[test]
    fn symbol_is_concat_spreadable_exists() {
        let sym = SymbolConstructor::is_concat_spreadable();
        assert_eq!(sym.description(), Some("Symbol.isConcatSpreadable"));
    }

    #[test]
    fn symbol_to_primitive_exists() {
        let sym = SymbolConstructor::to_primitive();
        assert_eq!(sym.description(), Some("Symbol.toPrimitive"));
    }

    #[test]
    fn symbol_unscopables_exists() {
        let sym = SymbolConstructor::unscopables();
        assert_eq!(sym.description(), Some("Symbol.unscopables"));
    }

    #[test]
    fn symbol_match_exists() {
        let sym = SymbolConstructor::match_symbol();
        assert_eq!(sym.description(), Some("Symbol.match"));
    }

    #[test]
    fn symbol_replace_exists() {
        let sym = SymbolConstructor::replace();
        assert_eq!(sym.description(), Some("Symbol.replace"));
    }

    #[test]
    fn symbol_search_exists() {
        let sym = SymbolConstructor::search();
        assert_eq!(sym.description(), Some("Symbol.search"));
    }

    #[test]
    fn symbol_split_exists() {
        let sym = SymbolConstructor::split();
        assert_eq!(sym.description(), Some("Symbol.split"));
    }

    #[test]
    fn well_known_symbols_are_all_unique() {
        let symbols = vec![
            SymbolConstructor::iterator(),
            SymbolConstructor::async_iterator(),
            SymbolConstructor::to_string_tag(),
            SymbolConstructor::has_instance(),
            SymbolConstructor::species(),
            SymbolConstructor::is_concat_spreadable(),
            SymbolConstructor::to_primitive(),
            SymbolConstructor::unscopables(),
            SymbolConstructor::match_symbol(),
            SymbolConstructor::replace(),
            SymbolConstructor::search(),
            SymbolConstructor::split(),
        ];

        // Check all pairs are different
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                assert_ne!(
                    symbols[i].id(),
                    symbols[j].id(),
                    "Symbol {} and {} should be different",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn well_known_symbols_are_stable() {
        // Call twice to ensure same instance
        assert_eq!(
            SymbolConstructor::iterator().id(),
            SymbolConstructor::iterator().id()
        );
        assert_eq!(
            SymbolConstructor::async_iterator().id(),
            SymbolConstructor::async_iterator().id()
        );
        assert_eq!(
            SymbolConstructor::to_string_tag().id(),
            SymbolConstructor::to_string_tag().id()
        );
    }
}

#[cfg(test)]
mod symbol_js_value_integration_tests {
    use super::*;

    #[test]
    fn js_value_symbol_creation() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let value = JsValue::symbol(sym);
        assert!(value.is_symbol());
    }

    #[test]
    fn js_value_symbol_as_symbol() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let id = sym.id();
        let value = JsValue::symbol(sym);

        let retrieved = value.as_symbol().unwrap();
        assert_eq!(retrieved.id(), id);
    }

    #[test]
    fn js_value_symbol_to_js_string() {
        let sym = SymbolConstructor::new(Some("mySymbol".to_string()));
        let value = JsValue::symbol(sym);
        assert_eq!(value.to_js_string(), "Symbol(mySymbol)");
    }

    #[test]
    fn symbol_cannot_convert_to_number() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let result = sym.to_number();

        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.message.contains("TypeError")),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn symbol_cannot_implicitly_convert_to_string() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        // Implicit string conversion should fail
        let result = sym.to_string_implicit();

        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.message.contains("TypeError")),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn symbol_explicit_to_string_works() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        // Explicit Symbol.prototype.toString() should work
        assert_eq!(sym.to_string(), "Symbol(test)");
    }

    #[test]
    fn symbol_equality_same_symbol() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let value1 = JsValue::symbol(sym.clone());
        let value2 = JsValue::symbol(sym);

        assert!(value1.equals(&value2));
    }

    #[test]
    fn symbol_equality_different_symbols() {
        let sym1 = SymbolConstructor::new(Some("test".to_string()));
        let sym2 = SymbolConstructor::new(Some("test".to_string()));
        let value1 = JsValue::symbol(sym1);
        let value2 = JsValue::symbol(sym2);

        assert!(!value1.equals(&value2));
    }
}

#[cfg(test)]
mod symbol_as_property_key_tests {
    use super::*;

    #[test]
    fn symbol_can_be_object_property_key() {
        let obj = JsValue::object();
        let sym = SymbolConstructor::new(Some("myProp".to_string()));

        obj.set_symbol(&sym, JsValue::number(42.0));
        let retrieved = obj.get_symbol(&sym);

        assert_eq!(retrieved, Some(JsValue::number(42.0)));
    }

    #[test]
    fn different_symbols_are_different_keys() {
        let obj = JsValue::object();
        let sym1 = SymbolConstructor::new(Some("prop".to_string()));
        let sym2 = SymbolConstructor::new(Some("prop".to_string()));

        obj.set_symbol(&sym1, JsValue::number(1.0));
        obj.set_symbol(&sym2, JsValue::number(2.0));

        assert_eq!(obj.get_symbol(&sym1), Some(JsValue::number(1.0)));
        assert_eq!(obj.get_symbol(&sym2), Some(JsValue::number(2.0)));
    }

    #[test]
    fn symbol_property_not_in_string_keys() {
        let obj = JsValue::object();
        let sym = SymbolConstructor::new(Some("hidden".to_string()));

        obj.set_symbol(&sym, JsValue::number(42.0));
        obj.set("visible", JsValue::string("hello"));

        // Symbol property should not be enumerable with string keys
        assert!(obj.has_own("visible"));
        assert!(!obj.has_own("hidden"));
        assert!(obj.has_own_symbol(&sym));
    }

    #[test]
    fn same_registered_symbol_accesses_same_property() {
        let obj = JsValue::object();
        let sym1 = SymbolConstructor::for_key("shared");
        obj.set_symbol(&sym1, JsValue::number(100.0));

        let sym2 = SymbolConstructor::for_key("shared");
        let retrieved = obj.get_symbol(&sym2);

        assert_eq!(retrieved, Some(JsValue::number(100.0)));
    }

    #[test]
    fn well_known_symbol_as_property_key() {
        let obj = JsValue::object();
        let iterator_sym = SymbolConstructor::iterator();

        obj.set_symbol(&iterator_sym, JsValue::string("iterator_func"));
        let retrieved = obj.get_symbol(&iterator_sym);

        assert_eq!(retrieved, Some(JsValue::string("iterator_func")));
    }

    #[test]
    fn object_has_own_symbol_property() {
        let obj = JsValue::object();
        let sym = SymbolConstructor::new(Some("test".to_string()));

        assert!(!obj.has_own_symbol(&sym));
        obj.set_symbol(&sym, JsValue::boolean(true));
        assert!(obj.has_own_symbol(&sym));
    }
}

#[cfg(test)]
mod symbol_type_checking_tests {
    use super::*;

    #[test]
    fn typeof_symbol_is_symbol() {
        let sym = SymbolConstructor::new(None);
        let value = JsValue::symbol(sym);
        assert_eq!(value.type_of(), "symbol");
    }

    #[test]
    fn symbol_is_not_object() {
        let sym = SymbolConstructor::new(None);
        let value = JsValue::symbol(sym);
        assert!(!value.is_object());
    }

    #[test]
    fn symbol_is_not_string() {
        let sym = SymbolConstructor::new(None);
        let value = JsValue::symbol(sym);
        assert!(!value.is_string());
    }

    #[test]
    fn symbol_is_not_number() {
        let sym = SymbolConstructor::new(None);
        let value = JsValue::symbol(sym);
        assert!(!value.is_number());
    }

    #[test]
    fn symbol_object_to_string_tag() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        let value = JsValue::symbol(sym);
        // [object Symbol] when using Object.prototype.toString
        assert_eq!(builtins::ObjectPrototype::to_string(&value).unwrap().as_string().unwrap(), "[object Symbol]");
    }
}
