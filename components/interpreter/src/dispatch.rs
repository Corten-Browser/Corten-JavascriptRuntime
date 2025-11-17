//! Dispatch loop for bytecode execution
//!
//! Handles individual opcode execution.

use async_runtime::PromiseState;
use bytecode_system::{BytecodeChunk, Opcode, UpvalueDescriptor};
use builtins::{ConsoleObject, JsValue as BuiltinValue, MathObject};
use core_types::{ErrorKind, JsError, Value};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::context::ExecutionContext;
use crate::gc_integration::{GCObject, VMHeap};
use crate::promise_integration::{PromiseConstructor, PromiseObject};
use crate::upvalue::{new_upvalue_handle, Upvalue, UpvalueHandle};

/// Exception handler for try/catch/finally blocks
#[derive(Debug, Clone)]
struct TryHandler {
    /// Offset to jump to for catch block (if any)
    catch_offset: Option<usize>,
    /// Offset to jump to for finally block (if any)
    finally_offset: Option<usize>,
    /// Stack height when try block started (for unwinding)
    stack_height: usize,
}

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
    /// Stack of active try blocks for exception handling
    try_stack: Vec<TryHandler>,
    /// Currently thrown exception (if any)
    current_exception: Option<Value>,
    /// GC heap for JavaScript object allocation (shared with VM)
    heap: Option<Rc<VMHeap>>,
    /// Registry of closures: maps closure ID to (function_index, captured_upvalues)
    closure_registry: HashMap<usize, (usize, Vec<UpvalueHandle>)>,
    /// Next available closure ID
    next_closure_id: usize,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field("globals", &self.globals)
            .field("stack", &self.stack)
            .field("console", &"ConsoleObject")
            .field("open_upvalues_count", &self.open_upvalues.len())
            .field("current_upvalues_count", &self.current_upvalues.len())
            .field("try_stack_depth", &self.try_stack.len())
            .field("has_exception", &self.current_exception.is_some())
            .field("has_heap", &self.heap.is_some())
            .field("closure_registry_size", &self.closure_registry.len())
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

        // Inject Promise constructor as a NativeFunction
        // Promise.resolve(), Promise.reject() are accessed via property lookup
        globals.insert(
            "Promise".to_string(),
            Value::NativeFunction("Promise".to_string()),
        );

        Self {
            globals,
            stack: Vec::with_capacity(256),
            console,
            open_upvalues: HashMap::new(),
            current_upvalues: Vec::new(),
            try_stack: Vec::new(),
            current_exception: None,
            heap: None,
            closure_registry: HashMap::new(),
            next_closure_id: 0,
        }
    }

    /// Set the GC heap reference
    ///
    /// This should be called before executing bytecode to enable GC-managed object creation.
    pub fn set_heap(&mut self, heap: Rc<VMHeap>) {
        self.heap = Some(heap);
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

    /// Throw an exception, unwinding the try stack to find a handler
    ///
    /// Returns Ok(()) if a handler was found and execution should continue,
    /// or Err if no handler was found (uncaught exception).
    fn throw_exception(
        &mut self,
        value: Value,
        ctx: &mut ExecutionContext,
    ) -> Result<(), JsError> {
        self.current_exception = Some(value.clone());

        // Find nearest catch handler
        while let Some(handler) = self.try_stack.pop() {
            // Unwind stack to try block's height
            while self.stack.len() > handler.stack_height {
                self.stack.pop();
            }

            if let Some(catch_offset) = handler.catch_offset {
                // Jump to catch block, push exception value onto stack
                self.stack.push(value);
                ctx.instruction_pointer = catch_offset;
                self.current_exception = None;
                return Ok(());
            } else if let Some(finally_offset) = handler.finally_offset {
                // Must run finally block first (exception still pending)
                ctx.instruction_pointer = finally_offset;
                return Ok(());
            }
        }

        // No handler found - propagate error as uncaught exception
        Err(JsError {
            kind: ErrorKind::InternalError,
            message: format!("Uncaught exception: {:?}", value),
            stack: vec![],
            source_position: None,
        })
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
            bytecode_system::Value::String(s) => Value::String(s.clone()),
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
                Opcode::Not => {
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    // Logical NOT - invert truthiness
                    let result = Value::Boolean(!a.is_truthy());
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
                    // Create a GC-managed JavaScript object
                    if let Some(ref heap) = self.heap {
                        let gc_object = heap.create_object();
                        let boxed: Box<dyn Any> = Box::new(gc_object);
                        let value =
                            Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>);
                        self.stack.push(value);
                    } else {
                        // Fallback: use HeapObject with ID 0 (legacy behavior)
                        self.stack.push(Value::HeapObject(0));
                    }
                }
                Opcode::LoadProperty(name) => {
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    match obj {
                        Value::NativeObject(native_obj) => {
                            let borrowed = native_obj.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                // Check if it's a GCObject wrapped in Box<dyn Any>
                                if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                    let value = gc_object.get(&name);
                                    drop(borrowed);
                                    self.stack.push(value);
                                } else {
                                    drop(borrowed);
                                    self.stack.push(Value::Undefined);
                                }
                            } else if borrowed.is::<ConsoleObject>() {
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
                        Value::NativeFunction(fn_name) => {
                            // Handle static properties on constructor functions
                            if fn_name == "Promise" {
                                match name.as_str() {
                                    "resolve" => self
                                        .stack
                                        .push(Value::NativeFunction("Promise.resolve".to_string())),
                                    "reject" => self
                                        .stack
                                        .push(Value::NativeFunction("Promise.reject".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else {
                                self.stack.push(Value::Undefined);
                            }
                        }
                        _ => self.stack.push(Value::Undefined),
                    }
                }
                Opcode::StoreProperty(name) => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    match obj {
                        Value::NativeObject(native_obj) => {
                            let mut borrowed = native_obj.borrow_mut();
                            if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                                // Check if it's a GCObject wrapped in Box<dyn Any>
                                if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                    gc_object.set(name, value);
                                }
                            }
                            // For other NativeObjects, we just ignore the store (non-extensible)
                        }
                        _ => {
                            // Ignore stores to non-objects
                        }
                    }
                }
                Opcode::GetIndex => {
                    // Get value at computed index: obj[index]
                    let index = self.stack.pop().unwrap_or(Value::Undefined);
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    let result = match obj {
                        Value::NativeObject(native_obj) => {
                            let borrowed = native_obj.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                    // Convert index to string key
                                    let key = self.to_property_key(&index);
                                    gc_object.get(&key)
                                } else {
                                    Value::Undefined
                                }
                            } else {
                                Value::Undefined
                            }
                        }
                        Value::String(s) => {
                            // String indexing: "hello"[1] => "e"
                            match &index {
                                Value::Smi(i) => {
                                    if *i >= 0 {
                                        s.chars()
                                            .nth(*i as usize)
                                            .map(|c| Value::String(c.to_string()))
                                            .unwrap_or(Value::Undefined)
                                    } else {
                                        Value::Undefined
                                    }
                                }
                                Value::Double(n) => {
                                    if n.fract() == 0.0 && *n >= 0.0 {
                                        s.chars()
                                            .nth(*n as usize)
                                            .map(|c| Value::String(c.to_string()))
                                            .unwrap_or(Value::Undefined)
                                    } else {
                                        Value::Undefined
                                    }
                                }
                                _ => Value::Undefined,
                            }
                        }
                        _ => Value::Undefined,
                    };
                    self.stack.push(result);
                }
                Opcode::SetIndex => {
                    // Set value at computed index: obj[index] = value
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    let index = self.stack.pop().unwrap_or(Value::Undefined);
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    match obj {
                        Value::NativeObject(native_obj) => {
                            let mut borrowed = native_obj.borrow_mut();
                            if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                                if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                    let key = self.to_property_key(&index);
                                    gc_object.set(key, value);
                                }
                            }
                        }
                        _ => {
                            // Ignore index stores to non-objects
                        }
                    }
                }
                Opcode::CreateArray(count) => {
                    // Create array with elements from stack
                    if let Some(ref heap) = self.heap {
                        let gc_object = heap.create_object();
                        let boxed: Box<dyn Any> = Box::new(gc_object);
                        let obj_ref =
                            Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>;

                        // Pop elements from stack (in reverse order)
                        let mut elements = Vec::with_capacity(count);
                        for _ in 0..count {
                            elements.push(self.stack.pop().unwrap_or(Value::Undefined));
                        }
                        elements.reverse(); // Now elements[0] is first

                        // Store elements with numeric keys
                        {
                            let borrowed = obj_ref.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(_) = gc_obj.downcast_ref::<GCObject>() {
                                    drop(borrowed);
                                    let mut borrowed_mut = obj_ref.borrow_mut();
                                    if let Some(gc_obj) = borrowed_mut.downcast_mut::<Box<dyn Any>>()
                                    {
                                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                            for (i, elem) in elements.into_iter().enumerate() {
                                                gc_object.set(i.to_string(), elem);
                                            }
                                            // Set length property
                                            gc_object.set("length".to_string(), Value::Smi(count as i32));
                                        }
                                    }
                                }
                            }
                        }

                        self.stack.push(Value::NativeObject(obj_ref));
                    } else {
                        // Fallback: push empty array representation
                        self.stack.push(Value::HeapObject(0));
                    }
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

                        // Register the closure with its captured upvalues
                        let closure_id = self.next_closure_id;
                        self.next_closure_id += 1;
                        self.closure_registry
                            .insert(closure_id, (idx, captured_upvalues));

                        // Push a closure ID (with high bit set to distinguish from plain function index)
                        // We encode closure IDs starting from 1_000_000 to avoid collision with function indices
                        let encoded_id = 1_000_000 + closure_id;
                        self.stack.push(Value::HeapObject(encoded_id));
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
                        Value::HeapObject(idx) => {
                            // User-defined function - call directly with args
                            let result = self.call_function_with_args(idx, args, functions)?;
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
                Opcode::CallMethod(argc) => {
                    // Pop arguments first (in reverse order)
                    let mut args = Vec::with_capacity(argc as usize);
                    for _ in 0..argc {
                        args.push(self.stack.pop().unwrap_or(Value::Undefined));
                    }
                    args.reverse(); // Now args[0] is first argument

                    // Pop the method (function) from stack
                    let method = self.stack.pop().unwrap_or(Value::Undefined);
                    // Pop the receiver (this) from stack
                    let receiver = self.stack.pop().unwrap_or(Value::Undefined);

                    match method {
                        Value::NativeFunction(name) => {
                            let result = self.call_native_function(&name, args)?;
                            self.stack.push(result);
                        }
                        Value::HeapObject(idx) => {
                            // User-defined method - call with this binding
                            let result = self.call_method_with_this(idx, receiver, args, functions)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(JsError {
                                kind: ErrorKind::TypeError,
                                message: format!("{:?} is not a function", method),
                                stack: vec![],
                                source_position: None,
                            });
                        }
                    }
                }
                Opcode::CallNew(argc) => {
                    // Parser generates: push constructor, push arg1, push arg2, ..., CallNew
                    // Stack order: [..., constructor, arg1, arg2, ...] with last arg on top

                    // Pop arguments first (they're on top of stack)
                    let mut args = Vec::with_capacity(argc as usize);
                    for _ in 0..argc {
                        args.push(self.stack.pop().unwrap_or(Value::Undefined));
                    }
                    args.reverse(); // Now args[0] is first argument

                    // Now pop the constructor (it's below the arguments)
                    let constructor = self.stack.pop().unwrap_or(Value::Undefined);

                    match constructor {
                        Value::NativeFunction(name) => {
                            let result = self.call_native_function(&name, args)?;
                            self.stack.push(result);
                        }
                        Value::HeapObject(idx) => {
                            // User-defined constructor - call with new instance as this
                            let result = self.call_constructor(idx, args, functions)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(JsError {
                                kind: ErrorKind::TypeError,
                                message: format!("{:?} is not a constructor", constructor),
                                stack: vec![],
                                source_position: None,
                            });
                        }
                    }
                }

                // Exception handling opcodes
                Opcode::Throw => {
                    let value = self.stack.pop().unwrap_or(Value::Undefined);
                    self.throw_exception(value, ctx)?;
                }

                Opcode::PushTry(catch_offset) => {
                    self.try_stack.push(TryHandler {
                        catch_offset: Some(catch_offset),
                        finally_offset: None,
                        stack_height: self.stack.len(),
                    });
                }

                Opcode::PopTry => {
                    self.try_stack.pop();
                }

                Opcode::PushFinally(finally_offset) => {
                    if let Some(handler) = self.try_stack.last_mut() {
                        handler.finally_offset = Some(finally_offset);
                    }
                }

                Opcode::PopFinally => {
                    if let Some(handler) = self.try_stack.last_mut() {
                        handler.finally_offset = None;
                    }
                    // If we're still throwing, re-throw after finally completes
                    if let Some(exc) = self.current_exception.take() {
                        self.throw_exception(exc, ctx)?;
                    }
                }

                Opcode::Pop => {
                    self.stack.pop();
                }
                Opcode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                // Async opcodes
                Opcode::Await => {
                    let promise_value = self.stack.pop().unwrap_or(Value::Undefined);

                    // Check if it's a Promise
                    match &promise_value {
                        Value::NativeObject(obj) => {
                            let borrowed = obj.borrow();
                            if let Some(promise_obj) = borrowed.downcast_ref::<PromiseObject>() {
                                match promise_obj.state() {
                                    PromiseState::Fulfilled => {
                                        // Promise is already resolved - get the value immediately
                                        let result = promise_obj.value().cloned().unwrap_or(Value::Undefined);
                                        drop(borrowed);
                                        self.stack.push(result);
                                    }
                                    PromiseState::Rejected => {
                                        // Promise is rejected - throw the error
                                        if let Some(error) = promise_obj.error() {
                                            return Err(error.clone());
                                        } else {
                                            let error_value = Value::Undefined;
                                            drop(borrowed);
                                            self.throw_exception(error_value, ctx)?;
                                        }
                                    }
                                    PromiseState::Pending => {
                                        // Promise is pending - in a real implementation we would
                                        // suspend execution and schedule resume when promise resolves.
                                        // For now, return undefined (synchronous fallback)
                                        drop(borrowed);
                                        self.stack.push(Value::Undefined);
                                    }
                                }
                            } else {
                                // Not a Promise object - return as-is
                                drop(borrowed);
                                self.stack.push(promise_value);
                            }
                        }
                        _ => {
                            // Not a Promise - just return the value (like awaiting a non-thenable)
                            self.stack.push(promise_value);
                        }
                    }
                }

                Opcode::CreateAsyncFunction(idx, ref upvalue_descs) => {
                    // Create an async function wrapper
                    // In a full implementation, this would create a special async function
                    // that returns a Promise when called
                    if upvalue_descs.is_empty() {
                        // No captured variables - use function index directly
                        // Mark it as async by using a special encoding (high bit set)
                        let async_marker = 0x8000_0000;
                        self.stack.push(Value::HeapObject(idx | async_marker));
                    } else {
                        // With captured upvalues
                        let mut captured_upvalues = Vec::with_capacity(upvalue_descs.len());
                        for desc in upvalue_descs {
                            let upvalue_handle = self.capture_upvalue(desc, ctx);
                            captured_upvalues.push(upvalue_handle);
                        }
                        let async_marker = 0x8000_0000;
                        self.stack.push(Value::HeapObject(idx | async_marker));
                    }
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
            // Promise methods
            "Promise" => {
                // Promise constructor - create a new pending promise
                // In a full implementation, this would take an executor function
                Ok(PromiseConstructor::new_pending())
            }
            "Promise.resolve" => {
                // Create a resolved promise with the given value
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                Ok(PromiseConstructor::resolve(value))
            }
            "Promise.reject" => {
                // Create a rejected promise with the given error
                let reason = args.first().cloned().unwrap_or(Value::Undefined);
                let error = JsError {
                    kind: ErrorKind::TypeError,
                    message: format!("{:?}", reason),
                    stack: vec![],
                    source_position: None,
                };
                Ok(PromiseConstructor::reject(error))
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
            Value::String(s) => BuiltinValue::string(s.clone()),
            Value::NativeObject(_) => BuiltinValue::object(),
            Value::NativeFunction(name) => BuiltinValue::string(format!("function {}() {{ [native code] }}", name)),
        }
    }

    /// Execute a function call with pre-extracted arguments
    ///
    /// # Arguments
    /// * `func_idx_or_closure` - The function index or closure encoded ID
    /// * `args` - The function arguments (already in correct order)
    /// * `functions` - The function registry
    ///
    /// # Returns
    /// The return value of the function
    fn call_function_with_args(
        &mut self,
        func_idx_or_closure: usize,
        args: Vec<Value>,
        functions: &[BytecodeChunk],
    ) -> Result<Value, JsError> {
        // Check for recursion depth to prevent stack overflow
        static CALL_DEPTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let depth = CALL_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if depth > 10000 {
            CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            return Err(JsError {
                kind: ErrorKind::RangeError,
                message: "Maximum call stack size exceeded".to_string(),
                stack: vec![],
                source_position: None,
            });
        }

        // Determine the actual function index and closure upvalues
        let (fn_idx, closure_upvalues) = if func_idx_or_closure >= 1_000_000 {
            // This is a closure - decode and get upvalues from registry
            let closure_id = func_idx_or_closure - 1_000_000;
            match self.closure_registry.get(&closure_id) {
                Some((func_idx, upvalues)) => (*func_idx, Some(upvalues.clone())),
                None => {
                    CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    return Err(JsError {
                        kind: ErrorKind::ReferenceError,
                        message: format!("Invalid closure ID: {}", closure_id),
                        stack: vec![],
                        source_position: None,
                    });
                }
            }
        } else {
            // Plain function index
            (func_idx_or_closure, None)
        };

        // Get the function bytecode
        let fn_bytecode = match functions.get(fn_idx) {
            Some(chunk) => chunk.clone(),
            None => {
                CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                return Err(JsError {
                    kind: ErrorKind::ReferenceError,
                    message: format!("Invalid function index: {}", fn_idx),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        // Create new execution context for the function
        let mut fn_ctx = ExecutionContext::new(fn_bytecode);

        // Set arguments as registers (parameter passing)
        // Register 0 = first argument, Register 1 = second argument, etc.
        for (i, arg) in args.into_iter().enumerate() {
            fn_ctx.set_register(i, arg);
        }
        // Missing arguments are already initialized to Undefined

        // Save current upvalues and set closure's upvalues if this is a closure call
        let saved_upvalues = std::mem::take(&mut self.current_upvalues);
        let saved_open_upvalues = std::mem::take(&mut self.open_upvalues);
        if let Some(upvalues) = closure_upvalues {
            self.current_upvalues = upvalues;
        }

        // Recursively execute the function
        // This enables nested calls and recursion
        let result = self.execute(&mut fn_ctx, functions);

        // Restore previous upvalues
        self.current_upvalues = saved_upvalues;
        self.open_upvalues = saved_open_upvalues;

        // Decrement call depth counter
        CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        result
    }

    /// Execute a method call with `this` binding
    ///
    /// # Arguments
    /// * `func_idx_or_closure` - The function index or closure encoded ID
    /// * `receiver` - The receiver object (this binding)
    /// * `args` - The function arguments
    /// * `functions` - The function registry
    ///
    /// # Returns
    /// The return value of the method
    fn call_method_with_this(
        &mut self,
        func_idx_or_closure: usize,
        receiver: Value,
        args: Vec<Value>,
        functions: &[BytecodeChunk],
    ) -> Result<Value, JsError> {
        // Check for recursion depth
        static CALL_DEPTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let depth = CALL_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if depth > 10000 {
            CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            return Err(JsError {
                kind: ErrorKind::RangeError,
                message: "Maximum call stack size exceeded".to_string(),
                stack: vec![],
                source_position: None,
            });
        }

        // Determine the actual function index and closure upvalues
        let (fn_idx, closure_upvalues) = if func_idx_or_closure >= 1_000_000 {
            let closure_id = func_idx_or_closure - 1_000_000;
            match self.closure_registry.get(&closure_id) {
                Some((func_idx, upvalues)) => (*func_idx, Some(upvalues.clone())),
                None => {
                    CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    return Err(JsError {
                        kind: ErrorKind::ReferenceError,
                        message: format!("Invalid closure ID: {}", closure_id),
                        stack: vec![],
                        source_position: None,
                    });
                }
            }
        } else {
            (func_idx_or_closure, None)
        };

        // Get the function bytecode
        let fn_bytecode = match functions.get(fn_idx) {
            Some(chunk) => chunk.clone(),
            None => {
                CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                return Err(JsError {
                    kind: ErrorKind::ReferenceError,
                    message: format!("Invalid function index: {}", fn_idx),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        // Create new execution context
        let mut fn_ctx = ExecutionContext::new(fn_bytecode);

        // Set arguments in registers starting from 0 (matching parser's parameter allocation)
        for (i, arg) in args.into_iter().enumerate() {
            fn_ctx.set_register(i, arg);
        }

        // Save current globals state and set `this` as a global variable
        // The parser emits LoadGlobal("this") for `this` expressions
        let saved_this = self.globals.get("this").cloned();
        self.globals.insert("this".to_string(), receiver);

        // Save and restore upvalues
        let saved_upvalues = std::mem::take(&mut self.current_upvalues);
        let saved_open_upvalues = std::mem::take(&mut self.open_upvalues);
        if let Some(upvalues) = closure_upvalues {
            self.current_upvalues = upvalues;
        }

        let result = self.execute(&mut fn_ctx, functions);

        // Restore previous `this` binding
        if let Some(prev_this) = saved_this {
            self.globals.insert("this".to_string(), prev_this);
        } else {
            self.globals.remove("this");
        }

        self.current_upvalues = saved_upvalues;
        self.open_upvalues = saved_open_upvalues;
        CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        result
    }

    /// Execute a constructor call (new operator)
    ///
    /// # Arguments
    /// * `func_idx_or_closure` - The function index or closure encoded ID
    /// * `args` - The constructor arguments
    /// * `functions` - The function registry
    ///
    /// # Returns
    /// The newly created instance
    fn call_constructor(
        &mut self,
        func_idx_or_closure: usize,
        args: Vec<Value>,
        functions: &[BytecodeChunk],
    ) -> Result<Value, JsError> {
        // Check for recursion depth
        static CALL_DEPTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let depth = CALL_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if depth > 10000 {
            CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            return Err(JsError {
                kind: ErrorKind::RangeError,
                message: "Maximum call stack size exceeded".to_string(),
                stack: vec![],
                source_position: None,
            });
        }

        // Determine the actual function index and closure upvalues
        let (fn_idx, closure_upvalues) = if func_idx_or_closure >= 1_000_000 {
            let closure_id = func_idx_or_closure - 1_000_000;
            match self.closure_registry.get(&closure_id) {
                Some((func_idx, upvalues)) => (*func_idx, Some(upvalues.clone())),
                None => {
                    CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    return Err(JsError {
                        kind: ErrorKind::ReferenceError,
                        message: format!("Invalid closure ID: {}", closure_id),
                        stack: vec![],
                        source_position: None,
                    });
                }
            }
        } else {
            (func_idx_or_closure, None)
        };

        // Get the function bytecode
        let fn_bytecode = match functions.get(fn_idx) {
            Some(chunk) => chunk.clone(),
            None => {
                CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                return Err(JsError {
                    kind: ErrorKind::ReferenceError,
                    message: format!("Invalid function index: {}", fn_idx),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        // Create new instance object
        let instance = if let Some(ref heap) = self.heap {
            let gc_object = heap.create_object();
            let boxed: Box<dyn Any> = Box::new(gc_object);
            Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>)
        } else {
            // Fallback
            Value::HeapObject(0)
        };

        // Create new execution context
        let mut fn_ctx = ExecutionContext::new(fn_bytecode);

        // Set arguments in registers starting from 0 (matching parser's parameter allocation)
        // Parameters are allocated registers 0, 1, 2... by the parser
        for (i, arg) in args.into_iter().enumerate() {
            fn_ctx.set_register(i, arg);
        }

        // Save current globals state and set `this` as a global variable
        // The parser emits LoadGlobal("this") for `this` expressions
        let saved_this = self.globals.get("this").cloned();
        self.globals.insert("this".to_string(), instance.clone());

        // Save and restore upvalues
        let saved_upvalues = std::mem::take(&mut self.current_upvalues);
        let saved_open_upvalues = std::mem::take(&mut self.open_upvalues);
        if let Some(upvalues) = closure_upvalues {
            self.current_upvalues = upvalues;
        }

        let result = self.execute(&mut fn_ctx, functions);

        // Restore previous `this` binding
        if let Some(prev_this) = saved_this {
            self.globals.insert("this".to_string(), prev_this);
        } else {
            self.globals.remove("this");
        }

        self.current_upvalues = saved_upvalues;
        self.open_upvalues = saved_open_upvalues;
        CALL_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        // If constructor returns an object, use that; otherwise return the instance
        match result {
            Ok(Value::Undefined) | Ok(Value::Null) => Ok(instance),
            Ok(Value::NativeObject(_)) | Ok(Value::HeapObject(_)) => result,
            Ok(_) => Ok(instance), // Primitive return values are ignored
            Err(e) => Err(e),
        }
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
        match (&a, &b) {
            // String concatenation has priority
            (Value::String(s1), Value::String(s2)) => {
                Ok(Value::String(format!("{}{}", s1, s2)))
            }
            (Value::String(s), _) => {
                let b_str = self.to_string_value(&b);
                Ok(Value::String(format!("{}{}", s, b_str)))
            }
            (_, Value::String(s)) => {
                let a_str = self.to_string_value(&a);
                Ok(Value::String(format!("{}{}", a_str, s)))
            }
            // Numeric addition
            (Value::Smi(x), Value::Smi(y)) => Ok(Value::Smi(x.wrapping_add(*y))),
            (Value::Double(x), Value::Double(y)) => Ok(Value::Double(*x + *y)),
            (Value::Smi(x), Value::Double(y)) => Ok(Value::Double(*x as f64 + *y)),
            (Value::Double(x), Value::Smi(y)) => Ok(Value::Double(*x + *y as f64)),
            _ => Ok(Value::Double(f64::NAN)),
        }
    }

    fn to_string_value(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Smi(n) => n.to_string(),
            Value::Double(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::HeapObject(id) => format!("[object Object {}]", id),
            Value::NativeObject(_) => "[object Object]".to_string(),
            Value::NativeFunction(name) => format!("function {}() {{ [native code] }}", name),
        }
    }

    /// Convert a value to a property key (string)
    fn to_property_key(&self, value: &Value) -> String {
        match value {
            Value::Smi(n) => n.to_string(),
            Value::Double(n) => {
                // Check if it's an integer
                if n.fract() == 0.0 && *n >= 0.0 && *n <= (i32::MAX as f64) {
                    (*n as i32).to_string()
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            _ => "[object]".to_string(),
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
            Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
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
        // Globals contains console, Math, and Promise by default
        assert_eq!(dispatcher.globals.len(), 3);
        assert!(dispatcher.globals.contains_key("console"));
        assert!(dispatcher.globals.contains_key("Math"));
        assert!(dispatcher.globals.contains_key("Promise"));
        assert!(dispatcher.stack.is_empty());
        assert!(dispatcher.open_upvalues.is_empty());
        assert!(dispatcher.current_upvalues.is_empty());
    }

    #[test]
    fn test_dispatcher_default() {
        let dispatcher = Dispatcher::default();
        // Default has console, Math, and Promise globals
        assert_eq!(dispatcher.globals.len(), 3);
        assert!(dispatcher.globals.contains_key("console"));
        assert!(dispatcher.globals.contains_key("Math"));
        assert!(dispatcher.globals.contains_key("Promise"));
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
