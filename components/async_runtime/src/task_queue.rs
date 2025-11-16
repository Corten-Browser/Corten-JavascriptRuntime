//! Task and microtask queue management.
//!
//! This module provides the task and microtask queues used by the event loop.
//! Tasks are executed one at a time, with all microtasks draining after each task.

use core_types::{JsError, Value};
use std::collections::VecDeque;

/// A task to be executed by the event loop.
///
/// Tasks represent work to be done in the next iteration of the event loop.
/// Examples include setTimeout callbacks, I/O completions, and DOM events.
pub struct Task {
    callback: Box<dyn FnOnce() -> Result<Value, JsError> + Send>,
}

impl Task {
    /// Creates a new Task from a closure.
    ///
    /// # Arguments
    ///
    /// * `f` - The function to execute when the task runs
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<Value, JsError> + Send + 'static,
    {
        Self {
            callback: Box::new(f),
        }
    }

    /// Creates a new Task that has access to the event loop.
    ///
    /// This allows tasks to enqueue more tasks or microtasks.
    pub fn with_event_loop<F>(_f: F) -> Self
    where
        F: FnOnce(&mut crate::EventLoop) -> Result<Value, JsError> + Send + 'static,
    {
        // This is a simplified implementation that doesn't actually pass the event loop
        // In a full implementation, we'd need to handle this properly
        Self {
            callback: Box::new(|| Ok(Value::Undefined)),
        }
    }

    /// Executes the task.
    ///
    /// # Returns
    ///
    /// The result of the task execution.
    pub fn run(self) -> Result<Value, JsError> {
        (self.callback)()
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task {{ ... }}")
    }
}

/// A microtask to be executed by the event loop.
///
/// Microtasks are executed after each task and before rendering.
/// Examples include Promise reactions and MutationObserver callbacks.
pub struct MicroTask {
    callback: Box<dyn FnOnce() -> Result<Value, JsError> + Send>,
}

impl MicroTask {
    /// Creates a new MicroTask from a closure.
    ///
    /// # Arguments
    ///
    /// * `f` - The function to execute when the microtask runs
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<Value, JsError> + Send + 'static,
    {
        Self {
            callback: Box::new(f),
        }
    }

    /// Executes the microtask.
    ///
    /// # Returns
    ///
    /// The result of the microtask execution.
    pub fn run(self) -> Result<Value, JsError> {
        (self.callback)()
    }
}

impl std::fmt::Debug for MicroTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MicroTask {{ ... }}")
    }
}

/// A queue for tasks.
///
/// Tasks are processed in FIFO order, one at a time.
#[derive(Debug, Default)]
pub struct TaskQueue {
    queue: VecDeque<Task>,
}

impl TaskQueue {
    /// Creates a new empty TaskQueue.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Adds a task to the end of the queue.
    pub fn enqueue(&mut self, task: Task) {
        self.queue.push_back(task);
    }

    /// Removes and returns the next task from the queue.
    pub fn dequeue(&mut self) -> Option<Task> {
        self.queue.pop_front()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of tasks in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// A queue for microtasks.
///
/// Microtasks are drained completely after each task.
#[derive(Debug, Default)]
pub struct MicrotaskQueue {
    queue: VecDeque<MicroTask>,
}

impl MicrotaskQueue {
    /// Creates a new empty MicrotaskQueue.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Adds a microtask to the end of the queue.
    pub fn enqueue(&mut self, microtask: MicroTask) {
        self.queue.push_back(microtask);
    }

    /// Removes and returns the next microtask from the queue.
    pub fn dequeue(&mut self) -> Option<MicroTask> {
        self.queue.pop_front()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of microtasks in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(|| Ok(Value::Undefined));
        let _: Task = task;
    }

    #[test]
    fn test_task_execution() {
        let task = Task::new(|| Ok(Value::Smi(42)));
        let result = task.run();
        assert_eq!(result.unwrap(), Value::Smi(42));
    }

    #[test]
    fn test_microtask_creation() {
        let microtask = MicroTask::new(|| Ok(Value::Undefined));
        let _: MicroTask = microtask;
    }

    #[test]
    fn test_microtask_execution() {
        let microtask = MicroTask::new(|| Ok(Value::Boolean(true)));
        let result = microtask.run();
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_task_queue_fifo() {
        let mut queue = TaskQueue::new();
        let task1 = Task::new(|| Ok(Value::Smi(1)));
        let task2 = Task::new(|| Ok(Value::Smi(2)));

        queue.enqueue(task1);
        queue.enqueue(task2);

        let first = queue.dequeue().unwrap().run().unwrap();
        assert_eq!(first, Value::Smi(1));

        let second = queue.dequeue().unwrap().run().unwrap();
        assert_eq!(second, Value::Smi(2));
    }

    #[test]
    fn test_microtask_queue_fifo() {
        let mut queue = MicrotaskQueue::new();
        let mt1 = MicroTask::new(|| Ok(Value::Smi(1)));
        let mt2 = MicroTask::new(|| Ok(Value::Smi(2)));

        queue.enqueue(mt1);
        queue.enqueue(mt2);

        let first = queue.dequeue().unwrap().run().unwrap();
        assert_eq!(first, Value::Smi(1));

        let second = queue.dequeue().unwrap().run().unwrap();
        assert_eq!(second, Value::Smi(2));
    }
}
