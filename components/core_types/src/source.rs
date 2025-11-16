//! Source position and stack frame types for JavaScript error tracking.
//!
//! This module provides types for tracking source locations and call stacks
//! in JavaScript execution.

/// Represents a position in source code.
///
/// Used for error reporting and debugging to indicate where an issue occurred.
///
/// # Examples
///
/// ```
/// use core_types::SourcePosition;
///
/// let pos = SourcePosition {
///     line: 10,
///     column: 5,
///     offset: 150,
/// };
///
/// assert_eq!(pos.line, 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (0-indexed or 1-indexed depending on context)
    pub line: u32,
    /// Column number (0-indexed or 1-indexed depending on context)
    pub column: u32,
    /// Byte offset from the start of the source file
    pub offset: usize,
}

/// Represents a single frame in a JavaScript call stack.
///
/// Contains information about where in the code execution occurred,
/// useful for generating stack traces.
///
/// # Examples
///
/// ```
/// use core_types::StackFrame;
///
/// let frame = StackFrame {
///     function_name: Some("myFunction".to_string()),
///     source_url: Some("file:///main.js".to_string()),
///     line: 25,
///     column: 10,
/// };
///
/// assert_eq!(frame.function_name, Some("myFunction".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    /// Name of the function, or None for anonymous functions
    pub function_name: Option<String>,
    /// URL or file path of the source, or None if not available
    pub source_url: Option<String>,
    /// Line number where the call occurred
    pub line: u32,
    /// Column number where the call occurred
    pub column: u32,
}

#[cfg(test)]
mod tests {
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
    fn test_stack_frame_creation() {
        let frame = StackFrame {
            function_name: Some("test".to_string()),
            source_url: None,
            line: 1,
            column: 1,
        };
        assert_eq!(frame.function_name, Some("test".to_string()));
    }
}
