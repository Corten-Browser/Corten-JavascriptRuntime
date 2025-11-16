//! Bytecode instruction representation
//!
//! Contains instruction structure and source position tracking.

use crate::opcode::Opcode;

/// Source position for debugging information
///
/// Placeholder for core_types::SourcePosition dependency.
/// Will be replaced when core_types is integrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// Byte offset from start of source
    pub offset: u32,
}

impl SourcePosition {
    /// Create a new source position
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// A single bytecode instruction with optional source mapping
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// The opcode for this instruction
    pub opcode: Opcode,
    /// Optional source position for debugging
    pub source_position: Option<SourcePosition>,
}

impl Instruction {
    /// Create a new instruction without source position
    pub fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            source_position: None,
        }
    }

    /// Create a new instruction with source position
    pub fn with_position(opcode: Opcode, position: SourcePosition) -> Self {
        Self {
            opcode,
            source_position: Some(position),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_position_new() {
        let pos = SourcePosition::new(10, 20, 100);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 20);
        assert_eq!(pos.offset, 100);
    }

    #[test]
    fn test_instruction_new() {
        let inst = Instruction::new(Opcode::Add);
        assert!(matches!(inst.opcode, Opcode::Add));
        assert!(inst.source_position.is_none());
    }

    #[test]
    fn test_instruction_with_position() {
        let pos = SourcePosition::new(1, 1, 0);
        let inst = Instruction::with_position(Opcode::Sub, pos);
        assert!(matches!(inst.opcode, Opcode::Sub));
        assert!(inst.source_position.is_some());
    }
}
