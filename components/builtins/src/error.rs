//! JavaScript Error types implementation
//!
//! This module provides comprehensive Error support including:
//! - Base Error constructor
//! - All standard error subtypes (TypeError, ReferenceError, etc.)
//! - Stack trace generation and formatting
//! - Error.prototype methods

use std::fmt;
use std::rc::Rc;

/// The kind of JavaScript error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Generic Error
    Error,
    /// TypeError - type mismatch errors
    TypeError,
    /// ReferenceError - undefined variable access
    ReferenceError,
    /// SyntaxError - parse/syntax errors
    SyntaxError,
    /// RangeError - numeric range violations
    RangeError,
    /// URIError - malformed URI
    URIError,
    /// EvalError - eval failures (legacy)
    EvalError,
    /// AggregateError - multiple errors combined
    AggregateError,
}

impl ErrorKind {
    /// Returns true if this is an error type
    pub fn is_error_type(&self) -> bool {
        true
    }

    /// Get the error name as a string
    pub fn name(&self) -> &'static str {
        match self {
            ErrorKind::Error => "Error",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::ReferenceError => "ReferenceError",
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::RangeError => "RangeError",
            ErrorKind::URIError => "URIError",
            ErrorKind::EvalError => "EvalError",
            ErrorKind::AggregateError => "AggregateError",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents a single frame in a JavaScript stack trace
#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    function_name: String,
    file_name: String,
    line_number: u32,
    column_number: u32,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(function_name: String, file_name: String, line_number: u32, column_number: u32) -> Self {
        StackFrame {
            function_name,
            file_name,
            line_number,
            column_number,
        }
    }

    /// Get the function name
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Get the file name
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get the line number
    pub fn line_number(&self) -> u32 {
        self.line_number
    }

    /// Get the column number
    pub fn column_number(&self) -> u32 {
        self.column_number
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "    at {} ({}:{}:{})",
            self.function_name, self.file_name, self.line_number, self.column_number
        )
    }
}

/// JavaScript Error object representation
#[derive(Debug, Clone)]
pub struct JsErrorObject {
    /// The kind of error
    kind: ErrorKind,
    /// Custom error name (can be overwritten)
    name: String,
    /// Error message
    message: String,
    /// Stack frames
    stack_frames: Vec<StackFrame>,
    /// Optional cause (for error chaining)
    cause: Option<Rc<JsErrorObject>>,
    /// For AggregateError: collection of errors
    errors: Option<Vec<JsErrorObject>>,
    /// Stack trace limit (default 10)
    stack_trace_limit: usize,
}

impl JsErrorObject {
    /// Create a new error object
    pub fn new(kind: ErrorKind, message: String) -> Self {
        let name = kind.name().to_string();
        JsErrorObject {
            kind,
            name,
            message,
            stack_frames: Vec::new(),
            cause: None,
            errors: None,
            stack_trace_limit: 10,
        }
    }

    /// Get the error kind
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Get the error name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the error name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Set the error message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// Check if this is an error object
    pub fn is_error(&self) -> bool {
        true
    }

    /// Get the stack trace as a formatted string
    pub fn stack(&self) -> String {
        let mut result = self.to_string();

        let frames_to_show = self.stack_frames.len().min(self.stack_trace_limit);
        for frame in self.stack_frames.iter().take(frames_to_show) {
            result.push('\n');
            result.push_str(&frame.to_string());
        }

        result
    }

    /// Capture stack trace from provided frames
    pub fn capture_stack_trace(&mut self, frames: Vec<StackFrame>) {
        self.stack_frames = frames;
    }

    /// Get the cause of this error if any
    pub fn cause(&self) -> Option<&JsErrorObject> {
        self.cause.as_ref().map(|rc| rc.as_ref())
    }

    /// Set the cause of this error
    pub fn set_cause(&mut self, cause: JsErrorObject) {
        self.cause = Some(Rc::new(cause));
    }

    /// Get the errors array (for AggregateError)
    pub fn errors(&self) -> Option<&Vec<JsErrorObject>> {
        self.errors.as_ref()
    }

    /// Set the errors array (for AggregateError)
    pub fn set_errors(&mut self, errors: Vec<JsErrorObject>) {
        self.errors = Some(errors);
    }
}

impl fmt::Display for JsErrorObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.message)
        }
    }
}

impl PartialEq for JsErrorObject {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

/// Error constructor functions matching JavaScript Error API
pub struct ErrorConstructor;

impl ErrorConstructor {
    /// Construct a base Error
    pub fn construct(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::Error, message.unwrap_or_default())
    }

    /// Call Error() without new (behaves like new Error())
    pub fn call(message: Option<String>) -> JsErrorObject {
        Self::construct(message)
    }

    /// Create a TypeError
    pub fn type_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::TypeError, message.unwrap_or_default())
    }

    /// Create a ReferenceError
    pub fn reference_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::ReferenceError, message.unwrap_or_default())
    }

    /// Create a SyntaxError
    pub fn syntax_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::SyntaxError, message.unwrap_or_default())
    }

    /// Create a RangeError
    pub fn range_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::RangeError, message.unwrap_or_default())
    }

    /// Create a URIError
    pub fn uri_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::URIError, message.unwrap_or_default())
    }

    /// Create an EvalError
    pub fn eval_error(message: Option<String>) -> JsErrorObject {
        JsErrorObject::new(ErrorKind::EvalError, message.unwrap_or_default())
    }

    /// Create an AggregateError
    pub fn aggregate_error(errors: Vec<JsErrorObject>, message: Option<String>) -> JsErrorObject {
        let mut error = JsErrorObject::new(ErrorKind::AggregateError, message.unwrap_or_default());
        error.set_errors(errors);
        error
    }

    /// Create an error with a cause
    pub fn with_cause(
        kind: ErrorKind,
        message: Option<String>,
        cause: JsErrorObject,
    ) -> JsErrorObject {
        let mut error = JsErrorObject::new(kind, message.unwrap_or_default());
        error.set_cause(cause);
        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_names() {
        assert_eq!(ErrorKind::Error.name(), "Error");
        assert_eq!(ErrorKind::TypeError.name(), "TypeError");
        assert_eq!(ErrorKind::ReferenceError.name(), "ReferenceError");
        assert_eq!(ErrorKind::SyntaxError.name(), "SyntaxError");
        assert_eq!(ErrorKind::RangeError.name(), "RangeError");
        assert_eq!(ErrorKind::URIError.name(), "URIError");
        assert_eq!(ErrorKind::EvalError.name(), "EvalError");
        assert_eq!(ErrorKind::AggregateError.name(), "AggregateError");
    }

    #[test]
    fn test_stack_frame_display() {
        let frame = StackFrame::new("myFunc".to_string(), "file.js".to_string(), 10, 5);
        let display = frame.to_string();
        assert!(display.contains("myFunc"));
        assert!(display.contains("file.js"));
        assert!(display.contains("10"));
        assert!(display.contains("5"));
    }

    #[test]
    fn test_error_object_creation() {
        let error = JsErrorObject::new(ErrorKind::TypeError, "not a function".to_string());
        assert_eq!(error.kind(), ErrorKind::TypeError);
        assert_eq!(error.name(), "TypeError");
        assert_eq!(error.message(), "not a function");
        assert!(error.is_error());
    }

    #[test]
    fn test_error_to_string_with_message() {
        let error = JsErrorObject::new(ErrorKind::Error, "something wrong".to_string());
        assert_eq!(error.to_string(), "Error: something wrong");
    }

    #[test]
    fn test_error_to_string_without_message() {
        let error = JsErrorObject::new(ErrorKind::Error, "".to_string());
        assert_eq!(error.to_string(), "Error");
    }

    #[test]
    fn test_error_stack_generation() {
        let mut error = JsErrorObject::new(ErrorKind::TypeError, "test".to_string());
        let frames = vec![
            StackFrame::new("foo".to_string(), "test.js".to_string(), 1, 1),
            StackFrame::new("bar".to_string(), "test.js".to_string(), 5, 3),
        ];
        error.capture_stack_trace(frames);

        let stack = error.stack();
        assert!(stack.starts_with("TypeError: test"));
        assert!(stack.contains("at foo"));
        assert!(stack.contains("at bar"));
    }

    #[test]
    fn test_error_stack_trace_limit() {
        let mut error = JsErrorObject::new(ErrorKind::Error, "deep".to_string());
        let frames: Vec<StackFrame> = (0..100)
            .map(|i| StackFrame::new(format!("f{}", i), "test.js".to_string(), i, 1))
            .collect();
        error.capture_stack_trace(frames);

        let stack = error.stack();
        let lines: Vec<&str> = stack.lines().collect();
        // 1 for error message + 10 for frames (default limit)
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn test_error_constructor_type_error() {
        let error = ErrorConstructor::type_error(Some("not callable".to_string()));
        assert_eq!(error.kind(), ErrorKind::TypeError);
        assert_eq!(error.message(), "not callable");
    }

    #[test]
    fn test_aggregate_error_stores_errors() {
        let errors = vec![
            ErrorConstructor::type_error(Some("e1".to_string())),
            ErrorConstructor::range_error(Some("e2".to_string())),
        ];
        let agg = ErrorConstructor::aggregate_error(errors, Some("combined".to_string()));

        let stored = agg.errors().unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].kind(), ErrorKind::TypeError);
        assert_eq!(stored[1].kind(), ErrorKind::RangeError);
    }

    #[test]
    fn test_error_with_cause() {
        let cause = ErrorConstructor::type_error(Some("root".to_string()));
        let error = ErrorConstructor::with_cause(
            ErrorKind::Error,
            Some("wrapper".to_string()),
            cause,
        );

        let retrieved_cause = error.cause().unwrap();
        assert_eq!(retrieved_cause.kind(), ErrorKind::TypeError);
        assert_eq!(retrieved_cause.message(), "root");
    }

    #[test]
    fn test_error_name_modification() {
        let mut error = JsErrorObject::new(ErrorKind::Error, "test".to_string());
        error.set_name("CustomError".to_string());
        assert_eq!(error.name(), "CustomError");
        assert_eq!(error.to_string(), "CustomError: test");
    }

    #[test]
    fn test_error_message_modification() {
        let mut error = JsErrorObject::new(ErrorKind::Error, "original".to_string());
        error.set_message("modified".to_string());
        assert_eq!(error.message(), "modified");
    }
}
