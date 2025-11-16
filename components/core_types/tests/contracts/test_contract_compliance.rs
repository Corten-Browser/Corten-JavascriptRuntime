//! Contract compliance tests for core_types
//!
//! These tests verify that the implementation matches the contract defined
//! in contracts/core_types.yaml

use core_types::{ErrorKind, JsError, SourcePosition, StackFrame, Value};

#[cfg(test)]
mod value_contract_tests {
    use super::*;

    /// Contract: Value enum must have all specified variants
    #[test]
    fn test_value_has_undefined_variant() {
        let _: Value = Value::Undefined;
    }

    #[test]
    fn test_value_has_null_variant() {
        let _: Value = Value::Null;
    }

    #[test]
    fn test_value_has_boolean_variant() {
        let _: Value = Value::Boolean(true);
        let _: Value = Value::Boolean(false);
    }

    #[test]
    fn test_value_has_smi_variant() {
        let _: Value = Value::Smi(0);
        let _: Value = Value::Smi(i32::MAX);
        let _: Value = Value::Smi(i32::MIN);
    }

    #[test]
    fn test_value_has_heap_object_variant() {
        // Contract specifies HeapObject(*mut Object)
        // Implementation uses usize for safety
        let _: Value = Value::HeapObject(0);
    }

    #[test]
    fn test_value_has_double_variant() {
        let _: Value = Value::Double(0.0);
        let _: Value = Value::Double(f64::NAN);
        let _: Value = Value::Double(f64::INFINITY);
    }

    /// Contract: Value must have is_truthy method returning bool
    #[test]
    fn test_value_is_truthy_method_exists() {
        let val = Value::Undefined;
        let _: bool = val.is_truthy();
    }

    #[test]
    fn test_value_is_truthy_returns_correct_type() {
        // Test each variant returns bool
        let _: bool = Value::Undefined.is_truthy();
        let _: bool = Value::Null.is_truthy();
        let _: bool = Value::Boolean(true).is_truthy();
        let _: bool = Value::Smi(42).is_truthy();
        let _: bool = Value::HeapObject(0).is_truthy();
        let _: bool = Value::Double(3.14).is_truthy();
    }

    /// Contract: Value must have to_string method returning String
    #[test]
    fn test_value_to_string_method_exists() {
        let val = Value::Undefined;
        let _: String = val.to_string();
    }

    #[test]
    fn test_value_to_string_returns_correct_type() {
        // Test each variant returns String
        let _: String = Value::Undefined.to_string();
        let _: String = Value::Null.to_string();
        let _: String = Value::Boolean(true).to_string();
        let _: String = Value::Smi(42).to_string();
        let _: String = Value::HeapObject(0).to_string();
        let _: String = Value::Double(3.14).to_string();
    }

    /// Contract: Value must have type_of method returning String
    #[test]
    fn test_value_type_of_method_exists() {
        let val = Value::Undefined;
        let _: String = val.type_of();
    }

    #[test]
    fn test_value_type_of_returns_correct_type() {
        // Test each variant returns String
        let _: String = Value::Undefined.type_of();
        let _: String = Value::Null.type_of();
        let _: String = Value::Boolean(true).type_of();
        let _: String = Value::Smi(42).type_of();
        let _: String = Value::HeapObject(0).type_of();
        let _: String = Value::Double(3.14).type_of();
    }
}

#[cfg(test)]
mod error_kind_contract_tests {
    use super::*;

    /// Contract: ErrorKind enum must have all specified variants
    #[test]
    fn test_error_kind_has_syntax_error_variant() {
        let _: ErrorKind = ErrorKind::SyntaxError;
    }

    #[test]
    fn test_error_kind_has_type_error_variant() {
        let _: ErrorKind = ErrorKind::TypeError;
    }

    #[test]
    fn test_error_kind_has_reference_error_variant() {
        let _: ErrorKind = ErrorKind::ReferenceError;
    }

    #[test]
    fn test_error_kind_has_range_error_variant() {
        let _: ErrorKind = ErrorKind::RangeError;
    }

    #[test]
    fn test_error_kind_has_eval_error_variant() {
        let _: ErrorKind = ErrorKind::EvalError;
    }

    #[test]
    fn test_error_kind_has_uri_error_variant() {
        let _: ErrorKind = ErrorKind::URIError;
    }

    #[test]
    fn test_error_kind_has_internal_error_variant() {
        let _: ErrorKind = ErrorKind::InternalError;
    }
}

#[cfg(test)]
mod js_error_contract_tests {
    use super::*;

    /// Contract: JsError struct must have all specified fields
    #[test]
    fn test_js_error_has_kind_field() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: String::new(),
            stack: vec![],
            source_position: None,
        };
        let _: ErrorKind = error.kind;
    }

    #[test]
    fn test_js_error_has_message_field() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "test message".to_string(),
            stack: vec![],
            source_position: None,
        };
        let _: String = error.message;
    }

    #[test]
    fn test_js_error_has_stack_field() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: String::new(),
            stack: vec![StackFrame {
                function_name: None,
                source_url: None,
                line: 0,
                column: 0,
            }],
            source_position: None,
        };
        let _: Vec<StackFrame> = error.stack;
    }

    #[test]
    fn test_js_error_has_source_position_field() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: String::new(),
            stack: vec![],
            source_position: Some(SourcePosition {
                line: 1,
                column: 1,
                offset: 0,
            }),
        };
        let _: Option<SourcePosition> = error.source_position;
    }

    #[test]
    fn test_js_error_source_position_can_be_none() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: String::new(),
            stack: vec![],
            source_position: None,
        };
        assert!(error.source_position.is_none());
    }
}

#[cfg(test)]
mod source_position_contract_tests {
    use super::*;

    /// Contract: SourcePosition struct must have all specified fields
    #[test]
    fn test_source_position_has_line_field() {
        let pos = SourcePosition {
            line: 42,
            column: 0,
            offset: 0,
        };
        let _: u32 = pos.line;
    }

    #[test]
    fn test_source_position_has_column_field() {
        let pos = SourcePosition {
            line: 0,
            column: 42,
            offset: 0,
        };
        let _: u32 = pos.column;
    }

    #[test]
    fn test_source_position_has_offset_field() {
        let pos = SourcePosition {
            line: 0,
            column: 0,
            offset: 1000,
        };
        let _: usize = pos.offset;
    }

    #[test]
    fn test_source_position_field_types() {
        let pos = SourcePosition {
            line: u32::MAX,
            column: u32::MAX,
            offset: usize::MAX,
        };
        assert_eq!(pos.line, u32::MAX);
        assert_eq!(pos.column, u32::MAX);
        assert_eq!(pos.offset, usize::MAX);
    }
}

#[cfg(test)]
mod stack_frame_contract_tests {
    use super::*;

    /// Contract: StackFrame struct must have all specified fields
    #[test]
    fn test_stack_frame_has_function_name_field() {
        let frame = StackFrame {
            function_name: Some("test".to_string()),
            source_url: None,
            line: 0,
            column: 0,
        };
        let _: Option<String> = frame.function_name;
    }

    #[test]
    fn test_stack_frame_has_source_url_field() {
        let frame = StackFrame {
            function_name: None,
            source_url: Some("test.js".to_string()),
            line: 0,
            column: 0,
        };
        let _: Option<String> = frame.source_url;
    }

    #[test]
    fn test_stack_frame_has_line_field() {
        let frame = StackFrame {
            function_name: None,
            source_url: None,
            line: 42,
            column: 0,
        };
        let _: u32 = frame.line;
    }

    #[test]
    fn test_stack_frame_has_column_field() {
        let frame = StackFrame {
            function_name: None,
            source_url: None,
            line: 0,
            column: 42,
        };
        let _: u32 = frame.column;
    }

    #[test]
    fn test_stack_frame_optional_fields_can_be_none() {
        let frame = StackFrame {
            function_name: None,
            source_url: None,
            line: 0,
            column: 0,
        };
        assert!(frame.function_name.is_none());
        assert!(frame.source_url.is_none());
    }

    #[test]
    fn test_stack_frame_optional_fields_can_be_some() {
        let frame = StackFrame {
            function_name: Some("fn".to_string()),
            source_url: Some("url".to_string()),
            line: 0,
            column: 0,
        };
        assert!(frame.function_name.is_some());
        assert!(frame.source_url.is_some());
    }
}

#[cfg(test)]
mod safe_rust_contract_tests {
    use super::*;

    /// Contract: All operations must be safe Rust (no unsafe)
    /// This is enforced by #![deny(unsafe_code)] in lib.rs
    #[test]
    fn test_value_operations_are_safe() {
        let val = Value::Smi(42);
        // All these operations compile without unsafe blocks
        let _ = val.is_truthy();
        let _ = val.to_string();
        let _ = val.type_of();
        let _ = val.clone();
    }

    #[test]
    fn test_error_operations_are_safe() {
        let error = JsError {
            kind: ErrorKind::TypeError,
            message: "test".to_string(),
            stack: vec![],
            source_position: None,
        };
        // All these operations compile without unsafe blocks
        let _ = error.clone();
        let _ = error.kind.clone();
    }

    #[test]
    fn test_source_position_operations_are_safe() {
        let pos = SourcePosition {
            line: 1,
            column: 1,
            offset: 0,
        };
        // All these operations compile without unsafe blocks
        let _ = pos.clone();
    }

    #[test]
    fn test_stack_frame_operations_are_safe() {
        let frame = StackFrame {
            function_name: None,
            source_url: None,
            line: 0,
            column: 0,
        };
        // All these operations compile without unsafe blocks
        let _ = frame.clone();
    }
}
