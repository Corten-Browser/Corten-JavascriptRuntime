//! Deoptimization support
//!
//! Handles safe transition from compiled code back to interpreter
//! when speculation fails or guards are violated.

use crate::compiled_code::CompiledCode;
use bytecode_system::BytecodeChunk;
use core_types::Value;

/// Interpreter state for deoptimization
///
/// This represents the state that the interpreter needs to resume execution.
/// It mirrors the structure of interpreter::ExecutionContext but is defined
/// here to avoid cyclic dependencies.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpreterState {
    /// Register file for local variable storage
    pub registers: Vec<Value>,
    /// Current instruction pointer
    pub instruction_pointer: usize,
    /// The bytecode being executed
    pub bytecode: BytecodeChunk,
}

impl InterpreterState {
    /// Create a new interpreter state from bytecode
    pub fn new(bytecode: BytecodeChunk) -> Self {
        let register_count = bytecode.register_count as usize;
        Self {
            registers: vec![Value::Undefined; register_count],
            instruction_pointer: 0,
            bytecode,
        }
    }
}

/// Reason for deoptimization
#[derive(Debug, Clone, PartialEq)]
pub enum DeoptReason {
    /// Type guard failed
    TypeGuardFailure,
    /// Bounds check failed
    BoundsCheckFailure,
    /// Unknown property shape
    ShapeMismatch,
    /// Stack overflow
    StackOverflow,
    /// Explicit deopt request
    Explicit,
}

/// Information about a deoptimization event
#[derive(Debug, Clone)]
pub struct DeoptInfo {
    /// Reason for deoptimization
    pub reason: DeoptReason,
    /// Bytecode offset to resume at
    pub resume_offset: usize,
    /// Number of deoptimizations for this code
    pub deopt_count: u32,
}

impl DeoptInfo {
    /// Create new deopt info
    pub fn new(reason: DeoptReason, resume_offset: usize) -> Self {
        Self {
            reason,
            resume_offset,
            deopt_count: 1,
        }
    }
}

/// Deoptimizer for safe fallback to interpreter
///
/// Maps optimized frame state back to interpreter state when
/// speculation fails or guards are violated.
#[derive(Debug, Clone)]
pub struct Deoptimizer {
    /// History of deoptimizations (for tracking hot deopt points)
    deopt_history: Vec<DeoptInfo>,
    /// Maximum number of deoptimizations before disabling optimization
    max_deopt_count: u32,
}

impl Deoptimizer {
    /// Create a new deoptimizer
    pub fn new() -> Self {
        Self {
            deopt_history: Vec::new(),
            max_deopt_count: 10,
        }
    }

    /// Create deoptimizer with custom max deopt count
    pub fn with_max_count(max_count: u32) -> Self {
        Self {
            deopt_history: Vec::new(),
            max_deopt_count: max_count,
        }
    }

    /// Deoptimize from compiled code back to interpreter
    ///
    /// Returns the interpreter state needed to resume execution.
    /// The bytecode must be provided separately since CompiledCode now only
    /// contains native code pointers.
    pub fn deoptimize(
        &self,
        _compiled: &CompiledCode,
        bytecode: &BytecodeChunk,
    ) -> InterpreterState {
        // Create a fresh interpreter state from the original bytecode
        // In a real implementation, we would:
        // 1. Save current compiled frame state
        // 2. Map compiled registers to interpreter registers
        // 3. Reconstruct interpreter stack frames
        // 4. Set instruction pointer to correct bytecode offset
        // 5. Resume interpreter execution
        InterpreterState::new(bytecode.clone())
    }

    /// Deoptimize with reason tracking
    pub fn deoptimize_with_reason(
        &mut self,
        _compiled: &CompiledCode,
        bytecode: &BytecodeChunk,
        reason: DeoptReason,
        bytecode_offset: usize,
    ) -> InterpreterState {
        // Track the deoptimization
        let info = DeoptInfo::new(reason, bytecode_offset);
        self.deopt_history.push(info);

        // Create interpreter state
        let mut state = InterpreterState::new(bytecode.clone());

        // Set instruction pointer to resume point
        state.instruction_pointer = bytecode_offset;

        state
    }

    /// Check if too many deoptimizations have occurred
    pub fn should_disable_optimization(&self) -> bool {
        self.deopt_history.len() as u32 >= self.max_deopt_count
    }

    /// Get the deoptimization count
    pub fn deopt_count(&self) -> usize {
        self.deopt_history.len()
    }

    /// Get deoptimization history
    pub fn history(&self) -> &[DeoptInfo] {
        &self.deopt_history
    }

    /// Clear deoptimization history
    pub fn clear_history(&mut self) {
        self.deopt_history.clear();
    }

    /// Check if a specific reason has caused frequent deopts
    pub fn is_frequent_deopt_reason(&self, reason: &DeoptReason) -> bool {
        let count = self
            .deopt_history
            .iter()
            .filter(|info| &info.reason == reason)
            .count();
        count >= 3
    }
}

impl Default for Deoptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cranelift_backend::CraneliftBackend;
    use bytecode_system::{BytecodeChunk, Opcode};

    fn create_test_bytecode() -> BytecodeChunk {
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadUndefined);
        chunk.emit(Opcode::Return);
        chunk.register_count = 3;
        chunk
    }

    fn create_test_compiled_code(chunk: &BytecodeChunk) -> CompiledCode {
        let mut backend = CraneliftBackend::new().unwrap();
        let compiled_func = backend.compile_function(chunk).unwrap();
        CompiledCode::new(compiled_func.code_ptr, compiled_func.code_size, vec![])
    }

    #[test]
    fn test_deoptimizer_new() {
        let deopt = Deoptimizer::new();
        assert_eq!(deopt.deopt_count(), 0);
        assert!(!deopt.should_disable_optimization());
    }

    #[test]
    fn test_deoptimizer_default() {
        let deopt = Deoptimizer::default();
        assert_eq!(deopt.deopt_count(), 0);
    }

    #[test]
    fn test_deoptimizer_with_max_count() {
        let deopt = Deoptimizer::with_max_count(5);
        assert_eq!(deopt.max_deopt_count, 5);
    }

    #[test]
    fn test_deoptimize() {
        let deopt = Deoptimizer::new();
        let bytecode = create_test_bytecode();
        let compiled = create_test_compiled_code(&bytecode);

        let context = deopt.deoptimize(&compiled, &bytecode);
        assert_eq!(context.instruction_pointer, 0);
        assert_eq!(context.registers.len(), 3);
    }

    #[test]
    fn test_deoptimize_with_reason() {
        let mut deopt = Deoptimizer::new();
        let bytecode = create_test_bytecode();
        let compiled = create_test_compiled_code(&bytecode);

        let context =
            deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::TypeGuardFailure, 1);

        assert_eq!(context.instruction_pointer, 1);
        assert_eq!(deopt.deopt_count(), 1);
        assert_eq!(deopt.history()[0].reason, DeoptReason::TypeGuardFailure);
    }

    #[test]
    fn test_should_disable_optimization() {
        let mut deopt = Deoptimizer::with_max_count(3);
        let bytecode = create_test_bytecode();
        let compiled = create_test_compiled_code(&bytecode);

        assert!(!deopt.should_disable_optimization());

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::TypeGuardFailure, 0);
        assert!(!deopt.should_disable_optimization());

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::ShapeMismatch, 0);
        assert!(!deopt.should_disable_optimization());

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::BoundsCheckFailure, 0);
        assert!(deopt.should_disable_optimization());
    }

    #[test]
    fn test_clear_history() {
        let mut deopt = Deoptimizer::new();
        let bytecode = create_test_bytecode();
        let compiled = create_test_compiled_code(&bytecode);

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::Explicit, 0);
        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::Explicit, 0);

        assert_eq!(deopt.deopt_count(), 2);
        deopt.clear_history();
        assert_eq!(deopt.deopt_count(), 0);
    }

    #[test]
    fn test_is_frequent_deopt_reason() {
        let mut deopt = Deoptimizer::new();
        let bytecode = create_test_bytecode();
        let compiled = create_test_compiled_code(&bytecode);

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::TypeGuardFailure, 0);
        assert!(!deopt.is_frequent_deopt_reason(&DeoptReason::TypeGuardFailure));

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::TypeGuardFailure, 0);
        assert!(!deopt.is_frequent_deopt_reason(&DeoptReason::TypeGuardFailure));

        deopt.deoptimize_with_reason(&compiled, &bytecode, DeoptReason::TypeGuardFailure, 0);
        assert!(deopt.is_frequent_deopt_reason(&DeoptReason::TypeGuardFailure));

        // Other reasons should not be frequent
        assert!(!deopt.is_frequent_deopt_reason(&DeoptReason::ShapeMismatch));
    }

    #[test]
    fn test_deopt_info() {
        let info = DeoptInfo::new(DeoptReason::StackOverflow, 100);
        assert_eq!(info.reason, DeoptReason::StackOverflow);
        assert_eq!(info.resume_offset, 100);
        assert_eq!(info.deopt_count, 1);
    }
}
