//! Event loop implementation.
//!
//! This module provides the main event loop that coordinates task and microtask
//! execution following the JavaScript event loop model.

use crate::task_queue::{MicroTask, MicrotaskQueue, Task, TaskQueue};
use core_types::JsError;

/// The JavaScript event loop.
///
/// The event loop processes tasks and microtasks according to the HTML5 event loop
/// specification. Each iteration (turn) of the loop:
/// 1. Takes the oldest task from the task queue and executes it
/// 2. Drains all microtasks in the microtask queue
/// 3. Renders (if needed, not implemented here)
/// 4. Repeats
///
/// # Examples
///
/// ```
/// use async_runtime::{EventLoop, Task, MicroTask};
/// use core_types::Value;
///
/// let mut event_loop = EventLoop::new();
///
/// event_loop.enqueue_task(Task::new(|| Ok(Value::Undefined)));
/// event_loop.run_until_done().unwrap();
/// ```
#[derive(Debug, Default)]
pub struct EventLoop {
    task_queue: TaskQueue,
    microtask_queue: MicrotaskQueue,
}

impl EventLoop {
    /// Creates a new EventLoop with empty queues.
    pub fn new() -> Self {
        Self {
            task_queue: TaskQueue::new(),
            microtask_queue: MicrotaskQueue::new(),
        }
    }

    /// Runs the event loop until all tasks and microtasks are processed.
    ///
    /// This is similar to `run_until_complete` but doesn't require a VM.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all tasks completed successfully, or an error if any task failed.
    pub fn run_until_done(&mut self) -> Result<(), JsError> {
        // Process all tasks
        while !self.task_queue.is_empty() || !self.microtask_queue.is_empty() {
            // Execute one task (if available)
            if let Some(task) = self.task_queue.dequeue() {
                task.run()?;
            }

            // Drain all microtasks
            self.run_all_microtasks()?;
        }

        Ok(())
    }

    /// Adds a task to the task queue.
    ///
    /// The task will be executed in the next available iteration of the event loop.
    pub fn enqueue_task(&mut self, task: Task) {
        self.task_queue.enqueue(task);
    }

    /// Adds a microtask to the microtask queue.
    ///
    /// The microtask will be executed after the current task completes.
    pub fn enqueue_microtask(&mut self, microtask: MicroTask) {
        self.microtask_queue.enqueue(microtask);
    }

    /// Returns true if the task queue is empty.
    pub fn is_task_queue_empty(&self) -> bool {
        self.task_queue.is_empty()
    }

    /// Returns true if the microtask queue is empty.
    pub fn is_microtask_queue_empty(&self) -> bool {
        self.microtask_queue.is_empty()
    }

    /// Runs all microtasks in the queue until empty.
    ///
    /// This drains the microtask queue completely. New microtasks added during
    /// execution will also be processed before this method returns.
    pub fn run_all_microtasks(&mut self) -> Result<(), JsError> {
        while let Some(microtask) = self.microtask_queue.dequeue() {
            microtask.run()?;
        }
        Ok(())
    }

    /// Runs all tasks in the queue (without processing microtasks between them).
    ///
    /// This is primarily for testing purposes.
    pub fn run_all_tasks(&mut self) -> Result<(), JsError> {
        while let Some(task) = self.task_queue.dequeue() {
            task.run()?;
        }
        Ok(())
    }

    /// Processes one complete cycle: one task followed by all microtasks.
    ///
    /// This represents one iteration of the event loop.
    pub fn process_one_cycle(&mut self) -> Result<(), JsError> {
        // Execute one task if available
        if let Some(task) = self.task_queue.dequeue() {
            task.run()?;
        }

        // Drain all microtasks
        self.run_all_microtasks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::Value;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_new_event_loop() {
        let el = EventLoop::new();
        assert!(el.is_task_queue_empty());
        assert!(el.is_microtask_queue_empty());
    }

    #[test]
    fn test_enqueue_task() {
        let mut el = EventLoop::new();
        el.enqueue_task(Task::new(|| Ok(Value::Undefined)));
        assert!(!el.is_task_queue_empty());
    }

    #[test]
    fn test_enqueue_microtask() {
        let mut el = EventLoop::new();
        el.enqueue_microtask(MicroTask::new(|| Ok(Value::Undefined)));
        assert!(!el.is_microtask_queue_empty());
    }

    #[test]
    fn test_run_until_done_empty() {
        let mut el = EventLoop::new();
        let result = el.run_until_done();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_until_done_with_tasks() {
        let mut el = EventLoop::new();
        let counter = Arc::new(Mutex::new(0));

        let c = counter.clone();
        el.enqueue_task(Task::new(move || {
            *c.lock().unwrap() += 1;
            Ok(Value::Undefined)
        }));

        let c = counter.clone();
        el.enqueue_task(Task::new(move || {
            *c.lock().unwrap() += 1;
            Ok(Value::Undefined)
        }));

        el.run_until_done().unwrap();
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_microtasks_after_tasks() {
        let mut el = EventLoop::new();
        let order = Arc::new(Mutex::new(vec![]));

        let o = order.clone();
        el.enqueue_task(Task::new(move || {
            o.lock().unwrap().push('T');
            Ok(Value::Undefined)
        }));

        let o = order.clone();
        el.enqueue_microtask(MicroTask::new(move || {
            o.lock().unwrap().push('M');
            Ok(Value::Undefined)
        }));

        el.run_until_done().unwrap();

        // Task should run before microtask
        assert_eq!(*order.lock().unwrap(), vec!['T', 'M']);
    }
}
