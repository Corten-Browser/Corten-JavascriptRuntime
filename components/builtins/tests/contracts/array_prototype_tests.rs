//! Contract tests for ArrayPrototype

use builtins::{ArrayPrototype, JsValue, JsResult};

#[test]
fn test_array_push() {
    let arr = JsValue::array();
    let result = ArrayPrototype::push(&arr, JsValue::number(1.0));
    assert!(result.is_ok());
    assert_eq!(arr.array_length(), 1);
}

#[test]
fn test_array_pop() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::pop(&arr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::number(2.0));
    assert_eq!(arr.array_length(), 1);
}

#[test]
fn test_array_shift() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::shift(&arr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::number(1.0));
    assert_eq!(arr.array_length(), 1);
}

#[test]
fn test_array_unshift() {
    let arr = JsValue::array_from(vec![JsValue::number(2.0)]);
    let result = ArrayPrototype::unshift(&arr, JsValue::number(1.0));
    assert!(result.is_ok());
    assert_eq!(arr.array_length(), 2);
}

#[test]
fn test_array_slice() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::slice(&arr, 1, Some(3));
    assert!(result.is_ok());
    let sliced = result.unwrap();
    assert_eq!(sliced.array_length(), 2);
}

#[test]
fn test_array_splice() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::splice(&arr, 1, 1, vec![JsValue::number(4.0)]);
    assert!(result.is_ok());
}

#[test]
fn test_array_map() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    // Map with a function that doubles
    let result = ArrayPrototype::map(&arr, |v| {
        Ok(JsValue::number(v.as_number().unwrap() * 2.0))
    });
    assert!(result.is_ok());
}

#[test]
fn test_array_filter() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::filter(&arr, |v| {
        Ok(v.as_number().unwrap() > 1.0)
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap().array_length(), 2);
}

#[test]
fn test_array_reduce() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::reduce(&arr, JsValue::number(0.0), |acc, v| {
        Ok(JsValue::number(acc.as_number().unwrap() + v.as_number().unwrap()))
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_number().unwrap(), 6.0);
}

#[test]
fn test_array_for_each() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::for_each(&arr, |_| Ok(()));
    assert!(result.is_ok());
}

#[test]
fn test_array_find() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::find(&arr, |v| {
        Ok(v.as_number().unwrap() > 1.0)
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_number().unwrap(), 2.0);
}

#[test]
fn test_array_find_index() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::find_index(&arr, |v| {
        Ok(v.as_number().unwrap() > 1.0)
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_array_includes() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::includes(&arr, &JsValue::number(2.0));
    assert!(result);
}

#[test]
fn test_array_index_of() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::index_of(&arr, &JsValue::number(2.0));
    assert_eq!(result, Some(1));
}

#[test]
fn test_array_join() {
    let arr = JsValue::array_from(vec![
        JsValue::string("a"),
        JsValue::string("b"),
        JsValue::string("c"),
    ]);

    let result = ArrayPrototype::join(&arr, ",");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "a,b,c");
}

#[test]
fn test_array_sort() {
    let arr = JsValue::array_from(vec![
        JsValue::number(3.0),
        JsValue::number(1.0),
        JsValue::number(2.0),
    ]);

    let result = ArrayPrototype::sort(&arr);
    assert!(result.is_ok());
}

#[test]
fn test_array_reverse() {
    let arr = JsValue::array_from(vec![
        JsValue::number(1.0),
        JsValue::number(2.0),
        JsValue::number(3.0),
    ]);

    let result = ArrayPrototype::reverse(&arr);
    assert!(result.is_ok());
}
