//! Bytecode system for JavaScript runtime
//!
//! This crate provides bytecode instruction set definitions, compilation,
//! and optimization passes for the JavaScript runtime.
//!
//! # Features
//!
//! - Register-based bytecode architecture
//! - Complete opcode set for JavaScript operations
//! - Binary serialization support
//! - Optimization passes (dead code elimination, constant folding)
//!
//! # Example
//!
//! ```
//! use bytecode_system::{BytecodeChunk, Opcode, Value};
//!
//! let mut chunk = BytecodeChunk::new();
//!
//! // Add constants
//! let idx = chunk.add_constant(Value::Number(42.0));
//!
//! // Emit instructions
//! chunk.emit(Opcode::LoadConstant(idx));
//! chunk.emit(Opcode::Return);
//!
//! // Optimize
//! chunk.optimize();
//!
//! // Serialize
//! let bytes = chunk.to_bytes();
//! let restored = BytecodeChunk::from_bytes(&bytes).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod chunk;
pub mod instruction;
pub mod opcode;
pub mod optimizer;
pub mod value;

// Re-export main types at crate root
pub use chunk::BytecodeChunk;
pub use instruction::{Instruction, SourcePosition};
pub use opcode::{Opcode, RegisterId};
pub use optimizer::Optimizer;
pub use value::Value;
