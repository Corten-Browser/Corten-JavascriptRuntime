//! Dispatch loop for bytecode execution
//!
//! Handles individual opcode execution.

use bytecode_system::Opcode;
use core_types::{JsError, Value};
use std::collections::HashMap;

use crate::context::ExecutionContext;

/// Dispatch handler for executing bytecode
#[derive(Debug)]
pub struct Dispatcher {
    /// Global variables storage
    globals: HashMap<String, Value>,
    /// Stack for intermediate values
    stack: Vec<Value>,
}

impl Dispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            stack: Vec::with_capacity(256),
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
        }
    }

    /// Execute bytecode in the given context
    pub fn execute(&mut self, ctx: &mut ExecutionContext) -> Result<Value, JsError> {
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
                Opcode::LoadProperty(_name) => {
                    // Placeholder: property access
                    self.stack.pop();
                    self.stack.push(Value::Undefined);
                }
                Opcode::StoreProperty(_name) => {
                    // Placeholder: property store
                    self.stack.pop(); // value
                    self.stack.pop(); // object
                }
                Opcode::CreateClosure(_idx) => {
                    // Placeholder: create closure
                    self.stack.push(Value::HeapObject(0));
                }
                Opcode::Call(_argc) => {
                    // Placeholder: function call
                    self.stack.push(Value::Undefined);
                }
            }
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

        let result = dispatcher.execute(&mut ctx);
        assert_eq!(result.unwrap(), Value::Undefined);
    }
}
