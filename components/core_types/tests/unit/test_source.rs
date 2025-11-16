//! Unit tests for SourcePosition and StackFrame
//!
//! Following TDD: These tests are written FIRST before implementation.

use core_types::{SourcePosition, StackFrame};

#[cfg(test)]
mod source_position_tests {
    use super::*;

    #[test]
    fn test_source_position_creation() {
        let pos = SourcePosition {
            line: 10,
            column: 5,
            offset: 150,
        };

        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 150);
    }

    #[test]
    fn test_source_position_zero_values() {
        let pos = SourcePosition {
            line: 0,
            column: 0,
            offset: 0,
        };

        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_source_position_large_values() {
        let pos = SourcePosition {
            line: u32::MAX,
            column: u32::MAX,
            offset: usize::MAX,
        };

        assert_eq!(pos.line, u32::MAX);
        assert_eq!(pos.column, u32::MAX);
        assert_eq!(pos.offset, usize::MAX);
    }

    #[test]
    fn test_source_position_clone() {
        let pos1 = SourcePosition {
            line: 42,
            column: 7,
            offset: 1000,
        };
        let pos2 = pos1.clone();

        assert_eq!(pos1.line, pos2.line);
        assert_eq!(pos1.column, pos2.column);
        assert_eq!(pos1.offset, pos2.offset);
    }

    #[test]
    fn test_source_position_debug() {
        let pos = SourcePosition {
            line: 1,
            column: 2,
            offset: 3,
        };
        let debug_str = format!("{:?}", pos);

        assert!(debug_str.contains("line"));
        assert!(debug_str.contains("column"));
        assert!(debug_str.contains("offset"));
    }

    #[test]
    fn test_source_position_equality() {
        let pos1 = SourcePosition {
            line: 10,
            column: 20,
            offset: 100,
        };
        let pos2 = SourcePosition {
            line: 10,
            column: 20,
            offset: 100,
        };
        let pos3 = SourcePosition {
            line: 11,
            column: 20,
            offset: 100,
        };

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }
}

#[cfg(test)]
mod stack_frame_tests {
    use super::*;

    #[test]
    fn test_stack_frame_creation_with_all_fields() {
        let frame = StackFrame {
            function_name: Some("myFunction".to_string()),
            source_url: Some("file:///main.js".to_string()),
            line: 25,
            column: 10,
        };

        assert_eq!(frame.function_name, Some("myFunction".to_string()));
        assert_eq!(frame.source_url, Some("file:///main.js".to_string()));
        assert_eq!(frame.line, 25);
        assert_eq!(frame.column, 10);
    }

    #[test]
    fn test_stack_frame_anonymous_function() {
        let frame = StackFrame {
            function_name: None,
            source_url: Some("script.js".to_string()),
            line: 1,
            column: 1,
        };

        assert_eq!(frame.function_name, None);
        assert!(frame.source_url.is_some());
    }

    #[test]
    fn test_stack_frame_no_source_url() {
        let frame = StackFrame {
            function_name: Some("eval".to_string()),
            source_url: None,
            line: 0,
            column: 0,
        };

        assert!(frame.function_name.is_some());
        assert_eq!(frame.source_url, None);
    }

    #[test]
    fn test_stack_frame_completely_empty() {
        let frame = StackFrame {
            function_name: None,
            source_url: None,
            line: 0,
            column: 0,
        };

        assert_eq!(frame.function_name, None);
        assert_eq!(frame.source_url, None);
        assert_eq!(frame.line, 0);
        assert_eq!(frame.column, 0);
    }

    #[test]
    fn test_stack_frame_clone() {
        let frame1 = StackFrame {
            function_name: Some("test".to_string()),
            source_url: Some("test.js".to_string()),
            line: 100,
            column: 50,
        };
        let frame2 = frame1.clone();

        assert_eq!(frame1.function_name, frame2.function_name);
        assert_eq!(frame1.source_url, frame2.source_url);
        assert_eq!(frame1.line, frame2.line);
        assert_eq!(frame1.column, frame2.column);
    }

    #[test]
    fn test_stack_frame_debug() {
        let frame = StackFrame {
            function_name: Some("fn".to_string()),
            source_url: None,
            line: 1,
            column: 2,
        };
        let debug_str = format!("{:?}", frame);

        assert!(debug_str.contains("function_name"));
        assert!(debug_str.contains("line"));
        assert!(debug_str.contains("column"));
    }

    #[test]
    fn test_stack_frame_equality() {
        let frame1 = StackFrame {
            function_name: Some("foo".to_string()),
            source_url: Some("bar.js".to_string()),
            line: 5,
            column: 3,
        };
        let frame2 = StackFrame {
            function_name: Some("foo".to_string()),
            source_url: Some("bar.js".to_string()),
            line: 5,
            column: 3,
        };
        let frame3 = StackFrame {
            function_name: Some("different".to_string()),
            source_url: Some("bar.js".to_string()),
            line: 5,
            column: 3,
        };

        assert_eq!(frame1, frame2);
        assert_ne!(frame1, frame3);
    }
}
