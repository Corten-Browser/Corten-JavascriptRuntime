//! JavaScript error types and error handling.
//!
//! This module provides error types that correspond to JavaScript's built-in
//! error types, along with stack trace information.

use crate::{SourcePosition, StackFrame};

/// The kind of JavaScript error.
///
/// These correspond to JavaScript's built-in error constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Syntax error in JavaScript code
    SyntaxError,
    /// Type error (e.g., calling a non-function)
    TypeError,
    /// Reference to an undefined variable
    ReferenceError,
    /// Value out of allowed range
    RangeError,
    /// Error in eval() function
    EvalError,
    /// Error in URI handling functions
    URIError,
    /// Internal engine error
    InternalError,
}

/// A JavaScript error with message and stack trace.
///
/// This struct represents a JavaScript exception that can be thrown and caught.
/// It includes the error type, message, stack trace, and source position.
///
/// # Examples
///
/// ```
/// use core_types::{JsError, ErrorKind};
///
/// let error = JsError {
///     kind: ErrorKind::TypeError,
///     message: "undefined is not a function".to_string(),
///     stack: vec![],
///     source_position: None,
/// };
///
/// assert_eq!(error.message, "undefined is not a function");
/// ```
#[derive(Debug, Clone)]
pub struct JsError {
    /// The type of error
    pub kind: ErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Stack trace (call stack at the time of the error)
    pub stack: Vec<StackFrame>,
    /// Source position where the error occurred
    pub source_position: Option<SourcePosition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_variants() {
        let _syntax = ErrorKind::SyntaxError;
        let _type_err = ErrorKind::TypeError;
        let _ref_err = ErrorKind::ReferenceError;
        let _range = ErrorKind::RangeError;
        let _eval = ErrorKind::EvalError;
        let _uri = ErrorKind::URIError;
        let _internal = ErrorKind::InternalError;
    }

    #[test]
    fn test_js_error_creation() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "test".to_string(),
            stack: vec![],
            source_position: None,
        };
        assert!(matches!(error.kind, ErrorKind::TypeError));
    }
}
