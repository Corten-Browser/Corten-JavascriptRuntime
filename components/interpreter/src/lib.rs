//! Bytecode interpreter for JavaScript runtime
//!
//! This crate provides a bytecode virtual machine with:
//! - Stack-based execution with register allocation
//! - Inline caching for property access optimization
//! - Profiling data collection for JIT compilation triggers
//! - Tier transition detection (baseline/optimized JIT)
//!
//! # Example
//!
//! ```
//! use interpreter::VM;
//! use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
//! use core_types::Value;
//!
//! let mut vm = VM::new();
//! let mut chunk = BytecodeChunk::new();
//!
//! let idx = chunk.add_constant(BcValue::Number(42.0));
//! chunk.emit(Opcode::LoadConstant(idx));
//! chunk.emit(Opcode::Return);
//!
//! let result = vm.execute(&chunk).unwrap();
//! assert_eq!(result, Value::Smi(42));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod call_frame;
pub mod context;
pub mod dispatch;
pub mod gc_integration;
pub mod inline_cache;
pub mod profile;
pub mod promise_integration;
pub mod upvalue;
pub mod vm;

// Re-export main types at crate root
pub use call_frame::CallFrame;
pub use context::ExecutionContext;
pub use gc_integration::{GCObject, VMHeap};
pub use inline_cache::{InlineCache, ShapeId};
pub use profile::{BranchOutcome, ProfileData, TypeInfo};
pub use promise_integration::{PromiseConstructor, PromiseObject};
pub use upvalue::{Closure, Upvalue, UpvalueHandle};
pub use vm::VM;
