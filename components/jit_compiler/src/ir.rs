//! Intermediate representation for JIT compilation
//!
//! Provides an IR that sits between bytecode and machine code,
//! enabling optimization passes.

use bytecode_system::{BytecodeChunk, Opcode};
use interpreter::TypeInfo;

/// IR operation types
#[derive(Debug, Clone, PartialEq)]
pub enum IROpcode {
    /// Load a constant value
    LoadConst(usize),
    /// Load undefined
    LoadUndefined,
    /// Load null
    LoadNull,
    /// Load boolean true
    LoadTrue,
    /// Load boolean false
    LoadFalse,
    /// Load from register
    LoadReg(u32),
    /// Store to register
    StoreReg(u32),
    /// Add two values (with optional type specialization)
    Add(Option<TypeInfo>),
    /// Subtract two values
    Sub(Option<TypeInfo>),
    /// Multiply two values
    Mul(Option<TypeInfo>),
    /// Divide two values
    Div(Option<TypeInfo>),
    /// Modulo operation
    Mod(Option<TypeInfo>),
    /// Negate value
    Neg(Option<TypeInfo>),
    /// Equality check
    Equal,
    /// Strict equality check
    StrictEqual,
    /// Not equal check
    NotEqual,
    /// Strict not equal check
    StrictNotEqual,
    /// Less than comparison
    LessThan(Option<TypeInfo>),
    /// Less than or equal comparison
    LessThanEqual(Option<TypeInfo>),
    /// Greater than comparison
    GreaterThan(Option<TypeInfo>),
    /// Greater than or equal comparison
    GreaterThanEqual(Option<TypeInfo>),
    /// Unconditional jump
    Jump(usize),
    /// Jump if true
    JumpIfTrue(usize),
    /// Jump if false
    JumpIfFalse(usize),
    /// Return from function
    Return,
    /// Create object
    CreateObject,
    /// Load property
    LoadProperty(String),
    /// Store property
    StoreProperty(String),
    /// Load global variable
    LoadGlobal(String),
    /// Store global variable
    StoreGlobal(String),
    /// Load upvalue (captured variable)
    LoadUpvalue(u32),
    /// Store upvalue (captured variable)
    StoreUpvalue(u32),
    /// Close upvalue
    CloseUpvalue,
    /// Create closure
    CreateClosure(usize),
    /// Call function
    Call(u8),
    /// Type guard (for speculation)
    TypeGuard(TypeInfo),
    /// Deoptimization point
    DeoptPoint(usize),
}

/// Single IR instruction
#[derive(Debug, Clone, PartialEq)]
pub struct IRInstruction {
    /// The operation to perform
    pub opcode: IROpcode,
    /// Source bytecode offset (for debugging and OSR)
    pub bytecode_offset: usize,
}

impl IRInstruction {
    /// Create a new IR instruction
    pub fn new(opcode: IROpcode, bytecode_offset: usize) -> Self {
        Self {
            opcode,
            bytecode_offset,
        }
    }
}

/// IR function representation
#[derive(Debug, Clone)]
pub struct IRFunction {
    /// List of IR instructions
    pub instructions: Vec<IRInstruction>,
    /// Constant pool (shared with bytecode)
    pub constants: Vec<bytecode_system::Value>,
    /// Number of registers needed
    pub register_count: u32,
}

impl IRFunction {
    /// Create a new empty IR function
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            register_count: 0,
        }
    }

    /// Build IR from bytecode chunk
    pub fn from_bytecode(chunk: &BytecodeChunk) -> Self {
        let mut ir_func = Self::new();
        ir_func.constants = chunk.constants.clone();
        ir_func.register_count = chunk.register_count;

        for (offset, instruction) in chunk.instructions.iter().enumerate() {
            let ir_op = match &instruction.opcode {
                Opcode::LoadConstant(idx) => IROpcode::LoadConst(*idx),
                Opcode::LoadUndefined => IROpcode::LoadUndefined,
                Opcode::LoadNull => IROpcode::LoadNull,
                Opcode::LoadTrue => IROpcode::LoadTrue,
                Opcode::LoadFalse => IROpcode::LoadFalse,
                Opcode::LoadGlobal(name) => IROpcode::LoadGlobal(name.clone()),
                Opcode::StoreGlobal(name) => IROpcode::StoreGlobal(name.clone()),
                Opcode::LoadLocal(reg) => IROpcode::LoadReg(reg.0),
                Opcode::StoreLocal(reg) => IROpcode::StoreReg(reg.0),
                Opcode::Add => IROpcode::Add(None),
                Opcode::Sub => IROpcode::Sub(None),
                Opcode::Mul => IROpcode::Mul(None),
                Opcode::Div => IROpcode::Div(None),
                Opcode::Mod => IROpcode::Mod(None),
                Opcode::Neg => IROpcode::Neg(None),
                Opcode::Equal => IROpcode::Equal,
                Opcode::StrictEqual => IROpcode::StrictEqual,
                Opcode::NotEqual => IROpcode::NotEqual,
                Opcode::StrictNotEqual => IROpcode::StrictNotEqual,
                Opcode::LessThan => IROpcode::LessThan(None),
                Opcode::LessThanEqual => IROpcode::LessThanEqual(None),
                Opcode::GreaterThan => IROpcode::GreaterThan(None),
                Opcode::GreaterThanEqual => IROpcode::GreaterThanEqual(None),
                Opcode::Jump(target) => IROpcode::Jump(*target),
                Opcode::JumpIfTrue(target) => IROpcode::JumpIfTrue(*target),
                Opcode::JumpIfFalse(target) => IROpcode::JumpIfFalse(*target),
                Opcode::Return => IROpcode::Return,
                Opcode::CreateObject => IROpcode::CreateObject,
                Opcode::LoadProperty(name) => IROpcode::LoadProperty(name.clone()),
                Opcode::StoreProperty(name) => IROpcode::StoreProperty(name.clone()),
                Opcode::LoadUpvalue(idx) => IROpcode::LoadUpvalue(*idx),
                Opcode::StoreUpvalue(idx) => IROpcode::StoreUpvalue(*idx),
                Opcode::CloseUpvalue => IROpcode::CloseUpvalue,
                Opcode::CreateClosure(idx, _) => IROpcode::CreateClosure(*idx),
                Opcode::Call(argc) => IROpcode::Call(*argc),
            };

            ir_func.instructions.push(IRInstruction::new(ir_op, offset));
        }

        ir_func
    }

    /// Add an instruction to the IR
    pub fn emit(&mut self, opcode: IROpcode, bytecode_offset: usize) {
        self.instructions
            .push(IRInstruction::new(opcode, bytecode_offset));
    }

    /// Get number of instructions
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}

impl Default for IRFunction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_function_new() {
        let ir = IRFunction::new();
        assert!(ir.instructions.is_empty());
        assert!(ir.constants.is_empty());
        assert_eq!(ir.register_count, 0);
    }

    #[test]
    fn test_ir_function_default() {
        let ir = IRFunction::default();
        assert!(ir.instructions.is_empty());
    }

    #[test]
    fn test_ir_function_from_bytecode() {
        let mut chunk = BytecodeChunk::new();
        let idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);
        chunk.register_count = 5;

        let ir = IRFunction::from_bytecode(&chunk);
        assert_eq!(ir.instruction_count(), 2);
        assert_eq!(ir.constants.len(), 1);
        assert_eq!(ir.register_count, 5);
    }

    #[test]
    fn test_ir_function_emit() {
        let mut ir = IRFunction::new();
        ir.emit(IROpcode::LoadConst(0), 0);
        ir.emit(IROpcode::Return, 1);

        assert_eq!(ir.instruction_count(), 2);
        assert_eq!(ir.instructions[0].opcode, IROpcode::LoadConst(0));
        assert_eq!(ir.instructions[0].bytecode_offset, 0);
    }

    #[test]
    fn test_ir_instruction_creation() {
        let inst = IRInstruction::new(IROpcode::Add(Some(TypeInfo::Number)), 5);
        assert_eq!(inst.opcode, IROpcode::Add(Some(TypeInfo::Number)));
        assert_eq!(inst.bytecode_offset, 5);
    }

    #[test]
    fn test_ir_opcode_type_specialization() {
        let add_generic = IROpcode::Add(None);
        let add_number = IROpcode::Add(Some(TypeInfo::Number));

        assert_ne!(add_generic, add_number);
    }
}
