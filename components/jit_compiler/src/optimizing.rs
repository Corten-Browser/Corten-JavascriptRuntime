//! Optimizing JIT compiler
//!
//! Speculation-based compilation that uses profiling data to generate
//! highly optimized code with type guards and deoptimization support.

use crate::codegen::{CodeGenerator, CodegenConfig};
use crate::compiled_code::{CompilationTier, CompiledCode};
use crate::ir::{IRFunction, IROpcode};
use bytecode_system::BytecodeChunk;
use core_types::{ErrorKind, JsError};
use interpreter::{ProfileData, TypeInfo};

/// Statistics for optimizing JIT compilation
#[derive(Debug, Clone, Default)]
pub struct OptimizingStats {
    /// Number of functions compiled
    pub functions_compiled: u64,
    /// Total compilation time (microseconds)
    pub total_compilation_time_us: u64,
    /// Total code size generated
    pub total_code_size: usize,
    /// Number of type guards inserted
    pub type_guards_inserted: u64,
    /// Number of deopt points inserted
    pub deopt_points_inserted: u64,
}

/// Optimizing JIT compiler
///
/// Uses profiling data to perform speculative optimizations.
/// Generates faster code but takes longer to compile.
///
/// Characteristics:
/// - Slower compilation (10x slower than baseline)
/// - Maximum speedup (10-50x over interpreter)
/// - Uses type feedback for specialization
/// - Inserts type guards with deoptimization
/// - Performs advanced optimizations (inlining, escape analysis, etc.)
#[derive(Debug, Clone)]
pub struct OptimizingJIT {
    /// Code generator with optimizing configuration
    codegen: CodeGenerator,
    /// Compilation statistics
    stats: OptimizingStats,
    /// Minimum profile samples before type specialization
    min_samples: usize,
}

impl OptimizingJIT {
    /// Create a new optimizing JIT compiler
    pub fn new() -> Self {
        Self {
            codegen: CodeGenerator::new(CodegenConfig::optimizing()),
            stats: OptimizingStats::default(),
            min_samples: 10,
        }
    }

    /// Create optimizing JIT with custom sample threshold
    pub fn with_min_samples(min_samples: usize) -> Self {
        Self {
            codegen: CodeGenerator::new(CodegenConfig::optimizing()),
            stats: OptimizingStats::default(),
            min_samples,
        }
    }

    /// Compile a bytecode chunk with profiling information
    ///
    /// # Arguments
    /// * `chunk` - The bytecode chunk to compile
    /// * `profile` - Profiling data from interpreter execution
    ///
    /// # Returns
    /// * `Ok(CompiledCode)` - Successfully compiled optimized code
    /// * `Err(JsError)` - Compilation failed
    ///
    /// # Example
    /// ```
    /// use jit_compiler::OptimizingJIT;
    /// use bytecode_system::{BytecodeChunk, Opcode, Value};
    /// use interpreter::ProfileData;
    ///
    /// let mut jit = OptimizingJIT::new();
    /// let mut chunk = BytecodeChunk::new();
    /// let idx = chunk.add_constant(Value::Number(42.0));
    /// chunk.emit(Opcode::LoadConstant(idx));
    /// chunk.emit(Opcode::Return);
    ///
    /// let profile = ProfileData::new();
    /// let compiled = jit.compile(&chunk, &profile).unwrap();
    /// ```
    pub fn compile(
        &mut self,
        chunk: &BytecodeChunk,
        profile: &ProfileData,
    ) -> Result<CompiledCode, JsError> {
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
        let mut ir = IRFunction::from_bytecode(chunk);

        // Apply type specialization based on profile
        self.apply_type_specialization(&mut ir, profile);

        // Insert type guards for speculated types
        let guards_inserted = self.insert_type_guards(&mut ir, profile);
        self.stats.type_guards_inserted += guards_inserted as u64;

        // Insert deoptimization points
        let deopt_points = self.insert_deopt_points(&mut ir);
        self.stats.deopt_points_inserted += deopt_points as u64;

        // Apply optimization passes
        self.codegen.optimize(&mut ir);

        // Generate native code
        let codegen_result = self.codegen.generate(&ir);

        // Create compiled code object
        let mut compiled = CompiledCode::new(chunk.clone(), ir, CompilationTier::Optimized);

        // Add OSR entries from code generation
        for entry in codegen_result.osr_entries {
            compiled.add_osr_entry(entry);
        }

        // Update statistics
        self.stats.functions_compiled += 1;
        self.stats.total_code_size += codegen_result.code_size;

        Ok(compiled)
    }

    /// Apply type specialization based on profile data
    fn apply_type_specialization(&self, ir: &mut IRFunction, profile: &ProfileData) {
        if profile.type_feedback.len() < self.min_samples {
            return;
        }

        // Analyze dominant types
        let dominant_type = self.analyze_dominant_type(&profile.type_feedback);

        // Specialize operations based on dominant type
        for instruction in &mut ir.instructions {
            match &instruction.opcode {
                IROpcode::Add(None) => {
                    instruction.opcode = IROpcode::Add(dominant_type);
                }
                IROpcode::Sub(None) => {
                    instruction.opcode = IROpcode::Sub(dominant_type);
                }
                IROpcode::Mul(None) => {
                    instruction.opcode = IROpcode::Mul(dominant_type);
                }
                IROpcode::Div(None) => {
                    instruction.opcode = IROpcode::Div(dominant_type);
                }
                IROpcode::LessThan(None) => {
                    instruction.opcode = IROpcode::LessThan(dominant_type);
                }
                IROpcode::LessThanEqual(None) => {
                    instruction.opcode = IROpcode::LessThanEqual(dominant_type);
                }
                IROpcode::GreaterThan(None) => {
                    instruction.opcode = IROpcode::GreaterThan(dominant_type);
                }
                IROpcode::GreaterThanEqual(None) => {
                    instruction.opcode = IROpcode::GreaterThanEqual(dominant_type);
                }
                _ => {}
            }
        }
    }

    /// Analyze type feedback to find dominant type
    fn analyze_dominant_type(&self, feedback: &[TypeInfo]) -> Option<TypeInfo> {
        if feedback.is_empty() {
            return None;
        }

        // Count occurrences of each type
        let mut number_count = 0;
        let mut string_count = 0;
        let mut boolean_count = 0;
        let mut object_count = 0;

        for info in feedback {
            match info {
                TypeInfo::Number => number_count += 1,
                TypeInfo::String => string_count += 1,
                TypeInfo::Boolean => boolean_count += 1,
                TypeInfo::Object => object_count += 1,
                _ => {}
            }
        }

        // Find the most common type
        let total = feedback.len();
        let threshold = total * 90 / 100; // 90% threshold for monomorphic

        if number_count >= threshold {
            Some(TypeInfo::Number)
        } else if string_count >= threshold {
            Some(TypeInfo::String)
        } else if boolean_count >= threshold {
            Some(TypeInfo::Boolean)
        } else if object_count >= threshold {
            Some(TypeInfo::Object)
        } else {
            None // Polymorphic - don't specialize
        }
    }

    /// Insert type guards for speculated types
    fn insert_type_guards(&self, ir: &mut IRFunction, profile: &ProfileData) -> usize {
        if profile.type_feedback.is_empty() {
            return 0;
        }

        let dominant_type = self.analyze_dominant_type(&profile.type_feedback);
        if dominant_type.is_none() {
            return 0;
        }

        let type_info = dominant_type.unwrap();
        let mut guards_inserted = 0;

        // Insert guards before specialized operations
        let mut new_instructions = Vec::new();
        for (idx, instruction) in ir.instructions.iter().enumerate() {
            let needs_guard = matches!(
                &instruction.opcode,
                IROpcode::Add(Some(_))
                    | IROpcode::Sub(Some(_))
                    | IROpcode::Mul(Some(_))
                    | IROpcode::Div(Some(_))
            );

            if needs_guard {
                // Insert type guard before the operation
                new_instructions.push(crate::ir::IRInstruction::new(
                    IROpcode::TypeGuard(type_info),
                    instruction.bytecode_offset,
                ));
                guards_inserted += 1;
            }
            new_instructions.push(ir.instructions[idx].clone());
        }

        ir.instructions = new_instructions;
        guards_inserted
    }

    /// Insert deoptimization points after guards
    fn insert_deopt_points(&self, ir: &mut IRFunction) -> usize {
        let mut deopt_points = 0;
        let mut new_instructions = Vec::new();

        for instruction in &ir.instructions {
            new_instructions.push(instruction.clone());

            // Add deopt point after type guards
            if let IROpcode::TypeGuard(_) = &instruction.opcode {
                new_instructions.push(crate::ir::IRInstruction::new(
                    IROpcode::DeoptPoint(instruction.bytecode_offset),
                    instruction.bytecode_offset,
                ));
                deopt_points += 1;
            }
        }

        ir.instructions = new_instructions;
        deopt_points
    }

    /// Get compilation statistics
    pub fn stats(&self) -> &OptimizingStats {
        &self.stats
    }

    /// Reset compilation statistics
    pub fn reset_stats(&mut self) {
        self.stats = OptimizingStats::default();
    }
}

impl Default for OptimizingJIT {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::{Opcode, Value as BcValue};

    #[test]
    fn test_optimizing_jit_new() {
        let jit = OptimizingJIT::new();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_optimizing_jit_default() {
        let jit = OptimizingJIT::default();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_optimizing_jit_with_min_samples() {
        let jit = OptimizingJIT::with_min_samples(20);
        assert_eq!(jit.min_samples, 20);
    }

    #[test]
    fn test_compile_simple_function() {
        let mut jit = OptimizingJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let profile = ProfileData::new();
        let result = jit.compile(&chunk, &profile);
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert_eq!(compiled.tier(), CompilationTier::Optimized);
    }

    #[test]
    fn test_compile_empty_chunk_fails() {
        let mut jit = OptimizingJIT::new();
        let chunk = BytecodeChunk::new();
        let profile = ProfileData::new();

        let result = jit.compile(&chunk, &profile);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_specialization_with_number_feedback() {
        let mut jit = OptimizingJIT::with_min_samples(5);
        let mut chunk = BytecodeChunk::new();
        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(20.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let mut profile = ProfileData::new();
        // Add number type feedback
        for _ in 0..10 {
            profile.record_type(TypeInfo::Number);
        }

        let compiled = jit.compile(&chunk, &profile).unwrap();
        // Should have type guards inserted
        assert!(jit.stats().type_guards_inserted > 0);

        let _ = compiled;
    }

    #[test]
    fn test_no_specialization_with_insufficient_samples() {
        let mut jit = OptimizingJIT::with_min_samples(10);
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let mut profile = ProfileData::new();
        // Only 5 samples, not enough
        for _ in 0..5 {
            profile.record_type(TypeInfo::Number);
        }

        jit.compile(&chunk, &profile).unwrap();
        assert_eq!(jit.stats().type_guards_inserted, 0);
    }

    #[test]
    fn test_polymorphic_no_specialization() {
        let mut jit = OptimizingJIT::with_min_samples(5);
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let mut profile = ProfileData::new();
        // Mixed types - polymorphic
        for _ in 0..5 {
            profile.record_type(TypeInfo::Number);
        }
        for _ in 0..5 {
            profile.record_type(TypeInfo::String);
        }

        jit.compile(&chunk, &profile).unwrap();
        // Should not specialize for polymorphic code
        assert_eq!(jit.stats().type_guards_inserted, 0);
    }

    #[test]
    fn test_deopt_points_inserted() {
        let mut jit = OptimizingJIT::with_min_samples(5);
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let mut profile = ProfileData::new();
        for _ in 0..10 {
            profile.record_type(TypeInfo::Number);
        }

        jit.compile(&chunk, &profile).unwrap();
        // Deopt points should match type guards
        assert_eq!(
            jit.stats().deopt_points_inserted,
            jit.stats().type_guards_inserted
        );
    }

    #[test]
    fn test_stats_tracking() {
        let mut jit = OptimizingJIT::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);
        let profile = ProfileData::new();

        jit.compile(&chunk, &profile).unwrap();
        assert_eq!(jit.stats().functions_compiled, 1);

        jit.compile(&chunk, &profile).unwrap();
        assert_eq!(jit.stats().functions_compiled, 2);
    }

    #[test]
    fn test_reset_stats() {
        let mut jit = OptimizingJIT::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);
        let profile = ProfileData::new();

        jit.compile(&chunk, &profile).unwrap();
        assert_eq!(jit.stats().functions_compiled, 1);

        jit.reset_stats();
        assert_eq!(jit.stats().functions_compiled, 0);
    }

    #[test]
    fn test_analyze_dominant_type() {
        let jit = OptimizingJIT::new();

        // 100% numbers
        let feedback = vec![
            TypeInfo::Number,
            TypeInfo::Number,
            TypeInfo::Number,
            TypeInfo::Number,
            TypeInfo::Number,
        ];
        assert_eq!(jit.analyze_dominant_type(&feedback), Some(TypeInfo::Number));

        // Mixed - polymorphic
        let mixed = vec![
            TypeInfo::Number,
            TypeInfo::String,
            TypeInfo::Number,
            TypeInfo::String,
        ];
        assert_eq!(jit.analyze_dominant_type(&mixed), None);

        // Empty
        assert_eq!(jit.analyze_dominant_type(&[]), None);
    }

    #[test]
    fn test_compiled_code_execution() {
        let mut jit = OptimizingJIT::new();
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let profile = ProfileData::new();
        let compiled = jit.compile(&chunk, &profile).unwrap();
        let result = compiled.execute().unwrap();

        assert_eq!(result, core_types::Value::Smi(42));
    }
}
