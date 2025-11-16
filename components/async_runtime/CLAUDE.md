# async_runtime Component

**Type**: Feature Library (Level 2)
**Tech Stack**: Rust, tokio (optional)
**Version**: 0.1.0

## Purpose
Event loop implementation with task and microtask queues, Promise engine with async/await support, and ES module system.

## Dependencies
- `interpreter`: VM, ExecutionContext
- `builtins`: ObjectPrototype

## Token Budget
- Optimal: 60,000 tokens
- Warning: 80,000 tokens
- Critical: 100,000 tokens

## Exported Types

```rust
// Event loop
pub struct EventLoop {
    task_queue: VecDeque<Task>,
    microtask_queue: VecDeque<MicroTask>,
}

impl EventLoop {
    pub fn run_until_complete(&mut self, vm: &mut VM) -> Result<(), JsError>;
    pub fn enqueue_task(&mut self, task: Task);
    pub fn enqueue_microtask(&mut self, microtask: MicroTask);
}

// Promise
pub struct Promise {
    state: PromiseState,
    reactions: Vec<PromiseReaction>,
    result: Option<Value>,
}

pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

// Module system
pub struct Module {
    source: String,
    status: ModuleStatus,
    environment: ModuleEnvironment,
    imports: Vec<ImportEntry>,
    exports: Vec<ExportEntry>,
}

pub enum ModuleStatus {
    Unlinked,
    Linking,
    Linked,
    Evaluating,
    Evaluated,
    Error(JsError),
}

// Task queues
pub struct TaskQueue;
pub struct MicrotaskQueue;
```

## Key Implementation Requirements

### Event Loop
1. Process all microtasks after each task
2. Task queue priorities (DOM, user, network, timer)
3. Never block - async all I/O

### Promise Implementation
- Promise resolution procedure per spec
- Promise.all, Promise.race, Promise.any
- Promise.resolve, Promise.reject
- Chaining with .then(), .catch(), .finally()

### Async/Await
- Transform async functions to state machines
- Each await point becomes state transition
- Generator-based implementation

### ES Modules
- Module loading pipeline: fetch → parse → link → evaluate
- Top-level await support
- Dynamic import() with Promise API
- Circular dependency handling

## Mandatory Requirements

### 1. Test-Driven Development
- Test each queue operation
- 80%+ coverage
- TDD pattern in commits

### 2. File Structure
```
src/
  lib.rs             # Public exports
  event_loop.rs      # Main event loop
  task_queue.rs      # Task/microtask queues
  promise.rs         # Promise implementation
  async_function.rs  # Async/await support
  module.rs          # ES module system
  module_loader.rs   # Module fetching/linking
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[async_runtime] <type>: <description>
```

## Definition of Done
- [ ] Event loop functional
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Promise spec compliant
- [ ] Module system working
- [ ] Contract tests passing
