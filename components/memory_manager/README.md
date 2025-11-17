# Memory Manager

Garbage collector and heap management for the Corten JavaScript Runtime.

## Overview

The Memory Manager component provides:

- **Generational Garbage Collection**: Efficient memory reclamation with young and old generation heaps
- **Heap Management**: Memory allocation and deallocation for JavaScript values
- **Hidden Classes**: Object shape tracking for fast property access
- **Write Barriers**: Maintaining GC invariants for generational collection

## Architecture

```
┌─────────────────────────────────────────┐
│              Memory Manager             │
├─────────────┬───────────┬───────────────┤
│    Heap     │    GC     │ Hidden Class  │
│  (alloc)    │ (collect) │   (shapes)    │
├─────────────┴───────────┴───────────────┤
│              Write Barrier              │
└─────────────────────────────────────────┘
```

## Quick Start

```rust
use memory_manager::{Heap, HiddenClass, JSObject};

// Create a heap
let mut heap = Heap::new();

// Create a hidden class for objects
let class = HiddenClass::new();
let class_with_x = class.add_property("x".to_string());

// Create an object
let obj = JSObject::new(class_with_x.as_ref() as *const _);
```

## Modules

- **heap**: Memory allocation and heap management
- **gc**: Garbage collection algorithms
- **hidden_class**: Object shape tracking
- **object**: JavaScript object representation
- **write_barrier**: GC write barrier implementation

## Safety

This component uses unsafe Rust internally for performance-critical memory operations. All public APIs provide safe wrappers with documented safety invariants.

## Testing

```bash
cargo test
```

## Dependencies

- `core_types`: JavaScript value types

## License

Part of the Corten JavaScript Runtime project.
