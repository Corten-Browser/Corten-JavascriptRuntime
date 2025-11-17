//! Execution context for VM

use bytecode_system::BytecodeChunk;
use core_types::Value;

/// Execution context for a bytecode chunk
///
/// Contains the runtime state needed for executing bytecode:
/// registers for local variables, instruction pointer, and the bytecode itself.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {
    /// Register file for local variable storage
    pub registers: Vec<Value>,
    /// Current instruction pointer
    pub instruction_pointer: usize,
    /// The bytecode being executed
    pub bytecode: BytecodeChunk,
}

impl ExecutionContext {
    /// Create a new execution context for a bytecode chunk
    pub fn new(bytecode: BytecodeChunk) -> Self {
        let register_count = bytecode.register_count as usize;
        Self {
            registers: vec![Value::Undefined; register_count],
            instruction_pointer: 0,
            bytecode,
        }
    }

    /// Advance instruction pointer and return current instruction
    pub fn fetch(&mut self) -> Option<&bytecode_system::Instruction> {
        if self.instruction_pointer < self.bytecode.instructions.len() {
            let inst = &self.bytecode.instructions[self.instruction_pointer];
            self.instruction_pointer += 1;
            Some(inst)
        } else {
            None
        }
    }

    /// Get register value
    pub fn get_register(&self, index: usize) -> Value {
        self.registers
            .get(index)
            .cloned()
            .unwrap_or(Value::Undefined)
    }

    /// Set register value
    pub fn set_register(&mut self, index: usize, value: Value) {
        if index >= self.registers.len() {
            self.registers.resize(index + 1, Value::Undefined);
        }
        self.registers[index] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_new() {
        let mut chunk = BytecodeChunk::new();
        chunk.register_count = 5;

        let ctx = ExecutionContext::new(chunk);
        assert_eq!(ctx.registers.len(), 5);
        assert_eq!(ctx.instruction_pointer, 0);
    }

    #[test]
    fn test_execution_context_fetch() {
        use bytecode_system::Opcode;

        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::LoadUndefined);
        chunk.emit(Opcode::Return);

        let mut ctx = ExecutionContext::new(chunk);

        let inst1 = ctx.fetch();
        assert!(inst1.is_some());
        assert_eq!(ctx.instruction_pointer, 1);

        let inst2 = ctx.fetch();
        assert!(inst2.is_some());
        assert_eq!(ctx.instruction_pointer, 2);

        let inst3 = ctx.fetch();
        assert!(inst3.is_none());
    }

    #[test]
    fn test_execution_context_registers() {
        let chunk = BytecodeChunk::new();
        let mut ctx = ExecutionContext::new(chunk);

        ctx.set_register(0, Value::Smi(42));
        assert_eq!(ctx.get_register(0), Value::Smi(42));

        // Non-existent register returns Undefined
        assert_eq!(ctx.get_register(100), Value::Undefined);

        // Setting beyond current size extends the vector
        ctx.set_register(10, Value::Boolean(true));
        assert_eq!(ctx.get_register(10), Value::Boolean(true));
    }
}
