//! JavaScript value representation for builtins
//!
//! This module provides a high-level JavaScript value type for use with built-in methods.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::async_generator::AsyncGeneratorObject;
use crate::error::JsErrorObject;
use crate::iterator::GeneratorObject;
use crate::regexp::RegExpObject;
use crate::symbol::SymbolValue;
use crate::weakref::{FinalizationRegistryData, WeakRefData};
use num_bigint::BigInt as NumBigInt;

/// BigInt value wrapper for arbitrary precision integers
///
/// This type wraps num_bigint::BigInt to provide ES2024-compliant
/// BigInt support in JavaScript values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigIntValue {
    inner: NumBigInt,
}

impl BigIntValue {
    /// Create a new BigIntValue from a NumBigInt
    pub fn new(inner: NumBigInt) -> Self {
        BigIntValue { inner }
    }

    /// Get a reference to the inner BigInt
    pub fn inner(&self) -> &NumBigInt {
        &self.inner
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

impl fmt::Display for BigIntValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Error type for JavaScript operations
#[derive(Debug, Clone, PartialEq)]
pub struct JsError {
    /// The error message
    pub message: String,
}

impl JsError {
    /// Create a new generic error
    pub fn new(message: impl Into<String>) -> Self {
        JsError {
            message: message.into(),
        }
    }

    /// Create a TypeError
    pub fn type_error(message: impl Into<String>) -> Self {
        JsError::new(format!("TypeError: {}", message.into()))
    }

    /// Create a SyntaxError
    pub fn syntax_error(message: impl Into<String>) -> Self {
        JsError::new(format!("SyntaxError: {}", message.into()))
    }

    /// Create a RangeError
    pub fn range_error(message: impl Into<String>) -> Self {
        JsError::new(format!("RangeError: {}", message.into()))
    }
}

/// Result type for JavaScript operations
pub type JsResult<T> = Result<T, JsError>;

/// Internal object data
#[derive(Debug, Clone)]
pub struct ObjectData {
    /// Object properties map
    pub properties: HashMap<String, JsValue>,
    /// Symbol-keyed properties
    pub symbol_properties: HashMap<u64, JsValue>,
    /// Optional prototype reference
    pub prototype: Option<Box<JsValue>>,
    /// Whether object is extensible (None means default true)
    pub extensible: Option<bool>,
}

/// Internal array data
#[derive(Debug, Clone)]
pub struct ArrayData {
    /// Array elements
    pub elements: Vec<JsValue>,
}

/// Internal map data - preserves insertion order
#[derive(Debug, Clone)]
pub struct MapData {
    /// Map entries in insertion order
    pub entries: Vec<(JsValue, JsValue)>,
}

/// Internal set data - preserves insertion order
#[derive(Debug, Clone)]
pub struct SetData {
    /// Set values in insertion order
    pub values: Vec<JsValue>,
}

/// Internal weak map data - keys are object pointers
#[derive(Debug, Clone)]
pub struct WeakMapData {
    /// Map entries keyed by object pointer address
    pub entries: HashMap<usize, JsValue>,
}

/// Internal weak set data - values are object pointers
#[derive(Debug, Clone)]
pub struct WeakSetData {
    /// Set of object pointer addresses
    pub values: HashMap<usize, ()>,
}

/// Internal function data
pub struct FunctionData {
    /// The function implementation
    pub func: Box<dyn Fn(JsValue, Vec<JsValue>) -> JsResult<JsValue>>,
}

impl std::fmt::Debug for FunctionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionData").finish()
    }
}

impl Clone for FunctionData {
    fn clone(&self) -> Self {
        // Functions are not truly clonable, but we need Clone for JsValue
        // In practice, functions are wrapped in Rc<RefCell<>>
        panic!("FunctionData cannot be directly cloned")
    }
}

/// Internal constructor data
pub struct ConstructorData {
    /// The constructor implementation
    pub func: Box<dyn Fn(Vec<JsValue>) -> JsResult<JsValue>>,
}

impl std::fmt::Debug for ConstructorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstructorData").finish()
    }
}

impl Clone for ConstructorData {
    fn clone(&self) -> Self {
        panic!("ConstructorData cannot be directly cloned")
    }
}

/// JavaScript value representation
#[derive(Debug, Clone)]
pub enum JsValue {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number (IEEE 754 double)
    Number(f64),
    /// String value
    String(String),
    /// Symbol value
    Symbol(SymbolValue),
    /// Object with properties
    Object(Rc<RefCell<ObjectData>>),
    /// Array
    Array(Rc<RefCell<ArrayData>>),
    /// Map collection
    Map(Rc<RefCell<MapData>>),
    /// Set collection
    Set(Rc<RefCell<SetData>>),
    /// Error object
    Error(Rc<RefCell<JsErrorObject>>),
    /// RegExp object
    RegExp(Rc<RefCell<RegExpObject>>),
    /// Function object
    Function(Rc<RefCell<FunctionData>>),
    /// Constructor object
    Constructor(Rc<RefCell<ConstructorData>>),
    /// Proxy object
    Proxy(crate::proxy::ProxyObject),
    /// WeakMap collection
    WeakMap(Rc<RefCell<WeakMapData>>),
    /// WeakSet collection
    WeakSet(Rc<RefCell<WeakSetData>>),
    /// Generator object
    Generator(GeneratorObject),
    /// AsyncGenerator object
    AsyncGenerator(AsyncGeneratorObject),
    /// BigInt value (arbitrary precision integer)
    BigInt(BigIntValue),
    /// WeakRef object (weak reference to target object)
    WeakRef(Rc<RefCell<WeakRefData>>),
    /// FinalizationRegistry object (cleanup callbacks for GC'd objects)
    FinalizationRegistry(Rc<RefCell<FinalizationRegistryData>>),
}

impl JsValue {
    /// Create undefined value
    pub fn undefined() -> Self {
        JsValue::Undefined
    }

    /// Create null value
    pub fn null() -> Self {
        JsValue::Null
    }

    /// Create boolean value
    pub fn boolean(v: bool) -> Self {
        JsValue::Boolean(v)
    }

    /// Create number value
    pub fn number(v: f64) -> Self {
        JsValue::Number(v)
    }

    /// Create string value
    pub fn string(s: impl Into<String>) -> Self {
        JsValue::String(s.into())
    }

    /// Create a symbol value
    pub fn symbol(sym: SymbolValue) -> Self {
        JsValue::Symbol(sym)
    }

    /// Create empty object
    pub fn object() -> Self {
        JsValue::Object(Rc::new(RefCell::new(ObjectData {
            properties: HashMap::new(),
            symbol_properties: HashMap::new(),
            prototype: None,
            extensible: None, // Default to true
        })))
    }

    /// Create object with prototype
    pub fn object_with_proto(proto: &JsValue) -> Self {
        JsValue::Object(Rc::new(RefCell::new(ObjectData {
            properties: HashMap::new(),
            symbol_properties: HashMap::new(),
            prototype: Some(Box::new(proto.clone())),
            extensible: None, // Default to true
        })))
    }

    /// Create a function value
    pub fn function<F>(func: F) -> Self
    where
        F: Fn(JsValue, Vec<JsValue>) -> JsResult<JsValue> + 'static,
    {
        JsValue::Function(Rc::new(RefCell::new(FunctionData {
            func: Box::new(func),
        })))
    }

    /// Create a constructor value
    pub fn constructor<F>(func: F) -> Self
    where
        F: Fn(Vec<JsValue>) -> JsResult<JsValue> + 'static,
    {
        JsValue::Constructor(Rc::new(RefCell::new(ConstructorData {
            func: Box::new(func),
        })))
    }

    /// Create a proxy value
    pub fn from_proxy(proxy: crate::proxy::ProxyObject) -> Self {
        JsValue::Proxy(proxy)
    }

    /// Create empty array
    pub fn array() -> Self {
        JsValue::Array(Rc::new(RefCell::new(ArrayData {
            elements: Vec::new(),
        })))
    }

    /// Create array from values
    pub fn array_from(values: Vec<JsValue>) -> Self {
        JsValue::Array(Rc::new(RefCell::new(ArrayData { elements: values })))
    }

    /// Create an error value from a JsErrorObject
    pub fn from_error(error: JsErrorObject) -> Self {
        JsValue::Error(Rc::new(RefCell::new(error)))
    }

    /// Create an empty Map
    pub fn map() -> Self {
        JsValue::Map(Rc::new(RefCell::new(MapData {
            entries: Vec::new(),
        })))
    }

    /// Create an empty Set
    pub fn set_collection() -> Self {
        JsValue::Set(Rc::new(RefCell::new(SetData {
            values: Vec::new(),
        })))
    }

    /// Create a RegExp value
    pub fn regexp(re: RegExpObject) -> Self {
        JsValue::RegExp(Rc::new(RefCell::new(re)))
    }

    /// Create an empty WeakMap
    pub fn weak_map() -> Self {
        JsValue::WeakMap(Rc::new(RefCell::new(WeakMapData {
            entries: HashMap::new(),
        })))
    }

    /// Create an empty WeakSet
    pub fn weak_set() -> Self {
        JsValue::WeakSet(Rc::new(RefCell::new(WeakSetData {
            values: HashMap::new(),
        })))
    }

    /// Create a BigInt value
    pub fn bigint(value: BigIntValue) -> Self {
        JsValue::BigInt(value)
    }

    /// Check if value is undefined
    pub fn is_undefined(&self) -> bool {
        matches!(self, JsValue::Undefined)
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsValue::Null)
    }

    /// Check if value is boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, JsValue::Boolean(_))
    }

    /// Check if value is number
    pub fn is_number(&self) -> bool {
        matches!(self, JsValue::Number(_))
    }

    /// Check if value is string
    pub fn is_string(&self) -> bool {
        matches!(self, JsValue::String(_))
    }

    /// Check if value is object
    pub fn is_object(&self) -> bool {
        matches!(self, JsValue::Object(_))
    }

    /// Check if value is array
    pub fn is_array(&self) -> bool {
        matches!(self, JsValue::Array(_))
    }

    /// Check if value is an error
    pub fn is_error(&self) -> bool {
        matches!(self, JsValue::Error(_))
    }

    /// Check if value is a Map
    pub fn is_map(&self) -> bool {
        matches!(self, JsValue::Map(_))
    }

    /// Check if value is a Set
    pub fn is_set(&self) -> bool {
        matches!(self, JsValue::Set(_))
    }

    /// Check if value is a symbol
    pub fn is_symbol(&self) -> bool {
        matches!(self, JsValue::Symbol(_))
    }

    /// Check if value is a RegExp
    pub fn is_regexp(&self) -> bool {
        matches!(self, JsValue::RegExp(_))
    }

    /// Check if value is a function
    pub fn is_function(&self) -> bool {
        matches!(self, JsValue::Function(_))
    }

    /// Check if value is a constructor
    pub fn is_constructor(&self) -> bool {
        matches!(self, JsValue::Constructor(_))
    }

    /// Check if value is a proxy
    pub fn is_proxy(&self) -> bool {
        matches!(self, JsValue::Proxy(_))
    }

    /// Check if value is a WeakMap
    pub fn is_weak_map(&self) -> bool {
        matches!(self, JsValue::WeakMap(_))
    }

    /// Check if value is a WeakSet
    pub fn is_weak_set(&self) -> bool {
        matches!(self, JsValue::WeakSet(_))
    }

    /// Check if value is a BigInt
    pub fn is_bigint(&self) -> bool {
        matches!(self, JsValue::BigInt(_))
    }

    /// Check if value is a Generator
    pub fn is_generator(&self) -> bool {
        matches!(self, JsValue::Generator(_))
    }

    /// Check if value is an AsyncGenerator
    pub fn is_async_generator(&self) -> bool {
        matches!(self, JsValue::AsyncGenerator(_))
    }

    /// Check if value is a WeakRef
    pub fn is_weak_ref(&self) -> bool {
        matches!(self, JsValue::WeakRef(_))
    }

    /// Check if value is a FinalizationRegistry
    pub fn is_finalization_registry(&self) -> bool {
        matches!(self, JsValue::FinalizationRegistry(_))
    }

    /// Get as Generator object
    pub fn as_generator(&self) -> Option<GeneratorObject> {
        match self {
            JsValue::Generator(gen) => Some(gen.clone()),
            _ => None,
        }
    }

    /// Get as AsyncGenerator object
    pub fn as_async_generator(&self) -> Option<AsyncGeneratorObject> {
        match self {
            JsValue::AsyncGenerator(gen) => Some(gen.clone()),
            _ => None,
        }
    }

    /// Create a Generator value
    pub fn generator(gen: GeneratorObject) -> Self {
        JsValue::Generator(gen)
    }

    /// Create an AsyncGenerator value
    pub fn async_generator(gen: AsyncGeneratorObject) -> Self {
        JsValue::AsyncGenerator(gen)
    }

    /// Get the object pointer identity (used for WeakMap/WeakSet keys)
    ///
    /// Returns Some(address) for object types, None for primitives.
    /// This is used to implement weak references based on object identity.
    pub fn object_identity(&self) -> Option<usize> {
        match self {
            JsValue::Object(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Array(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Map(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Set(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Error(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::RegExp(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Function(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::Constructor(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::WeakMap(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::WeakSet(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::WeakRef(rc) => Some(Rc::as_ptr(rc) as usize),
            JsValue::FinalizationRegistry(rc) => Some(Rc::as_ptr(rc) as usize),
            // Primitives don't have object identity
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_)
            | JsValue::Proxy(_)
            | JsValue::Generator(_)
            | JsValue::AsyncGenerator(_)
            | JsValue::BigInt(_) => None,
        }
    }

    /// Get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            JsValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<String> {
        match self {
            JsValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get as error object
    pub fn as_error(&self) -> Option<JsErrorObject> {
        match self {
            JsValue::Error(err) => Some(err.borrow().clone()),
            _ => None,
        }
    }

    /// Get as symbol
    pub fn as_symbol(&self) -> Option<SymbolValue> {
        match self {
            JsValue::Symbol(sym) => Some(sym.clone()),
            _ => None,
        }
    }

    /// Get as RegExp object
    pub fn as_regexp(&self) -> Option<RegExpObject> {
        match self {
            JsValue::RegExp(re) => Some(re.borrow().clone()),
            _ => None,
        }
    }

    /// Get as BigInt value
    pub fn as_bigint(&self) -> Option<BigIntValue> {
        match self {
            JsValue::BigInt(bi) => Some(bi.clone()),
            _ => None,
        }
    }

    /// Get array length
    pub fn array_length(&self) -> usize {
        match self {
            JsValue::Array(arr) => arr.borrow().elements.len(),
            _ => 0,
        }
    }

    /// Set object property
    pub fn set(&self, key: &str, value: JsValue) {
        if let JsValue::Object(obj) = self {
            obj.borrow_mut().properties.insert(key.to_string(), value);
        }
    }

    /// Get object property
    pub fn get(&self, key: &str) -> Option<JsValue> {
        match self {
            JsValue::Object(obj) => obj.borrow().properties.get(key).cloned(),
            _ => None,
        }
    }

    /// Check if object has own property
    pub fn has_own(&self, key: &str) -> bool {
        match self {
            JsValue::Object(obj) => obj.borrow().properties.contains_key(key),
            _ => false,
        }
    }

    /// Set object property with symbol key
    pub fn set_symbol(&self, sym: &SymbolValue, value: JsValue) {
        if let JsValue::Object(obj) = self {
            obj.borrow_mut()
                .symbol_properties
                .insert(sym.id(), value);
        }
    }

    /// Get object property with symbol key
    pub fn get_symbol(&self, sym: &SymbolValue) -> Option<JsValue> {
        match self {
            JsValue::Object(obj) => obj.borrow().symbol_properties.get(&sym.id()).cloned(),
            _ => None,
        }
    }

    /// Check if object has own symbol property
    pub fn has_own_symbol(&self, sym: &SymbolValue) -> bool {
        match self {
            JsValue::Object(obj) => obj.borrow().symbol_properties.contains_key(&sym.id()),
            _ => false,
        }
    }

    /// Get object prototype
    pub fn get_prototype(&self) -> Option<JsValue> {
        match self {
            JsValue::Object(obj) => obj.borrow().prototype.as_ref().map(|p| (**p).clone()),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_js_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    }
                } else if *n == n.trunc() && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Symbol(sym) => sym.to_string(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let elements: Vec<String> = arr
                    .borrow()
                    .elements
                    .iter()
                    .map(|e| e.to_js_string())
                    .collect();
                elements.join(",")
            }
            JsValue::Map(_) => "[object Map]".to_string(),
            JsValue::Set(_) => "[object Set]".to_string(),
            JsValue::Error(err) => err.borrow().to_string(),
            JsValue::RegExp(re) => re.borrow().to_string(),
            JsValue::Function(_) => "function() { [native code] }".to_string(),
            JsValue::Constructor(_) => "function() { [native code] }".to_string(),
            JsValue::Proxy(_) => "[object Object]".to_string(),
            JsValue::WeakMap(_) => "[object WeakMap]".to_string(),
            JsValue::WeakSet(_) => "[object WeakSet]".to_string(),
            JsValue::Generator(_) => "[object Generator]".to_string(),
            JsValue::AsyncGenerator(_) => "[object AsyncGenerator]".to_string(),
            JsValue::BigInt(n) => format!("{}n", n),
            JsValue::WeakRef(_) => "[object WeakRef]".to_string(),
            JsValue::FinalizationRegistry(_) => "[object FinalizationRegistry]".to_string(),
        }
    }

    /// Get the type of the value (as JavaScript typeof would return)
    pub fn type_of(&self) -> &'static str {
        match self {
            JsValue::Undefined => "undefined",
            JsValue::Null => "object", // typeof null === "object" in JavaScript
            JsValue::Boolean(_) => "boolean",
            JsValue::Number(_) => "number",
            JsValue::String(_) => "string",
            JsValue::Symbol(_) => "symbol",
            JsValue::Object(_) => "object",
            JsValue::Array(_) => "object",
            JsValue::Map(_) => "object",
            JsValue::Set(_) => "object",
            JsValue::Error(_) => "object",
            JsValue::RegExp(_) => "object",
            JsValue::Function(_) => "function",
            JsValue::Constructor(_) => "function",
            JsValue::Proxy(_) => "object",
            JsValue::WeakMap(_) => "object",
            JsValue::WeakSet(_) => "object",
            JsValue::Generator(_) => "object",
            JsValue::AsyncGenerator(_) => "object",
            JsValue::BigInt(_) => "bigint",
            JsValue::WeakRef(_) => "object",
            JsValue::FinalizationRegistry(_) => "object",
        }
    }

    /// Check equality (loose comparison)
    pub fn equals(&self, other: &JsValue) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => {
                if a.is_nan() && b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Symbol(a), JsValue::Symbol(b)) => a.id() == b.id(),
            // Reference types - same instance only
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Map(a), JsValue::Map(b)) => Rc::ptr_eq(a, b),
            (JsValue::Set(a), JsValue::Set(b)) => Rc::ptr_eq(a, b),
            (JsValue::Error(a), JsValue::Error(b)) => Rc::ptr_eq(a, b),
            (JsValue::RegExp(a), JsValue::RegExp(b)) => Rc::ptr_eq(a, b),
            (JsValue::Function(a), JsValue::Function(b)) => Rc::ptr_eq(a, b),
            (JsValue::Constructor(a), JsValue::Constructor(b)) => Rc::ptr_eq(a, b),
            // Proxy comparison is by reference to the proxy data
            (JsValue::Proxy(_), JsValue::Proxy(_)) => false, // Proxies are compared by internal data ref
            (JsValue::WeakMap(a), JsValue::WeakMap(b)) => Rc::ptr_eq(a, b),
            (JsValue::WeakSet(a), JsValue::WeakSet(b)) => Rc::ptr_eq(a, b),
            (JsValue::Generator(_), JsValue::Generator(_)) => false, // Generators are not equal
            (JsValue::AsyncGenerator(_), JsValue::AsyncGenerator(_)) => false, // AsyncGenerators are not equal
            (JsValue::BigInt(a), JsValue::BigInt(b)) => a == b,
            (JsValue::WeakRef(a), JsValue::WeakRef(b)) => Rc::ptr_eq(a, b),
            (JsValue::FinalizationRegistry(a), JsValue::FinalizationRegistry(b)) => {
                Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }

    /// SameValueZero comparison for Map/Set key equality
    ///
    /// Like equals() but treats NaN === NaN and -0 === +0
    /// This is the algorithm used by Map and Set for key comparison.
    pub fn same_value_zero(&self, other: &JsValue) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => {
                // SameValueZero: NaN equals NaN, -0 equals +0
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b // This already treats -0 == +0
                }
            }
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Symbol(a), JsValue::Symbol(b)) => a.id() == b.id(),
            // Reference types - same instance only
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Map(a), JsValue::Map(b)) => Rc::ptr_eq(a, b),
            (JsValue::Set(a), JsValue::Set(b)) => Rc::ptr_eq(a, b),
            (JsValue::Error(a), JsValue::Error(b)) => Rc::ptr_eq(a, b),
            (JsValue::RegExp(a), JsValue::RegExp(b)) => Rc::ptr_eq(a, b),
            (JsValue::Function(a), JsValue::Function(b)) => Rc::ptr_eq(a, b),
            (JsValue::Constructor(a), JsValue::Constructor(b)) => Rc::ptr_eq(a, b),
            (JsValue::Proxy(_), JsValue::Proxy(_)) => false,
            (JsValue::WeakMap(a), JsValue::WeakMap(b)) => Rc::ptr_eq(a, b),
            (JsValue::WeakSet(a), JsValue::WeakSet(b)) => Rc::ptr_eq(a, b),
            (JsValue::Generator(_), JsValue::Generator(_)) => false,
            (JsValue::AsyncGenerator(_), JsValue::AsyncGenerator(_)) => false,
            (JsValue::BigInt(a), JsValue::BigInt(b)) => a == b,
            (JsValue::WeakRef(a), JsValue::WeakRef(b)) => Rc::ptr_eq(a, b),
            (JsValue::FinalizationRegistry(a), JsValue::FinalizationRegistry(b)) => {
                Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}


impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined() {
        let v = JsValue::undefined();
        assert!(v.is_undefined());
        assert_eq!(v.to_js_string(), "undefined");
    }

    #[test]
    fn test_null() {
        let v = JsValue::null();
        assert!(v.is_null());
        assert_eq!(v.to_js_string(), "null");
    }

    #[test]
    fn test_boolean() {
        let v = JsValue::boolean(true);
        assert!(v.is_boolean());
        assert_eq!(v.as_boolean(), Some(true));
        assert_eq!(v.to_js_string(), "true");
    }

    #[test]
    fn test_number() {
        let v = JsValue::number(42.0);
        assert!(v.is_number());
        assert_eq!(v.as_number(), Some(42.0));
        assert_eq!(v.to_js_string(), "42");
    }

    #[test]
    fn test_string() {
        let v = JsValue::string("hello");
        assert!(v.is_string());
        assert_eq!(v.as_string(), Some("hello".to_string()));
        assert_eq!(v.to_js_string(), "hello");
    }

    #[test]
    fn test_object() {
        let obj = JsValue::object();
        assert!(obj.is_object());
        obj.set("foo", JsValue::number(42.0));
        assert_eq!(obj.get("foo"), Some(JsValue::number(42.0)));
        assert!(obj.has_own("foo"));
        assert!(!obj.has_own("bar"));
    }

    #[test]
    fn test_array() {
        let arr = JsValue::array_from(vec![JsValue::number(1.0), JsValue::number(2.0)]);
        assert!(arr.is_array());
        assert_eq!(arr.array_length(), 2);
    }
}
