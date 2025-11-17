//! Bytecode opcodes for JavaScript runtime
//!
//! Defines all bytecode instructions for the register-based VM.

/// Register identifier for local variable slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterId(pub u32);

/// Descriptor for a captured variable (upvalue)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpvalueDescriptor {
    /// true if directly in parent scope, false if in grandparent+
    pub is_local: bool,
    /// Register index (if local) or upvalue index (if not)
    pub index: u32,
}

impl UpvalueDescriptor {
    /// Create a new upvalue descriptor
    pub fn new(is_local: bool, index: u32) -> Self {
        Self { is_local, index }
    }
}

/// Bytecode opcodes for JavaScript execution
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    // Literals
    /// Load constant from constant pool at given index
    LoadConstant(usize),
    /// Load undefined value
    LoadUndefined,
    /// Load null value
    LoadNull,
    /// Load boolean true
    LoadTrue,
    /// Load boolean false
    LoadFalse,

    // Variables
    /// Load global variable by name
    LoadGlobal(String),
    /// Store to global variable by name
    StoreGlobal(String),
    /// Load local variable from register
    LoadLocal(RegisterId),
    /// Store to local variable in register
    StoreLocal(RegisterId),

    // Upvalue operations for closures
    /// Load captured variable by upvalue index
    LoadUpvalue(u32),
    /// Store to captured variable
    StoreUpvalue(u32),
    /// Close over local variable (move from stack to heap)
    CloseUpvalue,

    // Arithmetic operations
    /// Add top two stack values
    Add,
    /// Subtract top from second-top
    Sub,
    /// Multiply top two stack values
    Mul,
    /// Divide second-top by top
    Div,
    /// Modulo second-top by top
    Mod,
    /// Negate top value
    Neg,

    // Comparison operations
    /// Loose equality (==)
    Equal,
    /// Strict equality (===)
    StrictEqual,
    /// Loose inequality (!=)
    NotEqual,
    /// Strict inequality (!==)
    StrictNotEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanEqual,

    // Control flow
    /// Unconditional jump to offset
    Jump(usize),
    /// Jump to offset if top of stack is truthy
    JumpIfTrue(usize),
    /// Jump to offset if top of stack is falsy
    JumpIfFalse(usize),
    /// Return from current function
    Return,

    // Object operations
    /// Create new empty object
    CreateObject,
    /// Load property from object
    LoadProperty(String),
    /// Store property to object
    StoreProperty(String),

    // Function operations
    /// Create closure from function at index with captured variables
    CreateClosure(usize, Vec<UpvalueDescriptor>),
    /// Call function with given number of arguments
    Call(u8),
}

impl Opcode {
    /// Check if this opcode is a terminator (ends basic block)
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Opcode::Return | Opcode::Jump(_) | Opcode::JumpIfTrue(_) | Opcode::JumpIfFalse(_)
        )
    }

    /// Check if this opcode is an unconditional terminator
    pub fn is_unconditional_terminator(&self) -> bool {
        matches!(self, Opcode::Return | Opcode::Jump(_))
    }

    /// Check if this opcode is a binary arithmetic operation
    pub fn is_binary_arithmetic(&self) -> bool {
        matches!(
            self,
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_is_terminator() {
        assert!(Opcode::Return.is_terminator());
        assert!(Opcode::Jump(0).is_terminator());
        assert!(Opcode::JumpIfTrue(0).is_terminator());
        assert!(Opcode::JumpIfFalse(0).is_terminator());
        assert!(!Opcode::Add.is_terminator());
    }

    #[test]
    fn test_opcode_is_unconditional_terminator() {
        assert!(Opcode::Return.is_unconditional_terminator());
        assert!(Opcode::Jump(0).is_unconditional_terminator());
        assert!(!Opcode::JumpIfTrue(0).is_unconditional_terminator());
    }

    #[test]
    fn test_opcode_is_binary_arithmetic() {
        assert!(Opcode::Add.is_binary_arithmetic());
        assert!(Opcode::Sub.is_binary_arithmetic());
        assert!(Opcode::Mul.is_binary_arithmetic());
        assert!(Opcode::Div.is_binary_arithmetic());
        assert!(Opcode::Mod.is_binary_arithmetic());
        assert!(!Opcode::Neg.is_binary_arithmetic());
    }
}
