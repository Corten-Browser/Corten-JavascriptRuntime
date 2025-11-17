//! Multi-tier JIT compilation system for JavaScript runtime
//!
//! This crate provides:
//! - Baseline JIT: Template-based compilation for quick speedup
//! - Optimizing JIT: Speculation-based optimization for maximum performance
//! - OSR: On-Stack Replacement for transitioning between tiers
//! - Deoptimization: Safe fallback to interpreter when speculation fails
//!
//! # Example
//!
//! ```
//! use jit_compiler::{BaselineJIT, OptimizingJIT};
//! use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
//! use interpreter::ProfileData;
//!
//! // Baseline JIT compilation
//! let mut baseline = BaselineJIT::new();
//! let mut chunk = BytecodeChunk::new();
//! let idx = chunk.add_constant(BcValue::Number(42.0));
//! chunk.emit(Opcode::LoadConstant(idx));
//! chunk.emit(Opcode::Return);
//!
//! let compiled = baseline.compile(&chunk).unwrap();
//! let result = compiled.execute().unwrap();
//!
//! // Optimizing JIT with profiling data
//! let mut opt_jit = OptimizingJIT::new();
//! let profile = ProfileData::new();
//! let optimized = opt_jit.compile(&chunk, &profile).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod baseline;
pub mod codegen;
pub mod compiled_code;
pub mod cranelift_backend;
pub mod deopt;
pub mod ir;
pub mod optimizing;
pub mod osr;

// Re-export main types at crate root
pub use baseline::BaselineJIT;
pub use compiled_code::CompiledCode;
pub use cranelift_backend::{CompiledFunction, CraneliftBackend};
pub use deopt::Deoptimizer;
pub use optimizing::OptimizingJIT;
pub use osr::{FrameMapping, OSREntry};
