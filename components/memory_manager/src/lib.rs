//! Memory Manager - Garbage collector and heap management
//!
//! This component provides:
//! - Generational garbage collection (young + old generation)
//! - Heap allocation and management
//! - Hidden classes for property access optimization
//! - Write barriers for remembered set maintenance
//! - Safe Rust wrappers for unsafe internals
//! - Concurrent and incremental garbage collection

pub mod concurrent_gc;
pub mod gc;
pub mod heap;
pub mod hidden_class;
pub mod object;
pub mod write_barrier;

// Re-export main types
pub use gc::*;
pub use heap::{GcStats, Heap};
pub use hidden_class::HiddenClass;
pub use object::JSObject;
pub use write_barrier::{write_barrier, write_barrier_gc, CardTable, Object, RememberedSet};

// Re-export concurrent GC types
pub use concurrent_gc::{
    AtomicMarkColor, ConcurrentConfig, ConcurrentMarker, ConcurrentStats, GcPhase,
    IncrementalConfig, IncrementalMarker, IncrementalStats, MarkStack, SafePoint,
    SafePointRequest, TriColor, WriteBarrierBuffer,
};
