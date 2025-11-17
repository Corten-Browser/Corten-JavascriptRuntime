//! Baseline JIT compiler
//!
//! Template-based compilation that provides fast compilation with modest speedup.
//! Compiles bytecode to native code using Cranelift as the backend.

use crate::compiled_code::CompiledCode;
use crate::cranelift_backend::CraneliftBackend;
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
/// Provides fast compilation using Cranelift code generation.
/// Each bytecode instruction is translated to native machine code.
///
/// Characteristics:
/// - Fast compilation using Cranelift
/// - Direct execution of native code
/// - Supports basic arithmetic and control flow
/// - No speculation or type guards (baseline only)
pub struct BaselineJIT {
    /// Cranelift backend for code generation
    backend: Option<CraneliftBackend>,
    /// Compilation statistics
    stats: BaselineStats,
}

impl BaselineJIT {
    /// Create a new baseline JIT compiler
    pub fn new() -> Self {
        let backend = CraneliftBackend::new().ok();
        Self {
            backend,
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
    /// let result = compiled.execute().unwrap();
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

        // Get the backend or return error
        let backend = self.backend.as_mut().ok_or_else(|| JsError {
            kind: ErrorKind::InternalError,
            message: "Cranelift backend not available".to_string(),
            stack: vec![],
            source_position: None,
        })?;

        // Compile to native code using Cranelift
        let compiled_func = backend.compile_function(chunk).map_err(|e| JsError {
            kind: ErrorKind::InternalError,
            message: format!("JIT compilation failed: {}", e),
            stack: vec![],
            source_position: None,
        })?;

        // Create CompiledCode from native code pointer
        let compiled = CompiledCode::new(compiled_func.code_ptr, compiled_func.code_size, vec![]);

        // Update statistics
        self.stats.functions_compiled += 1;
        self.stats.total_code_size += compiled_func.code_size;

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

    /// Check if the JIT backend is available
    pub fn is_available(&self) -> bool {
        self.backend.is_some()
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
    use core_types::Value;

    #[test]
    fn test_baseline_jit_new() {
        let jit = BaselineJIT::new();
        assert_eq!(jit.stats().functions_compiled, 0);
        assert!(jit.is_available());
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
        assert!(!compiled.code_ptr().is_null());
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

        assert_eq!(result, Value::Smi(100));
    }

    #[test]
    fn test_compile_boolean_operations() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadTrue);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        // True is represented as 1.0, which becomes Smi(1)
        assert_eq!(result, Value::Smi(1));
    }

    #[test]
    fn test_compile_and_execute_addition() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(32.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_subtraction() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(50.0));
        let idx2 = chunk.add_constant(BcValue::Number(8.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Sub);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_multiplication() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(6.0));
        let idx2 = chunk.add_constant(BcValue::Number(7.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Mul);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_division() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(84.0));
        let idx2 = chunk.add_constant(BcValue::Number(2.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Div);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_complex_expression() {
        // Compute: (10 + 20) * 2 - 18 = 42
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(20.0));
        let idx3 = chunk.add_constant(BcValue::Number(2.0));
        let idx4 = chunk.add_constant(BcValue::Number(18.0));

        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::LoadConstant(idx3));
        chunk.emit(Opcode::Mul);
        chunk.emit(Opcode::LoadConstant(idx4));
        chunk.emit(Opcode::Sub);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_negation() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(-42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Neg);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, Value::Smi(42));
    }

    #[test]
    fn test_compile_and_execute_double_result() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(3.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Div);
        chunk.emit(Opcode::Return);

        let compiled = jit.compile(&chunk).unwrap();
        let result = compiled.execute().unwrap();

        match result {
            Value::Double(val) => {
                assert!((val - 3.333333333333333).abs() < 1e-10);
            }
            _ => panic!("Expected Double value, got {:?}", result),
        }
    }

    #[test]
    fn test_invalidation_prevents_execution() {
        let mut jit = BaselineJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let mut compiled = jit.compile(&chunk).unwrap();

        // First execution should work
        let result1 = compiled.execute();
        assert!(result1.is_ok());

        // Invalidate
        compiled.invalidate();
        assert!(!compiled.is_valid());

        // Second execution should fail
        let result2 = compiled.execute();
        assert!(result2.is_err());
    }
}
