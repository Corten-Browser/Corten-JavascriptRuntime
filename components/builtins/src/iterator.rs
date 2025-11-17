//! JavaScript Iterator and Generator protocol implementation
//!
//! This module implements:
//! - Iterator protocol (IteratorResult, Iterator interface)
//! - Generator functions and objects
//! - Built-in iterators (Array, String, Map, Set, Object)
//! - Iterator helper methods (map, filter, take, drop, etc.)

use std::cell::RefCell;
use std::rc::Rc;

use crate::symbol::SymbolConstructor;
use crate::value::{JsError, JsResult, JsValue};

/// Iterator result object { value, done }
#[derive(Debug, Clone)]
pub struct IteratorResult {
    /// The value returned by the iterator
    pub value: JsValue,
    /// Whether the iterator is exhausted
    pub done: bool,
}

impl IteratorResult {
    /// Create a new iterator result with a value
    pub fn value(v: JsValue) -> Self {
        IteratorResult {
            value: v,
            done: false,
        }
    }

    /// Create a done iterator result
    pub fn done() -> Self {
        IteratorResult {
            value: JsValue::undefined(),
            done: true,
        }
    }

    /// Create a done result with a final value
    pub fn done_with_value(v: JsValue) -> Self {
        IteratorResult {
            value: v,
            done: true,
        }
    }

    /// Convert to JsValue object representation
    pub fn to_js_value(&self) -> JsValue {
        let obj = JsValue::object();
        obj.set("value", self.value.clone());
        obj.set("done", JsValue::boolean(self.done));
        obj
    }

    /// Create from JsValue object
    pub fn from_js_value(obj: &JsValue) -> JsResult<Self> {
        if !obj.is_object() {
            return Err(JsError::type_error("Iterator result must be an object"));
        }

        let value = obj.get("value").unwrap_or(JsValue::undefined());
        let done = obj
            .get("done")
            .and_then(|d| d.as_boolean())
            .unwrap_or(false);

        Ok(IteratorResult { value, done })
    }
}

/// Generator state enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeneratorState {
    /// Generator is created but not yet started
    Suspended,
    /// Generator is currently executing
    Executing,
    /// Generator has completed (return or end of function)
    Closed,
}

/// Internal data for a Generator object
#[derive(Debug)]
pub struct GeneratorData {
    /// Current state of the generator
    pub state: GeneratorState,
    /// Values yielded by the generator
    pub values: Vec<JsValue>,
    /// Current position in the values
    pub position: usize,
    /// The result to return when done
    pub return_value: Option<JsValue>,
}

impl Clone for GeneratorData {
    fn clone(&self) -> Self {
        GeneratorData {
            state: self.state,
            values: self.values.clone(),
            position: self.position,
            return_value: self.return_value.clone(),
        }
    }
}

/// Generator object implementation
#[derive(Debug, Clone)]
pub struct GeneratorObject {
    /// Internal generator data
    data: Rc<RefCell<GeneratorData>>,
}

impl GeneratorObject {
    /// Create a new generator with preset yield values
    pub fn new(values: Vec<JsValue>) -> Self {
        GeneratorObject {
            data: Rc::new(RefCell::new(GeneratorData {
                state: GeneratorState::Suspended,
                values,
                position: 0,
                return_value: None,
            })),
        }
    }

    /// Get the current state
    pub fn state(&self) -> GeneratorState {
        self.data.borrow().state
    }

    /// Generator.prototype.next(value?)
    ///
    /// Resumes the generator, optionally passing a value into it.
    pub fn next(&self, _value: Option<JsValue>) -> JsResult<IteratorResult> {
        let mut data = self.data.borrow_mut();

        match data.state {
            GeneratorState::Closed => Ok(IteratorResult::done()),
            GeneratorState::Executing => {
                Err(JsError::type_error("Generator is already executing"))
            }
            GeneratorState::Suspended => {
                data.state = GeneratorState::Executing;

                if data.position < data.values.len() {
                    let value = data.values[data.position].clone();
                    data.position += 1;
                    data.state = GeneratorState::Suspended;
                    Ok(IteratorResult::value(value))
                } else {
                    data.state = GeneratorState::Closed;
                    match &data.return_value {
                        Some(v) => Ok(IteratorResult::done_with_value(v.clone())),
                        None => Ok(IteratorResult::done()),
                    }
                }
            }
        }
    }

    /// Generator.prototype.return(value?)
    ///
    /// Returns the given value and finishes the generator.
    pub fn return_value(&self, value: Option<JsValue>) -> JsResult<IteratorResult> {
        let mut data = self.data.borrow_mut();

        if data.state == GeneratorState::Executing {
            return Err(JsError::type_error("Generator is already executing"));
        }

        data.state = GeneratorState::Closed;
        let return_val = value.unwrap_or(JsValue::undefined());
        Ok(IteratorResult::done_with_value(return_val))
    }

    /// Generator.prototype.throw(exception)
    ///
    /// Throws an exception into the generator.
    pub fn throw(&self, exception: JsValue) -> JsResult<IteratorResult> {
        let mut data = self.data.borrow_mut();

        if data.state == GeneratorState::Executing {
            return Err(JsError::type_error("Generator is already executing"));
        }

        data.state = GeneratorState::Closed;

        // In a real implementation, this would propagate the exception through the generator
        // For now, we just close the generator and return the exception
        Err(JsError::new(exception.to_js_string()))
    }

    /// Check if generator is iterable (Symbol.iterator returns self)
    pub fn is_iterable(&self) -> bool {
        true
    }
}

/// Generator function constructor
pub struct GeneratorFunction;

impl GeneratorFunction {
    /// Create a generator from a sequence of values
    ///
    /// This is a simplified version that creates a generator yielding preset values.
    pub fn from_values(values: Vec<JsValue>) -> GeneratorObject {
        GeneratorObject::new(values)
    }

    /// Create an empty generator
    pub fn empty() -> GeneratorObject {
        GeneratorObject::new(vec![])
    }
}

/// Array iterator implementation
#[derive(Debug, Clone)]
pub struct ArrayIterator {
    array: JsValue,
    index: usize,
    kind: IteratorKind,
}

/// Kind of iterator (keys, values, or entries)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IteratorKind {
    /// Iterate over keys (indices)
    Keys,
    /// Iterate over values
    Values,
    /// Iterate over [key, value] pairs
    Entries,
}

impl ArrayIterator {
    /// Create a new array iterator for values
    pub fn new(array: JsValue) -> Self {
        ArrayIterator {
            array,
            index: 0,
            kind: IteratorKind::Values,
        }
    }

    /// Create an iterator for keys
    pub fn keys(array: JsValue) -> Self {
        ArrayIterator {
            array,
            index: 0,
            kind: IteratorKind::Keys,
        }
    }

    /// Create an iterator for entries
    pub fn entries(array: JsValue) -> Self {
        ArrayIterator {
            array,
            index: 0,
            kind: IteratorKind::Entries,
        }
    }

    /// Get the next iterator result
    pub fn next(&mut self) -> IteratorResult {
        let len = self.array.array_length();

        if self.index >= len {
            return IteratorResult::done();
        }

        let result = match self.kind {
            IteratorKind::Keys => IteratorResult::value(JsValue::number(self.index as f64)),
            IteratorKind::Values => {
                if let JsValue::Array(arr) = &self.array {
                    let value = arr.borrow().elements[self.index].clone();
                    IteratorResult::value(value)
                } else {
                    IteratorResult::done()
                }
            }
            IteratorKind::Entries => {
                if let JsValue::Array(arr) = &self.array {
                    let value = arr.borrow().elements[self.index].clone();
                    let entry = JsValue::array_from(vec![
                        JsValue::number(self.index as f64),
                        value,
                    ]);
                    IteratorResult::value(entry)
                } else {
                    IteratorResult::done()
                }
            }
        };

        self.index += 1;
        result
    }
}

/// String iterator implementation
#[derive(Debug, Clone)]
pub struct StringIterator {
    string: String,
    index: usize,
}

impl StringIterator {
    /// Create a new string iterator
    pub fn new(s: String) -> Self {
        StringIterator {
            string: s,
            index: 0,
        }
    }

    /// Get the next iterator result
    pub fn next(&mut self) -> IteratorResult {
        let chars: Vec<char> = self.string.chars().collect();

        if self.index >= chars.len() {
            return IteratorResult::done();
        }

        let ch = chars[self.index].to_string();
        self.index += 1;
        IteratorResult::value(JsValue::string(ch))
    }
}

/// Object keys/values/entries iterator
#[derive(Debug, Clone)]
pub struct ObjectIterator {
    keys: Vec<String>,
    values: Vec<JsValue>,
    index: usize,
    kind: IteratorKind,
}

impl ObjectIterator {
    /// Create a keys iterator for an object
    pub fn keys(obj: &JsValue) -> Self {
        let (keys, values) = Self::extract_object_data(obj);
        ObjectIterator {
            keys,
            values,
            index: 0,
            kind: IteratorKind::Keys,
        }
    }

    /// Create a values iterator for an object
    pub fn values(obj: &JsValue) -> Self {
        let (keys, values) = Self::extract_object_data(obj);
        ObjectIterator {
            keys,
            values,
            index: 0,
            kind: IteratorKind::Values,
        }
    }

    /// Create an entries iterator for an object
    pub fn entries(obj: &JsValue) -> Self {
        let (keys, values) = Self::extract_object_data(obj);
        ObjectIterator {
            keys,
            values,
            index: 0,
            kind: IteratorKind::Entries,
        }
    }

    fn extract_object_data(obj: &JsValue) -> (Vec<String>, Vec<JsValue>) {
        if let JsValue::Object(o) = obj {
            let props = &o.borrow().properties;
            let keys: Vec<String> = props.keys().cloned().collect();
            let values: Vec<JsValue> = keys.iter().map(|k| props.get(k).unwrap().clone()).collect();
            (keys, values)
        } else {
            (vec![], vec![])
        }
    }

    /// Get the next iterator result
    pub fn next(&mut self) -> IteratorResult {
        if self.index >= self.keys.len() {
            return IteratorResult::done();
        }

        let result = match self.kind {
            IteratorKind::Keys => {
                IteratorResult::value(JsValue::string(self.keys[self.index].clone()))
            }
            IteratorKind::Values => IteratorResult::value(self.values[self.index].clone()),
            IteratorKind::Entries => {
                let entry = JsValue::array_from(vec![
                    JsValue::string(self.keys[self.index].clone()),
                    self.values[self.index].clone(),
                ]);
                IteratorResult::value(entry)
            }
        };

        self.index += 1;
        result
    }
}

/// Iterator helper to create iterators from various sources
pub struct Iterator;

impl Iterator {
    /// Iterator.from(object) - create an iterator from an iterable or array-like object
    pub fn from(obj: &JsValue) -> JsResult<Box<dyn IteratorProtocol>> {
        match obj {
            JsValue::Array(_) => Ok(Box::new(ArrayIterator::new(obj.clone()))),
            JsValue::String(s) => Ok(Box::new(StringIterator::new(s.clone()))),
            JsValue::Map(_) => {
                // For Map, iterate over entries
                if let JsValue::Map(map) = obj {
                    let entries = map.borrow().entries.clone();
                    let values: Vec<JsValue> = entries
                        .into_iter()
                        .map(|(k, v)| JsValue::array_from(vec![k, v]))
                        .collect();
                    let gen = GeneratorFunction::from_values(values);
                    Ok(Box::new(GeneratorIteratorAdapter(gen)))
                } else {
                    Err(JsError::type_error("Invalid map"))
                }
            }
            JsValue::Set(_) => {
                // For Set, iterate over values
                if let JsValue::Set(set) = obj {
                    let values = set.borrow().values.clone();
                    let gen = GeneratorFunction::from_values(values);
                    Ok(Box::new(GeneratorIteratorAdapter(gen)))
                } else {
                    Err(JsError::type_error("Invalid set"))
                }
            }
            JsValue::Object(o) => {
                // Check if object has Symbol.iterator method
                let iter_sym = SymbolConstructor::iterator();
                if o.borrow().symbol_properties.contains_key(&iter_sym.id()) {
                    // Object has custom iterator
                    Err(JsError::type_error("Custom iterators not yet supported"))
                } else {
                    // Treat as array-like if it has length property
                    Ok(Box::new(ObjectIterator::values(obj)))
                }
            }
            _ => Err(JsError::type_error(
                "Object is not iterable",
            )),
        }
    }
}

/// Trait for iterator protocol
pub trait IteratorProtocol {
    /// Get the next value from the iterator
    fn next(&mut self) -> IteratorResult;
}

impl IteratorProtocol for ArrayIterator {
    fn next(&mut self) -> IteratorResult {
        self.next()
    }
}

impl IteratorProtocol for StringIterator {
    fn next(&mut self) -> IteratorResult {
        self.next()
    }
}

impl IteratorProtocol for ObjectIterator {
    fn next(&mut self) -> IteratorResult {
        self.next()
    }
}

/// Adapter to make GeneratorObject implement IteratorProtocol
struct GeneratorIteratorAdapter(GeneratorObject);

impl IteratorProtocol for GeneratorIteratorAdapter {
    fn next(&mut self) -> IteratorResult {
        self.0.next(None).unwrap_or_else(|_| IteratorResult::done())
    }
}

/// Iterator helper methods (ES2024 Iterator Helpers)
pub struct IteratorHelpers;

impl IteratorHelpers {
    /// Map over iterator values
    pub fn map<F>(
        iter: &mut dyn IteratorProtocol,
        mapper: F,
    ) -> Vec<JsValue>
    where
        F: Fn(JsValue) -> JsResult<JsValue>,
    {
        let mut results = vec![];
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            if let Ok(mapped) = mapper(result.value) {
                results.push(mapped);
            }
        }
        results
    }

    /// Filter iterator values
    pub fn filter<F>(iter: &mut dyn IteratorProtocol, predicate: F) -> Vec<JsValue>
    where
        F: Fn(&JsValue) -> bool,
    {
        let mut results = vec![];
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            if predicate(&result.value) {
                results.push(result.value);
            }
        }
        results
    }

    /// Take first n values from iterator
    pub fn take(iter: &mut dyn IteratorProtocol, n: usize) -> Vec<JsValue> {
        let mut results = vec![];
        for _ in 0..n {
            let result = iter.next();
            if result.done {
                break;
            }
            results.push(result.value);
        }
        results
    }

    /// Drop first n values from iterator
    pub fn drop(iter: &mut dyn IteratorProtocol, n: usize) -> Vec<JsValue> {
        // Skip n values
        for _ in 0..n {
            let result = iter.next();
            if result.done {
                return vec![];
            }
        }

        // Collect the rest
        let mut results = vec![];
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            results.push(result.value);
        }
        results
    }

    /// Reduce iterator to a single value
    pub fn reduce<F>(
        iter: &mut dyn IteratorProtocol,
        initial: JsValue,
        reducer: F,
    ) -> JsResult<JsValue>
    where
        F: Fn(JsValue, JsValue) -> JsResult<JsValue>,
    {
        let mut accumulator = initial;
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            accumulator = reducer(accumulator, result.value)?;
        }
        Ok(accumulator)
    }

    /// Convert iterator to array
    pub fn to_array(iter: &mut dyn IteratorProtocol) -> Vec<JsValue> {
        let mut results = vec![];
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            results.push(result.value);
        }
        results
    }

    /// forEach - execute function for each value
    pub fn for_each<F>(iter: &mut dyn IteratorProtocol, callback: F)
    where
        F: Fn(JsValue),
    {
        loop {
            let result = iter.next();
            if result.done {
                break;
            }
            callback(result.value);
        }
    }

    /// find - return first value matching predicate
    pub fn find<F>(iter: &mut dyn IteratorProtocol, predicate: F) -> Option<JsValue>
    where
        F: Fn(&JsValue) -> bool,
    {
        loop {
            let result = iter.next();
            if result.done {
                return None;
            }
            if predicate(&result.value) {
                return Some(result.value);
            }
        }
    }

    /// some - check if any value matches predicate
    pub fn some<F>(iter: &mut dyn IteratorProtocol, predicate: F) -> bool
    where
        F: Fn(&JsValue) -> bool,
    {
        loop {
            let result = iter.next();
            if result.done {
                return false;
            }
            if predicate(&result.value) {
                return true;
            }
        }
    }

    /// every - check if all values match predicate
    pub fn every<F>(iter: &mut dyn IteratorProtocol, predicate: F) -> bool
    where
        F: Fn(&JsValue) -> bool,
    {
        loop {
            let result = iter.next();
            if result.done {
                return true;
            }
            if !predicate(&result.value) {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_result_value() {
        let result = IteratorResult::value(JsValue::number(42.0));
        assert!(!result.done);
        assert_eq!(result.value.as_number(), Some(42.0));
    }

    #[test]
    fn test_iterator_result_done() {
        let result = IteratorResult::done();
        assert!(result.done);
        assert!(result.value.is_undefined());
    }

    #[test]
    fn test_iterator_result_done_with_value() {
        let result = IteratorResult::done_with_value(JsValue::string("final"));
        assert!(result.done);
        assert_eq!(result.value.as_string(), Some("final".to_string()));
    }

    #[test]
    fn test_iterator_result_to_js_value() {
        let result = IteratorResult::value(JsValue::number(10.0));
        let obj = result.to_js_value();

        assert!(obj.is_object());
        assert_eq!(obj.get("value").unwrap().as_number(), Some(10.0));
        assert_eq!(obj.get("done").unwrap().as_boolean(), Some(false));
    }

    #[test]
    fn test_iterator_result_from_js_value() {
        let obj = JsValue::object();
        obj.set("value", JsValue::string("test"));
        obj.set("done", JsValue::boolean(true));

        let result = IteratorResult::from_js_value(&obj).unwrap();
        assert_eq!(result.value.as_string(), Some("test".to_string()));
        assert!(result.done);
    }

    #[test]
    fn test_generator_creation() {
        let gen = GeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        assert_eq!(gen.state(), GeneratorState::Suspended);
    }

    #[test]
    fn test_generator_next() {
        let gen = GeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
        ]);

        let r1 = gen.next(None).unwrap();
        assert!(!r1.done);
        assert_eq!(r1.value.as_number(), Some(1.0));

        let r2 = gen.next(None).unwrap();
        assert!(!r2.done);
        assert_eq!(r2.value.as_number(), Some(2.0));

        let r3 = gen.next(None).unwrap();
        assert!(r3.done);
    }

    #[test]
    fn test_generator_return() {
        let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        let result = gen.return_value(Some(JsValue::string("done"))).unwrap();
        assert!(result.done);
        assert_eq!(result.value.as_string(), Some("done".to_string()));

        // After return, next should return done
        let next = gen.next(None).unwrap();
        assert!(next.done);
    }

    #[test]
    fn test_generator_throw() {
        let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        let result = gen.throw(JsValue::string("Error!"));
        assert!(result.is_err());

        // After throw, generator should be closed
        assert_eq!(gen.state(), GeneratorState::Closed);
    }

    #[test]
    fn test_empty_generator() {
        let gen = GeneratorFunction::empty();

        let result = gen.next(None).unwrap();
        assert!(result.done);
    }

    #[test]
    fn test_array_iterator_values() {
        let arr = JsValue::array_from(vec![
            JsValue::number(10.0),
            JsValue::number(20.0),
            JsValue::number(30.0),
        ]);

        let mut iter = ArrayIterator::new(arr);

        let r1 = iter.next();
        assert!(!r1.done);
        assert_eq!(r1.value.as_number(), Some(10.0));

        let r2 = iter.next();
        assert_eq!(r2.value.as_number(), Some(20.0));

        let r3 = iter.next();
        assert_eq!(r3.value.as_number(), Some(30.0));

        let r4 = iter.next();
        assert!(r4.done);
    }

    #[test]
    fn test_array_iterator_keys() {
        let arr = JsValue::array_from(vec![
            JsValue::string("a"),
            JsValue::string("b"),
            JsValue::string("c"),
        ]);

        let mut iter = ArrayIterator::keys(arr);

        assert_eq!(iter.next().value.as_number(), Some(0.0));
        assert_eq!(iter.next().value.as_number(), Some(1.0));
        assert_eq!(iter.next().value.as_number(), Some(2.0));
        assert!(iter.next().done);
    }

    #[test]
    fn test_array_iterator_entries() {
        let arr = JsValue::array_from(vec![JsValue::string("x"), JsValue::string("y")]);

        let mut iter = ArrayIterator::entries(arr);

        let entry1 = iter.next();
        assert!(!entry1.done);
        if let JsValue::Array(e) = entry1.value {
            assert_eq!(e.borrow().elements[0].as_number(), Some(0.0));
            assert_eq!(e.borrow().elements[1].as_string(), Some("x".to_string()));
        } else {
            panic!("Expected array entry");
        }
    }

    #[test]
    fn test_string_iterator() {
        let mut iter = StringIterator::new("Hi!".to_string());

        assert_eq!(iter.next().value.as_string(), Some("H".to_string()));
        assert_eq!(iter.next().value.as_string(), Some("i".to_string()));
        assert_eq!(iter.next().value.as_string(), Some("!".to_string()));
        assert!(iter.next().done);
    }

    #[test]
    fn test_string_iterator_unicode() {
        let mut iter = StringIterator::new("A\u{1F600}Z".to_string()); // A + emoji + Z

        assert_eq!(iter.next().value.as_string(), Some("A".to_string()));
        assert_eq!(iter.next().value.as_string(), Some("\u{1F600}".to_string())); // emoji
        assert_eq!(iter.next().value.as_string(), Some("Z".to_string()));
        assert!(iter.next().done);
    }

    #[test]
    fn test_object_iterator_keys() {
        let obj = JsValue::object();
        obj.set("a", JsValue::number(1.0));
        obj.set("b", JsValue::number(2.0));

        let mut iter = ObjectIterator::keys(&obj);
        let keys: Vec<String> = IteratorHelpers::to_array(&mut iter)
            .into_iter()
            .filter_map(|v| v.as_string())
            .collect();

        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_object_iterator_values() {
        let obj = JsValue::object();
        obj.set("x", JsValue::number(100.0));
        obj.set("y", JsValue::number(200.0));

        let mut iter = ObjectIterator::values(&obj);
        let values: Vec<f64> = IteratorHelpers::to_array(&mut iter)
            .into_iter()
            .filter_map(|v| v.as_number())
            .collect();

        assert!(values.contains(&100.0));
        assert!(values.contains(&200.0));
    }

    #[test]
    fn test_object_iterator_entries() {
        let obj = JsValue::object();
        obj.set("key", JsValue::string("value"));

        let mut iter = ObjectIterator::entries(&obj);
        let entry = iter.next();
        assert!(!entry.done);

        if let JsValue::Array(arr) = entry.value {
            let elems = &arr.borrow().elements;
            assert_eq!(elems[0].as_string(), Some("key".to_string()));
            assert_eq!(elems[1].as_string(), Some("value".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_iterator_from_array() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);

        let mut iter = Iterator::from(&arr).unwrap();
        let values = IteratorHelpers::to_array(&mut *iter);

        assert_eq!(values.len(), 2);
        assert_eq!(values[0].as_number(), Some(1.0));
        assert_eq!(values[1].as_number(), Some(2.0));
    }

    #[test]
    fn test_iterator_from_string() {
        let s = JsValue::string("ab");
        let mut iter = Iterator::from(&s).unwrap();
        let values = IteratorHelpers::to_array(&mut *iter);

        assert_eq!(values.len(), 2);
        assert_eq!(values[0].as_string(), Some("a".to_string()));
        assert_eq!(values[1].as_string(), Some("b".to_string()));
    }

    #[test]
    fn test_iterator_helpers_map() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let mapped = IteratorHelpers::map(&mut iter, |v| {
            Ok(JsValue::number(v.as_number().unwrap() * 2.0))
        });

        assert_eq!(mapped.len(), 3);
        assert_eq!(mapped[0].as_number(), Some(2.0));
        assert_eq!(mapped[1].as_number(), Some(4.0));
        assert_eq!(mapped[2].as_number(), Some(6.0));
    }

    #[test]
    fn test_iterator_helpers_filter() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let filtered = IteratorHelpers::filter(&mut iter, |v| {
            v.as_number().unwrap() > 2.0
        });

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].as_number(), Some(3.0));
        assert_eq!(filtered[1].as_number(), Some(4.0));
    }

    #[test]
    fn test_iterator_helpers_take() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
            JsValue::number(5.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let taken = IteratorHelpers::take(&mut iter, 3);

        assert_eq!(taken.len(), 3);
        assert_eq!(taken[0].as_number(), Some(1.0));
        assert_eq!(taken[1].as_number(), Some(2.0));
        assert_eq!(taken[2].as_number(), Some(3.0));
    }

    #[test]
    fn test_iterator_helpers_drop() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let dropped = IteratorHelpers::drop(&mut iter, 2);

        assert_eq!(dropped.len(), 2);
        assert_eq!(dropped[0].as_number(), Some(3.0));
        assert_eq!(dropped[1].as_number(), Some(4.0));
    }

    #[test]
    fn test_iterator_helpers_reduce() {
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

        assert_eq!(sum.as_number(), Some(6.0));
    }

    #[test]
    fn test_iterator_helpers_find() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let found = IteratorHelpers::find(&mut iter, |v| v.as_number().unwrap() > 1.5);

        assert!(found.is_some());
        assert_eq!(found.unwrap().as_number(), Some(2.0));
    }

    #[test]
    fn test_iterator_helpers_find_not_found() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);

        let mut iter = ArrayIterator::new(arr);
        let found = IteratorHelpers::find(&mut iter, |v| v.as_number().unwrap() > 10.0);

        assert!(found.is_none());
    }

    #[test]
    fn test_iterator_helpers_some() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        assert!(IteratorHelpers::some(&mut iter, |v| {
            v.as_number().unwrap() > 2.0
        }));
    }

    #[test]
    fn test_iterator_helpers_some_false() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);

        let mut iter = ArrayIterator::new(arr);
        assert!(!IteratorHelpers::some(&mut iter, |v| {
            v.as_number().unwrap() > 10.0
        }));
    }

    #[test]
    fn test_iterator_helpers_every() {
        let arr = JsValue::array_from(vec![
            JsValue::number(2.0),
            JsValue::number(4.0),
            JsValue::number(6.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        assert!(IteratorHelpers::every(&mut iter, |v| {
            v.as_number().unwrap() % 2.0 == 0.0
        }));
    }

    #[test]
    fn test_iterator_helpers_every_false() {
        let arr = JsValue::array_from(vec![
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        assert!(!IteratorHelpers::every(&mut iter, |v| {
            v.as_number().unwrap() % 2.0 == 0.0
        }));
    }

    #[test]
    fn test_iterator_helpers_for_each() {
        let arr = JsValue::array_from(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let mut iter = ArrayIterator::new(arr);
        let sum = Rc::new(RefCell::new(0.0));
        let sum_clone = sum.clone();

        IteratorHelpers::for_each(&mut iter, move |v| {
            *sum_clone.borrow_mut() += v.as_number().unwrap();
        });

        assert_eq!(*sum.borrow(), 6.0);
    }

    #[test]
    fn test_generator_state_transitions() {
        let gen = GeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        assert_eq!(gen.state(), GeneratorState::Suspended);

        let _ = gen.next(None);
        assert_eq!(gen.state(), GeneratorState::Suspended);

        let _ = gen.next(None);
        assert_eq!(gen.state(), GeneratorState::Closed);
    }

    #[test]
    fn test_generator_is_iterable() {
        let gen = GeneratorFunction::empty();
        assert!(gen.is_iterable());
    }

    #[test]
    fn test_take_more_than_available() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);

        let mut iter = ArrayIterator::new(arr);
        let taken = IteratorHelpers::take(&mut iter, 10);

        assert_eq!(taken.len(), 2); // Only 2 available
    }

    #[test]
    fn test_drop_more_than_available() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);

        let mut iter = ArrayIterator::new(arr);
        let dropped = IteratorHelpers::drop(&mut iter, 10);

        assert_eq!(dropped.len(), 0); // All dropped
    }
}
