//! Virtual Machine for bytecode execution
//!
//! Main entry point for executing JavaScript bytecode.

use bytecode_system::BytecodeChunk;
use core_types::{JsError, Value};
use std::collections::HashMap;

use crate::call_frame::CallFrame;
use crate::context::ExecutionContext;
use crate::dispatch::Dispatcher;
use crate::gc_integration::VMHeap;
use crate::profile::ProfileData;

/// Virtual Machine for executing JavaScript bytecode
///
/// The VM manages the execution state including:
/// - Global object and variables
/// - Call stack for function invocations
/// - Memory heap (via memory_manager)
/// - Function registry for closures
/// - Execution counting for hot code detection
/// - Profile data collection for JIT optimization decisions
///
/// Note: JIT compilation is coordinated at the Runtime level (js_cli)
/// to avoid cyclic dependencies. The VM provides profiling hooks.
pub struct VM {
    /// Dispatcher for bytecode execution
    dispatcher: Dispatcher,
    /// Call stack for function invocations
    call_stack: Vec<CallFrame>,
    /// Registry of function bytecode chunks
    functions: Vec<BytecodeChunk>,
    /// GC-managed heap for JavaScript objects
    heap: VMHeap,
    /// Execution counts per function index (for hot code detection)
    execution_counts: HashMap<usize, u64>,
    /// Profile data per function (for optimizing JIT)
    profile_data: HashMap<usize, ProfileData>,
    /// Number of calls before baseline JIT compilation should be considered
    jit_threshold: u64,
    /// Number of calls before optimizing JIT compilation should be considered
    opt_threshold: u64,
}

impl std::fmt::Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VM")
            .field("dispatcher", &self.dispatcher)
            .field("call_stack", &self.call_stack)
            .field("functions_count", &self.functions.len())
            .field("heap", &self.heap)
            .field("execution_counts", &self.execution_counts)
            .field("jit_threshold", &self.jit_threshold)
            .field("opt_threshold", &self.opt_threshold)
            .finish()
    }
}

impl VM {
    /// Create a new VM instance
    ///
    /// Initializes an empty VM with no global variables.
    pub fn new() -> Self {
        let heap = VMHeap::new();
        let heap_rc = std::rc::Rc::new(heap);
        let mut dispatcher = Dispatcher::new();
        dispatcher.set_heap(heap_rc.clone());

        Self {
            dispatcher,
            call_stack: Vec::with_capacity(64),
            functions: Vec::new(),
            heap: VMHeap::new(), // TODO: Should use heap_rc, but VMHeap is not Clone
            execution_counts: HashMap::new(),
            profile_data: HashMap::new(),
            jit_threshold: 100,   // Baseline JIT after 100 calls
            opt_threshold: 10000, // Optimizing JIT after 10,000 calls
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
        // Register any nested functions from this chunk
        // This allows CreateClosure to find the function bytecode
        let base_idx = self.functions.len();
        for nested_fn in chunk.nested_functions() {
            self.functions.push(nested_fn.clone());
        }

        // Adjust closure indices in the chunk if needed
        let mut adjusted_chunk = chunk.clone();
        if base_idx > 0 && !chunk.nested_functions().is_empty() {
            // Adjust CreateClosure indices to account for existing functions
            for inst in &mut adjusted_chunk.instructions {
                match &mut inst.opcode {
                    bytecode_system::Opcode::CreateClosure(idx, _) => {
                        *idx = *idx + base_idx;
                    }
                    bytecode_system::Opcode::CreateAsyncFunction(idx, _) => {
                        *idx = *idx + base_idx;
                    }
                    _ => {}
                }
            }
        }

        let mut ctx = ExecutionContext::new(adjusted_chunk);
        self.dispatcher.execute(&mut ctx, &self.functions)
    }

    /// Execute a registered function by index
    ///
    /// This method:
    /// 1. Records the call for hot code detection
    /// 2. Executes the function in the interpreter
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function to execute
    ///
    /// # Returns
    /// * `Ok(Value)` - The result of execution
    /// * `Err(JsError)` - If execution fails
    pub fn execute_function(&mut self, func_idx: usize) -> Result<Value, JsError> {
        // Record the call for hot code detection
        self.record_call(func_idx);

        // Execute in interpreter
        if let Some(chunk) = self.functions.get(func_idx) {
            let chunk = chunk.clone();
            self.execute(&chunk)
        } else {
            Err(JsError {
                kind: core_types::ErrorKind::ReferenceError,
                message: format!("Function index {} not found", func_idx),
                stack: vec![],
                source_position: None,
            })
        }
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

    /// Record a function call for hot code detection
    ///
    /// Increments the execution count for the given function.
    /// External code (e.g., Runtime) can query this to decide when to trigger JIT.
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function being called
    pub fn record_call(&mut self, func_idx: usize) {
        let count = self.execution_counts.entry(func_idx).or_insert(0);
        *count += 1;
    }

    /// Check if a function should be baseline JIT compiled
    ///
    /// Returns true if the function has reached the baseline JIT threshold.
    pub fn should_baseline_compile(&self, func_idx: usize) -> bool {
        self.execution_counts
            .get(&func_idx)
            .map_or(false, |&count| count >= self.jit_threshold)
    }

    /// Check if a function should be optimizing JIT compiled
    ///
    /// Returns true if the function has reached the optimizing JIT threshold.
    pub fn should_optimizing_compile(&self, func_idx: usize) -> bool {
        self.execution_counts
            .get(&func_idx)
            .map_or(false, |&count| count >= self.opt_threshold)
    }

    /// Get functions that are hot and should be baseline compiled
    ///
    /// Returns a list of function indices that have reached the baseline threshold.
    pub fn get_hot_functions(&self) -> Vec<usize> {
        self.execution_counts
            .iter()
            .filter_map(|(&idx, &count)| {
                if count >= self.jit_threshold {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get functions that should be optimizing compiled
    ///
    /// Returns a list of function indices that have reached the optimizing threshold.
    pub fn get_very_hot_functions(&self) -> Vec<usize> {
        self.execution_counts
            .iter()
            .filter_map(|(&idx, &count)| {
                if count >= self.opt_threshold {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Set the baseline JIT threshold
    ///
    /// # Arguments
    /// * `threshold` - Number of calls before baseline JIT compilation
    pub fn set_jit_threshold(&mut self, threshold: u64) {
        self.jit_threshold = threshold;
    }

    /// Set the optimizing JIT threshold
    ///
    /// # Arguments
    /// * `threshold` - Number of calls before optimizing JIT compilation
    pub fn set_opt_threshold(&mut self, threshold: u64) {
        self.opt_threshold = threshold;
    }

    /// Get the current baseline JIT threshold
    pub fn jit_threshold(&self) -> u64 {
        self.jit_threshold
    }

    /// Get the current optimizing JIT threshold
    pub fn opt_threshold(&self) -> u64 {
        self.opt_threshold
    }

    /// Get execution count for a function
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function
    ///
    /// # Returns
    /// The number of times the function has been called
    pub fn get_execution_count(&self, func_idx: usize) -> u64 {
        self.execution_counts.get(&func_idx).copied().unwrap_or(0)
    }

    /// Get all execution counts
    pub fn execution_counts(&self) -> &HashMap<usize, u64> {
        &self.execution_counts
    }

    /// Record profile data for a function
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function
    /// * `profile` - The profile data to record
    pub fn record_profile_data(&mut self, func_idx: usize, profile: ProfileData) {
        self.profile_data.insert(func_idx, profile);
    }

    /// Get profile data for a function
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function
    ///
    /// # Returns
    /// The profile data if available
    pub fn get_profile_data(&self, func_idx: usize) -> Option<&ProfileData> {
        self.profile_data.get(&func_idx)
    }

    /// Get mutable profile data for a function, creating if necessary
    pub fn get_or_create_profile_data(&mut self, func_idx: usize) -> &mut ProfileData {
        self.profile_data
            .entry(func_idx)
            .or_insert_with(ProfileData::new)
    }

    /// Get reference to internal functions registry
    pub fn functions(&self) -> &[BytecodeChunk] {
        &self.functions
    }

    /// Get a specific function bytecode by index
    pub fn get_function(&self, func_idx: usize) -> Option<&BytecodeChunk> {
        self.functions.get(func_idx)
    }

    /// Reset execution counts (useful for testing)
    pub fn reset_execution_counts(&mut self) {
        self.execution_counts.clear();
    }

    /// Reset profile data (useful for testing)
    pub fn reset_profile_data(&mut self) {
        self.profile_data.clear();
    }

    /// Get reference to the GC heap
    pub fn heap(&self) -> &VMHeap {
        &self.heap
    }

    /// Get mutable reference to the GC heap
    pub fn heap_mut(&mut self) -> &mut VMHeap {
        &mut self.heap
    }

    /// Get GC statistics
    pub fn gc_stats(&self) -> memory_manager::GcStats {
        self.heap.gc_stats()
    }

    /// Trigger garbage collection
    pub fn collect_garbage(&mut self) {
        self.heap.collect_garbage();
    }

    /// Trigger full garbage collection
    pub fn full_gc(&mut self) {
        self.heap.full_gc();
    }

    /// Get the number of compiled functions
    ///
    /// Returns the count of functions that have been JIT compiled.
    /// Note: This is a stub for now - actual JIT compilation tracking
    /// would be done at the Runtime level.
    pub fn compiled_functions_count(&self) -> usize {
        // In the current architecture, JIT compilation is coordinated at the Runtime level
        // The VM provides profiling data but doesn't track compiled functions directly
        0
    }

    /// Check if baseline JIT is available
    ///
    /// Returns true if the baseline JIT compiler can be used.
    pub fn is_baseline_jit_available(&self) -> bool {
        // Baseline JIT is theoretically available if jit_compiler is linked
        // In practice, this depends on the runtime configuration
        true
    }

    /// Check if optimizing JIT is available
    ///
    /// Returns true if the optimizing JIT compiler can be used.
    pub fn is_optimizing_jit_available(&self) -> bool {
        // Optimizing JIT is theoretically available if jit_compiler is linked
        // In practice, this depends on the runtime configuration
        true
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
        assert_eq!(vm.jit_threshold(), 100);
        assert_eq!(vm.opt_threshold(), 10000);
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

    #[test]
    fn test_vm_jit_threshold_setting() {
        let mut vm = VM::new();
        assert_eq!(vm.jit_threshold(), 100);

        vm.set_jit_threshold(50);
        assert_eq!(vm.jit_threshold(), 50);

        vm.set_opt_threshold(5000);
        assert_eq!(vm.opt_threshold(), 5000);
    }

    #[test]
    fn test_vm_record_call_increments_count() {
        let mut vm = VM::new();

        assert_eq!(vm.get_execution_count(0), 0);

        vm.record_call(0);
        assert_eq!(vm.get_execution_count(0), 1);

        vm.record_call(0);
        assert_eq!(vm.get_execution_count(0), 2);

        vm.record_call(1);
        assert_eq!(vm.get_execution_count(1), 1);
        assert_eq!(vm.get_execution_count(0), 2);
    }

    #[test]
    fn test_vm_execution_counts_map() {
        let mut vm = VM::new();

        vm.record_call(0);
        vm.record_call(0);
        vm.record_call(1);
        vm.record_call(2);
        vm.record_call(2);
        vm.record_call(2);

        let counts = vm.execution_counts();
        assert_eq!(counts.get(&0), Some(&2));
        assert_eq!(counts.get(&1), Some(&1));
        assert_eq!(counts.get(&2), Some(&3));
    }

    #[test]
    fn test_vm_should_baseline_compile() {
        let mut vm = VM::new();
        vm.set_jit_threshold(5);

        // Not hot yet
        for _ in 0..4 {
            vm.record_call(0);
        }
        assert!(!vm.should_baseline_compile(0));

        // Now hot
        vm.record_call(0);
        assert!(vm.should_baseline_compile(0));
    }

    #[test]
    fn test_vm_should_optimizing_compile() {
        let mut vm = VM::new();
        vm.set_opt_threshold(10);

        // Not hot enough
        for _ in 0..9 {
            vm.record_call(0);
        }
        assert!(!vm.should_optimizing_compile(0));

        // Now hot enough
        vm.record_call(0);
        assert!(vm.should_optimizing_compile(0));
    }

    #[test]
    fn test_vm_get_hot_functions() {
        let mut vm = VM::new();
        vm.set_jit_threshold(5);

        // Make function 0 hot
        for _ in 0..5 {
            vm.record_call(0);
        }
        // Function 1 is not hot
        for _ in 0..3 {
            vm.record_call(1);
        }
        // Function 2 is hot
        for _ in 0..10 {
            vm.record_call(2);
        }

        let hot = vm.get_hot_functions();
        assert!(hot.contains(&0));
        assert!(!hot.contains(&1));
        assert!(hot.contains(&2));
    }

    #[test]
    fn test_vm_get_very_hot_functions() {
        let mut vm = VM::new();
        vm.set_jit_threshold(5);
        vm.set_opt_threshold(10);

        // Function 0 is baseline hot only
        for _ in 0..5 {
            vm.record_call(0);
        }
        // Function 1 is optimizing hot
        for _ in 0..15 {
            vm.record_call(1);
        }

        let very_hot = vm.get_very_hot_functions();
        assert!(!very_hot.contains(&0));
        assert!(very_hot.contains(&1));
    }

    #[test]
    fn test_vm_profile_data() {
        let mut vm = VM::new();

        assert!(vm.get_profile_data(0).is_none());

        let profile = ProfileData::new();
        vm.record_profile_data(0, profile);

        assert!(vm.get_profile_data(0).is_some());
    }

    #[test]
    fn test_vm_get_or_create_profile_data() {
        let mut vm = VM::new();

        // Should create new profile data
        let profile = vm.get_or_create_profile_data(0);
        profile.record_execution();

        // Should return existing profile data
        let same_profile = vm.get_or_create_profile_data(0);
        assert_eq!(same_profile.execution_count, 1);
    }

    #[test]
    fn test_vm_register_function() {
        let mut vm = VM::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);

        let idx = vm.register_function(chunk);
        assert_eq!(idx, 0);
        assert_eq!(vm.functions().len(), 1);

        let mut chunk2 = BytecodeChunk::new();
        chunk2.emit(Opcode::LoadTrue);
        chunk2.emit(Opcode::Return);

        let idx2 = vm.register_function(chunk2);
        assert_eq!(idx2, 1);
        assert_eq!(vm.functions().len(), 2);
    }

    #[test]
    fn test_vm_get_function() {
        let mut vm = VM::new();
        let mut chunk = BytecodeChunk::new();
        chunk.emit(Opcode::Return);
        vm.register_function(chunk);

        assert!(vm.get_function(0).is_some());
        assert!(vm.get_function(1).is_none());
    }

    #[test]
    fn test_vm_reset_execution_counts() {
        let mut vm = VM::new();
        vm.record_call(0);
        vm.record_call(1);

        vm.reset_execution_counts();
        assert_eq!(vm.get_execution_count(0), 0);
        assert_eq!(vm.get_execution_count(1), 0);
        assert!(vm.execution_counts().is_empty());
    }

    #[test]
    fn test_vm_reset_profile_data() {
        let mut vm = VM::new();
        vm.get_or_create_profile_data(0);
        vm.get_or_create_profile_data(1);

        vm.reset_profile_data();
        assert!(vm.get_profile_data(0).is_none());
        assert!(vm.get_profile_data(1).is_none());
    }

    #[test]
    fn test_vm_execute_function() {
        let mut vm = VM::new();
        let mut chunk = BytecodeChunk::new();

        let const_idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(const_idx));
        chunk.emit(Opcode::Return);

        let func_idx = vm.register_function(chunk);

        let result = vm.execute_function(func_idx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Smi(42));

        // Should have recorded the call
        assert_eq!(vm.get_execution_count(func_idx), 1);
    }

    #[test]
    fn test_vm_execute_function_not_found() {
        let mut vm = VM::new();

        let result = vm.execute_function(999);
        assert!(result.is_err());
    }
}
