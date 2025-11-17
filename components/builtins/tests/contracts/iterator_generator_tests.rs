//! Contract tests for Iterator and Generator protocol implementation
//!
//! These tests verify the ES2024 iterator and generator protocol compliance.

use builtins::iterator::{
    ArrayIterator, GeneratorFunction, GeneratorObject, GeneratorState, Iterator,
    IteratorHelpers, IteratorKind, IteratorProtocol, IteratorResult, ObjectIterator,
    StringIterator,
};
use builtins::value::JsValue;

// ====================
// IteratorResult Tests
// ====================

#[test]
fn test_iterator_result_contract_value() {
    let result = IteratorResult::value(JsValue::number(42.0));
    assert!(!result.done, "Iterator result should not be done when yielding value");
    assert_eq!(
        result.value.as_number().unwrap(),
        42.0,
        "Iterator result value should match"
    );
}

#[test]
fn test_iterator_result_contract_done() {
    let result = IteratorResult::done();
    assert!(result.done, "Done result must have done=true");
    assert!(
        result.value.is_undefined(),
        "Done result without value should have undefined"
    );
}

#[test]
fn test_iterator_result_contract_to_js_object() {
    let result = IteratorResult::value(JsValue::string("test"));
    let obj = result.to_js_value();

    assert!(obj.is_object(), "Iterator result should convert to object");
    assert_eq!(
        obj.get("value").unwrap().as_string().unwrap(),
        "test",
        "Object should have value property"
    );
    assert_eq!(
        obj.get("done").unwrap().as_boolean().unwrap(),
        false,
        "Object should have done property"
    );
}

#[test]
fn test_iterator_result_contract_from_js_object() {
    let obj = JsValue::object();
    obj.set("value", JsValue::number(100.0));
    obj.set("done", JsValue::boolean(true));

    let result = IteratorResult::from_js_value(&obj).unwrap();
    assert!(result.done);
    assert_eq!(result.value.as_number().unwrap(), 100.0);
}

#[test]
fn test_iterator_result_contract_from_invalid_type() {
    let not_object = JsValue::number(42.0);
    let result = IteratorResult::from_js_value(&not_object);
    assert!(result.is_err(), "Should fail for non-object input");
}

// ====================
// Generator Contract Tests
// ====================

#[test]
fn test_generator_contract_initial_state() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    assert_eq!(
        gen.state(),
        GeneratorState::Suspended,
        "New generator must be in Suspended state"
    );
}

#[test]
fn test_generator_contract_next_returns_iterator_result() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    let result = gen.next(None).unwrap();

    // Must return {value: any, done: boolean}
    assert!(!result.done);
    assert!(result.value.is_number());
}

#[test]
fn test_generator_contract_exhausted_returns_done() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    let _ = gen.next(None); // Consume first value
    let result = gen.next(None).unwrap();

    assert!(result.done, "Exhausted generator must return done=true");
}

#[test]
fn test_generator_contract_state_after_exhaustion() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    let _ = gen.next(None);
    let _ = gen.next(None);

    assert_eq!(
        gen.state(),
        GeneratorState::Closed,
        "Exhausted generator must be in Closed state"
    );
}

#[test]
fn test_generator_contract_return_closes_generator() {
    let gen = GeneratorFunction::from_values(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = gen.return_value(Some(JsValue::string("done"))).unwrap();

    assert!(result.done, "return() must set done=true");
    assert_eq!(
        result.value.as_string().unwrap(),
        "done",
        "return() must include provided value"
    );
    assert_eq!(
        gen.state(),
        GeneratorState::Closed,
        "return() must close generator"
    );
}

#[test]
fn test_generator_contract_throw_closes_generator() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    let result = gen.throw(JsValue::string("error"));

    assert!(result.is_err(), "throw() must return error");
    assert_eq!(
        gen.state(),
        GeneratorState::Closed,
        "throw() must close generator"
    );
}

#[test]
fn test_generator_contract_is_iterable() {
    let gen = GeneratorFunction::empty();
    assert!(
        gen.is_iterable(),
        "Generator must implement Symbol.iterator (return self)"
    );
}

#[test]
fn test_generator_contract_multiple_values() {
    let gen = GeneratorFunction::from_values(vec![
        JsValue::string("a"),
        JsValue::string("b"),
        JsValue::string("c"),
    ]);

    let r1 = gen.next(None).unwrap();
    assert_eq!(r1.value.as_string().unwrap(), "a");
    assert!(!r1.done);

    let r2 = gen.next(None).unwrap();
    assert_eq!(r2.value.as_string().unwrap(), "b");
    assert!(!r2.done);

    let r3 = gen.next(None).unwrap();
    assert_eq!(r3.value.as_string().unwrap(), "c");
    assert!(!r3.done);

    let r4 = gen.next(None).unwrap();
    assert!(r4.done);
}

// ====================
// ArrayIterator Contract Tests
// ====================

#[test]
fn test_array_iterator_contract_values() {
    let arr = JsValue::array_from(vec![
        JsValue::number(10.0),
        JsValue::number(20.0),
    ]);

    let mut iter = ArrayIterator::new(arr);

    let r1 = iter.next();
    assert_eq!(r1.value.as_number().unwrap(), 10.0);
    assert!(!r1.done);

    let r2 = iter.next();
    assert_eq!(r2.value.as_number().unwrap(), 20.0);
    assert!(!r2.done);

    let r3 = iter.next();
    assert!(r3.done);
}

#[test]
fn test_array_iterator_contract_keys() {
    let arr = JsValue::array_from(vec![
        JsValue::string("a"),
        JsValue::string("b"),
    ]);

    let mut iter = ArrayIterator::keys(arr);

    assert_eq!(iter.next().value.as_number().unwrap(), 0.0);
    assert_eq!(iter.next().value.as_number().unwrap(), 1.0);
    assert!(iter.next().done);
}

#[test]
fn test_array_iterator_contract_entries() {
    let arr = JsValue::array_from(vec![JsValue::string("x")]);

    let mut iter = ArrayIterator::entries(arr);
    let entry = iter.next();

    assert!(!entry.done);
    let entry_arr = entry.value;
    assert!(entry_arr.is_array());

    if let JsValue::Array(a) = entry_arr {
        let elems = &a.borrow().elements;
        assert_eq!(elems[0].as_number().unwrap(), 0.0); // Key
        assert_eq!(elems[1].as_string().unwrap(), "x"); // Value
    }
}

#[test]
fn test_array_iterator_contract_empty_array() {
    let arr = JsValue::array_from(vec![]);
    let mut iter = ArrayIterator::new(arr);

    assert!(iter.next().done, "Empty array iterator must be immediately done");
}

// ====================
// StringIterator Contract Tests
// ====================

#[test]
fn test_string_iterator_contract_basic() {
    let mut iter = StringIterator::new("AB".to_string());

    assert_eq!(iter.next().value.as_string().unwrap(), "A");
    assert_eq!(iter.next().value.as_string().unwrap(), "B");
    assert!(iter.next().done);
}

#[test]
fn test_string_iterator_contract_unicode() {
    let mut iter = StringIterator::new("A\u{1F600}".to_string());

    // Must iterate by Unicode code points, not UTF-16 code units
    assert_eq!(iter.next().value.as_string().unwrap(), "A");
    assert_eq!(iter.next().value.as_string().unwrap(), "\u{1F600}");
    assert!(iter.next().done);
}

#[test]
fn test_string_iterator_contract_empty() {
    let mut iter = StringIterator::new("".to_string());
    assert!(iter.next().done);
}

// ====================
// ObjectIterator Contract Tests
// ====================

#[test]
fn test_object_keys_iterator_contract() {
    let obj = JsValue::object();
    obj.set("a", JsValue::number(1.0));
    obj.set("b", JsValue::number(2.0));

    let mut iter = ObjectIterator::keys(&obj);
    let keys: Vec<String> = IteratorHelpers::to_array(&mut iter)
        .into_iter()
        .filter_map(|v| v.as_string())
        .collect();

    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
}

#[test]
fn test_object_values_iterator_contract() {
    let obj = JsValue::object();
    obj.set("x", JsValue::number(100.0));

    let mut iter = ObjectIterator::values(&obj);
    let values = IteratorHelpers::to_array(&mut iter);

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].as_number().unwrap(), 100.0);
}

#[test]
fn test_object_entries_iterator_contract() {
    let obj = JsValue::object();
    obj.set("key", JsValue::string("value"));

    let mut iter = ObjectIterator::entries(&obj);
    let entry = iter.next();

    assert!(!entry.done);
    if let JsValue::Array(arr) = entry.value {
        let elems = &arr.borrow().elements;
        assert_eq!(elems[0].as_string().unwrap(), "key");
        assert_eq!(elems[1].as_string().unwrap(), "value");
    } else {
        panic!("Entry must be an array");
    }
}

// ====================
// Iterator.from Contract Tests
// ====================

#[test]
fn test_iterator_from_contract_array() {
    let arr = JsValue::array_from(vec![JsValue::number(1.0)]);
    let iter = Iterator::from(&arr);

    assert!(iter.is_ok(), "Array must be iterable");
}

#[test]
fn test_iterator_from_contract_string() {
    let s = JsValue::string("test");
    let iter = Iterator::from(&s);

    assert!(iter.is_ok(), "String must be iterable");
}

#[test]
fn test_iterator_from_contract_map() {
    let map = JsValue::map();
    let iter = Iterator::from(&map);

    assert!(iter.is_ok(), "Map must be iterable");
}

#[test]
fn test_iterator_from_contract_set() {
    let set = JsValue::set_collection();
    let iter = Iterator::from(&set);

    assert!(iter.is_ok(), "Set must be iterable");
}

#[test]
fn test_iterator_from_contract_non_iterable() {
    let num = JsValue::number(42.0);
    let iter = Iterator::from(&num);

    assert!(iter.is_err(), "Number must not be iterable");
}

// ====================
// Iterator Helper Methods Contract Tests
// ====================

#[test]
fn test_iterator_helpers_map_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let mapped = IteratorHelpers::map(&mut iter, |v| {
        Ok(JsValue::number(v.as_number().unwrap() * 10.0))
    });

    assert_eq!(mapped.len(), 2);
    assert_eq!(mapped[0].as_number().unwrap(), 10.0);
    assert_eq!(mapped[1].as_number().unwrap(), 20.0);
}

#[test]
fn test_iterator_helpers_filter_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let filtered = IteratorHelpers::filter(&mut iter, |v| {
        v.as_number().unwrap() > 1.5
    });

    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].as_number().unwrap(), 2.0);
    assert_eq!(filtered[1].as_number().unwrap(), 3.0);
}

#[test]
fn test_iterator_helpers_take_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let taken = IteratorHelpers::take(&mut iter, 2);

    assert_eq!(taken.len(), 2, "take(n) must return at most n elements");
}

#[test]
fn test_iterator_helpers_drop_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let dropped = IteratorHelpers::drop(&mut iter, 1);

    assert_eq!(dropped.len(), 2, "drop(n) must skip first n elements");
    assert_eq!(dropped[0].as_number().unwrap(), 2.0);
}

#[test]
fn test_iterator_helpers_reduce_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let sum = IteratorHelpers::reduce(&mut iter, JsValue::number(0.0), |acc, val| {
        Ok(JsValue::number(
            acc.as_number().unwrap() + val.as_number().unwrap(),
        ))
    })
    .unwrap();

    assert_eq!(sum.as_number().unwrap(), 6.0);
}

#[test]
fn test_iterator_helpers_find_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(5.0),
        JsValue::number(3.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let found = IteratorHelpers::find(&mut iter, |v| v.as_number().unwrap() > 4.0);

    assert!(found.is_some());
    assert_eq!(found.unwrap().as_number().unwrap(), 5.0);
}

#[test]
fn test_iterator_helpers_some_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let has_even = IteratorHelpers::some(&mut iter, |v| {
        v.as_number().unwrap() % 2.0 == 0.0
    });

    assert!(has_even, "some() must return true if any element matches");
}

#[test]
fn test_iterator_helpers_every_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(2.0),
        JsValue::number(4.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let all_even = IteratorHelpers::every(&mut iter, |v| {
        v.as_number().unwrap() % 2.0 == 0.0
    });

    assert!(all_even, "every() must return true if all elements match");
}

#[test]
fn test_iterator_helpers_to_array_contract() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let mut iter = ArrayIterator::new(arr);
    let collected = IteratorHelpers::to_array(&mut iter);

    assert_eq!(collected.len(), 2);
}

// ====================
// JsValue Generator Integration Tests
// ====================

#[test]
fn test_jsvalue_generator_contract() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(42.0)]);
    let val = JsValue::generator(gen);

    assert!(val.is_generator(), "JsValue must identify Generator type");
    assert_eq!(val.type_of(), "object", "typeof generator must be 'object'");
    assert_eq!(
        val.to_js_string(),
        "[object Generator]",
        "Generator string representation"
    );
}

#[test]
fn test_jsvalue_generator_as_generator_contract() {
    let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);
    let val = JsValue::generator(gen);

    let retrieved = val.as_generator();
    assert!(retrieved.is_some());

    let gen_obj = retrieved.unwrap();
    let result = gen_obj.next(None).unwrap();
    assert_eq!(result.value.as_number().unwrap(), 1.0);
}
