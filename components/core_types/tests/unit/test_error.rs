//! Unit tests for JsError and ErrorKind
//!
//! Following TDD: These tests are written FIRST before implementation.

use core_types::{ErrorKind, JsError, SourcePosition, StackFrame};

#[cfg(test)]
mod error_kind_tests {
    use super::*;

    #[test]
    fn test_error_kind_syntax_error() {
        let kind = ErrorKind::SyntaxError;
        assert!(matches!(kind, ErrorKind::SyntaxError));
    }

    #[test]
    fn test_error_kind_type_error() {
        let kind = ErrorKind::TypeError;
        assert!(matches!(kind, ErrorKind::TypeError));
    }

    #[test]
    fn test_error_kind_reference_error() {
        let kind = ErrorKind::ReferenceError;
        assert!(matches!(kind, ErrorKind::ReferenceError));
    }

    #[test]
    fn test_error_kind_range_error() {
        let kind = ErrorKind::RangeError;
        assert!(matches!(kind, ErrorKind::RangeError));
    }

    #[test]
    fn test_error_kind_eval_error() {
        let kind = ErrorKind::EvalError;
        assert!(matches!(kind, ErrorKind::EvalError));
    }

    #[test]
    fn test_error_kind_uri_error() {
        let kind = ErrorKind::URIError;
        assert!(matches!(kind, ErrorKind::URIError));
    }

    #[test]
    fn test_error_kind_internal_error() {
        let kind = ErrorKind::InternalError;
        assert!(matches!(kind, ErrorKind::InternalError));
    }

    #[test]
    fn test_error_kind_clone() {
        let kind1 = ErrorKind::TypeError;
        let kind2 = kind1.clone();
        assert!(matches!(kind2, ErrorKind::TypeError));
    }

    #[test]
    fn test_error_kind_debug() {
        let kind = ErrorKind::SyntaxError;
        let debug_str = format!("{:?}", kind);
        assert!(debug_str.contains("SyntaxError"));
    }

    #[test]
    fn test_error_kind_equality() {
        assert_eq!(ErrorKind::TypeError, ErrorKind::TypeError);
        assert_ne!(ErrorKind::TypeError, ErrorKind::RangeError);
    }
}

#[cfg(test)]
mod js_error_tests {
    use super::*;

    #[test]
    fn test_js_error_creation_minimal() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "undefined is not a function".to_string(),
            stack: vec![],
            source_position: None,
        };

        assert!(matches!(error.kind, ErrorKind::TypeError));
        assert_eq!(error.message, "undefined is not a function");
        assert!(error.stack.is_empty());
        assert!(error.source_position.is_none());
    }

    #[test]
    fn test_js_error_with_source_position() {
        let error = JsError {
            kind: ErrorKind::SyntaxError,
            message: "Unexpected token".to_string(),
            stack: vec![],
            source_position: Some(SourcePosition {
                line: 10,
                column: 5,
                offset: 150,
            }),
        };

        assert!(error.source_position.is_some());
        let pos = error.source_position.unwrap();
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_js_error_with_stack_frames() {
        let error = JsError {
            kind: ErrorKind::ReferenceError,
            message: "x is not defined".to_string(),
            stack: vec![
                StackFrame {
                    function_name: Some("innerFunction".to_string()),
                    source_url: Some("app.js".to_string()),
                    line: 25,
                    column: 10,
                },
                StackFrame {
                    function_name: Some("outerFunction".to_string()),
                    source_url: Some("app.js".to_string()),
                    line: 30,
                    column: 5,
                },
            ],
            source_position: None,
        };

        assert_eq!(error.stack.len(), 2);
        assert_eq!(
            error.stack[0].function_name,
            Some("innerFunction".to_string())
        );
        assert_eq!(
            error.stack[1].function_name,
            Some("outerFunction".to_string())
        );
    }

    #[test]
    fn test_js_error_full_construction() {
        let error = JsError {
            kind: ErrorKind::InternalError,
            message: "Out of memory".to_string(),
            stack: vec![StackFrame {
                function_name: Some("allocate".to_string()),
                source_url: Some("memory.js".to_string()),
                line: 100,
                column: 1,
            }],
            source_position: Some(SourcePosition {
                line: 100,
                column: 1,
                offset: 2500,
            }),
        };

        assert!(matches!(error.kind, ErrorKind::InternalError));
        assert_eq!(error.message, "Out of memory");
        assert_eq!(error.stack.len(), 1);
        assert!(error.source_position.is_some());
    }

    #[test]
    fn test_js_error_empty_message() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: String::new(),
            stack: vec![],
            source_position: None,
        };

        assert!(error.message.is_empty());
    }

    #[test]
    fn test_js_error_long_stack() {
        let frames: Vec<StackFrame> = (0..100)
            .map(|i| StackFrame {
                function_name: Some(format!("fn_{}", i)),
                source_url: Some("deep.js".to_string()),
                line: i,
                column: 0,
            })
            .collect();

        let error = JsError {
            kind: ErrorKind::RangeError,
            message: "Maximum call stack size exceeded".to_string(),
            stack: frames,
            source_position: None,
        };

        assert_eq!(error.stack.len(), 100);
        assert_eq!(error.stack[0].function_name, Some("fn_0".to_string()));
        assert_eq!(error.stack[99].function_name, Some("fn_99".to_string()));
    }

    #[test]
    fn test_js_error_clone() {
        let error1 = JsError {
            kind: ErrorKind::EvalError,
            message: "eval is disabled".to_string(),
            stack: vec![StackFrame {
                function_name: Some("eval".to_string()),
                source_url: None,
                line: 1,
                column: 1,
            }],
            source_position: Some(SourcePosition {
                line: 1,
                column: 1,
                offset: 0,
            }),
        };
        let error2 = error1.clone();

        assert_eq!(error1.message, error2.message);
        assert_eq!(error1.stack.len(), error2.stack.len());
    }

    #[test]
    fn test_js_error_debug() {
        let error = JsError {
            kind: ErrorKind::URIError,
            message: "URI malformed".to_string(),
            stack: vec![],
            source_position: None,
        };
        let debug_str = format!("{:?}", error);

        assert!(debug_str.contains("URIError"));
        assert!(debug_str.contains("URI malformed"));
    }
}
