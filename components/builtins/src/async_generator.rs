//! ES2024 AsyncGenerator protocol implementation
//!
//! This module implements:
//! - AsyncGeneratorFunction constructor
//! - AsyncGenerator object with internal state machine
//! - Asynchronous iteration protocol (next, return, throw)
//! - Queue management for concurrent operations
//! - Promise-based result delivery

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::iterator::IteratorResult;
use crate::value::{JsError, JsResult, JsValue};

/// AsyncGenerator internal state
///
/// Represents the lifecycle of an async generator according to ECMAScript spec.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsyncGeneratorState {
    /// Generator created but never executed (initial state)
    SuspendedStart,
    /// Generator paused at a yield expression
    SuspendedYield,
    /// Generator is currently executing code
    Executing,
    /// Generator is processing a return operation
    AwaitingReturn,
    /// Generator has finished execution
    Completed,
}

/// Type of async generator request
#[derive(Debug, Clone)]
pub enum AsyncGeneratorRequestKind {
    /// next(value?) request
    Next,
    /// return(value?) request
    Return,
    /// throw(exception) request
    Throw,
}

/// A pending request in the async generator queue
#[derive(Debug, Clone)]
pub struct AsyncGeneratorRequest {
    /// The kind of request (next, return, throw)
    pub kind: AsyncGeneratorRequestKind,
    /// The value associated with the request
    pub value: Option<JsValue>,
    /// Whether this request has been processed
    pub completed: bool,
    /// The result of this request (set when completed)
    pub result: Option<Result<IteratorResult, JsError>>,
}

impl AsyncGeneratorRequest {
    /// Create a next request
    pub fn next(value: Option<JsValue>) -> Self {
        AsyncGeneratorRequest {
            kind: AsyncGeneratorRequestKind::Next,
            value,
            completed: false,
            result: None,
        }
    }

    /// Create a return request
    pub fn return_request(value: Option<JsValue>) -> Self {
        AsyncGeneratorRequest {
            kind: AsyncGeneratorRequestKind::Return,
            value,
            completed: false,
            result: None,
        }
    }

    /// Create a throw request
    pub fn throw(exception: JsValue) -> Self {
        AsyncGeneratorRequest {
            kind: AsyncGeneratorRequestKind::Throw,
            value: Some(exception),
            completed: false,
            result: None,
        }
    }
}

/// Promise state for async generator results
#[derive(Debug, Clone)]
pub enum PromiseState {
    /// Promise is pending resolution
    Pending,
    /// Promise has been fulfilled with a value
    Fulfilled(JsValue),
    /// Promise has been rejected with an error
    Rejected(JsError),
}

/// A simple Promise representation for async generator results
#[derive(Debug, Clone)]
pub struct AsyncGeneratorPromise {
    /// Internal state of the promise
    state: Rc<RefCell<PromiseState>>,
    /// Request ID for tracking
    request_id: usize,
}

impl AsyncGeneratorPromise {
    /// Create a new pending promise
    pub fn new(request_id: usize) -> Self {
        AsyncGeneratorPromise {
            state: Rc::new(RefCell::new(PromiseState::Pending)),
            request_id,
        }
    }

    /// Resolve the promise with a value
    pub fn resolve(&self, value: JsValue) {
        *self.state.borrow_mut() = PromiseState::Fulfilled(value);
    }

    /// Reject the promise with an error
    pub fn reject(&self, error: JsError) {
        *self.state.borrow_mut() = PromiseState::Rejected(error);
    }

    /// Check if the promise is pending
    pub fn is_pending(&self) -> bool {
        matches!(*self.state.borrow(), PromiseState::Pending)
    }

    /// Check if the promise is fulfilled
    pub fn is_fulfilled(&self) -> bool {
        matches!(*self.state.borrow(), PromiseState::Fulfilled(_))
    }

    /// Check if the promise is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(*self.state.borrow(), PromiseState::Rejected(_))
    }

    /// Get the fulfilled value (if fulfilled)
    pub fn value(&self) -> Option<JsValue> {
        match &*self.state.borrow() {
            PromiseState::Fulfilled(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Get the rejection error (if rejected)
    pub fn error(&self) -> Option<JsError> {
        match &*self.state.borrow() {
            PromiseState::Rejected(e) => Some(e.clone()),
            _ => None,
        }
    }

    /// Get the request ID
    pub fn request_id(&self) -> usize {
        self.request_id
    }

    /// Get the current state
    pub fn state(&self) -> PromiseState {
        self.state.borrow().clone()
    }

    /// Convert promise result to JsValue (for async iteration)
    pub fn to_js_value(&self) -> JsValue {
        match &*self.state.borrow() {
            PromiseState::Pending => {
                let obj = JsValue::object();
                obj.set("state", JsValue::string("pending"));
                obj
            }
            PromiseState::Fulfilled(value) => {
                let obj = JsValue::object();
                obj.set("state", JsValue::string("fulfilled"));
                obj.set("value", value.clone());
                obj
            }
            PromiseState::Rejected(err) => {
                let obj = JsValue::object();
                obj.set("state", JsValue::string("rejected"));
                obj.set("reason", JsValue::string(&err.message));
                obj
            }
        }
    }
}

/// Internal data for an AsyncGenerator object
#[derive(Debug)]
pub struct AsyncGeneratorData {
    /// Current state of the async generator
    pub state: AsyncGeneratorState,
    /// Queue of pending requests
    pub queue: VecDeque<AsyncGeneratorRequest>,
    /// Values to be yielded (simplified model)
    pub values: Vec<JsValue>,
    /// Current position in the values
    pub position: usize,
    /// The result to return when done
    pub return_value: Option<JsValue>,
    /// Request counter for promise tracking
    pub request_counter: usize,
    /// Map of request IDs to promises
    pub promises: Vec<AsyncGeneratorPromise>,
}

impl Clone for AsyncGeneratorData {
    fn clone(&self) -> Self {
        AsyncGeneratorData {
            state: self.state,
            queue: self.queue.clone(),
            values: self.values.clone(),
            position: self.position,
            return_value: self.return_value.clone(),
            request_counter: self.request_counter,
            promises: self.promises.clone(),
        }
    }
}

/// AsyncGenerator object implementation
///
/// Implements the ES2024 AsyncGenerator protocol with:
/// - State machine for execution control
/// - Queue for concurrent operations
/// - Promise-based result delivery
#[derive(Debug, Clone)]
pub struct AsyncGeneratorObject {
    /// Internal async generator data
    data: Rc<RefCell<AsyncGeneratorData>>,
}

impl AsyncGeneratorObject {
    /// Create a new async generator with preset yield values
    ///
    /// This is a simplified model where the generator yields preset values.
    /// In a real implementation, this would execute async generator function code.
    pub fn new(values: Vec<JsValue>) -> Self {
        AsyncGeneratorObject {
            data: Rc::new(RefCell::new(AsyncGeneratorData {
                state: AsyncGeneratorState::SuspendedStart,
                queue: VecDeque::new(),
                values,
                position: 0,
                return_value: None,
                request_counter: 0,
                promises: Vec::new(),
            })),
        }
    }

    /// Get the current state
    pub fn state(&self) -> AsyncGeneratorState {
        self.data.borrow().state
    }

    /// Get the number of pending requests in the queue
    pub fn queue_length(&self) -> usize {
        self.data.borrow().queue.len()
    }

    /// AsyncGenerator.prototype.next(value?)
    ///
    /// Returns a Promise that resolves to {value, done}
    pub fn next(&self, value: Option<JsValue>) -> AsyncGeneratorPromise {
        let mut data = self.data.borrow_mut();

        // Create request and promise
        let request_id = data.request_counter;
        data.request_counter += 1;

        let request = AsyncGeneratorRequest::next(value);
        let promise = AsyncGeneratorPromise::new(request_id);

        // Add to queue
        data.queue.push_back(request);
        data.promises.push(promise.clone());

        // Process queue if not already executing
        drop(data);
        self.process_queue();

        promise
    }

    /// AsyncGenerator.prototype.return(value?)
    ///
    /// Returns a Promise that resolves to {value, done: true}
    pub fn return_value(&self, value: Option<JsValue>) -> AsyncGeneratorPromise {
        let mut data = self.data.borrow_mut();

        // Create request and promise
        let request_id = data.request_counter;
        data.request_counter += 1;

        let request = AsyncGeneratorRequest::return_request(value);
        let promise = AsyncGeneratorPromise::new(request_id);

        // Add to queue
        data.queue.push_back(request);
        data.promises.push(promise.clone());

        // Process queue if not already executing
        drop(data);
        self.process_queue();

        promise
    }

    /// AsyncGenerator.prototype.throw(exception)
    ///
    /// Returns a Promise that may resolve or reject based on generator state
    pub fn throw(&self, exception: JsValue) -> AsyncGeneratorPromise {
        let mut data = self.data.borrow_mut();

        // Create request and promise
        let request_id = data.request_counter;
        data.request_counter += 1;

        let request = AsyncGeneratorRequest::throw(exception);
        let promise = AsyncGeneratorPromise::new(request_id);

        // Add to queue
        data.queue.push_back(request);
        data.promises.push(promise.clone());

        // Process queue if not already executing
        drop(data);
        self.process_queue();

        promise
    }

    /// Process the queue of pending requests
    ///
    /// This simulates the async generator event loop.
    fn process_queue(&self) {
        let mut data = self.data.borrow_mut();

        // Don't process if already executing
        if data.state == AsyncGeneratorState::Executing {
            return;
        }

        // Process all pending requests
        while let Some(mut request) = data.queue.pop_front() {
            let promise_idx = data.promises.len() - data.queue.len() - 1;

            match request.kind {
                AsyncGeneratorRequestKind::Next => {
                    let result = self.execute_next(&mut data, request.value.clone());
                    if promise_idx < data.promises.len() {
                        match &result {
                            Ok(iter_result) => {
                                data.promises[promise_idx]
                                    .resolve(iter_result.to_js_value());
                            }
                            Err(err) => {
                                data.promises[promise_idx].reject(err.clone());
                            }
                        }
                    }
                    request.result = Some(result);
                    request.completed = true;
                }
                AsyncGeneratorRequestKind::Return => {
                    let result = self.execute_return(&mut data, request.value.clone());
                    if promise_idx < data.promises.len() {
                        match &result {
                            Ok(iter_result) => {
                                data.promises[promise_idx]
                                    .resolve(iter_result.to_js_value());
                            }
                            Err(err) => {
                                data.promises[promise_idx].reject(err.clone());
                            }
                        }
                    }
                    request.result = Some(result);
                    request.completed = true;
                }
                AsyncGeneratorRequestKind::Throw => {
                    let result = self.execute_throw(&mut data, request.value.clone().unwrap());
                    if promise_idx < data.promises.len() {
                        match &result {
                            Ok(iter_result) => {
                                data.promises[promise_idx]
                                    .resolve(iter_result.to_js_value());
                            }
                            Err(err) => {
                                data.promises[promise_idx].reject(err.clone());
                            }
                        }
                    }
                    request.result = Some(result);
                    request.completed = true;
                }
            }
        }
    }

    /// Execute a next operation
    fn execute_next(
        &self,
        data: &mut AsyncGeneratorData,
        _value: Option<JsValue>,
    ) -> JsResult<IteratorResult> {
        match data.state {
            AsyncGeneratorState::Completed => Ok(IteratorResult::done()),
            AsyncGeneratorState::Executing => {
                Err(JsError::type_error("AsyncGenerator is already executing"))
            }
            AsyncGeneratorState::AwaitingReturn => {
                Err(JsError::type_error("AsyncGenerator is awaiting return"))
            }
            AsyncGeneratorState::SuspendedStart | AsyncGeneratorState::SuspendedYield => {
                data.state = AsyncGeneratorState::Executing;

                if data.position < data.values.len() {
                    let value = data.values[data.position].clone();
                    data.position += 1;
                    data.state = AsyncGeneratorState::SuspendedYield;
                    Ok(IteratorResult::value(value))
                } else {
                    data.state = AsyncGeneratorState::Completed;
                    match &data.return_value {
                        Some(v) => Ok(IteratorResult::done_with_value(v.clone())),
                        None => Ok(IteratorResult::done()),
                    }
                }
            }
        }
    }

    /// Execute a return operation
    fn execute_return(
        &self,
        data: &mut AsyncGeneratorData,
        value: Option<JsValue>,
    ) -> JsResult<IteratorResult> {
        if data.state == AsyncGeneratorState::Executing {
            return Err(JsError::type_error("AsyncGenerator is already executing"));
        }

        // Mark as awaiting return, then complete
        data.state = AsyncGeneratorState::AwaitingReturn;

        // Complete the generator
        data.state = AsyncGeneratorState::Completed;
        let return_val = value.unwrap_or(JsValue::undefined());
        Ok(IteratorResult::done_with_value(return_val))
    }

    /// Execute a throw operation
    fn execute_throw(
        &self,
        data: &mut AsyncGeneratorData,
        exception: JsValue,
    ) -> JsResult<IteratorResult> {
        if data.state == AsyncGeneratorState::Executing {
            return Err(JsError::type_error("AsyncGenerator is already executing"));
        }

        // If already completed, the exception is just thrown
        if data.state == AsyncGeneratorState::Completed {
            return Err(JsError::new(exception.to_js_string()));
        }

        // If suspended at start, close and throw
        if data.state == AsyncGeneratorState::SuspendedStart {
            data.state = AsyncGeneratorState::Completed;
            return Err(JsError::new(exception.to_js_string()));
        }

        // Otherwise, propagate the exception through the generator
        data.state = AsyncGeneratorState::Completed;
        Err(JsError::new(exception.to_js_string()))
    }

    /// Check if the async generator is iterable
    pub fn is_iterable(&self) -> bool {
        true
    }

    /// Get the Symbol.asyncIterator method (returns self)
    pub fn get_async_iterator(&self) -> AsyncGeneratorObject {
        self.clone()
    }
}

/// AsyncGeneratorFunction constructor
///
/// Creates async generator functions and objects.
pub struct AsyncGeneratorFunction;

impl AsyncGeneratorFunction {
    /// Create an async generator from a sequence of values
    ///
    /// This is a simplified version that creates a generator yielding preset values.
    /// In a real implementation, this would compile and execute async generator code.
    pub fn from_values(values: Vec<JsValue>) -> AsyncGeneratorObject {
        AsyncGeneratorObject::new(values)
    }

    /// Create an empty async generator
    pub fn empty() -> AsyncGeneratorObject {
        AsyncGeneratorObject::new(vec![])
    }

    /// Create an async generator that yields values from an async iterable
    ///
    /// This simulates yield* delegation.
    pub fn from_async_iterable(values: Vec<JsValue>) -> AsyncGeneratorObject {
        // In a real implementation, this would handle actual async iteration
        // For now, we just create a generator with the values
        AsyncGeneratorObject::new(values)
    }
}

/// Helper for for-await-of iteration
pub struct AsyncIteratorHelper;

impl AsyncIteratorHelper {
    /// Collect all values from an async generator
    ///
    /// This simulates for-await-of behavior, collecting all yielded values.
    pub fn collect(gen: &AsyncGeneratorObject) -> Vec<JsValue> {
        let mut results = vec![];

        loop {
            let promise = gen.next(None);

            // Wait for promise to resolve (synchronous in this simplified model)
            if let Some(value) = promise.value() {
                // Extract iterator result
                if let Some(done_val) = value.get("done") {
                    if done_val.as_boolean() == Some(true) {
                        break;
                    }
                }

                if let Some(result_value) = value.get("value") {
                    results.push(result_value);
                }
            } else if promise.is_rejected() {
                // Error occurred, stop iteration
                break;
            } else {
                // Promise is still pending (shouldn't happen in sync model)
                break;
            }
        }

        results
    }

    /// Map over async generator values
    pub fn map<F>(gen: &AsyncGeneratorObject, mapper: F) -> Vec<JsValue>
    where
        F: Fn(JsValue) -> JsResult<JsValue>,
    {
        let values = Self::collect(gen);
        values
            .into_iter()
            .filter_map(|v| mapper(v).ok())
            .collect()
    }

    /// Filter async generator values
    pub fn filter<F>(gen: &AsyncGeneratorObject, predicate: F) -> Vec<JsValue>
    where
        F: Fn(&JsValue) -> bool,
    {
        let values = Self::collect(gen);
        values.into_iter().filter(|v| predicate(v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_generator_creation() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        assert_eq!(gen.state(), AsyncGeneratorState::SuspendedStart);
        assert_eq!(gen.queue_length(), 0);
    }

    #[test]
    fn test_async_generator_next_returns_promise() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(42.0)]);

        let promise = gen.next(None);

        // Promise should be fulfilled (in our sync model)
        assert!(promise.is_fulfilled());
        assert!(!promise.is_pending());
        assert!(!promise.is_rejected());
    }

    #[test]
    fn test_async_generator_next_yields_values() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        // First next
        let p1 = gen.next(None);
        assert!(p1.is_fulfilled());
        let v1 = p1.value().unwrap();
        assert_eq!(v1.get("value").unwrap().as_number(), Some(1.0));
        assert_eq!(v1.get("done").unwrap().as_boolean(), Some(false));

        // Second next
        let p2 = gen.next(None);
        let v2 = p2.value().unwrap();
        assert_eq!(v2.get("value").unwrap().as_number(), Some(2.0));
        assert_eq!(v2.get("done").unwrap().as_boolean(), Some(false));

        // Third next
        let p3 = gen.next(None);
        let v3 = p3.value().unwrap();
        assert_eq!(v3.get("value").unwrap().as_number(), Some(3.0));
        assert_eq!(v3.get("done").unwrap().as_boolean(), Some(false));

        // Fourth next (done)
        let p4 = gen.next(None);
        let v4 = p4.value().unwrap();
        assert_eq!(v4.get("done").unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_async_generator_return_completes() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
        ]);

        // Call next once
        let _ = gen.next(None);

        // Call return
        let p_ret = gen.return_value(Some(JsValue::string("finished")));
        assert!(p_ret.is_fulfilled());

        let ret_val = p_ret.value().unwrap();
        assert_eq!(ret_val.get("done").unwrap().as_boolean(), Some(true));
        assert_eq!(
            ret_val.get("value").unwrap().as_string(),
            Some("finished".to_string())
        );

        // Generator should be completed
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);

        // Subsequent next should return done
        let p_next = gen.next(None);
        let next_val = p_next.value().unwrap();
        assert_eq!(next_val.get("done").unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_async_generator_throw_propagates_error() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        // Call next to start generator
        let _ = gen.next(None);

        // Call throw
        let p_throw = gen.throw(JsValue::string("Error!"));
        assert!(p_throw.is_rejected());

        let err = p_throw.error().unwrap();
        assert!(err.message.contains("Error!"));

        // Generator should be completed
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);
    }

    #[test]
    fn test_async_generator_state_transitions() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        // Initial state
        assert_eq!(gen.state(), AsyncGeneratorState::SuspendedStart);

        // After first next
        let _ = gen.next(None);
        assert_eq!(gen.state(), AsyncGeneratorState::SuspendedYield);

        // After second next (exhausted)
        let _ = gen.next(None);
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);
    }

    #[test]
    fn test_async_generator_empty() {
        let gen = AsyncGeneratorFunction::empty();

        let p = gen.next(None);
        let v = p.value().unwrap();
        assert_eq!(v.get("done").unwrap().as_boolean(), Some(true));
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);
    }

    #[test]
    fn test_async_generator_queue_management() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        // Queue multiple requests (they're processed immediately in sync model)
        let p1 = gen.next(None);
        let p2 = gen.next(None);
        let p3 = gen.next(None);

        // All should be fulfilled
        assert!(p1.is_fulfilled());
        assert!(p2.is_fulfilled());
        assert!(p3.is_fulfilled());

        // Values should be sequential
        assert_eq!(
            p1.value().unwrap().get("value").unwrap().as_number(),
            Some(1.0)
        );
        assert_eq!(
            p2.value().unwrap().get("value").unwrap().as_number(),
            Some(2.0)
        );
        assert_eq!(
            p3.value().unwrap().get("value").unwrap().as_number(),
            Some(3.0)
        );
    }

    #[test]
    fn test_async_generator_promise_request_id() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        let p1 = gen.next(None);
        let p2 = gen.next(None);

        // Each promise should have unique request ID
        assert_eq!(p1.request_id(), 0);
        assert_eq!(p2.request_id(), 1);
    }

    #[test]
    fn test_async_generator_is_iterable() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);
        assert!(gen.is_iterable());
    }

    #[test]
    fn test_async_generator_get_async_iterator() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);
        let iter = gen.get_async_iterator();

        // Should return self (generator is its own iterator)
        assert_eq!(iter.state(), gen.state());
    }

    #[test]
    fn test_async_generator_throw_on_suspended_start() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        // Throw without calling next first
        let p = gen.throw(JsValue::string("Early error"));
        assert!(p.is_rejected());
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);
    }

    #[test]
    fn test_async_generator_throw_on_completed() {
        let gen = AsyncGeneratorFunction::empty();

        // Complete the generator
        let _ = gen.next(None);
        assert_eq!(gen.state(), AsyncGeneratorState::Completed);

        // Throw on completed generator
        let p = gen.throw(JsValue::string("Late error"));
        assert!(p.is_rejected());
    }

    #[test]
    fn test_async_generator_return_without_value() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        let p = gen.return_value(None);
        let v = p.value().unwrap();

        assert_eq!(v.get("done").unwrap().as_boolean(), Some(true));
        assert!(v.get("value").unwrap().is_undefined());
    }

    #[test]
    fn test_promise_to_js_value_pending() {
        let promise = AsyncGeneratorPromise::new(0);
        let obj = promise.to_js_value();

        assert_eq!(obj.get("state").unwrap().as_string(), Some("pending".to_string()));
    }

    #[test]
    fn test_promise_to_js_value_fulfilled() {
        let promise = AsyncGeneratorPromise::new(0);
        promise.resolve(JsValue::number(42.0));
        let obj = promise.to_js_value();

        assert_eq!(obj.get("state").unwrap().as_string(), Some("fulfilled".to_string()));
        assert_eq!(obj.get("value").unwrap().as_number(), Some(42.0));
    }

    #[test]
    fn test_promise_to_js_value_rejected() {
        let promise = AsyncGeneratorPromise::new(0);
        promise.reject(JsError::new("test error"));
        let obj = promise.to_js_value();

        assert_eq!(obj.get("state").unwrap().as_string(), Some("rejected".to_string()));
        assert!(obj.get("reason").unwrap().as_string().unwrap().contains("test error"));
    }

    #[test]
    fn test_async_iterator_helper_collect() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let values = AsyncIteratorHelper::collect(&gen);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_number(), Some(1.0));
        assert_eq!(values[1].as_number(), Some(2.0));
        assert_eq!(values[2].as_number(), Some(3.0));
    }

    #[test]
    fn test_async_iterator_helper_map() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
        ]);

        let doubled = AsyncIteratorHelper::map(&gen, |v| {
            Ok(JsValue::number(v.as_number().unwrap() * 2.0))
        });

        assert_eq!(doubled.len(), 3);
        assert_eq!(doubled[0].as_number(), Some(2.0));
        assert_eq!(doubled[1].as_number(), Some(4.0));
        assert_eq!(doubled[2].as_number(), Some(6.0));
    }

    #[test]
    fn test_async_iterator_helper_filter() {
        let gen = AsyncGeneratorFunction::from_values(vec![
            JsValue::number(1.0),
            JsValue::number(2.0),
            JsValue::number(3.0),
            JsValue::number(4.0),
        ]);

        let evens = AsyncIteratorHelper::filter(&gen, |v| {
            v.as_number().unwrap() % 2.0 == 0.0
        });

        assert_eq!(evens.len(), 2);
        assert_eq!(evens[0].as_number(), Some(2.0));
        assert_eq!(evens[1].as_number(), Some(4.0));
    }

    #[test]
    fn test_async_generator_from_async_iterable() {
        let gen = AsyncGeneratorFunction::from_async_iterable(vec![
            JsValue::string("a"),
            JsValue::string("b"),
            JsValue::string("c"),
        ]);

        let values = AsyncIteratorHelper::collect(&gen);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_string(), Some("a".to_string()));
        assert_eq!(values[1].as_string(), Some("b".to_string()));
        assert_eq!(values[2].as_string(), Some("c".to_string()));
    }

    #[test]
    fn test_async_generator_multiple_returns() {
        let gen = AsyncGeneratorFunction::from_values(vec![JsValue::number(1.0)]);

        // First return
        let p1 = gen.return_value(Some(JsValue::string("first")));
        assert!(p1.is_fulfilled());

        // Second return (on completed generator)
        let p2 = gen.return_value(Some(JsValue::string("second")));
        assert!(p2.is_fulfilled());

        // Both should have done=true
        assert_eq!(
            p1.value().unwrap().get("done").unwrap().as_boolean(),
            Some(true)
        );
        assert_eq!(
            p2.value().unwrap().get("done").unwrap().as_boolean(),
            Some(true)
        );
    }
}
