//! Cranelift-based JIT compiler backend
//!
//! Provides real native code generation using Cranelift for JavaScript bytecode.

use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

/// Cranelift-based JIT compiler backend
///
/// Compiles JavaScript bytecode to native machine code using Cranelift.
pub struct CraneliftBackend {
    module: JITModule,
    ctx: Context,
    func_counter: u32,
}

/// Result of compiling a function
#[derive(Debug)]
pub struct CompiledFunction {
    /// Pointer to the compiled native code
    pub code_ptr: *const u8,
    /// Size of the compiled code in bytes
    pub code_size: usize,
}

impl CraneliftBackend {
    /// Create a new Cranelift backend
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| e.to_string())?;

        let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| e.to_string())?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        let ctx = module.make_context();

        Ok(Self {
            module,
            ctx,
            func_counter: 0,
        })
    }

    /// Compile bytecode to native function
    ///
    /// Returns a pointer to the compiled function that can be called.
    /// The function signature is `() -> f64` (returns a number).
    pub fn compile_function(&mut self, chunk: &BytecodeChunk) -> Result<CompiledFunction, String> {
        // Build Cranelift IR from bytecode
        self.build_ir(chunk)?;

        // Generate unique function name
        let func_name = format!("js_func_{}", self.func_counter);
        self.func_counter += 1;

        // Declare function
        let id = self
            .module
            .declare_function(&func_name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // Define function
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Clear context for reuse
        self.module.clear_context(&mut self.ctx);

        // Finalize definitions
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        // Get function pointer
        let code_ptr = self.module.get_finalized_function(id);
        let code_size = 0; // Cranelift doesn't easily expose this

        Ok(CompiledFunction {
            code_ptr,
            code_size,
        })
    }

    fn build_ir(&mut self, chunk: &BytecodeChunk) -> Result<(), String> {
        // Set up function signature: () -> f64
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::F64));
        self.ctx.func.signature = sig;

        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Translate bytecode to Cranelift IR
        let mut value_stack: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let mut has_return = false;

        for instruction in &chunk.instructions {
            match &instruction.opcode {
                Opcode::LoadConstant(idx) => {
                    if let Some(constant) = chunk.constants.get(*idx) {
                        if let BcValue::Number(n) = constant {
                            let val = builder.ins().f64const(*n);
                            value_stack.push(val);
                        } else {
                            // For non-number constants, push NaN as placeholder
                            let val = builder.ins().f64const(f64::NAN);
                            value_stack.push(val);
                        }
                    } else {
                        return Err(format!("Invalid constant index: {}", idx));
                    }
                }
                Opcode::LoadUndefined | Opcode::LoadNull => {
                    // Represent undefined/null as NaN
                    let val = builder.ins().f64const(f64::NAN);
                    value_stack.push(val);
                }
                Opcode::LoadTrue => {
                    let val = builder.ins().f64const(1.0);
                    value_stack.push(val);
                }
                Opcode::LoadFalse => {
                    let val = builder.ins().f64const(0.0);
                    value_stack.push(val);
                }
                Opcode::Add => {
                    if value_stack.len() >= 2 {
                        let b = value_stack.pop().unwrap();
                        let a = value_stack.pop().unwrap();
                        let result = builder.ins().fadd(a, b);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Add".to_string());
                    }
                }
                Opcode::Sub => {
                    if value_stack.len() >= 2 {
                        let b = value_stack.pop().unwrap();
                        let a = value_stack.pop().unwrap();
                        let result = builder.ins().fsub(a, b);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Sub".to_string());
                    }
                }
                Opcode::Mul => {
                    if value_stack.len() >= 2 {
                        let b = value_stack.pop().unwrap();
                        let a = value_stack.pop().unwrap();
                        let result = builder.ins().fmul(a, b);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Mul".to_string());
                    }
                }
                Opcode::Div => {
                    if value_stack.len() >= 2 {
                        let b = value_stack.pop().unwrap();
                        let a = value_stack.pop().unwrap();
                        let result = builder.ins().fdiv(a, b);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Div".to_string());
                    }
                }
                Opcode::Mod => {
                    // Modulo is more complex for floats, use fmod-like operation
                    // For now, we'll compute a - b * floor(a / b)
                    if value_stack.len() >= 2 {
                        let b = value_stack.pop().unwrap();
                        let a = value_stack.pop().unwrap();
                        let div = builder.ins().fdiv(a, b);
                        let floored = builder.ins().floor(div);
                        let mul = builder.ins().fmul(b, floored);
                        let result = builder.ins().fsub(a, mul);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Mod".to_string());
                    }
                }
                Opcode::Neg => {
                    if let Some(a) = value_stack.pop() {
                        let result = builder.ins().fneg(a);
                        value_stack.push(result);
                    } else {
                        return Err("Stack underflow on Neg".to_string());
                    }
                }
                Opcode::Return => {
                    let ret_val = value_stack.pop().unwrap_or_else(|| builder.ins().f64const(0.0));
                    builder.ins().return_(&[ret_val]);
                    has_return = true;
                }
                // Skip other opcodes for now - they'll be handled in future iterations
                _ => {
                    // For unsupported opcodes, we'll just continue
                    // In a production system, this would need proper handling
                }
            }
        }

        // Default return if no explicit return
        if !has_return {
            let zero = builder.ins().f64const(0.0);
            builder.ins().return_(&[zero]);
        }

        builder.finalize();
        Ok(())
    }
}

impl Default for CraneliftBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default CraneliftBackend")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::{BytecodeChunk, Opcode, Value as BcValue};

    #[test]
    fn test_cranelift_backend_creation() {
        let backend = CraneliftBackend::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_compile_constant_return() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk);
        assert!(compiled.is_ok());

        let func = compiled.unwrap();
        assert!(!func.code_ptr.is_null());

        // Execute the compiled function
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(func.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_addition() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(32.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_subtraction() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(50.0));
        let idx2 = chunk.add_constant(BcValue::Number(8.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Sub);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_multiplication() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(6.0));
        let idx2 = chunk.add_constant(BcValue::Number(7.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Mul);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_division() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(84.0));
        let idx2 = chunk.add_constant(BcValue::Number(2.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Div);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_complex_expression() {
        // Compute: (10 + 20) * 2 - 18 = 42
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(10.0));
        let idx2 = chunk.add_constant(BcValue::Number(20.0));
        let idx3 = chunk.add_constant(BcValue::Number(2.0));
        let idx4 = chunk.add_constant(BcValue::Number(18.0));

        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Add);
        chunk.emit(Opcode::LoadConstant(idx3));
        chunk.emit(Opcode::Mul);
        chunk.emit(Opcode::LoadConstant(idx4));
        chunk.emit(Opcode::Sub);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_negation() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx = chunk.add_constant(BcValue::Number(-42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Neg);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_modulo() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx1 = chunk.add_constant(BcValue::Number(47.0));
        let idx2 = chunk.add_constant(BcValue::Number(5.0));
        chunk.emit(Opcode::LoadConstant(idx1));
        chunk.emit(Opcode::LoadConstant(idx2));
        chunk.emit(Opcode::Mod);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_compile_multiple_functions() {
        let mut backend = CraneliftBackend::new().unwrap();

        // Compile first function
        let mut chunk1 = BytecodeChunk::new();
        let idx = chunk1.add_constant(BcValue::Number(10.0));
        chunk1.emit(Opcode::LoadConstant(idx));
        chunk1.emit(Opcode::Return);

        let compiled1 = backend.compile_function(&chunk1).unwrap();
        let func1: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled1.code_ptr) };

        // Compile second function
        let mut chunk2 = BytecodeChunk::new();
        let idx = chunk2.add_constant(BcValue::Number(20.0));
        chunk2.emit(Opcode::LoadConstant(idx));
        chunk2.emit(Opcode::Return);

        let compiled2 = backend.compile_function(&chunk2).unwrap();
        let func2: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled2.code_ptr) };

        // Both functions should work independently
        assert_eq!(func1(), 10.0);
        assert_eq!(func2(), 20.0);
    }

    #[test]
    fn test_compile_boolean_values() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        chunk.emit(Opcode::LoadTrue);
        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_compile_empty_return() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        chunk.emit(Opcode::Return);

        let compiled = backend.compile_function(&chunk).unwrap();
        let func_ptr: extern "C" fn() -> f64 = unsafe { std::mem::transmute(compiled.code_ptr) };
        let result = func_ptr();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_stack_underflow_add() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        let idx = chunk.add_constant(BcValue::Number(42.0));
        chunk.emit(Opcode::LoadConstant(idx));
        chunk.emit(Opcode::Add); // Only one value on stack
        chunk.emit(Opcode::Return);

        let result = backend.compile_function(&chunk);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Stack underflow"));
    }

    #[test]
    fn test_invalid_constant_index() {
        let mut backend = CraneliftBackend::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        chunk.emit(Opcode::LoadConstant(999)); // Invalid index
        chunk.emit(Opcode::Return);

        let result = backend.compile_function(&chunk);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid constant index"));
    }
}
