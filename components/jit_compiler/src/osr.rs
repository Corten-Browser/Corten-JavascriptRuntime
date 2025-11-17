//! On-Stack Replacement (OSR) support
//!
//! OSR allows transitioning between execution tiers while code is running:
//! - Enter compiled code from running interpreter
//! - Exit compiled code back to interpreter
//! - Frame reconstruction at transition points

use crate::deopt::InterpreterState;
use core_types::{ErrorKind, JsError};

/// Frame mapping for OSR transitions
///
/// Maps registers and stack slots between interpreter and compiled code frames.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameMapping {
    /// Mapping of interpreter registers to native locations
    pub register_map: Vec<RegisterLocation>,
    /// Stack frame size in the native code
    pub native_frame_size: usize,
    /// Interpreter frame size
    pub interpreter_frame_size: usize,
}

/// Location of a register in native code
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterLocation {
    /// Register is in a CPU register (register number)
    Register(u8),
    /// Register is on the stack (offset from frame pointer)
    Stack(i32),
    /// Register value is a constant
    Constant(i64),
}

impl FrameMapping {
    /// Create a new frame mapping
    pub fn new() -> Self {
        Self {
            register_map: Vec::new(),
            native_frame_size: 0,
            interpreter_frame_size: 0,
        }
    }

    /// Add a register mapping
    pub fn add_register(&mut self, location: RegisterLocation) {
        self.register_map.push(location);
    }

    /// Set the native frame size
    pub fn set_native_frame_size(&mut self, size: usize) {
        self.native_frame_size = size;
    }

    /// Set the interpreter frame size
    pub fn set_interpreter_frame_size(&mut self, size: usize) {
        self.interpreter_frame_size = size;
    }
}

impl Default for FrameMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// On-Stack Replacement entry point
///
/// Represents a location where execution can transition from interpreter
/// to compiled code, or vice versa.
#[derive(Debug, Clone, PartialEq)]
pub struct OSREntry {
    /// Offset in bytecode where OSR can occur
    pub bytecode_offset: usize,
    /// Offset in native code for this entry point
    pub native_offset: usize,
    /// Frame mapping for this transition point
    pub frame_mapping: FrameMapping,
}

impl OSREntry {
    /// Create a new OSR entry
    pub fn new(bytecode_offset: usize, native_offset: usize) -> Self {
        Self {
            bytecode_offset,
            native_offset,
            frame_mapping: FrameMapping::new(),
        }
    }

    /// Create OSR entry with custom frame mapping
    pub fn with_mapping(
        bytecode_offset: usize,
        native_offset: usize,
        frame_mapping: FrameMapping,
    ) -> Self {
        Self {
            bytecode_offset,
            native_offset,
            frame_mapping,
        }
    }

    /// Enter compiled code at this OSR point
    ///
    /// Transfers execution from interpreter state to compiled code.
    /// This is a mock implementation that validates the transition.
    pub fn enter_at(&self, state: &InterpreterState) -> Result<(), JsError> {
        // Validate that state matches expected state
        if state.instruction_pointer > self.bytecode_offset {
            return Err(JsError {
                kind: ErrorKind::InternalError,
                message: format!(
                    "OSR entry at {} but state at {}",
                    self.bytecode_offset, state.instruction_pointer
                ),
                stack: vec![],
                source_position: None,
            });
        }

        // Validate frame mapping is compatible
        if !self.frame_mapping.register_map.is_empty()
            && state.registers.len() > self.frame_mapping.register_map.len()
        {
            return Err(JsError {
                kind: ErrorKind::InternalError,
                message: "Frame mapping incomplete for OSR entry".to_string(),
                stack: vec![],
                source_position: None,
            });
        }

        // In a real implementation, this would:
        // 1. Save interpreter state
        // 2. Construct native frame from interpreter registers
        // 3. Jump to native code at native_offset
        // 4. Continue execution in compiled code

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::BytecodeChunk;

    #[test]
    fn test_frame_mapping_new() {
        let mapping = FrameMapping::new();
        assert!(mapping.register_map.is_empty());
        assert_eq!(mapping.native_frame_size, 0);
        assert_eq!(mapping.interpreter_frame_size, 0);
    }

    #[test]
    fn test_frame_mapping_default() {
        let mapping = FrameMapping::default();
        assert!(mapping.register_map.is_empty());
    }

    #[test]
    fn test_frame_mapping_add_register() {
        let mut mapping = FrameMapping::new();
        mapping.add_register(RegisterLocation::Register(0));
        mapping.add_register(RegisterLocation::Stack(-8));
        mapping.add_register(RegisterLocation::Constant(42));

        assert_eq!(mapping.register_map.len(), 3);
        assert_eq!(mapping.register_map[0], RegisterLocation::Register(0));
        assert_eq!(mapping.register_map[1], RegisterLocation::Stack(-8));
        assert_eq!(mapping.register_map[2], RegisterLocation::Constant(42));
    }

    #[test]
    fn test_frame_mapping_set_sizes() {
        let mut mapping = FrameMapping::new();
        mapping.set_native_frame_size(64);
        mapping.set_interpreter_frame_size(32);

        assert_eq!(mapping.native_frame_size, 64);
        assert_eq!(mapping.interpreter_frame_size, 32);
    }

    #[test]
    fn test_osr_entry_new() {
        let entry = OSREntry::new(10, 100);
        assert_eq!(entry.bytecode_offset, 10);
        assert_eq!(entry.native_offset, 100);
        assert!(entry.frame_mapping.register_map.is_empty());
    }

    #[test]
    fn test_osr_entry_with_mapping() {
        let mut mapping = FrameMapping::new();
        mapping.add_register(RegisterLocation::Register(0));
        mapping.set_native_frame_size(32);

        let entry = OSREntry::with_mapping(5, 50, mapping);
        assert_eq!(entry.bytecode_offset, 5);
        assert_eq!(entry.native_offset, 50);
        assert_eq!(entry.frame_mapping.register_map.len(), 1);
        assert_eq!(entry.frame_mapping.native_frame_size, 32);
    }

    #[test]
    fn test_osr_entry_enter_at_success() {
        let entry = OSREntry::new(0, 0);
        let chunk = BytecodeChunk::new();
        let state = InterpreterState::new(chunk);

        let result = entry.enter_at(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_osr_entry_enter_at_invalid_position() {
        let entry = OSREntry::new(0, 0);
        let chunk = BytecodeChunk::new();
        let mut state = InterpreterState::new(chunk);
        state.instruction_pointer = 10; // Past the OSR entry point

        let result = entry.enter_at(&state);
        assert!(result.is_err());
    }
}
