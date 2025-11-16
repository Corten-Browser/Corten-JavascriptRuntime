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
pub mod console;
pub mod json;
pub mod math;
pub mod number;
pub mod object;
pub mod string;
pub mod value;

// Re-export main types for convenience
pub use array::ArrayPrototype;
pub use console::ConsoleObject;
pub use json::JSONObject;
pub use math::MathObject;
pub use number::NumberPrototype;
pub use object::ObjectPrototype;
pub use string::StringPrototype;
pub use value::{JsError, JsResult, JsValue};

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
}
