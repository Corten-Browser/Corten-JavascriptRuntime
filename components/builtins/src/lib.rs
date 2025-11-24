//! ECMAScript 2024 standard library implementation
//!
//! This crate provides the built-in objects and prototypes for a JavaScript runtime:
//! - Object.prototype methods
//! - Array.prototype methods
//! - String.prototype methods
//! - Number.prototype methods
//! - Math object
//! - JSON object
//! - Console object
//!
//! # Example
//!
//! ```
//! use builtins::{JsValue, ArrayPrototype, MathObject};
//!
//! // Create an array and use prototype methods
//! let arr = JsValue::array_from(vec![
//!     JsValue::number(1.0),
//!     JsValue::number(2.0),
//!     JsValue::number(3.0),
//! ]);
//!
//! let sum = ArrayPrototype::reduce(&arr, JsValue::number(0.0), |acc, v| {
//!     Ok(JsValue::number(acc.as_number().unwrap() + v.as_number().unwrap()))
//! }).unwrap();
//!
//! assert_eq!(sum.as_number().unwrap(), 6.0);
//!
//! // Use Math methods
//! assert_eq!(MathObject::sqrt(16.0), 4.0);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod array;
pub mod async_generator;
pub mod bigint;
pub mod collections;
pub mod console;
pub mod date;
pub mod error;
pub mod intl;
pub mod iterator;
pub mod json;
pub mod math;
pub mod number;
pub mod object;
pub mod proxy;
pub mod reflect;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod typed_arrays;
pub mod value;
pub mod weakref;

// Re-export main types for convenience
pub use array::ArrayPrototype;
pub use async_generator::{
    AsyncGeneratorFunction, AsyncGeneratorObject, AsyncGeneratorPromise, AsyncGeneratorState,
    AsyncIteratorHelper,
};
pub use bigint::{BigIntConstructor, BigIntPrototype};
pub use collections::{MapIterator, MapObject, SetIterator, SetObject, WeakMapObject, WeakSetObject};
pub use console::ConsoleObject;
pub use date::{DateConstructor, JsDate};
pub use error::{ErrorConstructor, ErrorKind, JsErrorObject, StackFrame};
pub use intl::{
    CaseFirst, Collator, CollatorOptions, CollatorSensitivity, CollatorUsage,
    CompactDisplay, CurrencyDisplay, DateTimeFormat, DateTimeFormatOptions, DateTimeStyle,
    HourCycle, Intl, ListFormat, ListFormatOptions, ListFormatStyle, ListFormatType, Locale,
    Notation, NumberFormat, NumberFormatOptions, NumberStyle, PluralCategory, PluralRules,
    PluralRulesOptions, PluralRulesType, RelativeTimeFormat, RelativeTimeFormatOptions,
    RelativeTimeNumeric, RelativeTimeStyle, RelativeTimeUnit, SignDisplay,
};
pub use iterator::{
    ArrayIterator, GeneratorFunction, GeneratorObject, GeneratorState, Iterator,
    IteratorHelpers, IteratorKind, IteratorProtocol, IteratorResult, ObjectIterator,
    StringIterator,
};
pub use json::JSONObject;
pub use math::MathObject;
pub use number::NumberPrototype;
pub use object::ObjectPrototype;
pub use proxy::{ProxyHandler, ProxyObject};
pub use reflect::ReflectObject;
pub use regexp::{RegExpMatch, RegExpObject};
pub use string::StringPrototype;
pub use symbol::{SymbolConstructor, SymbolValue};
pub use typed_arrays::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, TypedArray, TypedArrayKind, TypedArrayValue, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
pub use value::{BigIntValue, JsError, JsResult, JsValue};
pub use weakref::{
    FinalizationRegistryData, FinalizationRegistryObject, WeakObjectRef, WeakRefData,
    WeakRefObject,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_array_operations() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        // Map: double each element
        let doubled = ArrayPrototype::map(&arr, |v| {
            Ok(JsValue::number(v.as_number().unwrap() * 2.0))
        })
        .unwrap();

        // Filter: keep only values > 4
        let filtered = ArrayPrototype::filter(&doubled, |v| Ok(v.as_number().unwrap() > 4.0)).unwrap();

        // Should have [6.0] (doubled values: [2, 4, 6], filtered > 4: [6])
        assert_eq!(filtered.array_length(), 1);
    }

    #[test]
    fn test_integration_string_operations() {
        let s = "  Hello, World!  ";
        let trimmed = StringPrototype::trim(s);
        let lower = StringPrototype::to_lower_case(&trimmed);
        let replaced = StringPrototype::replace(&lower, "world", "rust").unwrap();

        assert_eq!(replaced, "hello, rust!");
    }

    #[test]
    fn test_integration_json_roundtrip() {
        let obj = JsValue::object();
        obj.set("name", JsValue::string("test"));
        obj.set("value", JsValue::number(42.0));

        let json_str = JSONObject::stringify(&obj).unwrap();
        let parsed = JSONObject::parse(&json_str).unwrap();

        assert!(parsed.is_object());
        assert_eq!(parsed.get("name").unwrap().as_string().unwrap(), "test");
    }

    #[test]
    fn test_integration_math_calculations() {
        let values = vec![1.0, 4.0, 9.0, 16.0];

        // Calculate square roots
        let roots: Vec<f64> = values.iter().map(|&x| MathObject::sqrt(x)).collect();
        assert_eq!(roots, vec![1.0, 2.0, 3.0, 4.0]);

        // Find max
        let max = MathObject::max(&roots);
        assert_eq!(max, 4.0);
    }

    #[test]
    fn test_integration_object_prototype() {
        let proto = JsValue::object();
        proto.set("inherited", JsValue::number(100.0));

        let obj = JsValue::object_with_proto(&proto);
        obj.set("own", JsValue::number(42.0));

        // Check hasOwnProperty
        assert_eq!(
            ObjectPrototype::has_own_property(&obj, "own").unwrap(),
            JsValue::boolean(true)
        );
        assert_eq!(
            ObjectPrototype::has_own_property(&obj, "inherited").unwrap(),
            JsValue::boolean(false)
        );

        // Check toString
        let str_rep = ObjectPrototype::to_string(&obj).unwrap();
        assert!(str_rep.as_string().unwrap().contains("Object"));
    }

    #[test]
    fn test_integration_console_output() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let output = Rc::new(RefCell::new(Vec::new()));
        let console = ConsoleObject::new_with_output(output.clone());

        console.log(&[JsValue::string("Starting")]);
        console.info(&[JsValue::string("Processing")]);
        console.warn(&[JsValue::string("Almost done")]);
        console.log(&[JsValue::string("Done")]);

        assert_eq!(output.borrow().len(), 4);
    }

    #[test]
    fn test_integration_iterator_protocol() {
        // Test array iterator with helper methods
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
            JsValue::number(5.0),
        ]);

        // Use Iterator.from to get an iterator
        let mut iter = Iterator::from(&arr).unwrap();

        // Take first 3 elements
        let taken = IteratorHelpers::take(&mut *iter, 3);
        assert_eq!(taken.len(), 3);

        // Create new iterator for filtering
        let arr2 = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
        ]);
        let mut iter2 = ArrayIterator::new(arr2);

        // Filter even numbers
        let evens = IteratorHelpers::filter(&mut iter2, |v| {
            v.as_number().unwrap() % 2.0 == 0.0
        });
        assert_eq!(evens.len(), 2);

        // Reduce to sum
        let arr3 = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);
        let mut iter3 = ArrayIterator::new(arr3);
        let sum = IteratorHelpers::reduce(&mut iter3, JsValue::number(0.0), |acc, val| {
            Ok(JsValue::number(
                acc.as_number().unwrap() + val.as_number().unwrap(),
            ))
        })
        .unwrap();
        assert_eq!(sum.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_integration_generator_workflow() {
        // Create a generator that yields values
        let gen = GeneratorFunction::from_values(vec![
            JsValue::string("start"),
            JsValue::string("middle"),
            JsValue::string("end"),
        ]);

        // Test generator protocol
        assert_eq!(gen.state(), GeneratorState::Suspended);

        let r1 = gen.next(None).unwrap();
        assert!(!r1.done);
        assert_eq!(r1.value.as_string().unwrap(), "start");

        let r2 = gen.next(None).unwrap();
        assert!(!r2.done);
        assert_eq!(r2.value.as_string().unwrap(), "middle");

        // Early return
        let ret = gen.return_value(Some(JsValue::string("finished"))).unwrap();
        assert!(ret.done);
        assert_eq!(ret.value.as_string().unwrap(), "finished");

        // Generator is now closed
        assert_eq!(gen.state(), GeneratorState::Closed);

        // Subsequent calls return done
        let r3 = gen.next(None).unwrap();
        assert!(r3.done);
    }

    #[test]
    fn test_integration_string_iterator() {
        let s = JsValue::string("Hello\u{1F600}!");

        // Iterate over string characters (including emoji)
        let mut iter = Iterator::from(&s).unwrap();
        let chars = IteratorHelpers::to_array(&mut *iter);

        assert_eq!(chars.len(), 7); // H e l l o emoji !
        assert_eq!(chars[0].as_string().unwrap(), "H");
        assert_eq!(chars[5].as_string().unwrap(), "\u{1F600}");
        assert_eq!(chars[6].as_string().unwrap(), "!");
    }

    #[test]
    fn test_integration_object_entries_iterator() {
        let obj = JsValue::object();
        obj.set("name", JsValue::string("test"));
        obj.set("value", JsValue::number(42.0));

        let mut iter = ObjectIterator::entries(&obj);
        let entries = IteratorHelpers::to_array(&mut iter);

        assert_eq!(entries.len(), 2);

        // Check that entries are [key, value] pairs
        for entry in entries {
            assert!(entry.is_array());
            let entry_arr = entry.array_length();
            assert_eq!(entry_arr, 2);
        }
    }

    #[test]
    fn test_integration_jsvalue_generator() {
        // Test JsValue::Generator variant
        let gen = GeneratorFunction::from_values(vec![JsValue::number(42.0)]);
        let val = JsValue::generator(gen);

        // Type checks
        assert!(val.is_generator());
        assert_eq!(val.type_of(), "object");
        assert_eq!(val.to_js_string(), "[object Generator]");

        // Extract and use generator
        let gen_obj = val.as_generator().unwrap();
        let result = gen_obj.next(None).unwrap();
        assert_eq!(result.value.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_integration_jsvalue_async_generator() {
        // Test JsValue::AsyncGenerator variant
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);
        let val = JsValue::async_generator(gen);

        // Type checks
        assert!(val.is_async_generator());
        assert!(!val.is_generator());
        assert_eq!(val.type_of(), "object");
        assert_eq!(val.to_js_string(), "[object AsyncGenerator]");

        // Extract and use async generator
        let gen_obj = val.as_async_generator().unwrap();

        // Test async iteration
        let promise = gen_obj.next(None);
        assert!(promise.is_fulfilled());

        let result = promise.value().unwrap();
        assert_eq!(result.get("value").unwrap().as_number(), Some(1.0));
        assert_eq!(result.get("done").unwrap().as_boolean(), Some(false));

        // Collect all values
        let all_values = AsyncIteratorHelper::collect(&gen_obj);
        assert_eq!(all_values.len(), 2); // Already consumed one
        assert_eq!(all_values[0].as_number(), Some(2.0));
        assert_eq!(all_values[1].as_number(), Some(3.0));

        // Generator should be completed
        assert_eq!(gen_obj.state(), AsyncGeneratorState::Completed);
    }

    #[test]
    fn test_integration_async_generator_with_return() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::string("start")]);

        let val = JsValue::async_generator(gen);
        let gen_obj = val.as_async_generator().unwrap();

        // Get first value
        let p1 = gen_obj.next(None);
        assert!(p1.is_fulfilled());

        // Early return
        let p_ret = gen_obj.return_value(Some(JsValue::string("early exit")));
        assert!(p_ret.is_fulfilled());

        let ret_val = p_ret.value().unwrap();
        assert_eq!(ret_val.get("done").unwrap().as_boolean(), Some(true));
        assert_eq!(
            ret_val.get("value").unwrap().as_string(),
            Some("early exit".to_string())
        );

        // Subsequent next should return done
        let p2 = gen_obj.next(None);
        let done_val = p2.value().unwrap();
        assert_eq!(done_val.get("done").unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_integration_async_generator_error_handling() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(42.0)]);
        let val = JsValue::async_generator(gen);
        let gen_obj = val.as_async_generator().unwrap();

        // Start the generator
        let _ = gen_obj.next(None);

        // Throw an error
        let p_throw = gen_obj.throw(JsValue::string("test error"));
        assert!(p_throw.is_rejected());

        // Error should be captured
        let err = p_throw.error().unwrap();
        assert!(err.message.contains("test error"));

        // Generator should be completed
        assert_eq!(gen_obj.state(), AsyncGeneratorState::Completed);
    }
}


