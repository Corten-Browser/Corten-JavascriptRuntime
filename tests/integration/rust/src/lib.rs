//! Integration test suite for Corten JavaScript Runtime
//!
//! This crate provides comprehensive integration tests that verify
//! components work together correctly across component boundaries.

/// Re-export components for test convenience
pub mod components {
    pub use bytecode_system;
    pub use core_types;
    pub use interpreter;
    pub use js_cli;
    pub use memory_manager;
    pub use parser;
}
