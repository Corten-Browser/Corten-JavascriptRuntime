//! Bytecode optimization passes
//!
//! Implements dead code elimination, constant folding, and peephole optimizations.

use crate::chunk::BytecodeChunk;
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use crate::value::Value;

/// Bytecode optimizer that applies multiple optimization passes
pub struct Optimizer {
    /// Maximum number of optimization passes to run
    max_passes: usize,
}

impl Optimizer {
    /// Create a new optimizer with default settings
    pub fn new() -> Self {
        Self { max_passes: 10 }
    }

    /// Set maximum number of optimization passes
    pub fn with_max_passes(mut self, max: usize) -> Self {
        self.max_passes = max;
        self
    }

    /// Run all optimization passes on the chunk
    pub fn optimize(&mut self, chunk: &mut BytecodeChunk) {
        for _ in 0..self.max_passes {
            let changed = self.run_single_pass(chunk);
            if !changed {
                break;
            }
        }
    }

    /// Run a single optimization pass, returns true if any changes were made
    fn run_single_pass(&self, chunk: &mut BytecodeChunk) -> bool {
        let mut changed = false;

        // Dead code elimination
        if self.eliminate_dead_code(chunk) {
            changed = true;
        }

        // Constant folding
        if self.fold_constants(chunk) {
            changed = true;
        }

        // Peephole optimizations
        if self.peephole_optimize(chunk) {
            changed = true;
        }

        changed
    }

    /// Remove unreachable code after unconditional terminators
    fn eliminate_dead_code(&self, chunk: &mut BytecodeChunk) -> bool {
        if chunk.instructions.is_empty() {
            return false;
        }

        let mut new_instructions = Vec::new();
        let mut dead = false;

        for inst in chunk.instructions.drain(..) {
            if dead {
                // Skip dead code, but mark that we made changes
                continue;
            }

            let is_unconditional = inst.opcode.is_unconditional_terminator();
            new_instructions.push(inst);

            if is_unconditional {
                dead = true;
            }
        }

        let changed = new_instructions.len() < chunk.instructions.capacity();
        chunk.instructions = new_instructions;
        changed
    }

    /// Fold constant expressions at compile time
    fn fold_constants(&self, chunk: &mut BytecodeChunk) -> bool {
        if chunk.instructions.len() < 3 {
            return false;
        }

        let mut changed = false;
        let mut i = 0;

        while i + 2 < chunk.instructions.len() {
            // Look for pattern: LoadConstant, LoadConstant, BinaryOp
            if let (Opcode::LoadConstant(idx1), Opcode::LoadConstant(idx2), ref op) = (
                &chunk.instructions[i].opcode,
                &chunk.instructions[i + 1].opcode,
                &chunk.instructions[i + 2].opcode,
            ) {
                if op.is_binary_arithmetic() {
                    // Get the constant values
                    if *idx1 < chunk.constants.len() && *idx2 < chunk.constants.len() {
                        if let (Some(n1), Some(n2)) = (
                            chunk.constants[*idx1].as_number(),
                            chunk.constants[*idx2].as_number(),
                        ) {
                            // Compute the result
                            let result = match op {
                                Opcode::Add => Some(n1 + n2),
                                Opcode::Sub => Some(n1 - n2),
                                Opcode::Mul => Some(n1 * n2),
                                Opcode::Div => {
                                    if n2 != 0.0 {
                                        Some(n1 / n2)
                                    } else {
                                        None
                                    }
                                }
                                Opcode::Mod => {
                                    if n2 != 0.0 {
                                        Some(n1 % n2)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(folded) = result {
                                // Add the folded constant
                                let new_idx = chunk.constants.len();
                                chunk.constants.push(Value::Number(folded));

                                // Preserve source position from the operation
                                let pos = chunk.instructions[i + 2].source_position;

                                // Replace three instructions with one
                                let new_inst = Instruction {
                                    opcode: Opcode::LoadConstant(new_idx),
                                    source_position: pos,
                                };

                                chunk.instructions[i] = new_inst;
                                chunk.instructions.remove(i + 2);
                                chunk.instructions.remove(i + 1);

                                changed = true;
                                // Don't increment i, re-check this position
                                continue;
                            }
                        }
                    }
                }
            }

            i += 1;
        }

        changed
    }

    /// Apply peephole optimizations
    fn peephole_optimize(&self, chunk: &mut BytecodeChunk) -> bool {
        if chunk.instructions.is_empty() {
            return false;
        }

        let mut changed = false;

        // Double negation elimination
        if self.eliminate_double_negation(chunk) {
            changed = true;
        }

        changed
    }

    /// Remove consecutive Neg operations (double negation)
    fn eliminate_double_negation(&self, chunk: &mut BytecodeChunk) -> bool {
        if chunk.instructions.len() < 2 {
            return false;
        }

        let mut new_instructions = Vec::new();
        let mut i = 0;
        let mut changed = false;

        while i < chunk.instructions.len() {
            if i + 1 < chunk.instructions.len() {
                // Check for double negation
                if matches!(chunk.instructions[i].opcode, Opcode::Neg)
                    && matches!(chunk.instructions[i + 1].opcode, Opcode::Neg)
                {
                    // Skip both Neg instructions
                    i += 2;
                    changed = true;
                    continue;
                }
            }

            new_instructions.push(chunk.instructions[i].clone());
            i += 1;
        }

        chunk.instructions = new_instructions;
        changed
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_new() {
        let opt = Optimizer::new();
        assert_eq!(opt.max_passes, 10);
    }

    #[test]
    fn test_optimizer_with_max_passes() {
        let opt = Optimizer::new().with_max_passes(5);
        assert_eq!(opt.max_passes, 5);
    }

    #[test]
    fn test_optimizer_default() {
        let opt = Optimizer::default();
        assert_eq!(opt.max_passes, 10);
    }

    #[test]
    fn test_dead_code_basic() {
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);
        chunk.emit(Opcode::Add);

        let opt = Optimizer::new();
        let changed = opt.eliminate_dead_code(&mut chunk);

        assert!(changed);
        assert_eq!(chunk.instructions.len(), 1);
    }

    #[test]
    fn test_constant_folding_basic() {
        let mut chunk = BytecodeChunk::new();
        chunk.add_constant(Value::Number(2.0));
        chunk.add_constant(Value::Number(3.0));
        chunk.emit(Opcode::LoadConstant(0));
        chunk.emit(Opcode::LoadConstant(1));
        chunk.emit(Opcode::Add);

        let opt = Optimizer::new();
        let changed = opt.fold_constants(&mut chunk);

        assert!(changed);
        assert_eq!(chunk.instructions.len(), 1);
    }

    #[test]
    fn test_peephole_double_neg() {
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Neg);
        chunk.emit(Opcode::Neg);

        let opt = Optimizer::new();
        let changed = opt.eliminate_double_negation(&mut chunk);

        assert!(changed);
        assert_eq!(chunk.instructions.len(), 0);
    }
}
