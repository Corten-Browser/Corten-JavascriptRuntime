//! Parser error types and helpers

use core_types::{ErrorKind, JsError, SourcePosition};

/// Create a syntax error at a given position
pub fn syntax_error(message: impl Into<String>, position: Option<SourcePosition>) -> JsError {
    JsError {
        kind: ErrorKind::SyntaxError,
        message: message.into(),
        stack: vec![],
        source_position: position,
    }
}

/// Create an unexpected token error
pub fn unexpected_token(expected: &str, got: &str, position: Option<SourcePosition>) -> JsError {
    syntax_error(format!("Expected {}, got {}", expected, got), position)
}

/// Create an unexpected end of input error
pub fn unexpected_eof(position: Option<SourcePosition>) -> JsError {
    syntax_error("Unexpected end of input", position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error() {
        let err = syntax_error("test", None);
        assert!(matches!(err.kind, ErrorKind::SyntaxError));
    }

    #[test]
    fn test_unexpected_token() {
        let err = unexpected_token("identifier", "number", None);
        assert!(err.message.contains("Expected"));
    }
}
