//! Code generation utilities
//!
//! Provides helper functions for generating native code from IR.
//! In this mock implementation, we simulate code generation.

use crate::ir::{IRFunction, IROpcode};
use crate::osr::{FrameMapping, OSREntry, RegisterLocation};

/// Code generation configuration
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Enable bounds check elimination
    pub eliminate_bounds_checks: bool,
    /// Enable dead code elimination
    pub dead_code_elimination: bool,
    /// Enable constant propagation
    pub constant_propagation: bool,
    /// Generate OSR entries at loop headers
    pub generate_osr_entries: bool,
}

impl CodegenConfig {
    /// Create default configuration
    pub fn new() -> Self {
        Self {
            eliminate_bounds_checks: false,
            dead_code_elimination: true,
            constant_propagation: true,
            generate_osr_entries: true,
        }
    }

    /// Create baseline JIT configuration (fast compilation)
    pub fn baseline() -> Self {
        Self {
            eliminate_bounds_checks: false,
            dead_code_elimination: false,
            constant_propagation: false,
            generate_osr_entries: true,
        }
    }

    /// Create optimizing JIT configuration (aggressive optimization)
    pub fn optimizing() -> Self {
        Self {
            eliminate_bounds_checks: true,
            dead_code_elimination: true,
            constant_propagation: true,
            generate_osr_entries: true,
        }
    }
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Code generation result
#[derive(Debug, Clone)]
pub struct CodegenResult {
    /// Generated code bytes (mock)
    pub code_bytes: Vec<u8>,
    /// OSR entry points
    pub osr_entries: Vec<OSREntry>,
    /// Code size in bytes
    pub code_size: usize,
}

/// Code generator
///
/// Transforms IR into native code (mock implementation).
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    config: CodegenConfig,
}

impl CodeGenerator {
    /// Create new code generator with configuration
    pub fn new(config: CodegenConfig) -> Self {
        Self { config }
    }

    /// Generate code from IR function
    pub fn generate(&self, ir: &IRFunction) -> CodegenResult {
        let mut code_bytes = Vec::new();
        let mut osr_entries = Vec::new();

        // Track instruction positions for OSR
        let mut native_offset = 0;

        for instruction in &ir.instructions {
            // Generate OSR entry at backward jumps (loop headers)
            if self.config.generate_osr_entries {
                if let IROpcode::Jump(target) = &instruction.opcode {
                    if *target <= instruction.bytecode_offset {
                        // This is a backward jump (loop)
                        let mut frame_mapping = FrameMapping::new();
                        for i in 0..ir.register_count {
                            frame_mapping.add_register(RegisterLocation::Stack(-(i as i32 * 8)));
                        }
                        frame_mapping.set_native_frame_size(ir.register_count as usize * 8);
                        frame_mapping.set_interpreter_frame_size(ir.register_count as usize);

                        let entry = OSREntry::with_mapping(
                            instruction.bytecode_offset,
                            native_offset,
                            frame_mapping,
                        );
                        osr_entries.push(entry);
                    }
                }
            }

            // Generate "native code" (mock bytes)
            let inst_bytes = self.generate_instruction(&instruction.opcode);
            code_bytes.extend_from_slice(&inst_bytes);
            native_offset += inst_bytes.len();
        }

        CodegenResult {
            code_size: code_bytes.len(),
            code_bytes,
            osr_entries,
        }
    }

    /// Generate mock bytes for a single instruction
    fn generate_instruction(&self, opcode: &IROpcode) -> Vec<u8> {
        // Mock code generation - just create placeholder bytes
        // Real implementation would generate actual x86/ARM instructions
        match opcode {
            IROpcode::LoadConst(_) => vec![0x48, 0xB8], // mov rax, imm64
            IROpcode::LoadUndefined => vec![0x48, 0x31, 0xC0], // xor rax, rax
            IROpcode::LoadNull => vec![0x48, 0x31, 0xC0], // xor rax, rax
            IROpcode::LoadTrue => vec![0xB8, 0x01],     // mov eax, 1
            IROpcode::LoadFalse => vec![0x48, 0x31, 0xC0], // xor rax, rax
            IROpcode::Add(_) => vec![0x48, 0x01, 0xD8], // add rax, rbx
            IROpcode::Sub(_) => vec![0x48, 0x29, 0xD8], // sub rax, rbx
            IROpcode::Mul(_) => vec![0x48, 0x0F, 0xAF, 0xC3], // imul rax, rbx
            IROpcode::Div(_) => vec![0x48, 0xF7, 0xF3], // div rbx
            IROpcode::Return => vec![0xC3],             // ret
            IROpcode::Jump(_) => vec![0xE9, 0x00, 0x00, 0x00, 0x00], // jmp rel32
            IROpcode::JumpIfTrue(_) => vec![0x0F, 0x85, 0x00, 0x00, 0x00, 0x00], // jnz rel32
            IROpcode::JumpIfFalse(_) => vec![0x0F, 0x84, 0x00, 0x00, 0x00, 0x00], // jz rel32
            IROpcode::TypeGuard(_) => vec![0x48, 0x85, 0xC0], // test rax, rax
            IROpcode::DeoptPoint(_) => vec![0xCC],      // int3
            _ => vec![0x90],                            // nop
        }
    }

    /// Apply optimization passes (if configured)
    pub fn optimize(&self, ir: &mut IRFunction) {
        if self.config.dead_code_elimination {
            self.eliminate_dead_code(ir);
        }
        if self.config.constant_propagation {
            self.propagate_constants(ir);
        }
    }

    /// Simple dead code elimination
    fn eliminate_dead_code(&self, ir: &mut IRFunction) {
        // Remove unreachable code after unconditional return
        let mut found_return = false;
        ir.instructions.retain(|inst| {
            if found_return {
                false
            } else {
                if let IROpcode::Return = inst.opcode {
                    found_return = true;
                }
                true
            }
        });
    }

    /// Simple constant propagation
    fn propagate_constants(&self, _ir: &mut IRFunction) {
        // In a real implementation, this would:
        // 1. Track constant values through operations
        // 2. Replace variable loads with constants when possible
        // 3. Fold constant expressions
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new(CodegenConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::{BytecodeChunk, Opcode};

    #[test]
    fn test_codegen_config_new() {
        let config = CodegenConfig::new();
        assert!(!config.eliminate_bounds_checks);
        assert!(config.dead_code_elimination);
        assert!(config.constant_propagation);
        assert!(config.generate_osr_entries);
    }

    #[test]
    fn test_codegen_config_baseline() {
        let config = CodegenConfig::baseline();
        assert!(!config.eliminate_bounds_checks);
        assert!(!config.dead_code_elimination);
        assert!(!config.constant_propagation);
        assert!(config.generate_osr_entries);
    }

    #[test]
    fn test_codegen_config_optimizing() {
        let config = CodegenConfig::optimizing();
        assert!(config.eliminate_bounds_checks);
        assert!(config.dead_code_elimination);
        assert!(config.constant_propagation);
        assert!(config.generate_osr_entries);
    }

    #[test]
    fn test_code_generator_creation() {
        let config = CodegenConfig::new();
        let _gen = CodeGenerator::new(config);
    }

    #[test]
    fn test_code_generator_default() {
        let _gen = CodeGenerator::default();
    }

    #[test]
    fn test_generate_simple_code() {
        let gen = CodeGenerator::default();
        let mut ir = IRFunction::new();
        ir.emit(IROpcode::LoadConst(0), 0);
        ir.emit(IROpcode::Return, 1);

        let result = gen.generate(&ir);
        assert!(result.code_size > 0);
        assert!(!result.code_bytes.is_empty());
    }

    #[test]
    fn test_generate_osr_entries_for_loops() {
        let gen = CodeGenerator::new(CodegenConfig::optimizing());

        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadConstant(0));
        chunk.add_constant(bytecode_system::Value::Number(0.0));
        chunk.emit(Opcode::Jump(0)); // Loop back to start
        chunk.register_count = 2;

        let ir = IRFunction::from_bytecode(&chunk);
        let result = gen.generate(&ir);

        // Should have an OSR entry for the backward jump
        assert!(!result.osr_entries.is_empty());
    }

    #[test]
    fn test_dead_code_elimination() {
        let gen = CodeGenerator::new(CodegenConfig::optimizing());

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::LoadConst(0), 0);
        ir.emit(IROpcode::Return, 1);
        ir.emit(IROpcode::LoadConst(1), 2); // Dead code
        ir.emit(IROpcode::Add(None), 3); // Dead code

        assert_eq!(ir.instruction_count(), 4);

        gen.optimize(&mut ir);

        // Dead code after return should be eliminated
        assert_eq!(ir.instruction_count(), 2);
    }

    #[test]
    fn test_no_optimization_when_disabled() {
        let gen = CodeGenerator::new(CodegenConfig::baseline());

        let mut ir = IRFunction::new();
        ir.emit(IROpcode::LoadConst(0), 0);
        ir.emit(IROpcode::Return, 1);
        ir.emit(IROpcode::LoadConst(1), 2);

        gen.optimize(&mut ir);

        // Should not eliminate dead code
        assert_eq!(ir.instruction_count(), 3);
    }
}
