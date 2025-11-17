//! Virtual Machine for bytecode execution
//!
//! Main entry point for executing JavaScript bytecode.

use bytecode_system::BytecodeChunk;
use core_types::{JsError, Value};

use crate::call_frame::CallFrame;
use crate::context::ExecutionContext;
use crate::dispatch::Dispatcher;

/// Virtual Machine for executing JavaScript bytecode
///
/// The VM manages the execution state including:
/// - Global object and variables
/// - Call stack for function invocations
/// - Memory heap (via memory_manager)
/// - Function registry for closures
#[derive(Debug)]
pub struct VM {
    /// Dispatcher for bytecode execution
    dispatcher: Dispatcher,
    /// Call stack for function invocations
    call_stack: Vec<CallFrame>,
    /// Registry of function bytecode chunks
    functions: Vec<BytecodeChunk>,
}

impl VM {
    /// Create a new VM instance
    ///
    /// Initializes an empty VM with no global variables.
    pub fn new() -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            call_stack: Vec::with_capacity(64),
            functions: Vec::new(),
        }
    }

    /// Register a function bytecode chunk and return its index
    ///
    /// # Arguments
    ///
    /// * `chunk` - The bytecode chunk for the function body
    ///
    /// # Returns
    ///
    /// The index (function ID) that can be used with CreateClosure
    pub fn register_function(&mut self, chunk: BytecodeChunk) -> usize {
        let idx = self.functions.len();
        self.functions.push(chunk);
        idx
    }

    /// Execute a bytecode chunk and return the result
    ///
    /// # Arguments
    ///
    /// * `chunk` - The bytecode chunk to execute
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The return value of the execution
    /// * `Err(JsError)` - If an error occurs during execution
    ///
    /// # Example
    ///
    /// ```
    /// use interpreter::VM;
    /// use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
    /// use core_types::Value;
    ///
    /// let mut vm = VM::new();
    /// let mut chunk = BytecodeChunk::new();
    ///
    /// let idx = chunk.add_constant(BcValue::Number(42.0));
    /// chunk.emit(Opcode::LoadConstant(idx));
    /// chunk.emit(Opcode::Return);
    ///
    /// let result = vm.execute(&chunk).unwrap();
    /// assert_eq!(result, Value::Smi(42));
    /// ```
    pub fn execute(&mut self, chunk: &BytecodeChunk) -> Result<Value, JsError> {
        let mut ctx = ExecutionContext::new(chunk.clone());
        self.dispatcher.execute(&mut ctx, &self.functions)
    }

    /// Get a global variable by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the global variable
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The value if the global exists
    /// * `None` - If the global does not exist
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.dispatcher.get_global(name)
    }

    /// Set a global variable
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the global variable
    /// * `value` - The value to set
    pub fn set_global(&mut self, name: String, value: Value) {
        self.dispatcher.set_global(name, value);
    }

    /// Get the current call stack depth
    pub fn call_stack_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Push a call frame onto the stack
    pub fn push_call_frame(&mut self, frame: CallFrame) {
        self.call_stack.push(frame);
    }

    /// Pop a call frame from the stack
    pub fn pop_call_frame(&mut self) -> Option<CallFrame> {
        self.call_stack.pop()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::Opcode;

    #[test]
    fn test_vm_new() {
        let vm = VM::new();
        assert_eq!(vm.call_stack_depth(), 0);
    }

    #[test]
    fn test_vm_default() {
        let vm = VM::default();
        assert_eq!(vm.call_stack_depth(), 0);
    }

    #[test]
    fn test_vm_globals() {
        let mut vm = VM::new();
        vm.set_global("test".to_string(), Value::Smi(100));
        assert_eq!(vm.get_global("test"), Some(Value::Smi(100)));
        assert_eq!(vm.get_global("nonexistent"), None);
    }

    #[test]
    fn test_vm_call_stack() {
        let mut vm = VM::new();

        let frame = CallFrame::new(10, 0, 1);
        vm.push_call_frame(frame.clone());

        assert_eq!(vm.call_stack_depth(), 1);

        let popped = vm.pop_call_frame();
        assert_eq!(popped, Some(frame));
        assert_eq!(vm.call_stack_depth(), 0);
    }

    #[test]
    fn test_vm_execute_simple() {
        let mut vm = VM::new();
        let mut chunk = BytecodeChunk::new();

        chunk.emit(Opcode::LoadTrue);
        chunk.emit(Opcode::Return);

        let result = vm.execute(&chunk);
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }
}
