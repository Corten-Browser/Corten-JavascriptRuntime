//! Core JavaScript value types and error handling.
//!
//! This crate provides the foundational types for a JavaScript runtime,
//! including value representation, error types, and source location tracking.
//!
//! # Overview
//!
//! - [`Value`] - Tagged representation of JavaScript values
//! - [`JsError`] - JavaScript errors with stack traces
//! - [`ErrorKind`] - Types of JavaScript errors
//! - [`SourcePosition`] - Source code location
//! - [`StackFrame`] - Call stack frame information
//!
//! # Examples
//!
//! ```
//! use core_types::{Value, JsError, ErrorKind};
//!
//! // Create JavaScript values
//! let num = Value::Smi(42);
//! assert!(num.is_truthy());
//! assert_eq!(num.type_of(), "number");
//!
//! // Create an error
//! let error = JsError {
//!     kind: ErrorKind::TypeError,
//!     message: "undefined is not a function".to_string(),
//!     stack: vec![],
//!     source_position: None,
//! };
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

mod error;
mod profile;
mod source;
mod value;

pub use error::{ErrorKind, JsError};
pub use profile::{BranchOutcome, ProfileData, TypeInfo};
pub use source::{SourcePosition, StackFrame};
pub use value::Value;
