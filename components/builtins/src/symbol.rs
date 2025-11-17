//! JavaScript Symbol primitive type implementation
//!
//! Symbols are unique, immutable primitive values that can be used as property keys.
//! This module implements:
//! - Symbol() constructor for creating unique symbols
//! - Symbol.for() global registry for shared symbols
//! - Symbol.keyFor() for reverse lookup
//! - Well-known symbols (Symbol.iterator, Symbol.asyncIterator, etc.)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use crate::value::JsError;

/// Global counter for generating unique symbol IDs
static SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global registry for Symbol.for()
static SYMBOL_REGISTRY: LazyLock<Mutex<HashMap<String, SymbolValue>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Well-known symbol storage
static WELL_KNOWN_SYMBOLS: LazyLock<WellKnownSymbols> = LazyLock::new(|| {
    WellKnownSymbols {
        iterator: SymbolValue::create_well_known("Symbol.iterator"),
        async_iterator: SymbolValue::create_well_known("Symbol.asyncIterator"),
        to_string_tag: SymbolValue::create_well_known("Symbol.toStringTag"),
        has_instance: SymbolValue::create_well_known("Symbol.hasInstance"),
        species: SymbolValue::create_well_known("Symbol.species"),
        is_concat_spreadable: SymbolValue::create_well_known("Symbol.isConcatSpreadable"),
        to_primitive: SymbolValue::create_well_known("Symbol.toPrimitive"),
        unscopables: SymbolValue::create_well_known("Symbol.unscopables"),
        match_symbol: SymbolValue::create_well_known("Symbol.match"),
        replace: SymbolValue::create_well_known("Symbol.replace"),
        search: SymbolValue::create_well_known("Symbol.search"),
        split: SymbolValue::create_well_known("Symbol.split"),
    }
});

/// Storage for well-known symbols
struct WellKnownSymbols {
    iterator: SymbolValue,
    async_iterator: SymbolValue,
    to_string_tag: SymbolValue,
    has_instance: SymbolValue,
    species: SymbolValue,
    is_concat_spreadable: SymbolValue,
    to_primitive: SymbolValue,
    unscopables: SymbolValue,
    match_symbol: SymbolValue,
    replace: SymbolValue,
    search: SymbolValue,
    split: SymbolValue,
}

/// A JavaScript Symbol value
///
/// Symbols are unique, immutable identifiers that can be used as object property keys.
/// Each symbol has a unique internal ID and an optional description for debugging.
#[derive(Debug, Clone)]
pub struct SymbolValue {
    /// Unique identifier for this symbol
    id: u64,
    /// Optional description for debugging
    description: Option<String>,
}

impl SymbolValue {
    /// Create a new unique symbol with optional description
    fn new(description: Option<String>) -> Self {
        let id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        SymbolValue { id, description }
    }

    /// Create a well-known symbol (internal use only)
    fn create_well_known(description: &str) -> Self {
        let id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        SymbolValue {
            id,
            description: Some(description.to_string()),
        }
    }

    /// Get the unique ID of this symbol
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the description of this symbol
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Convert symbol to its string representation
    ///
    /// Returns "Symbol(description)" or "Symbol()" if no description
    pub fn to_string(&self) -> String {
        match &self.description {
            Some(desc) if !desc.is_empty() => format!("Symbol({})", desc),
            _ => "Symbol()".to_string(),
        }
    }

    /// Return the symbol value itself (Symbol.prototype.valueOf)
    pub fn value_of(&self) -> SymbolValue {
        self.clone()
    }

    /// Attempt to convert symbol to number (always fails with TypeError)
    ///
    /// JavaScript does not allow implicit conversion of symbols to numbers.
    pub fn to_number(&self) -> Result<f64, JsError> {
        Err(JsError::type_error(
            "Cannot convert a Symbol value to a number",
        ))
    }

    /// Attempt implicit string conversion (always fails with TypeError)
    ///
    /// JavaScript does not allow implicit conversion of symbols to strings.
    /// Use to_string() for explicit conversion.
    pub fn to_string_implicit(&self) -> Result<String, JsError> {
        Err(JsError::type_error(
            "Cannot convert a Symbol value to a string",
        ))
    }
}

impl PartialEq for SymbolValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SymbolValue {}

impl std::hash::Hash for SymbolValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Symbol constructor and static methods
///
/// Provides the Symbol() constructor and static methods like Symbol.for(),
/// Symbol.keyFor(), and access to well-known symbols.
pub struct SymbolConstructor;

impl SymbolConstructor {
    /// Create a new unique symbol with optional description
    ///
    /// Each call creates a symbol with a unique ID, even if the same description is used.
    ///
    /// # Example
    /// ```
    /// use builtins::symbol::SymbolConstructor;
    ///
    /// let sym1 = SymbolConstructor::new(Some("test".to_string()));
    /// let sym2 = SymbolConstructor::new(Some("test".to_string()));
    /// assert_ne!(sym1.id(), sym2.id()); // Different symbols
    /// ```
    pub fn new(description: Option<String>) -> SymbolValue {
        SymbolValue::new(description)
    }

    /// Get or create a symbol in the global registry
    ///
    /// Symbol.for(key) searches for existing symbols with the given key in the global registry.
    /// If found, returns that symbol. Otherwise, creates a new symbol and adds it to the registry.
    ///
    /// # Example
    /// ```
    /// use builtins::symbol::SymbolConstructor;
    ///
    /// let sym1 = SymbolConstructor::for_key("shared");
    /// let sym2 = SymbolConstructor::for_key("shared");
    /// assert_eq!(sym1.id(), sym2.id()); // Same symbol
    /// ```
    pub fn for_key(key: &str) -> SymbolValue {
        let mut registry = SYMBOL_REGISTRY.lock().unwrap();

        if let Some(sym) = registry.get(key) {
            sym.clone()
        } else {
            let sym = SymbolValue::new(Some(key.to_string()));
            registry.insert(key.to_string(), sym.clone());
            sym
        }
    }

    /// Get the key for a registered symbol
    ///
    /// Symbol.keyFor(sym) retrieves the key for a symbol in the global registry.
    /// Returns None if the symbol was not created via Symbol.for().
    ///
    /// # Example
    /// ```
    /// use builtins::symbol::SymbolConstructor;
    ///
    /// let sym = SymbolConstructor::for_key("mykey");
    /// assert_eq!(SymbolConstructor::key_for(&sym), Some("mykey".to_string()));
    ///
    /// let local = SymbolConstructor::new(Some("test".to_string()));
    /// assert_eq!(SymbolConstructor::key_for(&local), None);
    /// ```
    pub fn key_for(symbol: &SymbolValue) -> Option<String> {
        let registry = SYMBOL_REGISTRY.lock().unwrap();

        for (key, sym) in registry.iter() {
            if sym.id() == symbol.id() {
                return Some(key.clone());
            }
        }
        None
    }

    // Well-known symbols

    /// Symbol.iterator - The well-known symbol used for the default iterator
    pub fn iterator() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.iterator.clone()
    }

    /// Symbol.asyncIterator - The well-known symbol used for async iteration
    pub fn async_iterator() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.async_iterator.clone()
    }

    /// Symbol.toStringTag - The well-known symbol used to create the default string description
    pub fn to_string_tag() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.to_string_tag.clone()
    }

    /// Symbol.hasInstance - The well-known symbol used for instanceof checks
    pub fn has_instance() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.has_instance.clone()
    }

    /// Symbol.species - The well-known symbol used for constructor selection
    pub fn species() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.species.clone()
    }

    /// Symbol.isConcatSpreadable - The well-known symbol used for array concatenation
    pub fn is_concat_spreadable() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.is_concat_spreadable.clone()
    }

    /// Symbol.toPrimitive - The well-known symbol used for type conversion
    pub fn to_primitive() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.to_primitive.clone()
    }

    /// Symbol.unscopables - The well-known symbol used for with statement scoping
    pub fn unscopables() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.unscopables.clone()
    }

    /// Symbol.match - The well-known symbol used for string matching
    pub fn match_symbol() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.match_symbol.clone()
    }

    /// Symbol.replace - The well-known symbol used for string replacement
    pub fn replace() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.replace.clone()
    }

    /// Symbol.search - The well-known symbol used for string searching
    pub fn search() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.search.clone()
    }

    /// Symbol.split - The well-known symbol used for string splitting
    pub fn split() -> SymbolValue {
        WELL_KNOWN_SYMBOLS.split.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let sym = SymbolConstructor::new(Some("test".to_string()));
        assert_eq!(sym.description(), Some("test"));
    }

    #[test]
    fn test_symbol_uniqueness() {
        let sym1 = SymbolConstructor::new(Some("same".to_string()));
        let sym2 = SymbolConstructor::new(Some("same".to_string()));
        assert_ne!(sym1.id(), sym2.id());
    }

    #[test]
    fn test_symbol_for_registry() {
        let sym1 = SymbolConstructor::for_key("registry_test");
        let sym2 = SymbolConstructor::for_key("registry_test");
        assert_eq!(sym1.id(), sym2.id());
    }

    #[test]
    fn test_symbol_key_for() {
        let sym = SymbolConstructor::for_key("lookup_test");
        assert_eq!(
            SymbolConstructor::key_for(&sym),
            Some("lookup_test".to_string())
        );
    }

    #[test]
    fn test_symbol_to_string() {
        let sym = SymbolConstructor::new(Some("desc".to_string()));
        assert_eq!(sym.to_string(), "Symbol(desc)");
    }

    #[test]
    fn test_symbol_to_number_error() {
        let sym = SymbolConstructor::new(None);
        assert!(sym.to_number().is_err());
    }

    #[test]
    fn test_symbol_implicit_string_error() {
        let sym = SymbolConstructor::new(None);
        assert!(sym.to_string_implicit().is_err());
    }

    #[test]
    fn test_well_known_symbols_exist() {
        let _ = SymbolConstructor::iterator();
        let _ = SymbolConstructor::async_iterator();
        let _ = SymbolConstructor::to_string_tag();
        let _ = SymbolConstructor::has_instance();
        let _ = SymbolConstructor::species();
        let _ = SymbolConstructor::is_concat_spreadable();
        let _ = SymbolConstructor::to_primitive();
        let _ = SymbolConstructor::unscopables();
        let _ = SymbolConstructor::match_symbol();
        let _ = SymbolConstructor::replace();
        let _ = SymbolConstructor::search();
        let _ = SymbolConstructor::split();
    }

    #[test]
    fn test_symbol_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();

        let sym1 = SymbolConstructor::new(Some("a".to_string()));
        let sym2 = SymbolConstructor::new(Some("b".to_string()));

        set.insert(sym1.clone());
        set.insert(sym2.clone());
        set.insert(sym1.clone()); // Duplicate

        assert_eq!(set.len(), 2);
    }
}
