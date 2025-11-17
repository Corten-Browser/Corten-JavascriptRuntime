//! Virtual Machine for bytecode execution
//!
//! Main entry point for executing JavaScript bytecode.

use bytecode_system::BytecodeChunk;
use core_types::{JsError, Value};
use jit_compiler::{BaselineJIT, CompiledCode, OptimizingJIT};
use std::collections::HashMap;

use crate::call_frame::CallFrame;
use crate::context::ExecutionContext;
use crate::dispatch::Dispatcher;
use crate::gc_integration::VMHeap;
use crate::jit_context::JITContext;
use crate::profile::ProfileData;

/// Virtual Machine for executing JavaScript bytecode
///
/// The VM manages the execution state including:
/// - Global object and variables
/// - Call stack for function invocations
/// - Memory heap (via memory_manager)
/// - Function registry for closures
/// - JIT compilation and hot code detection
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
    /// Compiled code cache (function index -> compiled code)
    compiled_code: HashMap<usize, CompiledCode>,
    /// Profile data per function (for optimizing JIT)
    profile_data: HashMap<usize, ProfileData>,
    /// Baseline JIT compiler (template-based, fast compilation)
    baseline_jit: Option<BaselineJIT>,
    /// Optimizing JIT compiler (speculation-based, slow compilation)
    optimizing_jit: Option<OptimizingJIT>,
    /// Number of calls before baseline JIT compilation
    jit_threshold: u64,
    /// Number of calls before optimizing JIT compilation
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
            .field("compiled_code_count", &self.compiled_code.len())
            .field("jit_threshold", &self.jit_threshold)
            .field("opt_threshold", &self.opt_threshold)
            .finish()
    }
}

impl VM {
    /// Create a new VM instance
    ///
    /// Initializes an empty VM with no global variables.
    /// JIT compilers are initialized but disabled by default (high thresholds).
    pub fn new() -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            call_stack: Vec::with_capacity(64),
            functions: Vec::new(),
            heap: VMHeap::new(),
            execution_counts: HashMap::new(),
            compiled_code: HashMap::new(),
            profile_data: HashMap::new(),
            baseline_jit: BaselineJIT::new().into(),
            optimizing_jit: OptimizingJIT::new().into(),
            jit_threshold: 100,      // Baseline JIT after 100 calls
            opt_threshold: 10000,    // Optimizing JIT after 10,000 calls
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

    /// Execute a registered function by index with JIT awareness
    ///
    /// This method:
    /// 1. Records the call for hot code detection
    /// 2. Checks if compiled code exists
    /// 3. Executes compiled code if available, otherwise interprets
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

        // Check if we have valid compiled code
        if let Some(compiled) = self.compiled_code.get(&func_idx) {
            if compiled.is_valid() {
                // Execute compiled code
                match compiled.execute() {
                    Ok(result) => return Ok(result),
                    Err(_) => {
                        // Deoptimize: fall back to interpreter
                        eprintln!("[JIT] Deoptimizing function {} - falling back to interpreter", func_idx);
                        self.invalidate_compiled(func_idx);
                    }
                }
            }
        }

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

    /// Create a JIT context for tracking function calls during execution
    ///
    /// This is useful for integrating JIT awareness into external execution loops.
    ///
    /// # Example
    /// ```ignore
    /// let mut jit_ctx = vm.create_jit_context();
    /// // ... execute some code that records calls via jit_ctx ...
    /// vm.process_pending_jit(&mut jit_ctx);
    /// ```
    pub fn create_jit_context(&mut self) -> JITContext<'_> {
        JITContext::new(
            &mut self.execution_counts,
            &self.compiled_code,
            &mut self.profile_data,
            self.jit_threshold,
            self.opt_threshold,
        )
    }

    /// Process pending JIT compilation requests from a JIT context
    ///
    /// This method compiles functions that have reached their JIT thresholds.
    pub fn process_pending_jit(&mut self, jit_ctx: &mut JITContext<'_>) {
        let (pending_baseline, pending_opt) = jit_ctx.drain_pending();

        // Process baseline JIT requests
        for func_idx in pending_baseline {
            self.trigger_baseline_jit(func_idx);
        }

        // Process optimizing JIT requests
        for func_idx in pending_opt {
            self.trigger_optimizing_jit(func_idx);
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
    /// Increments the execution count for the given function and triggers
    /// JIT compilation when thresholds are reached.
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function being called
    pub fn record_call(&mut self, func_idx: usize) {
        let count = self.execution_counts.entry(func_idx).or_insert(0);
        *count += 1;

        // Check if we should trigger JIT compilation
        if *count == self.jit_threshold {
            self.trigger_baseline_jit(func_idx);
        } else if *count == self.opt_threshold {
            self.trigger_optimizing_jit(func_idx);
        }
    }

    /// Trigger baseline JIT compilation for a function
    ///
    /// Compiles the function using the baseline (template) JIT compiler.
    /// This provides a modest speedup with fast compilation.
    fn trigger_baseline_jit(&mut self, func_idx: usize) {
        if let Some(ref mut jit) = self.baseline_jit {
            if let Some(chunk) = self.functions.get(func_idx) {
                match jit.compile(chunk) {
                    Ok(compiled) => {
                        eprintln!("[JIT] Baseline compiled function {}", func_idx);
                        self.compiled_code.insert(func_idx, compiled);
                    }
                    Err(e) => {
                        eprintln!("[JIT] Baseline compilation failed for function {}: {}", func_idx, e);
                    }
                }
            }
        }
    }

    /// Trigger optimizing JIT compilation for a function
    ///
    /// Compiles the function using the optimizing (speculative) JIT compiler.
    /// This provides maximum speedup but takes longer to compile.
    fn trigger_optimizing_jit(&mut self, func_idx: usize) {
        if let Some(ref mut jit) = self.optimizing_jit {
            if let Some(chunk) = self.functions.get(func_idx) {
                // Get profile data for this function
                let profile = self
                    .profile_data
                    .get(&func_idx)
                    .cloned()
                    .unwrap_or_default();

                match jit.compile(chunk, &profile) {
                    Ok(compiled) => {
                        eprintln!("[JIT] Optimizing compiled function {}", func_idx);
                        self.compiled_code.insert(func_idx, compiled);
                    }
                    Err(e) => {
                        eprintln!("[JIT] Optimizing compilation failed for function {}: {}", func_idx, e);
                    }
                }
            }
        }
    }

    /// Get compiled code for a function if available
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function
    ///
    /// # Returns
    /// The compiled code if the function has been JIT compiled, None otherwise
    pub fn get_compiled(&self, func_idx: usize) -> Option<&CompiledCode> {
        self.compiled_code.get(&func_idx)
    }

    /// Get mutable reference to compiled code for a function
    ///
    /// Used for invalidating compiled code during deoptimization.
    pub fn get_compiled_mut(&mut self, func_idx: usize) -> Option<&mut CompiledCode> {
        self.compiled_code.get_mut(&func_idx)
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

    /// Get the number of functions that have been JIT compiled
    pub fn compiled_functions_count(&self) -> usize {
        self.compiled_code.len()
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
        self.profile_data.entry(func_idx).or_insert_with(ProfileData::new)
    }

    /// Check if baseline JIT is available
    pub fn is_baseline_jit_available(&self) -> bool {
        self.baseline_jit.as_ref().map_or(false, |jit| jit.is_available())
    }

    /// Check if optimizing JIT is available
    pub fn is_optimizing_jit_available(&self) -> bool {
        self.optimizing_jit.as_ref().map_or(false, |jit| jit.is_available())
    }

    /// Invalidate compiled code for a function (deoptimization)
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function to invalidate
    pub fn invalidate_compiled(&mut self, func_idx: usize) {
        if let Some(compiled) = self.compiled_code.get_mut(&func_idx) {
            compiled.invalidate();
        }
    }

    /// Remove compiled code for a function
    ///
    /// # Arguments
    /// * `func_idx` - The index of the function
    pub fn remove_compiled(&mut self, func_idx: usize) -> Option<CompiledCode> {
        self.compiled_code.remove(&func_idx)
    }

    /// Get reference to internal functions registry
    pub fn functions(&self) -> &[BytecodeChunk] {
        &self.functions
    }

    /// Get a reference to the GC heap
    ///
    /// The heap manages JavaScript object allocation and garbage collection.
    pub fn heap(&self) -> &VMHeap {
        &self.heap
    }

    /// Get a mutable reference to the GC heap
    ///
    /// The heap manages JavaScript object allocation and garbage collection.
    pub fn heap_mut(&mut self) -> &mut VMHeap {
        &mut self.heap
    }

    /// Trigger garbage collection
    ///
    /// Performs a young generation collection.
    pub fn collect_garbage(&self) {
        self.heap.collect_garbage();
    }

    /// Trigger a full garbage collection
    ///
    /// Collects both young and old generations.
    pub fn full_gc(&self) {
        self.heap.full_gc();
    }

    /// Get GC statistics
    pub fn gc_stats(&self) -> memory_manager::GcStats {
        self.heap.gc_stats()
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
        assert_eq!(vm.compiled_functions_count(), 0);
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
        // Disable JIT for this test by setting very high thresholds
        vm.set_jit_threshold(u64::MAX);
        vm.set_opt_threshold(u64::MAX);

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
        vm.set_jit_threshold(u64::MAX);

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
    fn test_vm_jit_availability() {
        let vm = VM::new();
        // JIT compilers should be available if Cranelift is working
        assert!(vm.is_baseline_jit_available());
        assert!(vm.is_optimizing_jit_available());
    }

    #[test]
    fn test_vm_get_compiled_not_compiled() {
        let vm = VM::new();
        assert!(vm.get_compiled(0).is_none());
        assert!(vm.get_compiled(999).is_none());
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
    fn test_vm_jit_trigger_at_threshold() {
        let mut vm = VM::new();
        vm.set_jit_threshold(5);

        // Register a simple function
        let mut chunk = BytecodeChunk::new();
        let const_idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(const_idx));
        chunk.emit(Opcode::Return);
        vm.register_function(chunk);

        // Call 4 times - no JIT yet
        for _ in 0..4 {
            vm.record_call(0);
        }
        assert!(vm.get_compiled(0).is_none());

        // 5th call triggers JIT
        vm.record_call(0);
        assert!(vm.get_compiled(0).is_some());
        assert_eq!(vm.compiled_functions_count(), 1);
    }

    #[test]
    fn test_vm_optimizing_jit_trigger() {
        let mut vm = VM::new();
        vm.set_jit_threshold(5);
        vm.set_opt_threshold(10);

        // Register a simple function
        let mut chunk = BytecodeChunk::new();
        let const_idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(const_idx));
        chunk.emit(Opcode::Return);
        vm.register_function(chunk);

        // Call up to baseline threshold
        for _ in 0..5 {
            vm.record_call(0);
        }
        assert!(vm.get_compiled(0).is_some());

        // Continue to optimizing threshold
        for _ in 5..10 {
            vm.record_call(0);
        }
        // Should have recompiled with optimizing JIT
        assert!(vm.get_compiled(0).is_some());
    }

    #[test]
    fn test_vm_invalidate_compiled() {
        let mut vm = VM::new();
        vm.set_jit_threshold(1);

        // Register and compile a function
        let mut chunk = BytecodeChunk::new();
        let const_idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(const_idx));
        chunk.emit(Opcode::Return);
        vm.register_function(chunk);

        vm.record_call(0);
        assert!(vm.get_compiled(0).is_some());

        // Invalidate
        vm.invalidate_compiled(0);
        let compiled = vm.get_compiled(0).unwrap();
        assert!(!compiled.is_valid());
    }

    #[test]
    fn test_vm_remove_compiled() {
        let mut vm = VM::new();
        vm.set_jit_threshold(1);

        // Register and compile a function
        let mut chunk = BytecodeChunk::new();
        let const_idx = chunk.add_constant(bytecode_system::Value::Number(42.0));
        chunk.emit(Opcode::LoadConstant(const_idx));
        chunk.emit(Opcode::Return);
        vm.register_function(chunk);

        vm.record_call(0);
        assert!(vm.get_compiled(0).is_some());

        // Remove
        let removed = vm.remove_compiled(0);
        assert!(removed.is_some());
        assert!(vm.get_compiled(0).is_none());
    }
}
