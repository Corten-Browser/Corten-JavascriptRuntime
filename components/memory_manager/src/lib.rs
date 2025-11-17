//! Memory Manager - Garbage collector and heap management
//!
//! This component provides:
//! - Generational garbage collection (young + old generation)
//! - Heap allocation and management
//! - Hidden classes for property access optimization
//! - Write barriers for remembered set maintenance
//! - Safe Rust wrappers for unsafe internals

pub mod gc;
pub mod heap;
pub mod hidden_class;
pub mod object;
pub mod write_barrier;

// Re-export main types
pub use gc::*;
pub use heap::Heap;
pub use hidden_class::HiddenClass;
pub use object::JSObject;
pub use write_barrier::write_barrier;
