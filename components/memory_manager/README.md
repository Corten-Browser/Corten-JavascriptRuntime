# memory_manager

Memory management and garbage collection for the Corten JavaScript Runtime.

## Overview

This component provides a generational garbage collector and heap management system:

- **Generational GC**: Young and old generation with different collection strategies
- **Hidden Classes**: Property access optimization using V8-style hidden classes
- **JSObject**: JavaScript object representation with efficient property storage
- **Write Barriers**: GC correctness for remembered set and tri-color invariant

## Features

### Heap Management
- Arena-based allocation with bump pointer
- Young generation (1MB) for new allocations
- Old generation (4MB) for long-lived objects
- Automatic garbage collection when young generation fills

### Hidden Class System
- Objects with same property order share hidden classes
- Fast property access using offset-based lookups
- Transition mechanism when properties are added
- Memory-efficient object shape tracking

### Write Barriers
- Remembered set tracking for old-to-young pointers
- Tri-color invariant maintenance during concurrent marking
- Safe global heap initialization

## Usage

```rust
use memory_manager::{Heap, HiddenClass, JSObject};
use core_types::Value;

// Create a heap
let mut heap = Heap::new();

// Allocate memory
let ptr = heap.allocate(64);
assert!(!ptr.is_null());

// Create a JavaScript object
let class = Box::new(HiddenClass::new());
let class_ptr = Box::into_raw(class);
let mut obj = JSObject::new(class_ptr);

// Set and get properties
obj.set_property("name".to_string(), Value::Smi(42));
assert_eq!(obj.get_property("name"), Some(Value::Smi(42)));

// Trigger garbage collection
heap.collect_garbage();

// Clean up
unsafe { let _ = Box::from_raw(class_ptr); }
```

## API Contract

### Heap
- `new() -> Self` - Create new heap with default generation sizes
- `allocate(size: usize) -> *mut u8` - Allocate from young generation
- `collect_garbage()` - Trigger garbage collection
- `young_generation_size() -> usize` - Get young gen capacity
- `old_generation_size() -> usize` - Get old gen capacity

### HiddenClass
- `new() -> Self` - Create empty hidden class
- `add_property(name: String) -> Box<HiddenClass>` - Add property, get new class
- `lookup_property(name: &str) -> Option<u32>` - Get property offset

### JSObject
- `new(class: *const HiddenClass) -> Self` - Create object with class
- `get_property(name: &str) -> Option<Value>` - Get property value
- `set_property(name: String, value: Value)` - Set property value

### Functions
- `write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value)` - GC write barrier (unsafe)

## Dependencies

- `core_types`: Value, JsError types

## Testing

```bash
cargo test
```

**Test Coverage:**
- 39 unit tests
- 13 contract tests
- 6 documentation tests
- Total: 58 tests passing

**Quality Checks:**
- `cargo fmt` clean
- `cargo clippy` passes with `-D warnings`
- All unsafe blocks documented

## Implementation Details

### Unsafe Code Documentation

All unsafe blocks are documented with SAFETY comments explaining:
- Why the operation is safe
- What invariants must be maintained
- What preconditions must be met

### Memory Safety

- Null pointer checks before dereferencing
- Bounds checking on array accesses
- Proper cleanup with `Box::from_raw`
- No use-after-free scenarios

### GC Invariants

- **Remembered Set**: Tracks old-to-young pointers to ensure young GC scans all roots
- **Tri-color Invariant**: Prevents black objects from pointing to white objects during marking
- **Write Barriers**: Called on every pointer write to maintain invariants

## Files

```
src/
├── lib.rs            # Public API exports
├── heap.rs           # Heap, Arena, Object, ObjectHeader
├── hidden_class.rs   # HiddenClass, PropertyDescriptor
├── object.rs         # JSObject
└── barriers.rs       # write_barrier, global heap management

tests/
├── contracts/
│   ├── mod.rs
│   └── api_contract.rs  # Contract compliance tests
└── contract_tests.rs    # Test runner
```

## See Also

- `CLAUDE.md` - Implementation requirements and guidelines
- `component.yaml` - Component metadata and dependencies
- `contracts/memory_manager.yaml` - API contract specification
