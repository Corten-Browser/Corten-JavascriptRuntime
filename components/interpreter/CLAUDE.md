# interpreter Component

**Type**: Feature Library (Level 2)
**Tech Stack**: Rust, parking_lot
**Version**: 0.1.0

## Purpose
Bytecode virtual machine with dispatch loop, inline caching for property access optimization, and profiling instrumentation for JIT compilation decisions.

## Dependencies
- `bytecode_system`: Opcode, BytecodeChunk, Instruction
- `memory_manager`: Heap, JSObject, HiddenClass

## Token Budget
- Optimal: 65,000 tokens
- Warning: 85,000 tokens
- Critical: 105,000 tokens

## Exported Types

```rust
// Virtual Machine
pub struct VM {
    heap: Heap,
    global_object: *mut JSObject,
    call_stack: Vec<CallFrame>,
}

impl VM {
    pub fn execute(&mut self, chunk: &BytecodeChunk) -> Result<Value, JsError>;
}

// Execution context
pub struct ExecutionContext {
    pub registers: Vec<Value>,
    pub instruction_pointer: usize,
    pub bytecode: BytecodeChunk,
}

// Inline caching
pub enum InlineCache {
    Uninitialized,
    Monomorphic { shape: ShapeId, offset: u32 },
    Polymorphic { entries: ArrayVec<(ShapeId, u32), 4> },
    Megamorphic,
}

// Profiling data
pub struct ProfileData {
    pub execution_count: u64,
    pub type_feedback: Vec<TypeInfo>,
    pub branch_outcomes: Vec<BranchOutcome>,
}
```

## Key Implementation Requirements

### Dispatch Loop
```rust
pub fn dispatch(&mut self) -> Result<Value, JsError> {
    loop {
        let instruction = self.fetch();
        match instruction.opcode {
            Opcode::LoadConstant(idx) => self.load_constant(idx),
            Opcode::Add => self.add(),
            Opcode::Call(argc) => self.call(argc)?,
            Opcode::Return => break self.return_value(),
            // ... handle all opcodes
        }
    }
}
```

### Inline Caching
- Monomorphic: Single shape, direct offset access
- Polymorphic: Multiple shapes (up to 4)
- Megamorphic: Fallback to hash table lookup
- Cache property access patterns for performance

### Profiling
- Function invocation counters
- Loop iteration counters
- Type feedback at operations
- Branch outcomes for optimization

### Tier Transitions
- Counter thresholds trigger compilation
- ~500 executions → baseline JIT
- ~10,000 executions → optimizing JIT

## Mandatory Requirements

### 1. Test-Driven Development
- Test each opcode handler
- 80%+ coverage
- TDD pattern in commits

### 2. File Structure
```
src/
  lib.rs             # Public exports
  vm.rs              # Virtual machine
  context.rs         # ExecutionContext
  inline_cache.rs    # IC system
  profile.rs         # Profiling
  dispatch.rs        # Dispatch loop
  call_frame.rs      # Call stack
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[interpreter] <type>: <description>
```

## Definition of Done
- [ ] All opcodes implemented
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Inline caching working
- [ ] Profiling collecting data
- [ ] Contract tests passing
