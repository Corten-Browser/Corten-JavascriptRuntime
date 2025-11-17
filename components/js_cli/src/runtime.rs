//! Runtime orchestration for JavaScript execution
//!
//! The Runtime struct coordinates all components:
//! - Parser and BytecodeGenerator for parsing
//! - VM for execution
//! - JIT compilers for optimization
//! - EventLoop for async operations
//! - Builtins for standard library

use crate::error::{CliError, CliResult};
use async_runtime::EventLoop;
use core_types::Value;

/// Main runtime that orchestrates all JavaScript execution components
pub struct Runtime {
    /// Whether JIT compilation is enabled
    enable_jit: bool,
    /// Whether to print bytecode before execution
    print_bytecode: bool,
    /// Whether to print AST before execution
    print_ast: bool,
    /// Event loop for async operations
    event_loop: EventLoop,
    /// Persistent VM instance for maintaining state
    vm: interpreter::VM,
}

impl Runtime {
    /// Create a new runtime instance
    ///
    /// # Arguments
    /// * `enable_jit` - Whether to enable JIT compilation
    ///
    /// # Example
    /// ```
    /// use js_cli::Runtime;
    ///
    /// let runtime = Runtime::new(true);
    /// ```
    pub fn new(enable_jit: bool) -> Self {
        Self {
            enable_jit,
            print_bytecode: false,
            print_ast: false,
            event_loop: EventLoop::new(),
            vm: interpreter::VM::new(),
        }
    }

    /// Enable bytecode printing
    pub fn with_print_bytecode(mut self, enabled: bool) -> Self {
        self.print_bytecode = enabled;
        self
    }

    /// Enable AST printing
    pub fn with_print_ast(mut self, enabled: bool) -> Self {
        self.print_ast = enabled;
        self
    }

    /// Execute a JavaScript file
    ///
    /// # Arguments
    /// * `path` - Path to the JavaScript file
    ///
    /// # Returns
    /// The result value from executing the file
    ///
    /// # Errors
    /// Returns `CliError` if file cannot be read or execution fails
    ///
    /// # Example
    /// ```no_run
    /// use js_cli::Runtime;
    ///
    /// let mut runtime = Runtime::new(true);
    /// let result = runtime.execute_file("example.js").unwrap();
    /// ```
    pub fn execute_file(&mut self, path: &str) -> CliResult<Value> {
        // Read file content
        let source = std::fs::read_to_string(path)?;

        // Execute the source code
        self.execute_string(&source)
    }

    /// Execute a JavaScript source string
    ///
    /// # Arguments
    /// * `source` - JavaScript source code
    ///
    /// # Returns
    /// The result value from executing the code
    ///
    /// # Errors
    /// Returns `CliError` if parsing or execution fails
    ///
    /// # Example
    /// ```
    /// use js_cli::Runtime;
    /// use core_types::Value;
    ///
    /// let mut runtime = Runtime::new(false);
    /// let result = runtime.execute_string("let x = 42;").unwrap();
    /// ```
    pub fn execute_string(&mut self, source: &str) -> CliResult<Value> {
        // Parse the source code
        let mut parser = parser::Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| CliError::ParseError(format!("Parse error: {:?}", e)))?;

        // Optionally print AST
        if self.print_ast {
            println!("AST: {:#?}", ast);
        }

        // Generate bytecode
        let mut generator = parser::BytecodeGenerator::new();
        let bytecode = generator
            .generate(&ast)
            .map_err(|e| CliError::ParseError(format!("Bytecode generation error: {:?}", e)))?;

        // Optionally print bytecode
        if self.print_bytecode {
            println!("Bytecode: {:#?}", bytecode);
        }

        // Execute using persistent VM
        let result = self.vm.execute(&bytecode).map_err(CliError::JsError)?;

        // Run event loop to process pending promises and microtasks
        self.event_loop.run_until_done().map_err(CliError::JsError)?;

        Ok(result)
    }

    /// Queue a microtask for execution in the event loop
    ///
    /// # Arguments
    /// * `task` - A closure to execute as a microtask
    pub fn queue_microtask(&mut self, task: impl FnOnce() -> Result<Value, core_types::JsError> + Send + 'static) {
        self.event_loop.enqueue_microtask(async_runtime::MicroTask::new(task));
    }

    /// Get access to the event loop for advanced async operations
    pub fn event_loop(&mut self) -> &mut EventLoop {
        &mut self.event_loop
    }

    /// Get access to the VM for direct manipulation
    pub fn vm(&mut self) -> &mut interpreter::VM {
        &mut self.vm
    }

    /// Start the REPL (Read-Eval-Print Loop)
    ///
    /// # Returns
    /// `Ok(())` when REPL exits normally
    ///
    /// # Errors
    /// Returns `CliError` if REPL encounters a fatal error
    ///
    /// # Example
    /// ```no_run
    /// use js_cli::Runtime;
    ///
    /// let mut runtime = Runtime::new(true);
    /// runtime.repl().unwrap();
    /// ```
    pub fn repl(&mut self) -> CliResult<()> {
        crate::repl::run_repl(self)
    }

    /// Check if JIT is enabled
    pub fn is_jit_enabled(&self) -> bool {
        self.enable_jit
    }

    /// Check if bytecode printing is enabled
    pub fn is_print_bytecode_enabled(&self) -> bool {
        self.print_bytecode
    }

    /// Check if AST printing is enabled
    pub fn is_print_ast_enabled(&self) -> bool {
        self.print_ast
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new(true);
        assert!(runtime.is_jit_enabled());

        let runtime = Runtime::new(false);
        assert!(!runtime.is_jit_enabled());
    }

    #[test]
    fn test_runtime_builder_pattern() {
        let runtime = Runtime::new(true)
            .with_print_bytecode(true)
            .with_print_ast(true);

        assert!(runtime.is_jit_enabled());
        assert!(runtime.is_print_bytecode_enabled());
        assert!(runtime.is_print_ast_enabled());
    }
}
