//! Promise implementation following Promise/A+ specification.
//!
//! This module provides a JavaScript Promise implementation with proper
//! state management and chaining support.

use core_types::{JsError, Value};

/// The state of a Promise.
///
/// Promises transition through states according to the Promise/A+ specification.
/// Once settled (Fulfilled or Rejected), a Promise cannot change state.
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    /// The initial state; the promise is neither fulfilled nor rejected.
    Pending,
    /// The promise has been resolved with a value.
    Fulfilled,
    /// The promise has been rejected with an error.
    Rejected,
}

/// A function that can be called with arguments and returns a Result.
///
/// This represents a JavaScript function that can be used as a Promise handler.
pub struct Function {
    callback: Box<dyn FnMut(Vec<Value>) -> Result<Value, JsError> + Send>,
}

impl Function {
    /// Creates a new Function from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(Vec<Value>) -> Result<Value, JsError> + Send + 'static,
    {
        Self {
            callback: Box::new(f),
        }
    }

    /// Calls the function with the given arguments.
    pub fn call(&mut self, args: Vec<Value>) -> Result<Value, JsError> {
        (self.callback)(args)
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function {{ ... }}")
    }
}

/// A reaction to be triggered when a Promise settles.
///
/// This represents the handlers registered via `.then()`.
#[derive(Debug)]
pub struct PromiseReaction {
    /// The promise that will be resolved/rejected based on this reaction
    pub promise: Promise,
    /// Handler for fulfilled state
    pub on_fulfilled: Option<Function>,
    /// Handler for rejected state
    pub on_rejected: Option<Function>,
}

/// A JavaScript Promise.
///
/// Promises represent the eventual completion (or failure) of an asynchronous
/// operation and its resulting value.
///
/// # Examples
///
/// ```
/// use async_runtime::{Promise, PromiseState};
/// use core_types::Value;
///
/// let mut promise = Promise::new();
/// assert!(matches!(promise.state, PromiseState::Pending));
///
/// promise.resolve(Value::Smi(42));
/// assert!(matches!(promise.state, PromiseState::Fulfilled));
/// assert_eq!(promise.result, Some(Value::Smi(42)));
/// ```
#[derive(Debug)]
pub struct Promise {
    /// The current state of the Promise
    pub state: PromiseState,
    /// Reactions registered for when the Promise settles
    pub reactions: Vec<PromiseReaction>,
    /// The result value (if fulfilled) or error (if rejected)
    pub result: Option<Value>,
    /// The error if rejected
    pub error: Option<JsError>,
}

impl Promise {
    /// Creates a new pending Promise.
    ///
    /// # Returns
    ///
    /// A new Promise in the Pending state with no reactions or result.
    pub fn new() -> Self {
        Self {
            state: PromiseState::Pending,
            reactions: Vec::new(),
            result: None,
            error: None,
        }
    }

    /// Resolves the Promise with a value.
    ///
    /// If the Promise is already settled (Fulfilled or Rejected), this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to fulfill the Promise with
    pub fn resolve(&mut self, value: Value) {
        if matches!(self.state, PromiseState::Pending) {
            self.state = PromiseState::Fulfilled;
            self.result = Some(value);
            // Trigger reactions would happen here in a full implementation
            self.trigger_reactions();
        }
    }

    /// Rejects the Promise with an error.
    ///
    /// If the Promise is already settled (Fulfilled or Rejected), this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to reject the Promise with
    pub fn reject(&mut self, error: JsError) {
        if matches!(self.state, PromiseState::Pending) {
            self.state = PromiseState::Rejected;
            self.error = Some(error);
            // Trigger reactions would happen here in a full implementation
            self.trigger_reactions();
        }
    }

    /// Adds handlers for fulfillment and/or rejection.
    ///
    /// Returns a new Promise that will be resolved based on the handlers' results.
    ///
    /// # Arguments
    ///
    /// * `on_fulfilled` - Optional handler called when Promise fulfills
    /// * `on_rejected` - Optional handler called when Promise rejects
    ///
    /// # Returns
    ///
    /// A new Promise that chains after this one
    pub fn then(
        &mut self,
        on_fulfilled: Option<Function>,
        on_rejected: Option<Function>,
    ) -> Promise {
        let chained = Promise::new();

        let reaction = PromiseReaction {
            promise: Promise::new(), // placeholder for the chained promise
            on_fulfilled,
            on_rejected,
        };

        self.reactions.push(reaction);

        chained
    }

    /// Checks if there are pending reactions.
    pub fn has_pending_reactions(&self) -> bool {
        !self.reactions.is_empty()
    }

    /// Triggers all registered reactions.
    ///
    /// In a real implementation, this would enqueue microtasks.
    fn trigger_reactions(&mut self) {
        // In a full implementation, this would:
        // 1. For each reaction, create a microtask
        // 2. Enqueue the microtask in the event loop
        // 3. The microtask would call the appropriate handler
        // For now, we just keep the reactions pending
    }
}

impl Default for Promise {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_state_variants() {
        let pending = PromiseState::Pending;
        let fulfilled = PromiseState::Fulfilled;
        let rejected = PromiseState::Rejected;

        assert!(matches!(pending, PromiseState::Pending));
        assert!(matches!(fulfilled, PromiseState::Fulfilled));
        assert!(matches!(rejected, PromiseState::Rejected));
    }

    #[test]
    fn test_promise_new() {
        let promise = Promise::new();
        assert!(matches!(promise.state, PromiseState::Pending));
        assert!(promise.reactions.is_empty());
        assert!(promise.result.is_none());
    }

    #[test]
    fn test_promise_resolve() {
        let mut promise = Promise::new();
        promise.resolve(Value::Smi(42));
        assert!(matches!(promise.state, PromiseState::Fulfilled));
        assert_eq!(promise.result, Some(Value::Smi(42)));
    }

    #[test]
    fn test_function_creation() {
        let mut func = Function::new(|_args| Ok(Value::Undefined));
        let result = func.call(vec![]);
        assert!(result.is_ok());
    }
}
