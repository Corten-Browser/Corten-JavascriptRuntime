//! Profiling data collection for JIT compilation decisions
//!
//! This module re-exports profiling types from core_types for backwards compatibility.
//! The actual implementation is in core_types to avoid cyclic dependencies.

// Re-export all profiling types from core_types
pub use core_types::{BranchOutcome, ProfileData, TypeInfo};
