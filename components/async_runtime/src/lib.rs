//! Async runtime for JavaScript execution.
//!
//! This crate provides the async runtime components for a JavaScript engine:
//! - Event loop with task and microtask queues
//! - Promise implementation following the Promise/A+ specification
//! - ES module system with full lifecycle support
//!
//! # Overview
//!
//! The async runtime handles asynchronous JavaScript operations:
//! - [`EventLoop`] - Main event loop coordinating task execution
//! - [`Promise`] - Promise/A+ compliant implementation
//! - [`Module`] - ES module system with import/export support
//!
//! # Examples
//!
//! ## Event Loop Usage
//!
//! ```
//! use async_runtime::{EventLoop, Task};
//! use core_types::Value;
//!
//! let mut event_loop = EventLoop::new();
//! event_loop.enqueue_task(Task::new(|| Ok(Value::Undefined)));
//! event_loop.run_until_done().unwrap();
//! ```
//!
//! ## Promise Usage
//!
//! ```
//! use async_runtime::{Promise, PromiseState};
//! use core_types::Value;
//!
//! let mut promise = Promise::new();
//! promise.resolve(Value::Smi(42));
//! assert!(matches!(promise.state, PromiseState::Fulfilled));
//! ```
//!
//! ## Module Usage
//!
//! ```
//! use async_runtime::{Module, ModuleStatus};
//!
//! let mut module = Module::new("export default 42;".to_string());
//! module.link().unwrap();
//! module.evaluate().unwrap();
//! assert!(matches!(module.status, ModuleStatus::Evaluated));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod event_loop;
pub mod module;
pub mod promise;
pub mod task_queue;

// Re-export main types at crate root
pub use event_loop::EventLoop;
pub use module::{ExportEntry, ImportEntry, Module, ModuleStatus};
pub use promise::{Function, Promise, PromiseReaction, PromiseState};
pub use task_queue::{MicroTask, MicrotaskQueue, Task, TaskQueue};
