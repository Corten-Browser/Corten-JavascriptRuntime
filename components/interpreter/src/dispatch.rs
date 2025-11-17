//! Dispatch loop for bytecode execution
//!
//! Handles individual opcode execution.

use bytecode_system::{BytecodeChunk, Opcode, UpvalueDescriptor};
use builtins::{ConsoleObject, JsValue as BuiltinValue, MathObject};
use core_types::{ErrorKind, JsError, Value};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::context::ExecutionContext;
use crate::upvalue::{new_upvalue_handle, Closure, Upvalue, UpvalueHandle};

/// Dispatch handler for executing bytecode
pub struct Dispatcher {
    /// Global variables storage
    globals: HashMap<String, Value>,
    /// Stack for intermediate values
    stack: Vec<Value>,
    /// Console object for native console methods
    console: Rc<RefCell<ConsoleObject>>,
    /// Open upvalues tracked for the current execution (key: stack index)
    open_upvalues: HashMap<usize, UpvalueHandle>,
    /// Current closure's upvalues (set when executing a closure)
    current_upvalues: Vec<UpvalueHandle>,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field("globals", &self.globals)
            .field("stack", &self.stack)
            .field("console", &"ConsoleObject")
            .field("open_upvalues_count", &self.open_upvalues.len())
            .field("current_upvalues_count", &self.current_upvalues.len())
            .finish()
    }
}

impl Dispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        let console = Rc::new(RefCell::new(ConsoleObject::new()));
        let mut globals = HashMap::new();

        // Inject console global
        globals.insert(
            "console".to_string(),
            Value::NativeObject(console.clone() as Rc<RefCell<dyn Any>>),
        );

        // Inject Math global (static object, no state)
        globals.insert(
            "Math".to_string(),
            Value::NativeObject(Rc::new(RefCell::new(MathObject)) as Rc<RefCell<dyn Any>>),
        );

        Self {
            globals,
            stack: Vec::with_capacity(256),
            console,
            open_upvalues: HashMap::new(),
            current_upvalues: Vec::new(),
        }
    }

    /// Capture an upvalue for a closure based on the descriptor
    fn capture_upvalue(
        &mut self,
        desc: &UpvalueDescriptor,
        ctx: &ExecutionContext,
    ) -> UpvalueHandle {
        if desc.is_local {
            // The variable is a local in the current scope
            // We need to create or reuse an open upvalue for this register
            let stack_idx = desc.index as usize;

            // Check if we already have an open upvalue for this location
            if let Some(handle) = self.open_upvalues.get(&stack_idx) {
                handle.clone()
            } else {
                // Create new open upvalue pointing to the register value
                let value = ctx.get_register(stack_idx);
                let handle = new_upvalue_handle(Upvalue::new_closed(value));
                self.open_upvalues.insert(stack_idx, handle.clone());
                handle
            }
        } else {
            // The variable is an upvalue in the parent scope (grandparent+ to us)
            // Get it from the current upvalues
            if let Some(handle) = self.current_upvalues.get(desc.index as usize) {
                handle.clone()
            } else {
                // This shouldn't happen with correct compilation, but create a closed undefined upvalue
                new_upvalue_handle(Upvalue::new_closed(Value::Undefined))
            }
        }
    }

    /// Convert bytecode_system::Value to core_types::Value
    fn convert_bc_value(bc_value: &bytecode_system::Value) -> Value {
        match bc_value {
            bytecode_system::Value::Undefined => Value::Undefined,
            bytecode_system::Value::Null => Value::Null,
            bytecode_system::Value::Boolean(b) => Value::Boolean(*b),
            bytecode_system::Value::Number(n) => {
                // Try to convert to Smi if it's a small integer
                if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    Value::Smi(*n as i32)
                } else {
                    Value::Double(*n)
                }
            }
            bytecode_system::Value::String(_) => {
                // Strings would need to be heap-allocated, for now return undefined
                Value::Undefined
            }
            bytecode_system::Value::Closure(closure_data) => {
                // Create a HeapObject reference for the closure
                Value::HeapObject(closure_data.function_index)
            }
        }
    }

    /// Execute bytecode in the given context
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context with bytecode and registers
    /// * `functions` - Registry of function bytecode chunks
    pub fn execute(
        &mut self,
        ctx: &mut ExecutionContext,
        functions: &[BytecodeChunk],
    ) -> Result<Value, JsError> {
        loop {
            let inst = match ctx.fetch() {
                Some(i) => i.clone(),
                None => {
                    // No more instructions, return undefined
                    return Ok(Value::Undefined);
                }
            };

            match inst.opcode {
                Opcode::LoadConstant(idx) => {
                    let value = ctx
                        .bytecode
                        .constants
                        .get(idx)
                        .map(Self::convert_bc_value)
                        .unwrap_or(Value::Undefined);
                    self.stack.push(value);
                }
                Opcode::LoadUndefined => {
                    self.stack.push(Value::Undefined);
                }
                Opcode::LoadNull => {
                    self.stack.push(Value::Null);
                }
                Opcode::LoadTrue => {
                    self.stack.push(Value::Boolean(true));
                }
                Opcode::LoadFalse => {
                    self.stack.push(Value::Boolean(false));
                }
                Opcode::LoadGlobal(name) => {
                    let value = self.globals.get(&name).cloned().unwrap_or(Value::Undefined);
                    self.stack.push(value);
                }
                Opcode::StoreGlobal(name) => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    self.globals.insert(name, value);
                }
                Opcode::LoadLocal(reg_id) => {
                    let value = ctx.get_register(reg_id.0 as usize);
                    self.stack.push(value);
                }
                Opcode::StoreLocal(reg_id) => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    ctx.set_register(reg_id.0 as usize, value);
                }
                Opcode::LoadUpvalue(idx) => {
                    // Load value from captured upvalue
                    if let Some(upvalue_handle) = self.current_upvalues.get(idx as usize) {
                        let upvalue = upvalue_handle.borrow();
                        let value = upvalue.get(&[]);
                        self.stack.push(value);
                    } else {
                        self.stack.push(Value::Undefined);
                    }
                }
                Opcode::StoreUpvalue(idx) => {
                    // Store value to captured upvalue
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    if let Some(upvalue_handle) = self.current_upvalues.get(idx as usize) {
                        let upvalue = upvalue_handle.borrow();
                        upvalue.set(value, &mut []);
                    }
                }
                Opcode::CloseUpvalue => {
                    // Close over local variables when scope ends
                    // This is typically emitted when a scope ends that had captured variables
                    // For now, this is a no-op since we're using closed upvalues directly
                }
                Opcode::Add => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.add(a, b)?;
                    self.stack.push(result);
                }
                Opcode::Sub => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.sub(a, b)?;
                    self.stack.push(result);
                }
                Opcode::Mul => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.mul(a, b)?;
                    self.stack.push(result);
                }
                Opcode::Div => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.div(a, b)?;
                    self.stack.push(result);
                }
                Opcode::Mod => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.modulo(a, b)?;
                    self.stack.push(result);
                }
                Opcode::Neg => {
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.neg(a)?;
                    self.stack.push(result);
                }
                Opcode::Equal => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.equal(a, b);
                    self.stack.push(result);
                }
                Opcode::StrictEqual => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.strict_equal(a, b);
                    self.stack.push(result);
                }
                Opcode::NotEqual => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.not_equal(a, b);
                    self.stack.push(result);
                }
                Opcode::StrictNotEqual => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.strict_not_equal(a, b);
                    self.stack.push(result);
                }
                Opcode::LessThan => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.less_than(a, b);
                    self.stack.push(result);
                }
                Opcode::LessThanEqual => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.less_than_equal(a, b);
                    self.stack.push(result);
                }
                Opcode::GreaterThan => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.greater_than(a, b);
                    self.stack.push(result);
                }
                Opcode::GreaterThanEqual => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.greater_than_equal(a, b);
                    self.stack.push(result);
                }
                Opcode::Jump(target) => {
                    ctx.instruction_pointer = target;
                }
                Opcode::JumpIfTrue(target) => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    if value.is_truthy() {
                        ctx.instruction_pointer = target;
                    }
                }
                Opcode::JumpIfFalse(target) => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    if !value.is_truthy() {
                        ctx.instruction_pointer = target;
                    }
                }
                Opcode::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::Undefined);
                    return Ok(return_value);
                }
                Opcode::CreateObject => {
                    // Placeholder: create object with ID
                    self.stack.push(Value::HeapObject(0));
                }
                Opcode::LoadProperty(name) => {
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    match obj {
                        Value::NativeObject(native_obj) => {
                            let borrowed = native_obj.borrow();
                            if borrowed.is::<ConsoleObject>() {
                                // Return method based on property name
                                match name.as_str() {
                                    "log" => self
                                        .stack
                                        .push(Value::NativeFunction("console.log".to_string())),
                                    "error" => self
                                        .stack
                                        .push(Value::NativeFunction("console.error".to_string())),
                                    "warn" => self
                                        .stack
                                        .push(Value::NativeFunction("console.warn".to_string())),
                                    "info" => self
                                        .stack
                                        .push(Value::NativeFunction("console.info".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else if borrowed.is::<MathObject>() {
                                match name.as_str() {
                                    "abs" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.abs".to_string())),
                                    "ceil" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.ceil".to_string())),
                                    "floor" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.floor".to_string())),
                                    "round" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.round".to_string())),
                                    "sqrt" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.sqrt".to_string())),
                                    "pow" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.pow".to_string())),
                                    "sin" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.sin".to_string())),
                                    "cos" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.cos".to_string())),
                                    "tan" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.tan".to_string())),
                                    "random" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.random".to_string())),
                                    "max" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.max".to_string())),
                                    "min" => self
                                        .stack
                                        .push(Value::NativeFunction("Math.min".to_string())),
                                    "PI" => self.stack.push(Value::Double(MathObject::PI)),
                                    "E" => self.stack.push(Value::Double(MathObject::E)),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else {
                                self.stack.push(Value::Undefined);
                            }
                        }
                        _ => self.stack.push(Value::Undefined),
                    }
                }
                Opcode::StoreProperty(_name) => {
                    // Placeholder: property store
                    self.stack.pop(); // value
                    self.stack.pop(); // object
                }
                Opcode::CreateClosure(idx, upvalue_descs) => {
                    // Create a closure by capturing upvalues from the current scope
                    if upvalue_descs.is_empty() {
                        // No captured variables, just push the function index
                        self.stack.push(Value::HeapObject(idx));
                    } else {
                        // Capture upvalues based on descriptors
                        let mut captured_upvalues = Vec::with_capacity(upvalue_descs.len());
                        for desc in &upvalue_descs {
                            let upvalue_handle = self.capture_upvalue(desc, ctx);
                            captured_upvalues.push(upvalue_handle);
                        }

                        // Store the closure with captured upvalues
                        // For now, we store the closure data in the global registry
                        // and return a HeapObject reference with an encoded ID
                        // The captured upvalues will be retrieved when the closure is called

                        // Create a composite ID that encodes both function index and closure instance
                        // We'll use a simple scheme: store closure info and use a special marker
                        // For simplicity, we just store the function index and track upvalues separately
                        // A more complete implementation would use a closure registry
                        self.stack.push(Value::HeapObject(idx));

                        // Store captured upvalues for this closure (simplified approach)
                        // In a full implementation, we'd have a closure registry
                        // For now, we'll rely on the call site to set up upvalues properly
                    }
                }
                Opcode::Call(argc) => {
                    // Pop arguments first (in reverse order)
                    let mut args = Vec::with_capacity(argc as usize);
                    for _ in 0..argc {
                        args.push(self.stack.pop().unwrap_or(Value::Undefined));
                    }
                    args.reverse(); // Now args[0] is first argument

                    // Pop the callee (function) from stack
                    let callee = self.stack.pop().unwrap_or(Value::Undefined);

                    match callee {
                        Value::NativeFunction(name) => {
                            let result = self.call_native_function(&name, args)?;
                            self.stack.push(result);
                        }
                        Value::HeapObject(_) => {
                            // User-defined function - push args back and call
                            for arg in args.into_iter().rev() {
                                self.stack.push(arg);
                            }
                            self.stack.push(callee);
                            let result = self.call_function(argc, functions)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(JsError {
                                kind: ErrorKind::TypeError,
                                message: format!("{:?} is not a function", callee),
                                stack: vec![],
                                source_position: None,
                            });
                        }
                    }
                }
                Opcode::LoadUpvalue(_idx) => {
                    // Placeholder: load captured variable
                    // Upvalues are used for closures to access parent scope variables
                    self.stack.push(Value::Undefined);
                }
                Opcode::StoreUpvalue(_idx) => {
                    // Placeholder: store to captured variable
                    self.stack.pop();
                }
                Opcode::CloseUpvalue => {
                    // Placeholder: close over local variable (move from stack to heap)
                    // This is used when a closure outlives its creating scope
                }
            }
        }
    }

    /// Call a native function by name
    fn call_native_function(&self, name: &str, args: Vec<Value>) -> Result<Value, JsError> {
        match name {
            // Console methods
            "console.log" => {
                let builtin_args: Vec<BuiltinValue> = args.iter().map(Self::to_builtin_value).collect();
                self.console.borrow().log(&builtin_args);
                Ok(Value::Undefined)
            }
            "console.error" => {
                let builtin_args: Vec<BuiltinValue> = args.iter().map(Self::to_builtin_value).collect();
                self.console.borrow().error(&builtin_args);
                Ok(Value::Undefined)
            }
            "console.warn" => {
                let builtin_args: Vec<BuiltinValue> = args.iter().map(Self::to_builtin_value).collect();
                self.console.borrow().warn(&builtin_args);
                Ok(Value::Undefined)
            }
            "console.info" => {
                let builtin_args: Vec<BuiltinValue> = args.iter().map(Self::to_builtin_value).collect();
                self.console.borrow().info(&builtin_args);
                Ok(Value::Undefined)
            }
            // Math methods
            "Math.abs" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::abs(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.ceil" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::ceil(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.floor" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::floor(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.round" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::round(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.sqrt" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::sqrt(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.pow" => {
                if args.len() >= 2 {
                    let base = self.to_number(&args[0]);
                    let exp = self.to_number(&args[1]);
                    Ok(Value::Double(MathObject::pow(base, exp)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.sin" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::sin(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.cos" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::cos(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.tan" => {
                if let Some(n) = args.first().map(|v| self.to_number(v)) {
                    Ok(Value::Double(MathObject::tan(n)))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            "Math.random" => Ok(Value::Double(MathObject::random())),
            "Math.max" => {
                let nums: Vec<f64> = args.iter().map(|v| self.to_number(v)).collect();
                Ok(Value::Double(MathObject::max(&nums)))
            }
            "Math.min" => {
                let nums: Vec<f64> = args.iter().map(|v| self.to_number(v)).collect();
                Ok(Value::Double(MathObject::min(&nums)))
            }
            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("{} is not a function", name),
                stack: vec![],
                source_position: None,
            }),
        }
    }

    /// Convert core_types::Value to builtins::JsValue
    fn to_builtin_value(value: &Value) -> BuiltinValue {
        match value {
            Value::Undefined => BuiltinValue::undefined(),
            Value::Null => BuiltinValue::null(),
            Value::Boolean(b) => BuiltinValue::boolean(*b),
            Value::Smi(n) => BuiltinValue::number(*n as f64),
            Value::Double(n) => BuiltinValue::number(*n),
            Value::HeapObject(_) => BuiltinValue::object(),
            Value::NativeObject(_) => BuiltinValue::object(),
            Value::NativeFunction(name) => BuiltinValue::string(format!("function {}() {{ [native code] }}", name)),
        }
    }

    /// Execute a function call
    ///
    /// Stack layout before call (bottom to top):
    /// [..., arg1, arg2, ..., argN, callee]
    ///
    /// After call:
    /// [..., return_value]
    fn call_function(&mut self, argc: u8, functions: &[BytecodeChunk]) -> Result<Value, JsError> {
        // Pop the callee (function) from stack
        let callee = self.stack.pop().unwrap_or(Value::Undefined);

        // Determine the function index from the callee
        let fn_idx = match callee {
            Value::HeapObject(idx) => idx,
            _ => {
                // Not a function - TypeError
                // Pop arguments to clean up stack
                for _ in 0..argc {
                    self.stack.pop();
                }
                return Err(JsError {
                    kind: ErrorKind::TypeError,
                    message: format!("{:?} is not a function", callee),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        // Get the function bytecode
        let fn_bytecode = match functions.get(fn_idx) {
            Some(chunk) => chunk.clone(),
            None => {
                // Invalid function index
                for _ in 0..argc {
                    self.stack.pop();
                }
                return Err(JsError {
                    kind: ErrorKind::ReferenceError,
                    message: format!("Invalid function index: {}", fn_idx),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        // Pop arguments from stack (in reverse order, so arg1 is first)
        let mut args = Vec::with_capacity(argc as usize);
        for _ in 0..argc {
            args.push(self.stack.pop().unwrap_or(Value::Undefined));
        }
        args.reverse(); // Now args[0] is first argument

        // Create new execution context for the function
        let mut fn_ctx = ExecutionContext::new(fn_bytecode);

        // Set arguments as registers (parameter passing)
        // Register 0 = first argument, Register 1 = second argument, etc.
        for (i, arg) in args.into_iter().enumerate() {
            fn_ctx.set_register(i, arg);
        }
        // Missing arguments are already initialized to Undefined

        // Recursively execute the function
        // This enables nested calls and recursion
        self.execute(&mut fn_ctx, functions)
    }

    /// Get global variable
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    /// Set global variable
    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    // Arithmetic operations

    fn add(&self, a: Value, b: Value) -> Result<Value, JsError> {
        match (a, b) {
            (Value::Smi(x), Value::Smi(y)) => Ok(Value::Smi(x.wrapping_add(y))),
            (Value::Double(x), Value::Double(y)) => Ok(Value::Double(x + y)),
            (Value::Smi(x), Value::Double(y)) => Ok(Value::Double(x as f64 + y)),
            (Value::Double(x), Value::Smi(y)) => Ok(Value::Double(x + y as f64)),
            _ => Ok(Value::Double(f64::NAN)),
        }
    }

    fn sub(&self, a: Value, b: Value) -> Result<Value, JsError> {
        match (a, b) {
            (Value::Smi(x), Value::Smi(y)) => Ok(Value::Smi(x.wrapping_sub(y))),
            (Value::Double(x), Value::Double(y)) => Ok(Value::Double(x - y)),
            (Value::Smi(x), Value::Double(y)) => Ok(Value::Double(x as f64 - y)),
            (Value::Double(x), Value::Smi(y)) => Ok(Value::Double(x - y as f64)),
            _ => Ok(Value::Double(f64::NAN)),
        }
    }

    fn mul(&self, a: Value, b: Value) -> Result<Value, JsError> {
        match (a, b) {
            (Value::Smi(x), Value::Smi(y)) => Ok(Value::Smi(x.wrapping_mul(y))),
            (Value::Double(x), Value::Double(y)) => Ok(Value::Double(x * y)),
            (Value::Smi(x), Value::Double(y)) => Ok(Value::Double(x as f64 * y)),
            (Value::Double(x), Value::Smi(y)) => Ok(Value::Double(x * y as f64)),
            _ => Ok(Value::Double(f64::NAN)),
        }
    }

    fn div(&self, a: Value, b: Value) -> Result<Value, JsError> {
        // Division always returns Double in JavaScript
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Ok(Value::Double(a_num / b_num))
    }

    fn modulo(&self, a: Value, b: Value) -> Result<Value, JsError> {
        match (&a, &b) {
            (Value::Smi(x), Value::Smi(y)) => {
                if *y != 0 {
                    Ok(Value::Smi(x % y))
                } else {
                    Ok(Value::Double(f64::NAN))
                }
            }
            _ => {
                let a_num = self.to_number(&a);
                let b_num = self.to_number(&b);
                Ok(Value::Double(a_num % b_num))
            }
        }
    }

    fn neg(&self, a: Value) -> Result<Value, JsError> {
        match a {
            Value::Smi(x) => Ok(Value::Smi(-x)),
            Value::Double(x) => Ok(Value::Double(-x)),
            _ => Ok(Value::Double(f64::NAN)),
        }
    }

    fn to_number(&self, value: &Value) -> f64 {
        match value {
            Value::Smi(n) => *n as f64,
            Value::Double(n) => *n,
            Value::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::HeapObject(_) => f64::NAN,
            Value::NativeObject(_) => f64::NAN,
            Value::NativeFunction(_) => f64::NAN,
        }
    }

    // Comparison operations

    fn equal(&self, a: Value, b: Value) -> Value {
        // Loose equality - performs type coercion
        let result = match (&a, &b) {
            (Value::Smi(x), Value::Smi(y)) => x == y,
            (Value::Double(x), Value::Double(y)) => x == y,
            (Value::Smi(x), Value::Double(y)) => (*x as f64) == *y,
            (Value::Double(x), Value::Smi(y)) => *x == (*y as f64),
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Null) | (Value::Null, Value::Undefined) => true,
            _ => false,
        };
        Value::Boolean(result)
    }

    fn strict_equal(&self, a: Value, b: Value) -> Value {
        // Strict equality - no type coercion
        Value::Boolean(a == b)
    }

    fn not_equal(&self, a: Value, b: Value) -> Value {
        let eq = self.equal(a, b);
        match eq {
            Value::Boolean(b) => Value::Boolean(!b),
            _ => Value::Boolean(true),
        }
    }

    fn strict_not_equal(&self, a: Value, b: Value) -> Value {
        let eq = self.strict_equal(a, b);
        match eq {
            Value::Boolean(b) => Value::Boolean(!b),
            _ => Value::Boolean(true),
        }
    }

    fn less_than(&self, a: Value, b: Value) -> Value {
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Value::Boolean(a_num < b_num)
    }

    fn less_than_equal(&self, a: Value, b: Value) -> Value {
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Value::Boolean(a_num <= b_num)
    }

    fn greater_than(&self, a: Value, b: Value) -> Value {
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Value::Boolean(a_num > b_num)
    }

    fn greater_than_equal(&self, a: Value, b: Value) -> Value {
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Value::Boolean(a_num >= b_num)
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode_system::BytecodeChunk;

    #[test]
    fn test_dispatcher_new() {
        let dispatcher = Dispatcher::new();
        assert!(dispatcher.globals.is_empty());
        assert!(dispatcher.stack.is_empty());
    }

    #[test]
    fn test_dispatcher_default() {
        let dispatcher = Dispatcher::default();
        assert!(dispatcher.globals.is_empty());
    }

    #[test]
    fn test_dispatcher_globals() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.set_global("x".to_string(), Value::Smi(42));
        assert_eq!(dispatcher.get_global("x"), Some(Value::Smi(42)));
        assert_eq!(dispatcher.get_global("y"), None);
    }

    #[test]
    fn test_to_number() {
        let dispatcher = Dispatcher::new();
        assert_eq!(dispatcher.to_number(&Value::Smi(42)), 42.0);
        assert_eq!(dispatcher.to_number(&Value::Double(3.14)), 3.14);
        assert_eq!(dispatcher.to_number(&Value::Boolean(true)), 1.0);
        assert_eq!(dispatcher.to_number(&Value::Boolean(false)), 0.0);
        assert!(dispatcher.to_number(&Value::Undefined).is_nan());
        assert_eq!(dispatcher.to_number(&Value::Null), 0.0);
    }

    #[test]
    fn test_empty_bytecode() {
        let mut dispatcher = Dispatcher::new();
        let chunk = BytecodeChunk::new();
        let mut ctx = ExecutionContext::new(chunk);
        let functions = vec![];

        let result = dispatcher.execute(&mut ctx, &functions);
        assert_eq!(result.unwrap(), Value::Undefined);
    }
}
