//! Upvalue support for closures
//!
//! This module provides the runtime structures for captured variables
//! in closures. Upvalues allow inner functions to access variables
//! from their enclosing scopes.

use core_types::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Upvalue represents a captured variable from an outer scope
///
/// An upvalue can be in one of two states:
/// - Open: The variable still lives on the stack
/// - Closed: The variable has been moved to the heap when its scope ended
#[derive(Debug, Clone)]
pub enum Upvalue {
    /// Variable still lives on the stack at the given index
    Open {
        /// Stack index where the variable lives
        stack_index: usize,
    },
    /// Variable has been closed over (moved to heap)
    Closed {
        /// Heap-allocated value
        value: Rc<RefCell<Value>>,
    },
}

impl Upvalue {
    /// Create a new open upvalue pointing to a stack location
    pub fn new_open(stack_index: usize) -> Self {
        Upvalue::Open { stack_index }
    }

    /// Create a new closed upvalue with a heap-allocated value
    pub fn new_closed(value: Value) -> Self {
        Upvalue::Closed {
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Close this upvalue by moving the value from stack to heap
    ///
    /// This is called when the scope containing the captured variable ends
    pub fn close(&mut self, value: Value) {
        *self = Upvalue::Closed {
            value: Rc::new(RefCell::new(value)),
        };
    }

    /// Get the value of this upvalue
    ///
    /// For open upvalues, reads from the stack.
    /// For closed upvalues, reads from the heap.
    pub fn get(&self, stack: &[Value]) -> Value {
        match self {
            Upvalue::Open { stack_index } => stack
                .get(*stack_index)
                .cloned()
                .unwrap_or(Value::Undefined),
            Upvalue::Closed { value } => value.borrow().clone(),
        }
    }

    /// Set the value of this upvalue
    ///
    /// For open upvalues, writes to the stack.
    /// For closed upvalues, writes to the heap.
    pub fn set(&self, new_value: Value, stack: &mut [Value]) {
        match self {
            Upvalue::Open { stack_index } => {
                if *stack_index < stack.len() {
                    stack[*stack_index] = new_value;
                }
            }
            Upvalue::Closed { value } => {
                *value.borrow_mut() = new_value;
            }
        }
    }

    /// Check if this upvalue is still open
    pub fn is_open(&self) -> bool {
        matches!(self, Upvalue::Open { .. })
    }

    /// Get the stack index if this is an open upvalue
    pub fn stack_index(&self) -> Option<usize> {
        match self {
            Upvalue::Open { stack_index } => Some(*stack_index),
            Upvalue::Closed { .. } => None,
        }
    }
}

/// A handle to a shared upvalue
pub type UpvalueHandle = Rc<RefCell<Upvalue>>;

/// Create a new upvalue handle
pub fn new_upvalue_handle(upvalue: Upvalue) -> UpvalueHandle {
    Rc::new(RefCell::new(upvalue))
}

/// Closure combines function code with its captured environment
///
/// A closure consists of a function index (pointing to bytecode) and
/// a list of upvalue handles that represent the captured variables.
#[derive(Debug, Clone)]
pub struct Closure {
    /// Index of the function in the function registry
    pub function_index: usize,
    /// Captured variables (upvalues)
    pub upvalues: Vec<UpvalueHandle>,
}

impl Closure {
    /// Create a new closure with the given function index and upvalues
    pub fn new(function_index: usize, upvalues: Vec<UpvalueHandle>) -> Self {
        Self {
            function_index,
            upvalues,
        }
    }

    /// Create a closure with no captured variables
    pub fn without_upvalues(function_index: usize) -> Self {
        Self {
            function_index,
            upvalues: Vec::new(),
        }
    }

    /// Get the number of captured variables
    pub fn upvalue_count(&self) -> usize {
        self.upvalues.len()
    }

    /// Get a specific upvalue by index
    pub fn get_upvalue(&self, index: usize) -> Option<&UpvalueHandle> {
        self.upvalues.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_upvalue_creation() {
        let upvalue = Upvalue::new_open(5);
        assert!(upvalue.is_open());
        assert_eq!(upvalue.stack_index(), Some(5));
    }

    #[test]
    fn test_closed_upvalue_creation() {
        let upvalue = Upvalue::new_closed(Value::Smi(42));
        assert!(!upvalue.is_open());
        assert_eq!(upvalue.stack_index(), None);
    }

    #[test]
    fn test_open_upvalue_get() {
        let upvalue = Upvalue::new_open(1);
        let stack = vec![Value::Smi(10), Value::Smi(20), Value::Smi(30)];
        let value = upvalue.get(&stack);
        assert_eq!(value, Value::Smi(20));
    }

    #[test]
    fn test_open_upvalue_get_out_of_bounds() {
        let upvalue = Upvalue::new_open(10);
        let stack = vec![Value::Smi(10)];
        let value = upvalue.get(&stack);
        assert_eq!(value, Value::Undefined);
    }

    #[test]
    fn test_closed_upvalue_get() {
        let upvalue = Upvalue::new_closed(Value::Smi(99));
        let stack = vec![];
        let value = upvalue.get(&stack);
        assert_eq!(value, Value::Smi(99));
    }

    #[test]
    fn test_open_upvalue_set() {
        let upvalue = Upvalue::new_open(1);
        let mut stack = vec![Value::Smi(10), Value::Smi(20), Value::Smi(30)];
        upvalue.set(Value::Smi(100), &mut stack);
        assert_eq!(stack[1], Value::Smi(100));
    }

    #[test]
    fn test_closed_upvalue_set() {
        let upvalue = Upvalue::new_closed(Value::Smi(50));
        let mut stack = vec![];
        upvalue.set(Value::Smi(75), &mut stack);
        // The closed upvalue's internal value should be updated
        let value = upvalue.get(&stack);
        assert_eq!(value, Value::Smi(75));
    }

    #[test]
    fn test_close_upvalue() {
        let mut upvalue = Upvalue::new_open(2);
        assert!(upvalue.is_open());

        upvalue.close(Value::Smi(123));
        assert!(!upvalue.is_open());

        let stack = vec![];
        assert_eq!(upvalue.get(&stack), Value::Smi(123));
    }

    #[test]
    fn test_shared_closed_upvalue() {
        // Test that closed upvalues share their heap storage
        let upvalue = Upvalue::new_closed(Value::Smi(1));

        // Clone the upvalue (simulates sharing between closures)
        let upvalue_clone = upvalue.clone();

        // Both should read the same value
        let stack = vec![];
        assert_eq!(upvalue.get(&stack), Value::Smi(1));
        assert_eq!(upvalue_clone.get(&stack), Value::Smi(1));

        // Update through one handle
        let mut stack_mut = vec![];
        upvalue.set(Value::Smi(999), &mut stack_mut);

        // Both should see the update (they share the Rc<RefCell>)
        assert_eq!(upvalue.get(&stack), Value::Smi(999));
        assert_eq!(upvalue_clone.get(&stack), Value::Smi(999));
    }

    #[test]
    fn test_upvalue_handle_creation() {
        let upvalue = Upvalue::new_open(3);
        let handle = new_upvalue_handle(upvalue);

        assert!(handle.borrow().is_open());
        assert_eq!(handle.borrow().stack_index(), Some(3));
    }

    #[test]
    fn test_closure_creation() {
        let closure = Closure::new(0, vec![]);
        assert_eq!(closure.function_index, 0);
        assert_eq!(closure.upvalue_count(), 0);
    }

    #[test]
    fn test_closure_without_upvalues() {
        let closure = Closure::without_upvalues(5);
        assert_eq!(closure.function_index, 5);
        assert_eq!(closure.upvalue_count(), 0);
    }

    #[test]
    fn test_closure_with_upvalues() {
        let uv1 = new_upvalue_handle(Upvalue::new_open(0));
        let uv2 = new_upvalue_handle(Upvalue::new_closed(Value::Smi(100)));

        let closure = Closure::new(3, vec![uv1, uv2]);
        assert_eq!(closure.function_index, 3);
        assert_eq!(closure.upvalue_count(), 2);

        // Check we can access the upvalues
        let first_uv = closure.get_upvalue(0).unwrap();
        assert!(first_uv.borrow().is_open());

        let second_uv = closure.get_upvalue(1).unwrap();
        assert!(!second_uv.borrow().is_open());
    }

    #[test]
    fn test_closure_get_upvalue_out_of_bounds() {
        let closure = Closure::without_upvalues(0);
        assert!(closure.get_upvalue(0).is_none());
    }

    #[test]
    fn test_multiple_closures_share_upvalues() {
        // Simulate two inner functions capturing the same outer variable
        let shared_upvalue = new_upvalue_handle(Upvalue::new_closed(Value::Smi(42)));

        let closure1 = Closure::new(1, vec![shared_upvalue.clone()]);
        let closure2 = Closure::new(2, vec![shared_upvalue.clone()]);

        // Both closures should see the same value
        let stack = vec![];
        let val1 = closure1.get_upvalue(0).unwrap().borrow().get(&stack);
        let val2 = closure2.get_upvalue(0).unwrap().borrow().get(&stack);
        assert_eq!(val1, Value::Smi(42));
        assert_eq!(val2, Value::Smi(42));

        // Mutate through closure1
        closure1
            .get_upvalue(0)
            .unwrap()
            .borrow()
            .set(Value::Smi(100), &mut vec![]);

        // closure2 should see the change
        let val2_after = closure2.get_upvalue(0).unwrap().borrow().get(&stack);
        assert_eq!(val2_after, Value::Smi(100));
    }
}
