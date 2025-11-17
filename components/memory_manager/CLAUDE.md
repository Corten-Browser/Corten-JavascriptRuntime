# Memory Manager Component

## Component Information

- **Name**: memory_manager
- **Version**: 0.1.0
- **Type**: Core
- **Language**: Rust

## Description

Garbage collector and heap management for the JavaScript runtime. This component provides memory allocation, generational garbage collection, and object layout optimization through hidden classes.

## Tech Stack

- **Rust**: Primary implementation language
- **Unsafe internals**: Low-level memory operations require unsafe Rust
- **Safe wrappers**: All unsafe operations wrapped in safe, documented APIs
- **No external GC libraries**: Custom implementation for JavaScript semantics

## Responsibilities

1. **Heap Management**
   - Memory allocation for JavaScript values
   - Young and old generation heap spaces
   - Memory reclamation through garbage collection

2. **Generational Garbage Collection**
   - Young generation (nursery) for short-lived objects
   - Old generation for long-lived objects
   - Promotion policies for objects surviving collections
   - Write barriers for maintaining remembered sets

3. **Hidden Classes**
   - Object shape tracking for property access optimization
   - Transition chains for property additions
   - Inline cache support for fast property access

4. **Object Layout**
   - Efficient property storage based on hidden classes
   - Array element storage for indexed access
   - Memory-efficient representation

## Dependencies

- `core_types`: JavaScript value types (Value, String, etc.)

## Module Structure

```
src/
├── lib.rs           # Module declarations and re-exports
├── heap.rs          # Heap allocation and management
├── gc.rs            # GC algorithms and policies
├── hidden_class.rs  # Hidden class implementation
├── object.rs        # JSObject representation
└── write_barrier.rs # Write barrier for generational GC
```

## Safety Requirements

- **All unsafe blocks must be documented** with safety invariants
- **Safe wrappers required** for all public APIs using unsafe internals
- **Memory safety** must be maintained across all operations
- **No undefined behavior** from improper pointer usage

## Quality Standards

- Minimum 80% test coverage
- All unsafe code blocks documented
- Integration tests with core_types
- Performance benchmarks for GC operations

## Testing Strategy

1. **Unit Tests**: Individual component functionality
2. **Integration Tests**: Interaction between heap, GC, and objects
3. **Property Tests**: Invariant checking (e.g., no dangling pointers)
4. **Stress Tests**: Large allocation and collection scenarios
5. **Memory Safety Tests**: Verify no leaks or corruption

## Key Interfaces

### Heap
```rust
impl Heap {
    fn new() -> Self;
    fn allocate(&mut self, size: usize) -> *mut u8;
    fn collect_garbage(&mut self);
    fn young_generation_size(&self) -> usize;
    fn old_generation_size(&self) -> usize;
}
```

### HiddenClass
```rust
impl HiddenClass {
    fn new() -> Self;
    fn add_property(&self, name: String) -> Box<HiddenClass>;
    fn lookup_property(&self, name: &str) -> Option<u32>;
}
```

### JSObject
```rust
impl JSObject {
    fn new(class: *const HiddenClass) -> Self;
    fn get_property(&self, name: &str) -> Option<Value>;
    fn set_property(&mut self, name: String, value: Value);
}
```

### Write Barrier
```rust
unsafe fn write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value);
```

## Development Notes

- Start with simple mark-and-sweep, optimize later
- Hidden classes are immutable; transitions create new classes
- Write barriers are critical for GC correctness
- Consider concurrent/incremental GC for future optimization
