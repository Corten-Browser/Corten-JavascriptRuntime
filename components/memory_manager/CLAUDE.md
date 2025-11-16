# memory_manager Component

**Type**: Core Library (Level 1)
**Tech Stack**: Rust, parking_lot, crossbeam
**Version**: 0.1.0

## Purpose
Garbage collector implementation with generational collection, hidden class system for property access optimization, and heap management using arena allocation.

## Dependencies
- `core_types`: Value, JsError

## Token Budget
- Optimal: 60,000 tokens
- Warning: 80,000 tokens
- Critical: 100,000 tokens

## Exported Types

```rust
// Heap management
pub struct Heap {
    pub young_gen: Arena<Object>,
    pub old_gen: Arena<Object>,
    pub remembered_set: HashSet<*mut Object>,
}

// Hidden classes for property optimization
pub struct HiddenClass {
    pub properties: Vec<PropertyDescriptor>,
    pub transitions: HashMap<String, Box<HiddenClass>>,
    pub prototype: Option<ObjectRef>,
}

// JavaScript object representation
pub struct JSObject {
    pub class: *const HiddenClass,
    pub properties: Vec<Value>,
    pub elements: Vec<Value>,
}

// Write barrier for GC
pub unsafe fn write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value);

// Arena allocator
pub struct Arena<T> {
    // Semi-space for copying collector
}
```

## Key Implementation Requirements

### Generational GC
1. **Young Generation**: Semi-space copying collector
   - Fast bump-pointer allocation
   - Scavenge when from-space full
   - Copy survivors to to-space

2. **Old Generation**: Mark-and-sweep
   - Tri-color marking (white/gray/black)
   - Concurrent marking capability
   - Write barriers for remembered set

### Write Barriers
```rust
unsafe fn write_barrier(obj: *mut Object, slot: *mut Value, new_val: Value) {
    *slot = new_val;
    // Track old-to-young pointers
    if is_in_old_gen(obj) && is_in_young_gen(new_val) {
        REMEMBERED_SET.insert(obj);
    }
    // Maintain tri-color invariant during marking
    if is_marking() && is_black(obj) && is_white(new_val) {
        mark_gray(new_val);
    }
}
```

### Hidden Classes
- Objects with same properties in same order share hidden class
- Transitions create new hidden classes
- Enable fast property access (offset-based)

## Mandatory Requirements

### 1. Test-Driven Development
- Write tests FIRST
- 80%+ coverage
- TDD pattern in git commits

### 2. Unsafe Rust Documentation
Every unsafe block must have:
```rust
// SAFETY: <explain why this is safe>
unsafe {
    // code
}
```

### 3. Memory Safety Testing
- Test for memory leaks
- Test write barriers
- Test GC correctness

### 4. File Structure
```
src/
  lib.rs           # Public exports
  heap.rs          # Heap and Arena
  gc.rs            # Garbage collector
  hidden_class.rs  # Hidden class system
  object.rs        # JSObject
  barriers.rs      # Write barriers
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[memory_manager] <type>: <description>
```

## Definition of Done
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] All unsafe blocks documented
- [ ] GC correctness tests passing
- [ ] No memory leaks
- [ ] `cargo fmt` && `cargo clippy`
- [ ] Contract tests passing
