//! Dispatch loop for bytecode execution
//!
//! Handles individual opcode execution.

use async_runtime::PromiseState;
use bytecode_system::{BytecodeChunk, Opcode, UpvalueDescriptor};
use builtins::{BigIntValue, ConsoleObject, JSONObject, JsValue as BuiltinValue, MathObject, NumberObject};
use core_types::{ErrorKind, JsError, Value};
use num_traits::Zero;
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

        // Inject JSON global object
        globals.insert(
            "JSON".to_string(),
            Value::NativeObject(Rc::new(RefCell::new(JSONObject)) as Rc<RefCell<dyn Any>>),
        );

        // Inject Array constructor
        globals.insert(
            "Array".to_string(),
            Value::NativeFunction("Array".to_string()),
        );

        // Inject Error constructors
        globals.insert(
            "Error".to_string(),
            Value::NativeFunction("Error".to_string()),
        );
        globals.insert(
            "TypeError".to_string(),
            Value::NativeFunction("TypeError".to_string()),
        );
        globals.insert(
            "ReferenceError".to_string(),
            Value::NativeFunction("ReferenceError".to_string()),
        );
        globals.insert(
            "SyntaxError".to_string(),
            Value::NativeFunction("SyntaxError".to_string()),
        );
        globals.insert(
            "RangeError".to_string(),
            Value::NativeFunction("RangeError".to_string()),
        );
        globals.insert(
            "URIError".to_string(),
            Value::NativeFunction("URIError".to_string()),
        );
        globals.insert(
            "EvalError".to_string(),
            Value::NativeFunction("EvalError".to_string()),
        );

        // Inject Number constructor
        globals.insert(
            "Number".to_string(),
            Value::NativeFunction("Number".to_string()),
        );

        // Inject Object constructor
        globals.insert(
            "Object".to_string(),
            Value::NativeFunction("Object".to_string()),
        );

        // Inject String constructor
        globals.insert(
            "String".to_string(),
            Value::NativeFunction("String".to_string()),
        );

        // Inject Boolean constructor
        globals.insert(
            "Boolean".to_string(),
            Value::NativeFunction("Boolean".to_string()),
        );

        // Inject Array constructor
        globals.insert(
            "Array".to_string(),
            Value::NativeFunction("Array".to_string()),
        );

        // Inject global constants
        globals.insert("NaN".to_string(), Value::Double(f64::NAN));
        globals.insert("Infinity".to_string(), Value::Double(f64::INFINITY));
        globals.insert("undefined".to_string(), Value::Undefined);

        // Inject global functions
        globals.insert(
            "isNaN".to_string(),
            Value::NativeFunction("isNaN".to_string()),
        );
        globals.insert(
            "isFinite".to_string(),
            Value::NativeFunction("isFinite".to_string()),
        );
        globals.insert(
            "parseInt".to_string(),
            Value::NativeFunction("parseInt".to_string()),
        );
        globals.insert(
            "parseFloat".to_string(),
            Value::NativeFunction("parseFloat".to_string()),
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
        self.heap = Some(heap.clone());

        // Initialize Error.prototype objects now that we have a heap
        let error_types = ["Error", "TypeError", "ReferenceError", "SyntaxError",
                          "RangeError", "URIError", "EvalError"];

        for error_type in &error_types {
            let proto_obj = heap.create_object();
            let proto_key = format!("{}.prototype", error_type);
            let boxed: Box<dyn Any> = Box::new(proto_obj);
            let proto_value = Value::NativeObject(
                Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
            );
            self.globals.insert(proto_key, proto_value);
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
            bytecode_system::Value::BigInt(n) => Value::BigInt(n.clone()),
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
                Opcode::Exp => {
                    let b = self.stack.pop().unwrap_or(Value::Undefined);
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.exponentiate(a, b)?;
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
                Opcode::Typeof => {
                    let a = self.stack.pop().unwrap_or(Value::Undefined);
                    // typeof operator - returns type as string
                    self.stack.push(Value::String(a.type_of()));
                }
                Opcode::Void => {
                    // void operator - discard value and push undefined
                    let _discarded = self.stack.pop();
                    self.stack.push(Value::Undefined);
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
                Opcode::Instanceof => {
                    let constructor = self.stack.pop().unwrap_or(Value::Undefined);
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.instanceof_check(obj, constructor);
                    self.stack.push(result);
                }
                Opcode::In => {
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);
                    let prop = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = self.in_check(prop, obj);
                    self.stack.push(result);
                }
                Opcode::DeleteProperty(ref _prop_name) => {
                    let _obj = self.stack.pop().unwrap_or(Value::Undefined);
                    // Delete property from object - for now always return true
                    // TODO: implement actual property deletion on HeapObject
                    self.stack.push(Value::Boolean(true));
                }
                Opcode::DeleteGlobal(ref var_name) => {
                    // Delete global variable - always return true for now
                    // In strict mode this would throw, but we're lenient
                    self.globals.remove(var_name);
                    self.stack.push(Value::Boolean(true));
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
                                    // Check if this is an array (has numeric "length" property)
                                    let is_array = matches!(gc_object.get("length"), Value::Smi(_));

                                    let value = if is_array {
                                        // Check for array prototype methods first
                                        match name.as_str() {
                                            "map" => Value::NativeFunction("Array.prototype.map".to_string()),
                                            "filter" => Value::NativeFunction("Array.prototype.filter".to_string()),
                                            "forEach" => Value::NativeFunction("Array.prototype.forEach".to_string()),
                                            "reduce" => Value::NativeFunction("Array.prototype.reduce".to_string()),
                                            "find" => Value::NativeFunction("Array.prototype.find".to_string()),
                                            "findIndex" => Value::NativeFunction("Array.prototype.findIndex".to_string()),
                                            "some" => Value::NativeFunction("Array.prototype.some".to_string()),
                                            "every" => Value::NativeFunction("Array.prototype.every".to_string()),
                                            "includes" => Value::NativeFunction("Array.prototype.includes".to_string()),
                                            "indexOf" => Value::NativeFunction("Array.prototype.indexOf".to_string()),
                                            "push" => Value::NativeFunction("Array.prototype.push".to_string()),
                                            "pop" => Value::NativeFunction("Array.prototype.pop".to_string()),
                                            "shift" => Value::NativeFunction("Array.prototype.shift".to_string()),
                                            "unshift" => Value::NativeFunction("Array.prototype.unshift".to_string()),
                                            "slice" => Value::NativeFunction("Array.prototype.slice".to_string()),
                                            "splice" => Value::NativeFunction("Array.prototype.splice".to_string()),
                                            "concat" => Value::NativeFunction("Array.prototype.concat".to_string()),
                                            "join" => Value::NativeFunction("Array.prototype.join".to_string()),
                                            "reverse" => Value::NativeFunction("Array.prototype.reverse".to_string()),
                                            "sort" => Value::NativeFunction("Array.prototype.sort".to_string()),
                                            _ => gc_object.get(&name), // Regular property access
                                        }
                                    } else {
                                        // Not an array - check for Object.prototype methods first
                                        match name.as_str() {
                                            "toString" => Value::NativeFunction("Object.prototype.toString".to_string()),
                                            "valueOf" => Value::NativeFunction("Object.prototype.valueOf".to_string()),
                                            "hasOwnProperty" => Value::NativeFunction("Object.prototype.hasOwnProperty".to_string()),
                                            "propertyIsEnumerable" => Value::NativeFunction("Object.prototype.propertyIsEnumerable".to_string()),
                                            "isPrototypeOf" => Value::NativeFunction("Object.prototype.isPrototypeOf".to_string()),
                                            "toLocaleString" => Value::NativeFunction("Object.prototype.toLocaleString".to_string()),
                                            _ => gc_object.get(&name)  // Regular property access
                                        }
                                    };
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
                            } else if borrowed.is::<JSONObject>() {
                                // Return JSON method based on property name
                                match name.as_str() {
                                    "stringify" => self
                                        .stack
                                        .push(Value::NativeFunction("JSON.stringify".to_string())),
                                    "parse" => self
                                        .stack
                                        .push(Value::NativeFunction("JSON.parse".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else {
                                // Unknown NativeObject type
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
                            } else if matches!(fn_name.as_str(), "Error" | "TypeError" | "ReferenceError" |
                                             "SyntaxError" | "RangeError" | "URIError" | "EvalError") {
                                // Handle Error constructor properties
                                match name.as_str() {
                                    "prototype" => {
                                        // Return the pre-initialized prototype object
                                        let proto_key = format!("{}.prototype", fn_name);
                                        if let Some(proto) = self.globals.get(&proto_key) {
                                            self.stack.push(proto.clone());
                                        } else {
                                            // Prototype should have been initialized in set_heap()
                                            self.stack.push(Value::Undefined);
                                        }
                                    }
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else if fn_name == "Number" {
                                // Handle Number constructor properties
                                match name.as_str() {
                                    "NaN" => self.stack.push(Value::Double(NumberObject::NAN)),
                                    "POSITIVE_INFINITY" => self.stack.push(Value::Double(NumberObject::POSITIVE_INFINITY)),
                                    "NEGATIVE_INFINITY" => self.stack.push(Value::Double(NumberObject::NEGATIVE_INFINITY)),
                                    "MAX_VALUE" => self.stack.push(Value::Double(NumberObject::MAX_VALUE)),
                                    "MIN_VALUE" => self.stack.push(Value::Double(NumberObject::MIN_VALUE)),
                                    "MAX_SAFE_INTEGER" => self.stack.push(Value::Double(NumberObject::MAX_SAFE_INTEGER)),
                                    "MIN_SAFE_INTEGER" => self.stack.push(Value::Double(NumberObject::MIN_SAFE_INTEGER)),
                                    "EPSILON" => self.stack.push(Value::Double(NumberObject::EPSILON)),
                                    "isNaN" => self.stack.push(Value::NativeFunction("Number.isNaN".to_string())),
                                    "isFinite" => self.stack.push(Value::NativeFunction("Number.isFinite".to_string())),
                                    "isInteger" => self.stack.push(Value::NativeFunction("Number.isInteger".to_string())),
                                    "isSafeInteger" => self.stack.push(Value::NativeFunction("Number.isSafeInteger".to_string())),
                                    "parseInt" => self.stack.push(Value::NativeFunction("Number.parseInt".to_string())),
                                    "parseFloat" => self.stack.push(Value::NativeFunction("Number.parseFloat".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else if fn_name == "Array" {
                                // Handle Array constructor properties
                                match name.as_str() {
                                    "isArray" => self.stack.push(Value::NativeFunction("Array.isArray".to_string())),
                                    "of" => self.stack.push(Value::NativeFunction("Array.of".to_string())),
                                    "from" => self.stack.push(Value::NativeFunction("Array.from".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }
                            } else if fn_name == "Object" {
                                // Handle Object constructor properties
                                match name.as_str() {
                                    "keys" => self.stack.push(Value::NativeFunction("Object.keys".to_string())),
                                    "values" => self.stack.push(Value::NativeFunction("Object.values".to_string())),
                                    "entries" => self.stack.push(Value::NativeFunction("Object.entries".to_string())),
                                    "assign" => self.stack.push(Value::NativeFunction("Object.assign".to_string())),
                                    _ => self.stack.push(Value::Undefined),
                                }

                            } else {
                                self.stack.push(Value::Undefined);
                            }
                        }
                        Value::String(s) => {
                            // String primitive - handle length and prototype methods
                            let value = match name.as_str() {
                                "length" => Value::Smi(s.len() as i32),
                                "toString" | "valueOf" => Value::NativeFunction("String.prototype.toString".to_string()),
                                "charAt" => Value::NativeFunction("String.prototype.charAt".to_string()),
                                "charCodeAt" => Value::NativeFunction("String.prototype.charCodeAt".to_string()),
                                "indexOf" => Value::NativeFunction("String.prototype.indexOf".to_string()),
                                "lastIndexOf" => Value::NativeFunction("String.prototype.lastIndexOf".to_string()),
                                "slice" => Value::NativeFunction("String.prototype.slice".to_string()),
                                "substring" => Value::NativeFunction("String.prototype.substring".to_string()),
                                "toLowerCase" => Value::NativeFunction("String.prototype.toLowerCase".to_string()),
                                "toUpperCase" => Value::NativeFunction("String.prototype.toUpperCase".to_string()),
                                "trim" => Value::NativeFunction("String.prototype.trim".to_string()),
                                "trimStart" | "trimLeft" => Value::NativeFunction("String.prototype.trimStart".to_string()),
                                "trimEnd" | "trimRight" => Value::NativeFunction("String.prototype.trimEnd".to_string()),
                                "split" => Value::NativeFunction("String.prototype.split".to_string()),
                                "concat" => Value::NativeFunction("String.prototype.concat".to_string()),
                                "includes" => Value::NativeFunction("String.prototype.includes".to_string()),
                                "startsWith" => Value::NativeFunction("String.prototype.startsWith".to_string()),
                                "endsWith" => Value::NativeFunction("String.prototype.endsWith".to_string()),
                                "repeat" => Value::NativeFunction("String.prototype.repeat".to_string()),
                                "padStart" => Value::NativeFunction("String.prototype.padStart".to_string()),
                                "padEnd" => Value::NativeFunction("String.prototype.padEnd".to_string()),
                                _ => Value::Undefined,
                            };
                            self.stack.push(value);
                        }
                        Value::Smi(_) | Value::Double(_) => {
                            // Number primitive - handle prototype methods
                            let value = match name.as_str() {
                                "toString" => Value::NativeFunction("Number.prototype.toString".to_string()),
                                "valueOf" => Value::NativeFunction("Number.prototype.valueOf".to_string()),
                                "toFixed" => Value::NativeFunction("Number.prototype.toFixed".to_string()),
                                _ => Value::Undefined,
                            };
                            self.stack.push(value);
                        }
                        Value::Boolean(_) => {
                            // Boolean primitive - handle prototype methods
                            let value = match name.as_str() {
                                "toString" => Value::NativeFunction("Boolean.prototype.toString".to_string()),
                                "valueOf" => Value::NativeFunction("Boolean.prototype.valueOf".to_string()),
                                _ => Value::Undefined,
                            };
                            self.stack.push(value);
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
                                    gc_object.set(name, value.clone());
                                }
                            }
                            // For other NativeObjects, we just ignore the store (non-extensible)
                        }
                        Value::NativeFunction(ref fn_name) if matches!(fn_name.as_str(),
                            "Error" | "TypeError" | "ReferenceError" | "SyntaxError" |
                            "RangeError" | "URIError" | "EvalError") => {
                            // Handle Error constructor property assignment (e.g., Error.prototype = ...)
                            if name == "prototype" {
                                let proto_key = format!("{}.prototype", fn_name);
                                self.globals.insert(proto_key, value.clone());
                            }
                            // Ignore other property stores on Error constructors
                        }
                        _ => {
                            // Ignore stores to non-objects
                        }
                    }
                    // Push the assigned value back - assignment expressions return the assigned value
                    self.stack.push(value);
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
                                    gc_object.set(key, value.clone());
                                }
                            }
                        }
                        _ => {
                            // Ignore index stores to non-objects
                        }
                    }
                    // Push the assigned value back - assignment expressions return the assigned value
                    self.stack.push(value);
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
                Opcode::CreateRegExp(pattern_idx, flags_idx) => {
                    // Create a RegExp object
                    // For now, store pattern and flags as a string representation
                    // Real implementation would create proper RegExp object
                    let pattern = if let Some(bytecode_system::Value::String(p)) =
                        ctx.bytecode.constants.get(pattern_idx)
                    {
                        p.clone()
                    } else {
                        "".to_string()
                    };
                    let flags = if let Some(bytecode_system::Value::String(f)) =
                        ctx.bytecode.constants.get(flags_idx)
                    {
                        f.clone()
                    } else {
                        "".to_string()
                    };

                    // Create a simple object to represent the RegExp
                    // A full implementation would use a proper RegExp type
                    if let Some(ref heap) = self.heap {
                        let mut gc_object = heap.create_object();
                        // Set source and flags properties
                        gc_object.set("source".to_string(), Value::String(pattern.clone()));
                        gc_object.set("flags".to_string(), Value::String(flags.clone()));

                        let boxed: Box<dyn Any> = Box::new(gc_object);
                        let value =
                            Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>);
                        self.stack.push(value);
                    } else {
                        // Fallback: push as string representation
                        self.stack.push(Value::String(format!("/{}/{}", pattern, flags)));
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
                            // Check if this is a prototype method that needs the receiver
                            if name.starts_with("Array.prototype.") {
                                let result = self.call_array_prototype_method(&name, receiver, args, functions)?;
                                self.stack.push(result);
                            } else if name.starts_with("Object.prototype.") {
                                let result = self.call_object_prototype_method(&name, receiver, args)?;
                                self.stack.push(result);
                            } else if name.starts_with("String.prototype.") {
                                let result = self.call_string_prototype_method(&name, receiver, args)?;
                                self.stack.push(result);
                            } else if name.starts_with("Number.prototype.") {
                                let result = self.call_number_prototype_method(&name, receiver)?;
                                self.stack.push(result);
                            } else {
                                let result = self.call_native_function(&name, args)?;
                                self.stack.push(result);
                            }
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
            // JSON methods
            "JSON.stringify" => {
                if let Some(value) = args.first() {
                    let json_str = self.value_to_json_string(value);
                    Ok(Value::String(json_str))
                } else {
                    Ok(Value::String("undefined".to_string()))
                }
            }
            "JSON.parse" => {
                if let Some(Value::String(s)) = args.first() {
                    self.parse_json_string(s)
                } else {
                    Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "JSON.parse requires a string argument".to_string(),
                        stack: vec![],
                        source_position: None,
                    })
                }
            }
            // Array prototype methods (using method receiver pattern)
            // Note: For prototype methods, the first arg is `this` (the array)
            "Array.prototype.push" => {
                // This would need the array reference passed - simplified for now
                Err(JsError {
                    kind: ErrorKind::TypeError,
                    message: "Array.prototype.push is not yet implemented for runtime calls".to_string(),
                    stack: vec![],
                    source_position: None,
                })
            }
            "Array.prototype.map" | "Array.prototype.filter" | "Array.prototype.forEach" |
            "Array.prototype.reduce" | "Array.prototype.find" | "Array.prototype.findIndex" |
            "Array.prototype.some" | "Array.prototype.every" | "Array.prototype.includes" |
            "Array.prototype.indexOf" | "Array.prototype.pop" | "Array.prototype.shift" |
            "Array.prototype.unshift" | "Array.prototype.slice" | "Array.prototype.splice" |
            "Array.prototype.concat" | "Array.prototype.join" | "Array.prototype.reverse" |
            "Array.prototype.sort" => {
                // Array prototype methods require callback integration which
                // needs the call stack context. Return a descriptive error for now.
                Err(JsError {
                    kind: ErrorKind::TypeError,
                    message: format!("{} requires callback support not yet implemented", name),
                    stack: vec![],
                    source_position: None,
                })
            }
            // Global functions
            "isNaN" => {
                let n = args.first().map(|v| self.to_number(v)).unwrap_or(f64::NAN);
                Ok(Value::Boolean(n.is_nan()))
            }
            "isFinite" => {
                let n = args.first().map(|v| self.to_number(v)).unwrap_or(f64::NAN);
                Ok(Value::Boolean(n.is_finite()))
            }
            "parseInt" => {
                let s = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let radix = args.get(1).map(|v| self.to_number(v) as u32);
                Ok(Value::Double(NumberObject::parse_int(&s, radix)))
            }
            "parseFloat" => {
                let s = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                Ok(Value::Double(NumberObject::parse_float(&s)))
            }
            // Number constructor and methods
            "Number" => {
                // Number() type conversion
                let n = args.first().map(|v| self.to_number(v)).unwrap_or(0.0);
                Ok(Value::Double(n))
            }
            // String constructor
            "String" => {
                // String() type conversion
                let s = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                // Create a String object wrapper (for `new String()`)
                // For now, we return a string primitive which works for most cases
                Ok(Value::String(s.into()))
            }
            // Boolean constructor
            "Boolean" => {
                // Boolean() type conversion
                let b = args.first().map(|v| self.to_boolean(v)).unwrap_or(false);
                Ok(Value::Boolean(b))
            }
            // Array constructor
            "Array" => {
                // Array() constructor
                // Create an array using the heap if available
                if let Some(ref heap) = self.heap {
                    let mut arr_obj = heap.create_object();
                    if args.is_empty() {
                        // Empty array - just set length
                        arr_obj.set("length".to_string(), Value::Smi(0));
                    } else if args.len() == 1 {
                        // Single argument - if number, create array with that length
                        match args.first() {
                            Some(Value::Double(n)) => {
                                arr_obj.set("length".to_string(), Value::Smi(*n as i32));
                            }
                            Some(Value::Smi(n)) => {
                                arr_obj.set("length".to_string(), Value::Smi(*n));
                            }
                            _ => {
                                // Single non-number argument - create array with that element
                                arr_obj.set("0".to_string(), args[0].clone());
                                arr_obj.set("length".to_string(), Value::Smi(1));
                            }
                        }
                    } else {
                        // Multiple arguments - create array with those elements
                        for (i, arg) in args.iter().enumerate() {
                            arr_obj.set(i.to_string(), arg.clone());
                        }
                        arr_obj.set("length".to_string(), Value::Smi(args.len() as i32));
                    }
                    let boxed: Box<dyn Any> = Box::new(arr_obj);
                    Ok(Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>))
                } else {
                    // Fallback when heap not available - return empty array-like object
                    Ok(Value::NativeObject(Rc::new(RefCell::new(Vec::<Value>::new())) as Rc<RefCell<dyn Any>>))
                }
            }
            // Object constructor
            "Object" => {
                // Object() constructor - creates a new object
                if let Some(ref heap) = self.heap {
                    let obj = heap.create_object();
                    let boxed: Box<dyn Any> = Box::new(obj);
                    Ok(Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>))
                } else {
                    // Fallback when heap not available
                    Ok(Value::NativeObject(Rc::new(RefCell::new(std::collections::HashMap::<String, Value>::new())) as Rc<RefCell<dyn Any>>))
                }
            }
            "Number.isNaN" => {
                // Number.isNaN - strict check, doesn't coerce
                if let Some(Value::Double(n)) = args.first() {
                    Ok(Value::Boolean(n.is_nan()))
                } else if let Some(Value::Smi(_)) = args.first() {
                    Ok(Value::Boolean(false)) // SMIs are never NaN
                } else {
                    Ok(Value::Boolean(false)) // non-numbers return false
                }
            }
            "Number.isFinite" => {
                // Number.isFinite - strict check, doesn't coerce
                if let Some(Value::Double(n)) = args.first() {
                    Ok(Value::Boolean(n.is_finite()))
                } else if let Some(Value::Smi(_)) = args.first() {
                    Ok(Value::Boolean(true)) // SMIs are always finite
                } else {
                    Ok(Value::Boolean(false)) // non-numbers return false
                }
            }
            "Number.isInteger" => {
                if let Some(Value::Double(n)) = args.first() {
                    Ok(Value::Boolean(NumberObject::is_integer(*n)))
                } else if let Some(Value::Smi(_)) = args.first() {
                    Ok(Value::Boolean(true)) // SMIs are always integers
                } else {
                    Ok(Value::Boolean(false))
                }
            }
            "Number.isSafeInteger" => {
                if let Some(Value::Double(n)) = args.first() {
                    Ok(Value::Boolean(NumberObject::is_safe_integer(*n)))
                } else if let Some(Value::Smi(_)) = args.first() {
                    Ok(Value::Boolean(true)) // SMIs are always safe integers
                } else {
                    Ok(Value::Boolean(false))
                }
            }
            "Number.parseInt" => {
                let s = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let radix = args.get(1).map(|v| self.to_number(v) as u32);
                Ok(Value::Double(NumberObject::parse_int(&s, radix)))
            }
            "Number.parseFloat" => {
                let s = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                Ok(Value::Double(NumberObject::parse_float(&s)))
            }
            // Error constructors
            "Array.isArray" => {
                if let Some(value) = args.first() {
                    // Check if the value is a NativeObject with a "length" property (array)
                    let is_array = if let Value::NativeObject(obj_ref) = value {
                        let borrowed = obj_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                matches!(gc_object.get("length"), Value::Smi(_))
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    Ok(Value::Boolean(is_array))
                } else {
                    Ok(Value::Boolean(false))
                }
            }
            "Array.of" => {
                // Create array from arguments
                if let Some(ref heap) = self.heap {
                    let mut gc_object = heap.create_object();

                    // Store all arguments as array elements
                    for (i, arg) in args.iter().enumerate() {
                        gc_object.set(i.to_string(), arg.clone());
                    }

                    // Set length property
                    gc_object.set("length".to_string(), Value::Smi(args.len() as i32));

                    // Wrap and return as NativeObject
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Ok(Value::NativeObject(
                        Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "Array.from" => {
                // Array.from(arrayLike, mapFn?, thisArg?)
                // Basic implementation: handle arrays and strings
                if let Some(array_like) = args.first() {
                    if let Some(ref heap) = self.heap {
                        let mut gc_object = heap.create_object();
                        let mut elements = Vec::new();

                        match array_like {
                            Value::NativeObject(obj_ref) => {
                                // Try to get length property from NativeObject
                                let borrowed = obj_ref.borrow();
                                if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                    if let Some(source_obj) = gc_obj.downcast_ref::<GCObject>() {
                                        if let Value::Smi(len) = source_obj.get("length") {
                                            // It's an array-like object
                                            for i in 0..len {
                                                let elem = source_obj.get(&i.to_string());
                                                elements.push(elem);
                                            }
                                        }
                                    }
                                }
                            }
                            Value::String(s) => {
                                // Convert string to array of characters
                                for (i, ch) in s.chars().enumerate() {
                                    elements.push(Value::String(ch.to_string()));
                                }
                            }
                            _ => {
                                // For other types, create empty array
                            }
                        }

                        // Create new array with elements
                        for (i, elem) in elements.iter().enumerate() {
                            gc_object.set(i.to_string(), elem.clone());
                        }

                        gc_object.set("length".to_string(), Value::Smi(elements.len() as i32));

                        // Wrap and return as NativeObject
                        let boxed: Box<dyn Any> = Box::new(gc_object);
                        Ok(Value::NativeObject(
                            Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                        ))
                    } else {
                        Ok(Value::Undefined)
                    }
                } else {
                    Ok(Value::Undefined)
                }
            }
            // Object static methods
            "Object.keys" => {
                if let Some(value) = args.first() {
                    match value {
                        Value::NativeObject(obj_ref) => {
                            let borrowed = obj_ref.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                    // Get all keys from the object
                                    let keys = gc_object.keys();
                                    drop(borrowed);
                                    
                                    // Create an array with the keys
                                    if let Some(ref heap) = self.heap {
                                        let mut result_obj = heap.create_object();
                                        for (i, key) in keys.iter().enumerate() {
                                            result_obj.set(i.to_string(), Value::String(key.clone()));
                                        }
                                        result_obj.set("length".to_string(), Value::Smi(keys.len() as i32));
                                        
                                        let boxed: Box<dyn Any> = Box::new(result_obj);
                                        return Ok(Value::NativeObject(
                                            Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                        ));
                                    }
                                }
                            }
                            // Empty array fallback
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                        _ => {
                            // Empty array for non-objects
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                    }
                } else {
                    Err(JsError {
                        kind: ErrorKind::TypeError,
                        message: "Object.keys requires an argument".to_string(),
                        stack: vec![],
                        source_position: None,
                    })
                }
            }
            "Object.values" => {
                if let Some(value) = args.first() {
                    match value {
                        Value::NativeObject(obj_ref) => {
                            let borrowed = obj_ref.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                    // Get all keys and values
                                    let keys = gc_object.keys();
                                    let values: Vec<Value> = keys.iter()
                                        .map(|k| gc_object.get(k))
                                        .collect();
                                    drop(borrowed);
                                    
                                    // Create an array with the values
                                    if let Some(ref heap) = self.heap {
                                        let mut result_obj = heap.create_object();
                                        for (i, val) in values.iter().enumerate() {
                                            result_obj.set(i.to_string(), val.clone());
                                        }
                                        result_obj.set("length".to_string(), Value::Smi(values.len() as i32));
                                        
                                        let boxed: Box<dyn Any> = Box::new(result_obj);
                                        return Ok(Value::NativeObject(
                                            Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                        ));
                                    }
                                }
                            }
                            // Empty array fallback
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                        _ => {
                            // Empty array for non-objects
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                    }
                } else {
                    Err(JsError {
                        kind: ErrorKind::TypeError,
                        message: "Object.values requires an argument".to_string(),
                        stack: vec![],
                        source_position: None,
                    })
                }
            }
            "Object.entries" => {
                if let Some(value) = args.first() {
                    match value {
                        Value::NativeObject(obj_ref) => {
                            let borrowed = obj_ref.borrow();
                            if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                    // Get all keys and create [key, value] pairs
                                    let keys = gc_object.keys();
                                    let entries: Vec<(String, Value)> = keys.iter()
                                        .map(|k| (k.clone(), gc_object.get(k)))
                                        .collect();
                                    drop(borrowed);
                                    
                                    // Create an array with [key, value] pairs
                                    if let Some(ref heap) = self.heap {
                                        let mut result_obj = heap.create_object();
                                        for (i, (key, val)) in entries.iter().enumerate() {
                                            // Create inner array for [key, value]
                                            let mut pair_obj = heap.create_object();
                                            pair_obj.set("0".to_string(), Value::String(key.clone()));
                                            pair_obj.set("1".to_string(), val.clone());
                                            pair_obj.set("length".to_string(), Value::Smi(2));
                                            
                                            let pair_boxed: Box<dyn Any> = Box::new(pair_obj);
                                            let pair_value = Value::NativeObject(
                                                Rc::new(RefCell::new(pair_boxed)) as Rc<RefCell<dyn Any>>
                                            );
                                            result_obj.set(i.to_string(), pair_value);
                                        }
                                        result_obj.set("length".to_string(), Value::Smi(entries.len() as i32));
                                        
                                        let boxed: Box<dyn Any> = Box::new(result_obj);
                                        return Ok(Value::NativeObject(
                                            Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                        ));
                                    }
                                }
                            }
                            // Empty array fallback
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                        _ => {
                            // Empty array for non-objects
                            if let Some(ref heap) = self.heap {
                                let mut empty = heap.create_object();
                                empty.set("length".to_string(), Value::Smi(0));
                                let boxed: Box<dyn Any> = Box::new(empty);
                                Ok(Value::NativeObject(
                                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                                ))
                            } else {
                                Ok(Value::Undefined)
                            }
                        }
                    }
                } else {
                    Err(JsError {
                        kind: ErrorKind::TypeError,
                        message: "Object.entries requires an argument".to_string(),
                        stack: vec![],
                        source_position: None,
                    })
                }
            }
            "Object.assign" => {
                // Object.assign(target, ...sources)
                if args.is_empty() {
                    return Err(JsError {
                        kind: ErrorKind::TypeError,
                        message: "Object.assign requires at least one argument".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                }
                
                let target = args[0].clone();
                
                // Copy properties from all sources to target
                if let Value::NativeObject(target_ref) = &target {
                    for source_val in args.iter().skip(1) {
                        if let Value::NativeObject(source_ref) = source_val {
                            let source_borrowed = source_ref.borrow();
                            if let Some(source_gc_obj) = source_borrowed.downcast_ref::<Box<dyn Any>>() {
                                if let Some(source_gc_object) = source_gc_obj.downcast_ref::<GCObject>() {
                                    // Get all keys from source
                                    let keys = source_gc_object.keys();
                                    let key_value_pairs: Vec<(String, Value)> = keys.iter()
                                        .map(|k| (k.clone(), source_gc_object.get(k)))
                                        .collect();
                                    drop(source_borrowed);
                                    
                                    // Copy to target
                                    let mut target_borrowed = target_ref.borrow_mut();
                                    if let Some(target_gc_obj) = target_borrowed.downcast_mut::<Box<dyn Any>>() {
                                        if let Some(target_gc_object) = target_gc_obj.downcast_mut::<GCObject>() {
                                            for (key, value) in key_value_pairs {
                                                target_gc_object.set(key, value);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(target)
            }
            "Error" => self.create_error_object("Error", args),
            "TypeError" => self.create_error_object("TypeError", args),
            "ReferenceError" => self.create_error_object("ReferenceError", args),
            "SyntaxError" => self.create_error_object("SyntaxError", args),
            "RangeError" => self.create_error_object("RangeError", args),
            "URIError" => self.create_error_object("URIError", args),
            "EvalError" => self.create_error_object("EvalError", args),
            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("{} is not a function", name),
                stack: vec![],
                source_position: None,
            }),
        }
    }

    /// Create an Error object with the given name and message
    ///
    /// # Arguments
    ///
    /// * `error_name` - The name of the error type (e.g., "Error", "TypeError")
    /// * `args` - Constructor arguments (first arg is the message)
    ///
    /// # Returns
    ///
    /// A NativeObject wrapping an error with `name` and `message` properties
    fn create_error_object(&self, error_name: &str, args: Vec<Value>) -> Result<Value, JsError> {
        // Extract message from arguments
        let message = args.first()
            .map(|v| self.to_string_value(v))
            .unwrap_or_else(|| String::new());

        // Create error object using heap if available
        if let Some(ref heap) = self.heap {
            let mut error_obj = heap.create_object();
            error_obj.set("name".to_string(), Value::String(error_name.to_string()));
            error_obj.set("message".to_string(), Value::String(message));

            // Wrap the GCObject in Box<dyn Any> then in NativeObject
            let boxed: Box<dyn Any> = Box::new(error_obj);
            Ok(Value::NativeObject(
                Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
            ))
        } else {
            // Fallback: create a simple NativeObject without heap
            // This is a simplified error object for when heap is not available
            #[derive(Debug)]
            struct SimpleError {
                name: String,
                message: String,
            }

            let error = SimpleError {
                name: error_name.to_string(),
                message,
            };

            Ok(Value::NativeObject(
                Rc::new(RefCell::new(error)) as Rc<RefCell<dyn Any>>
            ))
        }
    }

    /// Call an Array prototype method with receiver and callback support
    fn call_array_prototype_method(
        &mut self,
        name: &str,
        receiver: Value,
        args: Vec<Value>,
        functions: &[BytecodeChunk],
    ) -> Result<Value, JsError> {
        // Extract array from receiver
        let (array_ref, array_len) = match &receiver {
            Value::NativeObject(obj) => {
                let borrowed = obj.borrow();
                if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                    if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                        if let Value::Smi(len) = gc_object.get("length") {
                            (obj.clone(), len as usize)
                        } else {
                            return Err(JsError {
                                kind: ErrorKind::TypeError,
                                message: "Cannot call array method on non-array".to_string(),
                                stack: vec![],
                                source_position: None,
                            });
                        }
                    } else {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Invalid array object".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                } else {
                    return Err(JsError {
                        kind: ErrorKind::TypeError,
                        message: "Invalid array object".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                }
            }
            _ => {
                return Err(JsError {
                    kind: ErrorKind::TypeError,
                    message: "Cannot call array method on non-array".to_string(),
                    stack: vec![],
                    source_position: None,
                });
            }
        };

        match name {
            "Array.prototype.map" => {
                // Get callback function from args
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.map callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                // Create result array
                let result_array = if let Some(ref heap) = self.heap {
                    let gc_object = heap.create_object();
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                } else {
                    return Err(JsError {
                        kind: ErrorKind::InternalError,
                        message: "Heap not initialized".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                };

                // Iterate over array elements and call callback
                for i in 0..array_len {
                    // Get element from source array
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    // Call callback(element, index, array)
                    let callback_args = vec![element, Value::Smi(i as i32), receiver.clone()];
                    let mapped_value = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    // Store result in new array
                    {
                        let mut borrowed = result_array.borrow_mut();
                        if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                gc_object.set(i.to_string(), mapped_value);
                            }
                        }
                    }
                }

                // Set length on result array
                {
                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(array_len as i32));
                        }
                    }
                }

                Ok(Value::NativeObject(result_array))
            }

            "Array.prototype.filter" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.filter callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                let result_array = if let Some(ref heap) = self.heap {
                    let gc_object = heap.create_object();
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                } else {
                    return Err(JsError {
                        kind: ErrorKind::InternalError,
                        message: "Heap not initialized".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                };

                let mut result_idx = 0;
                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element.clone(), Value::Smi(i as i32), receiver.clone()];
                    let predicate_result = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    if predicate_result.is_truthy() {
                        let mut borrowed = result_array.borrow_mut();
                        if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                gc_object.set(result_idx.to_string(), element);
                                result_idx += 1;
                            }
                        }
                    }
                }

                // Set length
                {
                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(result_idx as i32));
                        }
                    }
                }

                Ok(Value::NativeObject(result_array))
            }

            "Array.prototype.forEach" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.forEach callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element, Value::Smi(i as i32), receiver.clone()];
                    self.call_function_with_args(callback_idx, callback_args, functions)?;
                }

                Ok(Value::Undefined)
            }

            "Array.prototype.reduce" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.reduce callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                let mut accumulator = args.get(1).cloned();
                let start_idx = if accumulator.is_some() { 0 } else { 1 };

                // If no initial value, use first element
                if accumulator.is_none() {
                    if array_len == 0 {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Reduce of empty array with no initial value".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                    let first_elem = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get("0")
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };
                    accumulator = Some(first_elem);
                }

                let mut acc = accumulator.unwrap();

                for i in start_idx..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    // callback(accumulator, currentValue, currentIndex, array)
                    let callback_args = vec![acc, element, Value::Smi(i as i32), receiver.clone()];
                    acc = self.call_function_with_args(callback_idx, callback_args, functions)?;
                }

                Ok(acc)
            }

            "Array.prototype.find" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.find callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element.clone(), Value::Smi(i as i32), receiver.clone()];
                    let result = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    if result.is_truthy() {
                        return Ok(element);
                    }
                }

                Ok(Value::Undefined)
            }

            "Array.prototype.findIndex" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.findIndex callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element, Value::Smi(i as i32), receiver.clone()];
                    let result = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    if result.is_truthy() {
                        return Ok(Value::Smi(i as i32));
                    }
                }

                Ok(Value::Smi(-1))
            }

            "Array.prototype.some" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.some callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element, Value::Smi(i as i32), receiver.clone()];
                    let result = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    if result.is_truthy() {
                        return Ok(Value::Boolean(true));
                    }
                }

                Ok(Value::Boolean(false))
            }

            "Array.prototype.every" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                let callback_idx = match callback {
                    Value::HeapObject(idx) => idx,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::TypeError,
                            message: "Array.prototype.every callback must be a function".to_string(),
                            stack: vec![],
                            source_position: None,
                        });
                    }
                };

                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let callback_args = vec![element, Value::Smi(i as i32), receiver.clone()];
                    let result = self.call_function_with_args(callback_idx, callback_args, functions)?;

                    if !result.is_truthy() {
                        return Ok(Value::Boolean(false));
                    }
                }

                Ok(Value::Boolean(true))
            }

            "Array.prototype.includes" => {
                let search_element = args.first().cloned().unwrap_or(Value::Undefined);
                let from_index = args.get(1).map(|v| self.to_number(v) as i32).unwrap_or(0);

                let start = if from_index < 0 {
                    (array_len as i32 + from_index).max(0) as usize
                } else {
                    from_index as usize
                };

                for i in start..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    // Use strict equality
                    if element == search_element {
                        return Ok(Value::Boolean(true));
                    }
                }

                Ok(Value::Boolean(false))
            }

            "Array.prototype.indexOf" => {
                let search_element = args.first().cloned().unwrap_or(Value::Undefined);
                let from_index = args.get(1).map(|v| self.to_number(v) as i32).unwrap_or(0);

                let start = if from_index < 0 {
                    (array_len as i32 + from_index).max(0) as usize
                } else {
                    from_index as usize
                };

                for i in start..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    if element == search_element {
                        return Ok(Value::Smi(i as i32));
                    }
                }

                Ok(Value::Smi(-1))
            }

            "Array.prototype.push" => {
                // Get current length and add new elements
                let new_len = array_len + args.len();

                {
                    let mut borrowed = array_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            for (i, arg) in args.into_iter().enumerate() {
                                gc_object.set((array_len + i).to_string(), arg);
                            }
                            gc_object.set("length".to_string(), Value::Smi(new_len as i32));
                        }
                    }
                }

                Ok(Value::Smi(new_len as i32))
            }

            "Array.prototype.pop" => {
                if array_len == 0 {
                    return Ok(Value::Undefined);
                }

                let last_idx = array_len - 1;
                let last_elem = {
                    let borrowed = array_ref.borrow();
                    if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                            gc_object.get(&last_idx.to_string())
                        } else {
                            Value::Undefined
                        }
                    } else {
                        Value::Undefined
                    }
                };

                // Update length (don't actually delete, just decrement length)
                {
                    let mut borrowed = array_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(last_idx as i32));
                        }
                    }
                }

                Ok(last_elem)
            }

            "Array.prototype.shift" => {
                if array_len == 0 {
                    return Ok(Value::Undefined);
                }

                let first_elem = {
                    let borrowed = array_ref.borrow();
                    if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                            gc_object.get("0")
                        } else {
                            Value::Undefined
                        }
                    } else {
                        Value::Undefined
                    }
                };

                // Shift all elements down
                {
                    let mut borrowed = array_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            for i in 1..array_len {
                                let elem = gc_object.get(&i.to_string());
                                gc_object.set((i - 1).to_string(), elem);
                            }
                            gc_object.set("length".to_string(), Value::Smi((array_len - 1) as i32));
                        }
                    }
                }

                Ok(first_elem)
            }

            "Array.prototype.unshift" => {
                let shift_count = args.len();
                let new_len = array_len + shift_count;

                {
                    let mut borrowed = array_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            // Shift existing elements up
                            for i in (0..array_len).rev() {
                                let elem = gc_object.get(&i.to_string());
                                gc_object.set((i + shift_count).to_string(), elem);
                            }
                            // Insert new elements at beginning
                            for (i, arg) in args.into_iter().enumerate() {
                                gc_object.set(i.to_string(), arg);
                            }
                            gc_object.set("length".to_string(), Value::Smi(new_len as i32));
                        }
                    }
                }

                Ok(Value::Smi(new_len as i32))
            }

            "Array.prototype.slice" => {
                let start_arg = args.first().map(|v| self.to_number(v) as i32).unwrap_or(0);
                let end_arg = args.get(1).map(|v| self.to_number(v) as i32).unwrap_or(array_len as i32);

                let start = if start_arg < 0 {
                    (array_len as i32 + start_arg).max(0) as usize
                } else {
                    (start_arg as usize).min(array_len)
                };

                let end = if end_arg < 0 {
                    (array_len as i32 + end_arg).max(0) as usize
                } else {
                    (end_arg as usize).min(array_len)
                };

                let result_array = if let Some(ref heap) = self.heap {
                    let gc_object = heap.create_object();
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                } else {
                    return Err(JsError {
                        kind: ErrorKind::InternalError,
                        message: "Heap not initialized".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                };

                let slice_len = if end > start { end - start } else { 0 };

                for i in 0..slice_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&(start + i).to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set(i.to_string(), element);
                        }
                    }
                }

                {
                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(slice_len as i32));
                        }
                    }
                }

                Ok(Value::NativeObject(result_array))
            }

            "Array.prototype.concat" => {
                let result_array = if let Some(ref heap) = self.heap {
                    let gc_object = heap.create_object();
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>
                } else {
                    return Err(JsError {
                        kind: ErrorKind::InternalError,
                        message: "Heap not initialized".to_string(),
                        stack: vec![],
                        source_position: None,
                    });
                };

                let mut result_idx = 0;

                // Copy elements from original array
                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set(result_idx.to_string(), element);
                            result_idx += 1;
                        }
                    }
                }

                // Concat arguments (simplified - doesn't flatten arrays)
                for arg in args {
                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set(result_idx.to_string(), arg);
                            result_idx += 1;
                        }
                    }
                }

                {
                    let mut borrowed = result_array.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(result_idx as i32));
                        }
                    }
                }

                Ok(Value::NativeObject(result_array))
            }

            "Array.prototype.join" => {
                let separator = match args.first() {
                    Some(Value::String(s)) => s.clone(),
                    Some(v) => self.to_string_value(v),
                    None => ",".to_string(),
                };

                let mut parts = Vec::with_capacity(array_len);
                for i in 0..array_len {
                    let element = {
                        let borrowed = array_ref.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                gc_object.get(&i.to_string())
                            } else {
                                Value::Undefined
                            }
                        } else {
                            Value::Undefined
                        }
                    };

                    match element {
                        Value::Undefined | Value::Null => parts.push(String::new()),
                        _ => parts.push(self.to_string_value(&element)),
                    }
                }

                Ok(Value::String(parts.join(&separator)))
            }

            "Array.prototype.reverse" => {
                // Reverse in place
                {
                    let mut borrowed = array_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            let half = array_len / 2;
                            for i in 0..half {
                                let j = array_len - 1 - i;
                                let elem_i = gc_object.get(&i.to_string());
                                let elem_j = gc_object.get(&j.to_string());
                                gc_object.set(i.to_string(), elem_j);
                                gc_object.set(j.to_string(), elem_i);
                            }
                        }
                    }
                }

                Ok(receiver)
            }

            "Array.prototype.splice" | "Array.prototype.sort" => {
                // These are more complex - return error for now
                Err(JsError {
                    kind: ErrorKind::TypeError,
                    message: format!("{} is not fully implemented yet", name),
                    stack: vec![],
                    source_position: None,
                })
            }

            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("Unknown array method: {}", name),
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
            Value::BigInt(n) => BuiltinValue::bigint(BigIntValue::new(n.clone())),
        }
    }

    /// Convert a Value to JSON string
    fn value_to_json_string(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Smi(n) => n.to_string(),
            Value::Double(n) => {
                if n.is_nan() {
                    "null".to_string()
                } else if n.is_infinite() {
                    "null".to_string()
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
            Value::NativeObject(obj) => {
                let borrowed = obj.borrow();
                if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                    if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                        // Check if it's an array (has length property)
                        if let Value::Smi(len) = gc_object.get("length") {
                            // Stringify as array
                            let mut parts = Vec::new();
                            for i in 0..len {
                                let elem = gc_object.get(&i.to_string());
                                parts.push(self.value_to_json_string(&elem));
                            }
                            return format!("[{}]", parts.join(","));
                        } else {
                            // Stringify as object
                            // Note: GCObject doesn't expose keys iterator, so we output empty object
                            return "{}".to_string();
                        }
                    }
                }
                "{}".to_string()
            }
            Value::HeapObject(_) => "{}".to_string(),
            Value::NativeFunction(_) => "undefined".to_string(), // Functions become undefined in JSON
            Value::BigInt(n) => n.to_string(), // BigInt to string for JSON (per ES spec, should throw)
        }
    }

    /// Parse a JSON string into a Value
    fn parse_json_string(&self, json: &str) -> Result<Value, JsError> {
        let trimmed = json.trim();

        // Handle primitive values
        if trimmed == "null" {
            return Ok(Value::Null);
        }
        if trimmed == "true" {
            return Ok(Value::Boolean(true));
        }
        if trimmed == "false" {
            return Ok(Value::Boolean(false));
        }

        // Handle numbers
        if let Ok(n) = trimmed.parse::<f64>() {
            if n.fract() == 0.0 && n >= i32::MIN as f64 && n <= i32::MAX as f64 {
                return Ok(Value::Smi(n as i32));
            }
            return Ok(Value::Double(n));
        }

        // Handle strings
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let inner = &trimmed[1..trimmed.len()-1];
            let unescaped = inner
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t");
            return Ok(Value::String(unescaped));
        }

        // Handle arrays (simplified - single-level)
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(ref heap) = self.heap {
                let gc_object = heap.create_object();
                let boxed: Box<dyn Any> = Box::new(gc_object);
                let obj_ref = Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>;

                let inner = &trimmed[1..trimmed.len()-1].trim();
                if !inner.is_empty() {
                    // Simple comma split (doesn't handle nested structures)
                    let elements: Vec<&str> = inner.split(',').collect();
                    let len = elements.len();

                    let mut borrowed = obj_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            for (i, elem) in elements.iter().enumerate() {
                                if let Ok(val) = self.parse_json_string(elem.trim()) {
                                    gc_object.set(i.to_string(), val);
                                }
                            }
                            gc_object.set("length".to_string(), Value::Smi(len as i32));
                        }
                    }
                } else {
                    // Empty array
                    let mut borrowed = obj_ref.borrow_mut();
                    if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                        if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                            gc_object.set("length".to_string(), Value::Smi(0));
                        }
                    }
                }

                return Ok(Value::NativeObject(obj_ref));
            }
            return Ok(Value::Undefined);
        }

        // Handle objects (simplified)
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Some(ref heap) = self.heap {
                let gc_object = heap.create_object();
                let boxed: Box<dyn Any> = Box::new(gc_object);
                let obj_ref = Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>;

                // Simple key:value parsing (doesn't handle nested structures)
                let inner = &trimmed[1..trimmed.len()-1].trim();
                if !inner.is_empty() {
                    for pair in inner.split(',') {
                        let pair = pair.trim();
                        if let Some(colon_idx) = pair.find(':') {
                            let key = pair[..colon_idx].trim().trim_matches('"');
                            let val_str = pair[colon_idx+1..].trim();

                            if let Ok(val) = self.parse_json_string(val_str) {
                                let mut borrowed = obj_ref.borrow_mut();
                                if let Some(gc_obj) = borrowed.downcast_mut::<Box<dyn Any>>() {
                                    if let Some(gc_object) = gc_obj.downcast_mut::<GCObject>() {
                                        gc_object.set(key.to_string(), val);
                                    }
                                }
                            }
                        }
                    }
                }

                return Ok(Value::NativeObject(obj_ref));
            }
            return Ok(Value::Undefined);
        }

        Err(JsError {
            kind: ErrorKind::SyntaxError,
            message: format!("Unexpected token in JSON: {}", trimmed),
            stack: vec![],
            source_position: None,
        })
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
            // Type coercion for other types (null, undefined, boolean, etc.)
            _ => {
                let a_num = self.to_number(&a);
                let b_num = self.to_number(&b);
                Ok(Value::Double(a_num + b_num))
            }
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
            Value::BigInt(n) => n.to_string(),
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
            // Type coercion for other types (null, undefined, boolean, etc.)
            (a, b) => {
                let a_num = self.to_number(&a);
                let b_num = self.to_number(&b);
                Ok(Value::Double(a_num - b_num))
            }
        }
    }

    fn mul(&self, a: Value, b: Value) -> Result<Value, JsError> {
        match (a, b) {
            (Value::Smi(x), Value::Smi(y)) => Ok(Value::Smi(x.wrapping_mul(y))),
            (Value::Double(x), Value::Double(y)) => Ok(Value::Double(x * y)),
            (Value::Smi(x), Value::Double(y)) => Ok(Value::Double(x as f64 * y)),
            (Value::Double(x), Value::Smi(y)) => Ok(Value::Double(x * y as f64)),
            // Type coercion for other types (null, undefined, boolean, etc.)
            (a, b) => {
                let a_num = self.to_number(&a);
                let b_num = self.to_number(&b);
                Ok(Value::Double(a_num * b_num))
            }
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

    fn exponentiate(&self, a: Value, b: Value) -> Result<Value, JsError> {
        let a_num = self.to_number(&a);
        let b_num = self.to_number(&b);
        Ok(Value::Double(a_num.powf(b_num)))
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
            Value::BigInt(_) => f64::NAN, // BigInt cannot be implicitly converted to number
        }
    }

    fn to_boolean(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Smi(n) => *n != 0,
            Value::Double(n) => !n.is_nan() && *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Undefined => false,
            Value::Null => false,
            Value::HeapObject(_) => true,
            Value::NativeObject(_) => true,
            Value::NativeFunction(_) => true,
            Value::BigInt(n) => !n.is_zero(), // 0n is falsy
        }
    }

    // Comparison operations

    fn equal(&self, a: Value, b: Value) -> Value {
        // Loose equality - performs type coercion
        let result = match (&a, &b) {
            // Same type comparisons
            (Value::Smi(x), Value::Smi(y)) => x == y,
            (Value::Double(x), Value::Double(y)) => {
                // NaN != NaN
                if x.is_nan() && y.is_nan() {
                    false
                } else {
                    x == y
                }
            }
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            // null == undefined
            (Value::Undefined, Value::Null) | (Value::Null, Value::Undefined) => true,
            // Number type coercion
            (Value::Smi(x), Value::Double(y)) => (*x as f64) == *y,
            (Value::Double(x), Value::Smi(y)) => *x == (*y as f64),
            // String to number coercion
            (Value::String(s), Value::Smi(n)) | (Value::Smi(n), Value::String(s)) => {
                if let Ok(parsed) = s.parse::<i32>() {
                    parsed == *n
                } else if let Ok(parsed) = s.parse::<f64>() {
                    parsed == (*n as f64)
                } else {
                    false
                }
            }
            (Value::String(s), Value::Double(n)) | (Value::Double(n), Value::String(s)) => {
                if let Ok(parsed) = s.parse::<f64>() {
                    parsed == *n || (parsed.is_nan() && n.is_nan())
                } else {
                    n.is_nan() // unparseable string becomes NaN
                }
            }
            // Boolean to number coercion
            (Value::Boolean(b), Value::Smi(n)) | (Value::Smi(n), Value::Boolean(b)) => {
                let bool_val = if *b { 1 } else { 0 };
                bool_val == *n
            }
            (Value::Boolean(b), Value::Double(n)) | (Value::Double(n), Value::Boolean(b)) => {
                let bool_val = if *b { 1.0 } else { 0.0 };
                bool_val == *n
            }
            _ => false,
        };
        Value::Boolean(result)
    }

    fn strict_equal(&self, a: Value, b: Value) -> Value {
        // Strict equality - no type coercion, but Smi and Double are both "number"
        let result = match (&a, &b) {
            // Number comparisons - Smi and Double are both "number" type in JS
            (Value::Smi(x), Value::Smi(y)) => x == y,
            (Value::Double(x), Value::Double(y)) => {
                // NaN !== NaN
                if x.is_nan() && y.is_nan() {
                    false
                } else {
                    x == y
                }
            }
            (Value::Smi(x), Value::Double(y)) => (*x as f64) == *y,
            (Value::Double(x), Value::Smi(y)) => *x == (*y as f64),
            // String comparison
            (Value::String(x), Value::String(y)) => x == y,
            // Boolean comparison
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            // Undefined/null comparison
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            // Object identity (reference equality)
            (Value::HeapObject(x), Value::HeapObject(y)) => x == y,
            (Value::NativeObject(x), Value::NativeObject(y)) => Rc::ptr_eq(x, y),
            (Value::NativeFunction(x), Value::NativeFunction(y)) => x == y,
            // Different types - false
            _ => false,
        };
        Value::Boolean(result)
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

    fn instanceof_check(&self, obj: Value, constructor: Value) -> Value {
        // Basic instanceof implementation for Test262 compliance
        // In JavaScript: obj instanceof Constructor
        // Returns true if Constructor.prototype is in obj's prototype chain

        match (&obj, &constructor) {
            // Primitives are not instances of constructors
            (Value::Undefined, _) | (Value::Null, _) => Value::Boolean(false),
            (Value::Boolean(_), _) | (Value::Smi(_), _) | (Value::Double(_), _) => {
                Value::Boolean(false)
            }

            // Check if constructor is actually a constructor
            (_, Value::NativeFunction(name)) => {
                // For native constructors, check object properties
                // This is a simplified check - full implementation would check prototype chain
                match name.as_str() {
                    "Error" | "TypeError" | "ReferenceError" | "RangeError" |
                    "SyntaxError" | "URIError" | "EvalError" => {
                        // Check if obj is an error object
                        // For now, return false - would need to check object's constructor property
                        Value::Boolean(false)
                    }
                    "Array" => {
                        // Check if obj is an array
                        Value::Boolean(false)
                    }
                    "Object" => {
                        // All objects are instances of Object
                        Value::Boolean(matches!(obj, Value::HeapObject(_) | Value::NativeObject(_)))
                    }
                    "Function" => {
                        // Check if obj is a function
                        Value::Boolean(matches!(obj, Value::NativeFunction(_)))
                    }
                    _ => Value::Boolean(false),
                }
            }

            // HeapObject instanceof constructor would need prototype chain walking
            (Value::HeapObject(_), _) | (Value::NativeObject(_), _) => {
                // TODO: Implement prototype chain walking
                // For now, return false
                Value::Boolean(false)
            }

            _ => Value::Boolean(false),
        }
    }

    fn in_check(&self, _prop: Value, obj: Value) -> Value {
        // Basic 'in' operator implementation
        // Returns true if property exists in object or its prototype chain

        match obj {
            Value::HeapObject(_) | Value::NativeObject(_) => {
                // TODO: Check if property exists in object
                // For now, return false
                Value::Boolean(false)
            }
            _ => {
                // Non-objects don't have properties
                Value::Boolean(false)
            }
        }
    }

    /// Call an Object prototype method with receiver
    fn call_object_prototype_method(
        &self,
        name: &str,
        receiver: Value,
        args: Vec<Value>,
    ) -> Result<Value, JsError> {
        match name {
            "Object.prototype.toString" => {
                // Returns "[object Type]" format
                let type_tag = match &receiver {
                    Value::Undefined => "Undefined",
                    Value::Null => "Null",
                    Value::Boolean(_) => "Boolean",
                    Value::Smi(_) | Value::Double(_) => "Number",
                    Value::String(_) => "String",
                    Value::HeapObject(_) => "Object",
                    Value::NativeObject(obj) => {
                        let borrowed = obj.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                // Check if it's an array
                                if matches!(gc_object.get("length"), Value::Smi(_)) {
                                    "Array"
                                } else {
                                    "Object"
                                }
                            } else {
                                "Object"
                            }
                        } else {
                            "Object"
                        }
                    }
                    Value::NativeFunction(_) => "Function",
                    Value::BigInt(_) => "BigInt",
                };
                Ok(Value::String(format!("[object {}]", type_tag)))
            }
            "Object.prototype.valueOf" => {
                // Returns the receiver itself for objects
                Ok(receiver)
            }
            "Object.prototype.hasOwnProperty" => {
                // Check if property exists directly on object (not prototype)
                let prop_name = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();

                match &receiver {
                    Value::NativeObject(obj) => {
                        let borrowed = obj.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                let value = gc_object.get(&prop_name);
                                Ok(Value::Boolean(!matches!(value, Value::Undefined)))
                            } else {
                                Ok(Value::Boolean(false))
                            }
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    _ => Ok(Value::Boolean(false)),
                }
            }
            "Object.prototype.propertyIsEnumerable" => {
                // Simplified: just check if property exists
                let prop_name = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();

                match &receiver {
                    Value::NativeObject(obj) => {
                        let borrowed = obj.borrow();
                        if let Some(gc_obj) = borrowed.downcast_ref::<Box<dyn Any>>() {
                            if let Some(gc_object) = gc_obj.downcast_ref::<GCObject>() {
                                let value = gc_object.get(&prop_name);
                                Ok(Value::Boolean(!matches!(value, Value::Undefined)))
                            } else {
                                Ok(Value::Boolean(false))
                            }
                        } else {
                            Ok(Value::Boolean(false))
                        }
                    }
                    _ => Ok(Value::Boolean(false)),
                }
            }
            "Object.prototype.isPrototypeOf" | "Object.prototype.toLocaleString" => {
                // Simplified implementations
                if name == "Object.prototype.toLocaleString" {
                    self.call_object_prototype_method("Object.prototype.toString", receiver, args)
                } else {
                    Ok(Value::Boolean(false))
                }
            }
            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("Unknown Object.prototype method: {}", name),
                stack: vec![],
                source_position: None,
            }),
        }
    }

    /// Call a String prototype method with receiver
    fn call_string_prototype_method(
        &self,
        name: &str,
        receiver: Value,
        args: Vec<Value>,
    ) -> Result<Value, JsError> {
        // Get the string value from receiver
        let s = self.to_string_value(&receiver);

        match name {
            "String.prototype.toString" | "String.prototype.valueOf" => {
                Ok(Value::String(s))
            }
            "String.prototype.charAt" => {
                let index = args.first().map(|v| self.to_number(v) as usize).unwrap_or(0);
                let result = s.chars().nth(index).map(|c| c.to_string()).unwrap_or_default();
                Ok(Value::String(result))
            }
            "String.prototype.charCodeAt" => {
                let index = args.first().map(|v| self.to_number(v) as usize).unwrap_or(0);
                let result = s.chars().nth(index).map(|c| c as u32 as f64).unwrap_or(f64::NAN);
                Ok(Value::Double(result))
            }
            "String.prototype.indexOf" => {
                let search = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let start = args.get(1).map(|v| self.to_number(v) as usize).unwrap_or(0);
                let result = s[start..].find(&search).map(|i| (i + start) as i32).unwrap_or(-1);
                Ok(Value::Smi(result))
            }
            "String.prototype.lastIndexOf" => {
                let search = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let result = s.rfind(&search).map(|i| i as i32).unwrap_or(-1);
                Ok(Value::Smi(result))
            }
            "String.prototype.slice" => {
                let len = s.len() as i32;
                let start = args.first().map(|v| {
                    let n = self.to_number(v) as i32;
                    if n < 0 { (len + n).max(0) as usize } else { n.min(len) as usize }
                }).unwrap_or(0);
                let end = args.get(1).map(|v| {
                    let n = self.to_number(v) as i32;
                    if n < 0 { (len + n).max(0) as usize } else { n.min(len) as usize }
                }).unwrap_or(s.len());
                let result = if start <= end { s.get(start..end).unwrap_or("").to_string() } else { String::new() };
                Ok(Value::String(result))
            }
            "String.prototype.substring" => {
                let len = s.len();
                let start = args.first().map(|v| (self.to_number(v) as usize).min(len)).unwrap_or(0);
                let end = args.get(1).map(|v| (self.to_number(v) as usize).min(len)).unwrap_or(len);
                let (start, end) = if start <= end { (start, end) } else { (end, start) };
                Ok(Value::String(s.get(start..end).unwrap_or("").to_string()))
            }
            "String.prototype.toLowerCase" => {
                Ok(Value::String(s.to_lowercase()))
            }
            "String.prototype.toUpperCase" => {
                Ok(Value::String(s.to_uppercase()))
            }
            "String.prototype.trim" => {
                Ok(Value::String(s.trim().to_string()))
            }
            "String.prototype.trimStart" | "String.prototype.trimLeft" => {
                Ok(Value::String(s.trim_start().to_string()))
            }
            "String.prototype.trimEnd" | "String.prototype.trimRight" => {
                Ok(Value::String(s.trim_end().to_string()))
            }
            "String.prototype.split" => {
                let separator = args.first().map(|v| self.to_string_value(v));
                let limit = args.get(1).map(|v| self.to_number(v) as usize);

                let parts: Vec<&str> = if let Some(sep) = separator {
                    if let Some(lim) = limit {
                        s.split(&sep).take(lim).collect()
                    } else {
                        s.split(&sep).collect()
                    }
                } else {
                    vec![&s]
                };

                // Create array with parts
                if let Some(ref heap) = self.heap {
                    let mut gc_object = heap.create_object();
                    for (i, part) in parts.iter().enumerate() {
                        gc_object.set(i.to_string(), Value::String(part.to_string()));
                    }
                    gc_object.set("length".to_string(), Value::Smi(parts.len() as i32));
                    let boxed: Box<dyn Any> = Box::new(gc_object);
                    Ok(Value::NativeObject(Rc::new(RefCell::new(boxed)) as Rc<RefCell<dyn Any>>))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "String.prototype.concat" => {
                let mut result = s;
                for arg in args {
                    result.push_str(&self.to_string_value(&arg));
                }
                Ok(Value::String(result))
            }
            "String.prototype.includes" => {
                let search = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let start = args.get(1).map(|v| self.to_number(v) as usize).unwrap_or(0);
                Ok(Value::Boolean(s.get(start..).map(|sub| sub.contains(&search)).unwrap_or(false)))
            }
            "String.prototype.startsWith" => {
                let search = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                let start = args.get(1).map(|v| self.to_number(v) as usize).unwrap_or(0);
                Ok(Value::Boolean(s.get(start..).map(|sub| sub.starts_with(&search)).unwrap_or(false)))
            }
            "String.prototype.endsWith" => {
                let search = args.first().map(|v| self.to_string_value(v)).unwrap_or_default();
                Ok(Value::Boolean(s.ends_with(&search)))
            }
            "String.prototype.repeat" => {
                let count = args.first().map(|v| self.to_number(v) as usize).unwrap_or(0);
                Ok(Value::String(s.repeat(count)))
            }
            "String.prototype.padStart" => {
                let target_len = args.first().map(|v| self.to_number(v) as usize).unwrap_or(0);
                let pad_str = args.get(1).map(|v| self.to_string_value(v)).unwrap_or_else(|| " ".to_string());
                if s.len() >= target_len || pad_str.is_empty() {
                    Ok(Value::String(s))
                } else {
                    let pad_len = target_len - s.len();
                    let pad = pad_str.repeat((pad_len / pad_str.len()) + 1);
                    Ok(Value::String(format!("{}{}", &pad[..pad_len], s)))
                }
            }
            "String.prototype.padEnd" => {
                let target_len = args.first().map(|v| self.to_number(v) as usize).unwrap_or(0);
                let pad_str = args.get(1).map(|v| self.to_string_value(v)).unwrap_or_else(|| " ".to_string());
                if s.len() >= target_len || pad_str.is_empty() {
                    Ok(Value::String(s))
                } else {
                    let pad_len = target_len - s.len();
                    let pad = pad_str.repeat((pad_len / pad_str.len()) + 1);
                    Ok(Value::String(format!("{}{}", s, &pad[..pad_len])))
                }
            }
            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("Unknown String.prototype method: {}", name),
                stack: vec![],
                source_position: None,
            }),
        }
    }

    /// Call a Number prototype method with receiver
    fn call_number_prototype_method(
        &self,
        name: &str,
        receiver: Value,
    ) -> Result<Value, JsError> {
        let n = self.to_number(&receiver);

        match name {
            "Number.prototype.toString" => {
                Ok(Value::String(if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                    format!("{}", n as i64)
                } else {
                    format!("{}", n)
                }))
            }
            "Number.prototype.valueOf" => {
                Ok(Value::Double(n))
            }
            "Number.prototype.toFixed" => {
                // Simplified: just format with default precision
                Ok(Value::String(format!("{:.0}", n)))
            }
            _ => Err(JsError {
                kind: ErrorKind::TypeError,
                message: format!("Unknown Number.prototype method: {}", name),
                stack: vec![],
                source_position: None,
            }),
        }
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
        // Globals contains standard JavaScript builtins
        assert!(dispatcher.globals.len() >= 20, "Should have at least 20 standard globals");
        // Core objects
        assert!(dispatcher.globals.contains_key("console"));
        assert!(dispatcher.globals.contains_key("Math"));
        assert!(dispatcher.globals.contains_key("Promise"));
        assert!(dispatcher.globals.contains_key("JSON"));
        assert!(dispatcher.globals.contains_key("Array"));
        // Constructors
        assert!(dispatcher.globals.contains_key("Object"));
        assert!(dispatcher.globals.contains_key("Number"));
        assert!(dispatcher.globals.contains_key("String"));
        assert!(dispatcher.globals.contains_key("Boolean"));
        // Error constructors
        assert!(dispatcher.globals.contains_key("Error"));
        assert!(dispatcher.globals.contains_key("TypeError"));
        assert!(dispatcher.globals.contains_key("ReferenceError"));
        // Global constants and functions
        assert!(dispatcher.globals.contains_key("NaN"));
        assert!(dispatcher.globals.contains_key("Infinity"));
        assert!(dispatcher.globals.contains_key("undefined"));
        assert!(dispatcher.globals.contains_key("isNaN"));
        assert!(dispatcher.globals.contains_key("parseInt"));
        assert!(dispatcher.stack.is_empty());
        assert!(dispatcher.open_upvalues.is_empty());
        assert!(dispatcher.current_upvalues.is_empty());
    }

    #[test]
    fn test_dispatcher_default() {
        let dispatcher = Dispatcher::default();
        // Default has same globals as new()
        assert!(dispatcher.globals.len() >= 20, "Should have at least 20 standard globals");
        assert!(dispatcher.globals.contains_key("console"));
        assert!(dispatcher.globals.contains_key("Math"));
        assert!(dispatcher.globals.contains_key("Promise"));
        assert!(dispatcher.globals.contains_key("JSON"));
        assert!(dispatcher.globals.contains_key("Array"));
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
