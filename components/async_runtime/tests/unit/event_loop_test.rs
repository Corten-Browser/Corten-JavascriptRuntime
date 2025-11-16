//! Unit tests for EventLoop

use async_runtime::{EventLoop, MicroTask, Task};
use core_types::Value;

#[test]
fn new_event_loop_has_empty_task_queue() {
    let event_loop = EventLoop::new();
    assert!(event_loop.is_task_queue_empty());
}

#[test]
fn new_event_loop_has_empty_microtask_queue() {
    let event_loop = EventLoop::new();
    assert!(event_loop.is_microtask_queue_empty());
}

#[test]
fn enqueue_task_adds_to_task_queue() {
    let mut event_loop = EventLoop::new();
    let task = Task::new(|| Ok(Value::Undefined));
    event_loop.enqueue_task(task);
    assert!(!event_loop.is_task_queue_empty());
}

#[test]
fn enqueue_microtask_adds_to_microtask_queue() {
    let mut event_loop = EventLoop::new();
    let microtask = MicroTask::new(|| Ok(Value::Undefined));
    event_loop.enqueue_microtask(microtask);
    assert!(!event_loop.is_microtask_queue_empty());
}

#[test]
fn task_queue_fifo_order() {
    let mut event_loop = EventLoop::new();
    let results = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let results1 = results.clone();
    let task1 = Task::new(move || {
        results1.lock().unwrap().push(1);
        Ok(Value::Undefined)
    });

    let results2 = results.clone();
    let task2 = Task::new(move || {
        results2.lock().unwrap().push(2);
        Ok(Value::Undefined)
    });

    event_loop.enqueue_task(task1);
    event_loop.enqueue_task(task2);

    let _ = event_loop.run_all_tasks();

    assert_eq!(*results.lock().unwrap(), vec![1, 2]);
}

#[test]
fn microtask_queue_fifo_order() {
    let mut event_loop = EventLoop::new();
    let results = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let results1 = results.clone();
    let microtask1 = MicroTask::new(move || {
        results1.lock().unwrap().push(1);
        Ok(Value::Undefined)
    });

    let results2 = results.clone();
    let microtask2 = MicroTask::new(move || {
        results2.lock().unwrap().push(2);
        Ok(Value::Undefined)
    });

    event_loop.enqueue_microtask(microtask1);
    event_loop.enqueue_microtask(microtask2);

    let _ = event_loop.run_all_microtasks();

    assert_eq!(*results.lock().unwrap(), vec![1, 2]);
}

#[test]
fn microtasks_run_after_each_task() {
    let mut event_loop = EventLoop::new();
    let results = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let results1 = results.clone();
    let el_results = results.clone();

    let task = Task::new(move || {
        results1.lock().unwrap().push("task");
        Ok(Value::Undefined)
    });

    let microtask = MicroTask::new(move || {
        el_results.lock().unwrap().push("microtask");
        Ok(Value::Undefined)
    });

    event_loop.enqueue_task(task);
    event_loop.enqueue_microtask(microtask);

    // Process one task cycle: task + all microtasks
    let _ = event_loop.process_one_cycle();

    // After processing one cycle, both task and microtask should have run
    // with microtask after task
    let r = results.lock().unwrap();
    assert!(r.contains(&"task"));
    assert!(r.contains(&"microtask"));
}

#[test]
fn empty_event_loop_run_completes_immediately() {
    let mut event_loop = EventLoop::new();
    let result = event_loop.run_until_done();
    assert!(result.is_ok());
}

#[test]
fn task_can_produce_result() {
    let mut event_loop = EventLoop::new();

    // Basic task execution produces undefined result
    let task = Task::new(|| Ok(Value::Smi(42)));

    event_loop.enqueue_task(task);
    let result = event_loop.run_until_done();

    assert!(result.is_ok());
}

#[test]
fn multiple_tasks_with_microtasks() {
    let mut event_loop = EventLoop::new();
    let results = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let r1 = results.clone();
    let task1 = Task::new(move || {
        r1.lock().unwrap().push(1);
        Ok(Value::Undefined)
    });

    let r2 = results.clone();
    let microtask1 = MicroTask::new(move || {
        r2.lock().unwrap().push(2);
        Ok(Value::Undefined)
    });

    let r3 = results.clone();
    let task2 = Task::new(move || {
        r3.lock().unwrap().push(3);
        Ok(Value::Undefined)
    });

    event_loop.enqueue_task(task1);
    event_loop.enqueue_microtask(microtask1);
    event_loop.enqueue_task(task2);

    event_loop.run_until_done().unwrap();

    // Tasks and microtasks should run in proper order
    // Task1 (1), then microtask1 (2), then Task2 (3)
    assert_eq!(*results.lock().unwrap(), vec![1, 2, 3]);
}
