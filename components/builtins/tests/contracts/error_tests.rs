//! Contract tests for JavaScript Error types
//!
//! Tests verify ECMAScript 2024 compliance for:
//! - Error base constructor
//! - All error subtypes (TypeError, ReferenceError, etc.)
//! - Stack trace generation
//! - Error.prototype methods

use builtins::{
    ErrorConstructor, ErrorKind, JsErrorObject, JsValue, StackFrame,
};

#[cfg(test)]
mod error_constructor_tests {
    use super::*;

    #[test]
    fn test_error_constructor_with_message() {
        let error = ErrorConstructor::construct(Some("Something went wrong".to_string()));

        assert_eq!(error.name(), "Error");
        assert_eq!(error.message(), "Something went wrong");
        assert_eq!(error.kind(), ErrorKind::Error);
    }

    #[test]
    fn test_error_constructor_without_message() {
        let error = ErrorConstructor::construct(None);

        assert_eq!(error.name(), "Error");
        assert_eq!(error.message(), "");
        assert_eq!(error.kind(), ErrorKind::Error);
    }

    #[test]
    fn test_error_to_string() {
        let error = ErrorConstructor::construct(Some("test error".to_string()));
        assert_eq!(error.to_string(), "Error: test error");

        let error_no_msg = ErrorConstructor::construct(None);
        assert_eq!(error_no_msg.to_string(), "Error");
    }

    #[test]
    fn test_error_constructor_callable_without_new() {
        // In JavaScript, Error() without `new` behaves like new Error()
        let error = ErrorConstructor::call(Some("called without new".to_string()));

        assert_eq!(error.name(), "Error");
        assert_eq!(error.message(), "called without new");
    }

    #[test]
    fn test_error_has_stack_property() {
        let error = ErrorConstructor::construct(Some("has stack".to_string()));
        let stack = error.stack();

        assert!(!stack.is_empty());
        // Stack should be a string
        assert!(stack.contains("Error: has stack"));
    }
}

#[cfg(test)]
mod type_error_tests {
    use super::*;

    #[test]
    fn test_type_error_constructor() {
        let error = ErrorConstructor::type_error(Some("not a function".to_string()));

        assert_eq!(error.name(), "TypeError");
        assert_eq!(error.message(), "not a function");
        assert_eq!(error.kind(), ErrorKind::TypeError);
    }

    #[test]
    fn test_type_error_to_string() {
        let error = ErrorConstructor::type_error(Some("undefined is not a function".to_string()));
        assert_eq!(error.to_string(), "TypeError: undefined is not a function");
    }

    #[test]
    fn test_type_error_without_message() {
        let error = ErrorConstructor::type_error(None);
        assert_eq!(error.name(), "TypeError");
        assert_eq!(error.message(), "");
        assert_eq!(error.to_string(), "TypeError");
    }

    #[test]
    fn test_type_error_inherits_from_error() {
        let error = ErrorConstructor::type_error(Some("test".to_string()));
        assert!(error.is_error());
        assert!(error.stack().contains("TypeError"));
    }
}

#[cfg(test)]
mod reference_error_tests {
    use super::*;

    #[test]
    fn test_reference_error_constructor() {
        let error = ErrorConstructor::reference_error(Some("x is not defined".to_string()));

        assert_eq!(error.name(), "ReferenceError");
        assert_eq!(error.message(), "x is not defined");
        assert_eq!(error.kind(), ErrorKind::ReferenceError);
    }

    #[test]
    fn test_reference_error_to_string() {
        let error = ErrorConstructor::reference_error(Some("myVar is not defined".to_string()));
        assert_eq!(error.to_string(), "ReferenceError: myVar is not defined");
    }

    #[test]
    fn test_reference_error_without_message() {
        let error = ErrorConstructor::reference_error(None);
        assert_eq!(error.to_string(), "ReferenceError");
    }
}

#[cfg(test)]
mod syntax_error_tests {
    use super::*;

    #[test]
    fn test_syntax_error_constructor() {
        let error = ErrorConstructor::syntax_error(Some("Unexpected token".to_string()));

        assert_eq!(error.name(), "SyntaxError");
        assert_eq!(error.message(), "Unexpected token");
        assert_eq!(error.kind(), ErrorKind::SyntaxError);
    }

    #[test]
    fn test_syntax_error_to_string() {
        let error = ErrorConstructor::syntax_error(Some("Unexpected end of input".to_string()));
        assert_eq!(error.to_string(), "SyntaxError: Unexpected end of input");
    }
}

#[cfg(test)]
mod range_error_tests {
    use super::*;

    #[test]
    fn test_range_error_constructor() {
        let error = ErrorConstructor::range_error(Some("Invalid array length".to_string()));

        assert_eq!(error.name(), "RangeError");
        assert_eq!(error.message(), "Invalid array length");
        assert_eq!(error.kind(), ErrorKind::RangeError);
    }

    #[test]
    fn test_range_error_to_string() {
        let error = ErrorConstructor::range_error(Some("Maximum call stack size exceeded".to_string()));
        assert_eq!(error.to_string(), "RangeError: Maximum call stack size exceeded");
    }
}

#[cfg(test)]
mod uri_error_tests {
    use super::*;

    #[test]
    fn test_uri_error_constructor() {
        let error = ErrorConstructor::uri_error(Some("URI malformed".to_string()));

        assert_eq!(error.name(), "URIError");
        assert_eq!(error.message(), "URI malformed");
        assert_eq!(error.kind(), ErrorKind::URIError);
    }

    #[test]
    fn test_uri_error_to_string() {
        let error = ErrorConstructor::uri_error(Some("URI malformed".to_string()));
        assert_eq!(error.to_string(), "URIError: URI malformed");
    }
}

#[cfg(test)]
mod eval_error_tests {
    use super::*;

    #[test]
    fn test_eval_error_constructor() {
        let error = ErrorConstructor::eval_error(Some("eval failed".to_string()));

        assert_eq!(error.name(), "EvalError");
        assert_eq!(error.message(), "eval failed");
        assert_eq!(error.kind(), ErrorKind::EvalError);
    }

    #[test]
    fn test_eval_error_to_string() {
        let error = ErrorConstructor::eval_error(Some("eval is disabled".to_string()));
        assert_eq!(error.to_string(), "EvalError: eval is disabled");
    }
}

#[cfg(test)]
mod aggregate_error_tests {
    use super::*;

    #[test]
    fn test_aggregate_error_constructor() {
        let errors = vec![
            ErrorConstructor::type_error(Some("first error".to_string())),
            ErrorConstructor::range_error(Some("second error".to_string())),
        ];

        let error = ErrorConstructor::aggregate_error(errors, Some("Multiple errors".to_string()));

        assert_eq!(error.name(), "AggregateError");
        assert_eq!(error.message(), "Multiple errors");
        assert_eq!(error.kind(), ErrorKind::AggregateError);
    }

    #[test]
    fn test_aggregate_error_errors_property() {
        let errors = vec![
            ErrorConstructor::type_error(Some("error 1".to_string())),
            ErrorConstructor::reference_error(Some("error 2".to_string())),
            ErrorConstructor::syntax_error(Some("error 3".to_string())),
        ];

        let agg_error = ErrorConstructor::aggregate_error(errors, Some("All failed".to_string()));

        let inner_errors = agg_error.errors().expect("AggregateError should have errors");
        assert_eq!(inner_errors.len(), 3);
        assert_eq!(inner_errors[0].name(), "TypeError");
        assert_eq!(inner_errors[1].name(), "ReferenceError");
        assert_eq!(inner_errors[2].name(), "SyntaxError");
    }

    #[test]
    fn test_aggregate_error_to_string() {
        let errors = vec![
            ErrorConstructor::type_error(Some("e1".to_string())),
        ];

        let error = ErrorConstructor::aggregate_error(errors, Some("combined".to_string()));
        assert_eq!(error.to_string(), "AggregateError: combined");
    }

    #[test]
    fn test_aggregate_error_without_message() {
        let errors = vec![];
        let error = ErrorConstructor::aggregate_error(errors, None);
        assert_eq!(error.message(), "");
        assert_eq!(error.to_string(), "AggregateError");
    }
}

#[cfg(test)]
mod stack_trace_tests {
    use super::*;

    #[test]
    fn test_stack_frame_construction() {
        let frame = StackFrame::new(
            "myFunction".to_string(),
            "script.js".to_string(),
            10,
            5,
        );

        assert_eq!(frame.function_name(), "myFunction");
        assert_eq!(frame.file_name(), "script.js");
        assert_eq!(frame.line_number(), 10);
        assert_eq!(frame.column_number(), 5);
    }

    #[test]
    fn test_stack_frame_to_string() {
        let frame = StackFrame::new(
            "processData".to_string(),
            "app.js".to_string(),
            42,
            13,
        );

        let frame_str = frame.to_string();
        assert!(frame_str.contains("processData"));
        assert!(frame_str.contains("app.js"));
        assert!(frame_str.contains("42"));
        assert!(frame_str.contains("13"));
    }

    #[test]
    fn test_stack_frame_anonymous_function() {
        let frame = StackFrame::new(
            "<anonymous>".to_string(),
            "eval".to_string(),
            1,
            1,
        );

        assert_eq!(frame.function_name(), "<anonymous>");
    }

    #[test]
    fn test_error_capture_stack_trace() {
        let mut error = ErrorConstructor::construct(Some("test".to_string()));

        let frames = vec![
            StackFrame::new("foo".to_string(), "test.js".to_string(), 1, 1),
            StackFrame::new("bar".to_string(), "test.js".to_string(), 5, 3),
            StackFrame::new("main".to_string(), "test.js".to_string(), 10, 1),
        ];

        error.capture_stack_trace(frames);

        let stack = error.stack();
        assert!(stack.contains("foo"));
        assert!(stack.contains("bar"));
        assert!(stack.contains("main"));
        assert!(stack.contains("test.js"));
    }

    #[test]
    fn test_stack_trace_format() {
        let mut error = ErrorConstructor::type_error(Some("x is not a function".to_string()));

        let frames = vec![
            StackFrame::new("callIt".to_string(), "utils.js".to_string(), 15, 8),
            StackFrame::new("process".to_string(), "main.js".to_string(), 42, 5),
        ];

        error.capture_stack_trace(frames);

        let stack = error.stack();
        // Stack should start with error name and message
        let lines: Vec<&str> = stack.lines().collect();
        assert!(lines[0].contains("TypeError: x is not a function"));
        // Subsequent lines should be stack frames
        assert!(lines[1].contains("at callIt"));
        assert!(lines[2].contains("at process"));
    }

    #[test]
    fn test_stack_trace_limit() {
        let mut error = ErrorConstructor::construct(Some("deep".to_string()));

        // Create many frames
        let frames: Vec<StackFrame> = (0..100)
            .map(|i| StackFrame::new(format!("func{}", i), "test.js".to_string(), i as u32, 1))
            .collect();

        error.capture_stack_trace(frames);

        let stack = error.stack();
        let lines: Vec<&str> = stack.lines().collect();

        // Should respect stack trace limit (default 10 frames + 1 for error line)
        assert!(lines.len() <= 11);
    }
}

#[cfg(test)]
mod error_properties_tests {
    use super::*;

    #[test]
    fn test_error_name_is_writable() {
        let mut error = ErrorConstructor::construct(Some("test".to_string()));
        error.set_name("CustomError".to_string());

        assert_eq!(error.name(), "CustomError");
        assert_eq!(error.to_string(), "CustomError: test");
    }

    #[test]
    fn test_error_message_is_writable() {
        let mut error = ErrorConstructor::construct(Some("original".to_string()));
        error.set_message("modified".to_string());

        assert_eq!(error.message(), "modified");
    }

    #[test]
    fn test_error_cause_property() {
        let cause = ErrorConstructor::type_error(Some("root cause".to_string()));
        let error = ErrorConstructor::with_cause(
            ErrorKind::Error,
            Some("wrapped".to_string()),
            cause,
        );

        let retrieved_cause = error.cause().expect("Should have cause");
        assert_eq!(retrieved_cause.name(), "TypeError");
        assert_eq!(retrieved_cause.message(), "root cause");
    }

    #[test]
    fn test_error_without_cause() {
        let error = ErrorConstructor::construct(Some("no cause".to_string()));
        assert!(error.cause().is_none());
    }
}

#[cfg(test)]
mod js_value_integration_tests {
    use super::*;

    #[test]
    fn test_js_value_error_variant() {
        let error = ErrorConstructor::type_error(Some("test".to_string()));
        let value = JsValue::from_error(error);

        assert!(value.is_error());
        assert!(!value.is_object());
        assert!(!value.is_undefined());
    }

    #[test]
    fn test_js_value_as_error() {
        let error = ErrorConstructor::reference_error(Some("not defined".to_string()));
        let value = JsValue::from_error(error);

        let extracted = value.as_error().expect("Should extract error");
        assert_eq!(extracted.name(), "ReferenceError");
        assert_eq!(extracted.message(), "not defined");
    }

    #[test]
    fn test_error_to_js_string() {
        let error = ErrorConstructor::syntax_error(Some("Unexpected token".to_string()));
        let value = JsValue::from_error(error);

        let string_rep = value.to_js_string();
        assert_eq!(string_rep, "SyntaxError: Unexpected token");
    }

    #[test]
    fn test_error_equality() {
        let error1 = ErrorConstructor::type_error(Some("same".to_string()));
        let error2 = ErrorConstructor::type_error(Some("same".to_string()));

        let value1 = JsValue::from_error(error1);
        let value2 = JsValue::from_error(error2);

        // Errors are reference types, different instances should not be equal
        assert!(!value1.equals(&value2));

        // But same instance should equal itself
        assert!(value1.equals(&value1));
    }
}

#[cfg(test)]
mod error_kind_tests {
    use super::*;

    #[test]
    fn test_error_kind_display() {
        assert_eq!(ErrorKind::Error.to_string(), "Error");
        assert_eq!(ErrorKind::TypeError.to_string(), "TypeError");
        assert_eq!(ErrorKind::ReferenceError.to_string(), "ReferenceError");
        assert_eq!(ErrorKind::SyntaxError.to_string(), "SyntaxError");
        assert_eq!(ErrorKind::RangeError.to_string(), "RangeError");
        assert_eq!(ErrorKind::URIError.to_string(), "URIError");
        assert_eq!(ErrorKind::EvalError.to_string(), "EvalError");
        assert_eq!(ErrorKind::AggregateError.to_string(), "AggregateError");
    }

    #[test]
    fn test_error_kind_is_error_type() {
        assert!(ErrorKind::Error.is_error_type());
        assert!(ErrorKind::TypeError.is_error_type());
        assert!(ErrorKind::ReferenceError.is_error_type());
        assert!(ErrorKind::SyntaxError.is_error_type());
        assert!(ErrorKind::RangeError.is_error_type());
        assert!(ErrorKind::URIError.is_error_type());
        assert!(ErrorKind::EvalError.is_error_type());
        assert!(ErrorKind::AggregateError.is_error_type());
    }
}
