//! JavaScript Runtime CLI Library
//!
//! Provides the Runtime struct and supporting modules for the JavaScript CLI.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod cli;
pub mod error;
pub mod repl;
pub mod runtime;

pub use cli::Cli;
pub use error::{CliError, CliResult};
pub use runtime::Runtime;
