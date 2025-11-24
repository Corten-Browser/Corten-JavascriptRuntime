//! Structured Clone Algorithm implementation
//!
//! Implements the HTML specification's structured clone algorithm
//! for serializing and deserializing JavaScript values across
//! realms (e.g., between main thread and workers).
//!
//! Reference: https://html.spec.whatwg.org/multipage/structured-data.html

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Errors that can occur during structured cloning
#[derive(Debug, Clone, PartialEq)]
pub enum CloneError {
    /// The value contains a non-cloneable type
    DataCloneError(String),
    /// Circular reference detected but not supported in this context
    CircularReference,
    /// Maximum depth exceeded
    MaxDepthExceeded,
    /// Serialization error
    SerializationError(String),
    /// Deserialization error
    DeserializationError(String),
}

impl std::fmt::Display for CloneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneError::DataCloneError(msg) => write!(f, "DataCloneError: {}", msg),
            CloneError::CircularReference => write!(f, "Circular reference detected"),
            CloneError::MaxDepthExceeded => write!(f, "Maximum recursion depth exceeded"),
            CloneError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CloneError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl std::error::Error for CloneError {}

/// Represents a JavaScript value that can be cloned
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StructuredValue {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number (IEEE 754 double)
    Number(f64),
    /// BigInt (as string representation)
    BigInt(String),
    /// String value
    String(String),
    /// Date (milliseconds since epoch)
    Date(f64),
    /// RegExp (pattern, flags)
    RegExp { pattern: String, flags: String },
    /// ArrayBuffer (raw bytes)
    ArrayBuffer(Vec<u8>),
    /// SharedArrayBuffer reference ID (requires special handling)
    SharedArrayBuffer(u64),
    /// TypedArray (type, buffer data)
    TypedArray {
        kind: TypedArrayKind,
        buffer: Vec<u8>,
        byte_offset: usize,
        length: usize,
    },
    /// DataView
    DataView {
        buffer: Vec<u8>,
        byte_offset: usize,
        byte_length: usize,
    },
    /// Map entries
    Map(Vec<(StructuredValue, StructuredValue)>),
    /// Set values
    Set(Vec<StructuredValue>),
    /// Object (key-value pairs)
    Object(Vec<(String, StructuredValue)>),
    /// Array (indexed values with potential holes)
    Array(Vec<Option<StructuredValue>>),
    /// Error type (name, message, stack)
    Error {
        name: String,
        message: String,
        stack: Option<String>,
    },
    /// Blob (type, data)
    Blob {
        content_type: String,
        data: Vec<u8>,
    },
    /// File (name, type, data, last_modified)
    File {
        name: String,
        content_type: String,
        data: Vec<u8>,
        last_modified: u64,
    },
    /// ImageData (width, height, data)
    ImageData {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
    /// Reference to previously serialized value (for circular references)
    Reference(u32),
}

/// TypedArray kinds
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TypedArrayKind {
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Float32Array,
    Float64Array,
    BigInt64Array,
    BigUint64Array,
}

impl TypedArrayKind {
    /// Get bytes per element
    pub fn bytes_per_element(&self) -> usize {
        match self {
            TypedArrayKind::Int8Array | TypedArrayKind::Uint8Array | TypedArrayKind::Uint8ClampedArray => 1,
            TypedArrayKind::Int16Array | TypedArrayKind::Uint16Array => 2,
            TypedArrayKind::Int32Array | TypedArrayKind::Uint32Array | TypedArrayKind::Float32Array => 4,
            TypedArrayKind::Float64Array | TypedArrayKind::BigInt64Array | TypedArrayKind::BigUint64Array => 8,
        }
    }
}

/// Options for structured cloning
#[derive(Debug, Clone)]
pub struct CloneOptions {
    /// Maximum recursion depth
    pub max_depth: usize,
    /// List of transferable object IDs to transfer (not clone)
    pub transfer: Vec<u64>,
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            transfer: Vec::new(),
        }
    }
}

/// Structured clone context for tracking references
pub struct StructuredCloneContext {
    /// Map from original object ID to serialized index
    memory: HashMap<u64, u32>,
    /// Counter for reference IDs
    next_id: u32,
    /// Current recursion depth
    depth: usize,
    /// Clone options
    options: CloneOptions,
    /// Set of transferred object IDs
    transferred: HashSet<u64>,
}

impl StructuredCloneContext {
    /// Create a new clone context
    pub fn new(options: CloneOptions) -> Self {
        let transferred = options.transfer.iter().copied().collect();
        Self {
            memory: HashMap::new(),
            next_id: 0,
            depth: 0,
            options,
            transferred,
        }
    }

    /// Check if an object ID is being transferred
    pub fn is_transferred(&self, id: u64) -> bool {
        self.transferred.contains(&id)
    }

    /// Register an object and return its reference ID
    pub fn register(&mut self, object_id: u64) -> Option<u32> {
        if let Some(&ref_id) = self.memory.get(&object_id) {
            return Some(ref_id);
        }
        let ref_id = self.next_id;
        self.next_id += 1;
        self.memory.insert(object_id, ref_id);
        None
    }

    /// Enter a nested level
    pub fn enter(&mut self) -> Result<(), CloneError> {
        self.depth += 1;
        if self.depth > self.options.max_depth {
            return Err(CloneError::MaxDepthExceeded);
        }
        Ok(())
    }

    /// Exit a nested level
    pub fn exit(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

/// The Structured Clone Algorithm implementation
pub struct StructuredClone;

impl StructuredClone {
    /// Clone a value using the structured clone algorithm
    pub fn clone(value: &StructuredValue) -> Result<StructuredValue, CloneError> {
        Self::clone_with_options(value, CloneOptions::default())
    }

    /// Clone with options (transfer list, etc.)
    pub fn clone_with_options(
        value: &StructuredValue,
        options: CloneOptions,
    ) -> Result<StructuredValue, CloneError> {
        let mut ctx = StructuredCloneContext::new(options);
        Self::clone_internal(value, &mut ctx)
    }

    fn clone_internal(
        value: &StructuredValue,
        ctx: &mut StructuredCloneContext,
    ) -> Result<StructuredValue, CloneError> {
        ctx.enter()?;

        let result = match value {
            // Primitives are cloned directly
            StructuredValue::Undefined => Ok(StructuredValue::Undefined),
            StructuredValue::Null => Ok(StructuredValue::Null),
            StructuredValue::Boolean(b) => Ok(StructuredValue::Boolean(*b)),
            StructuredValue::Number(n) => Ok(StructuredValue::Number(*n)),
            StructuredValue::BigInt(s) => Ok(StructuredValue::BigInt(s.clone())),
            StructuredValue::String(s) => Ok(StructuredValue::String(s.clone())),
            StructuredValue::Date(d) => Ok(StructuredValue::Date(*d)),
            StructuredValue::RegExp { pattern, flags } => Ok(StructuredValue::RegExp {
                pattern: pattern.clone(),
                flags: flags.clone(),
            }),

            // ArrayBuffer - clone the bytes
            StructuredValue::ArrayBuffer(data) => {
                Ok(StructuredValue::ArrayBuffer(data.clone()))
            }

            // SharedArrayBuffer - not clonable (must be transferred)
            StructuredValue::SharedArrayBuffer(id) => {
                if ctx.is_transferred(*id) {
                    Ok(StructuredValue::SharedArrayBuffer(*id))
                } else {
                    Err(CloneError::DataCloneError(
                        "SharedArrayBuffer must be in transfer list".to_string(),
                    ))
                }
            }

            // TypedArray
            StructuredValue::TypedArray { kind, buffer, byte_offset, length } => {
                Ok(StructuredValue::TypedArray {
                    kind: *kind,
                    buffer: buffer.clone(),
                    byte_offset: *byte_offset,
                    length: *length,
                })
            }

            // DataView
            StructuredValue::DataView { buffer, byte_offset, byte_length } => {
                Ok(StructuredValue::DataView {
                    buffer: buffer.clone(),
                    byte_offset: *byte_offset,
                    byte_length: *byte_length,
                })
            }

            // Map - recursively clone entries
            StructuredValue::Map(entries) => {
                let cloned: Result<Vec<_>, _> = entries
                    .iter()
                    .map(|(k, v)| {
                        Ok((Self::clone_internal(k, ctx)?, Self::clone_internal(v, ctx)?))
                    })
                    .collect();
                Ok(StructuredValue::Map(cloned?))
            }

            // Set - recursively clone values
            StructuredValue::Set(values) => {
                let cloned: Result<Vec<_>, _> = values
                    .iter()
                    .map(|v| Self::clone_internal(v, ctx))
                    .collect();
                Ok(StructuredValue::Set(cloned?))
            }

            // Object - recursively clone properties
            StructuredValue::Object(props) => {
                let cloned: Result<Vec<_>, _> = props
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), Self::clone_internal(v, ctx)?)))
                    .collect();
                Ok(StructuredValue::Object(cloned?))
            }

            // Array - recursively clone elements
            StructuredValue::Array(elements) => {
                let cloned: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|elem| {
                        elem.as_ref()
                            .map(|v| Self::clone_internal(v, ctx))
                            .transpose()
                    })
                    .collect();
                Ok(StructuredValue::Array(cloned?))
            }

            // Error - clone name, message, and stack
            StructuredValue::Error { name, message, stack } => {
                Ok(StructuredValue::Error {
                    name: name.clone(),
                    message: message.clone(),
                    stack: stack.clone(),
                })
            }

            // Blob
            StructuredValue::Blob { content_type, data } => {
                Ok(StructuredValue::Blob {
                    content_type: content_type.clone(),
                    data: data.clone(),
                })
            }

            // File
            StructuredValue::File { name, content_type, data, last_modified } => {
                Ok(StructuredValue::File {
                    name: name.clone(),
                    content_type: content_type.clone(),
                    data: data.clone(),
                    last_modified: *last_modified,
                })
            }

            // ImageData
            StructuredValue::ImageData { width, height, data } => {
                Ok(StructuredValue::ImageData {
                    width: *width,
                    height: *height,
                    data: data.clone(),
                })
            }

            // Reference
            StructuredValue::Reference(id) => Ok(StructuredValue::Reference(*id)),
        };

        ctx.exit();
        result
    }

    /// Serialize a value to bytes
    pub fn serialize(value: &StructuredValue) -> Result<Vec<u8>, CloneError> {
        bincode::serialize(value)
            .map_err(|e| CloneError::SerializationError(e.to_string()))
    }

    /// Deserialize bytes to a value
    pub fn deserialize(bytes: &[u8]) -> Result<StructuredValue, CloneError> {
        bincode::deserialize(bytes)
            .map_err(|e| CloneError::DeserializationError(e.to_string()))
    }

    /// Check if a value type is cloneable
    pub fn is_cloneable(value: &StructuredValue) -> bool {
        match value {
            // Functions, Symbols, and certain objects are not cloneable
            StructuredValue::SharedArrayBuffer(_) => false, // Must be transferred
            _ => true,
        }
    }
}

/// Convenience function to post a message using structured clone
pub fn post_message(
    value: &StructuredValue,
    transfer: Vec<u64>,
) -> Result<Vec<u8>, CloneError> {
    let options = CloneOptions {
        transfer,
        ..Default::default()
    };
    let cloned = StructuredClone::clone_with_options(value, options)?;
    StructuredClone::serialize(&cloned)
}

/// Convenience function to receive a message
pub fn receive_message(bytes: &[u8]) -> Result<StructuredValue, CloneError> {
    StructuredClone::deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_primitives() {
        assert_eq!(
            StructuredClone::clone(&StructuredValue::Undefined).unwrap(),
            StructuredValue::Undefined
        );
        assert_eq!(
            StructuredClone::clone(&StructuredValue::Null).unwrap(),
            StructuredValue::Null
        );
        assert_eq!(
            StructuredClone::clone(&StructuredValue::Boolean(true)).unwrap(),
            StructuredValue::Boolean(true)
        );
        assert_eq!(
            StructuredClone::clone(&StructuredValue::Number(42.5)).unwrap(),
            StructuredValue::Number(42.5)
        );
        assert_eq!(
            StructuredClone::clone(&StructuredValue::String("hello".to_string())).unwrap(),
            StructuredValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_clone_array() {
        let arr = StructuredValue::Array(vec![
            Some(StructuredValue::Number(1.0)),
            None, // hole
            Some(StructuredValue::Number(3.0)),
        ]);
        let cloned = StructuredClone::clone(&arr).unwrap();
        assert_eq!(cloned, arr);
    }

    #[test]
    fn test_clone_object() {
        let obj = StructuredValue::Object(vec![
            ("name".to_string(), StructuredValue::String("test".to_string())),
            ("value".to_string(), StructuredValue::Number(42.0)),
        ]);
        let cloned = StructuredClone::clone(&obj).unwrap();
        assert_eq!(cloned, obj);
    }

    #[test]
    fn test_clone_nested() {
        let nested = StructuredValue::Object(vec![(
            "inner".to_string(),
            StructuredValue::Array(vec![
                Some(StructuredValue::Number(1.0)),
                Some(StructuredValue::Object(vec![(
                    "deep".to_string(),
                    StructuredValue::Boolean(true),
                )])),
            ]),
        )]);
        let cloned = StructuredClone::clone(&nested).unwrap();
        assert_eq!(cloned, nested);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let value = StructuredValue::Object(vec![
            ("str".to_string(), StructuredValue::String("hello".to_string())),
            ("num".to_string(), StructuredValue::Number(123.456)),
        ]);

        let bytes = StructuredClone::serialize(&value).unwrap();
        let deserialized = StructuredClone::deserialize(&bytes).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_shared_array_buffer_requires_transfer() {
        let sab = StructuredValue::SharedArrayBuffer(1);
        assert!(StructuredClone::clone(&sab).is_err());

        // With transfer list, it should work
        let options = CloneOptions {
            transfer: vec![1],
            ..Default::default()
        };
        assert!(StructuredClone::clone_with_options(&sab, options).is_ok());
    }

    #[test]
    fn test_max_depth() {
        let mut value = StructuredValue::Object(vec![]);
        for _ in 0..100 {
            value = StructuredValue::Object(vec![("nested".to_string(), value)]);
        }

        // Should succeed with default depth
        assert!(StructuredClone::clone(&value).is_ok());

        // Should fail with low depth limit
        let options = CloneOptions {
            max_depth: 10,
            ..Default::default()
        };
        assert!(StructuredClone::clone_with_options(&value, options).is_err());
    }

    #[test]
    fn test_typed_array() {
        let ta = StructuredValue::TypedArray {
            kind: TypedArrayKind::Uint8Array,
            buffer: vec![1, 2, 3, 4],
            byte_offset: 0,
            length: 4,
        };
        let cloned = StructuredClone::clone(&ta).unwrap();
        assert_eq!(cloned, ta);
    }

    #[test]
    fn test_error() {
        let err = StructuredValue::Error {
            name: "TypeError".to_string(),
            message: "test error".to_string(),
            stack: Some("at test".to_string()),
        };
        let cloned = StructuredClone::clone(&err).unwrap();
        assert_eq!(cloned, err);
    }
}
