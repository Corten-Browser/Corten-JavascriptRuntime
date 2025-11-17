//! Error types for the CLI

use core_types::JsError;
use std::fmt;

/// CLI-specific errors
#[derive(Debug)]
pub enum CliError {
    /// JavaScript execution error
    JsError(JsError),

    /// File I/O error
    IoError(std::io::Error),

    /// Parse error
    ParseError(String),

    /// REPL error
    ReplError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::JsError(e) => write!(f, "JavaScript error: {:?}", e),
            CliError::IoError(e) => write!(f, "File error: {}", e),
            CliError::ParseError(s) => write!(f, "Parse error: {}", s),
            CliError::ReplError(s) => write!(f, "REPL error: {}", s),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<JsError> for CliError {
    fn from(err: JsError) -> Self {
        CliError::JsError(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError(err)
    }
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;
