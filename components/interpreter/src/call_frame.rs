//! Call frame for function call stack management

/// Call frame representing a function invocation
///
/// Stored on the call stack to track function execution state.
#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    /// Instruction pointer to return to after function completes
    pub return_address: usize,
    /// Base register index for this frame's locals
    pub base_register: usize,
    /// Function identifier (index into function table)
    pub function_id: usize,
}

impl CallFrame {
    /// Create a new call frame
    pub fn new(return_address: usize, base_register: usize, function_id: usize) -> Self {
        Self {
            return_address,
            base_register,
            function_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_frame_new() {
        let frame = CallFrame::new(10, 5, 2);
        assert_eq!(frame.return_address, 10);
        assert_eq!(frame.base_register, 5);
        assert_eq!(frame.function_id, 2);
    }
}
