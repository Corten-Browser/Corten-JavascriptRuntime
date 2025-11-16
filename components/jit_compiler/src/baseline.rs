//! Baseline JIT compiler
//!
//! Template-based compilation that provides fast compilation with modest speedup.
//! Compiles bytecode to native code using fixed templates for each opcode.

use crate::codegen::{CodeGenerator, CodegenConfig};
use crate::compiled_code::{CompilationTier, CompiledCode};
use crate::ir::IRFunction;
use bytecode_system::BytecodeChunk;
use core_types::{ErrorKind, JsError};

/// Statistics for baseline JIT compilation
#[derive(Debug, Clone, Default)]
pub struct BaselineStats {
    /// Number of functions compiled
    pub functions_compiled: u64,
    /// Total compilation time (microseconds)
    pub total_compilation_time_us: u64,
    /// Total code size generated
    pub total_code_size: usize,
}

/// Baseline JIT compiler
///
/// Provides fast compilation using template-based code generation.
/// Each bytecode instruction maps to a fixed native code sequence.
///
/// Characteristics:
/// - Fast compilation (~10x faster than optimizing JIT)
/// - Modest speedup (2-3x over interpreter)
/// - Preserves interpreter inline caches
/// - No speculation or type guards
#[derive(Debug, Clone)]
pub struct BaselineJIT {
    /// Code generator with baseline configuration
    codegen: CodeGenerator,
    /// Compilation statistics
    stats: BaselineStats,
}

impl BaselineJIT {
    /// Create a new baseline JIT compiler
    pub fn new() -> Self {
        Self {
            codegen: CodeGenerator::new(CodegenConfig::baseline()),
            stats: BaselineStats::default(),
        }
    }

    /// Compile a bytecode chunk to native code
    ///
    /// # Arguments
    /// * `chunk` - The bytecode chunk to compile
    ///
    /// # Returns
    /// * `Ok(CompiledCode)` - Successfully compiled code
    /// * `Err(JsError)` - Compilation failed
    ///
    /// # Example
    /// ```
    /// use jit_compiler::BaselineJIT;
    /// use bytecode_system::{BytecodeChunk, Opcode, Value};
    ///
    /// let mut jit = BaselineJIT::new();
    /// let mut chunk = BytecodeChunk::new();
    /// let idx = chunk.add_constant(Value::Number(42.0));
    /// chunk.emit(Opcode::LoadConstant(idx));
    /// chunk.emit(Opcode::Return);
    ///
    /// let compiled = jit.compile(&chunk).unwrap();
    /// ```
    pub fn compile(&mut self, chunk: &BytecodeChunk) -> Result<CompiledCode, JsError> {
        // Validate input
        if chunk.instructions.is_empty() {
            return Err(JsError {
                kind: ErrorKind::InternalError,
                message: "Cannot compile empty bytecode chunk".to_string(),
                stack: vec![],
                source_position: None,
            });
        }

        // Convert bytecode to IR
        let ir = IRFunction::from_bytecode(chunk);

        // Generate native code
        let codegen_result = self.codegen.generate(&ir);

        // Create compiled code object
        let mut compiled = CompiledCode::new(chunk.clone(), ir, CompilationTier::Baseline);

        // Add OSR entries from code generation
        for entry in codegen_result.osr_entries {
            compiled.add_osr_entry(entry);
        }

        // Update statistics
        self.stats.functions_compiled += 1;
        self.stats.total_code_size += codegen_result.code_size;

        Ok(compiled)
    }

    /// Get compilation statistics
    pub fn stats(&self) -> &BaselineStats {
        &self.stats
    }

    /// Reset compilation statistics
    pub fn reset_stats(&mut self) {
        self.stats = BaselineStats::default();
    }
}

impl Default for BaselineJIT {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::{Opcode, Value as BcValue};

    #[test]
    fn test_baseline_jit_new() {
        let jit = BaselineJIT::new();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_baseline_jit_default() {
        let jit = BaselineJIT::default();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_compile_simple_function() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let result = jit.compile(&chunk);
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert_eq!(compiled.tier(), CompilationTier::Baseline);
        assert!(compiled.size > 0);
    }

    #[test]
    fn test_compile_empty_chunk_fails() {
        let mut jit = BaselineJIT::new();
        let chunk = BytecodeChunk::new();

        let result = jit.compile(&chunk);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.kind, ErrorKind::InternalError);
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_compile_with_arithmetic() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(20.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let result = jit.compile(&chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_loop() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadConstant(0));
        chunk.add_constant(BcValue::Number(0.0));
        chunk.emit(Opcode::Jump(0));
        chunk.register_count = 2;

        let result = jit.compile(&chunk);
        assert!(result.is_ok());

        let compiled = result.unwrap();
        // Should have OSR entry for the loop
        assert!(!compiled.osr_entries.is_empty());
    }

    #[test]
    fn test_stats_tracking() {
        let mut jit = BaselineJIT::new();

        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);

        jit.compile(&chunk).unwrap();
        assert_eq!(jit.stats().functions_compiled, 1);

        jit.compile(&chunk).unwrap();
        assert_eq!(jit.stats().functions_compiled, 2);
    }

    #[test]
    fn test_reset_stats() {
        let mut jit = BaselineJIT::new();

        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);

        jit.compile(&chunk).unwrap();
        assert_eq!(jit.stats().functions_compiled, 1);

        jit.reset_stats();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_compiled_code_execution() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(100.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, core_types::Value::Smi(100));
    }

    #[test]
    fn test_compile_boolean_operations() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadTrue);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, core_types::Value::Boolean(true));
    }
}
