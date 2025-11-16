# core_types Component

**Type**: Base Library (Level 0)
**Tech Stack**: Rust (no external dependencies)
**Version**: 0.1.0

## Purpose
Core JavaScript value representation including tagged pointers, error types, and source location tracking. This is the foundational component with NO dependencies.

## Token Budget
- Optimal: 40,000 tokens
- Warning: 60,000 tokens
- Critical: 80,000 tokens

## Exported Types

```rust
// Tagged value representation
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Smi(i32),           // Small integer (tagged)
    HeapObject(*mut Object),
    Double(f64),
}

// Error handling
pub struct JsError {
    pub kind: ErrorKind,
    pub message: String,
    pub stack: Vec<StackFrame>,
    pub source_position: Option<SourcePosition>,
}

pub enum ErrorKind {
    SyntaxError,
    TypeError,
    ReferenceError,
    RangeError,
    EvalError,
    URIError,
    InternalError,
}

// Source tracking
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

pub struct StackFrame {
    pub function_name: Option<String>,
    pub source_url: Option<String>,
    pub line: u32,
    pub column: u32,
}
```

## Mandatory Requirements

### 1. Test-Driven Development
- Write tests FIRST (RED)
- Implement minimal code (GREEN)
- Refactor (REFACTOR)
- Commit pattern must show TDD in git history

### 2. Rust Best Practices
- Use `Result<T, JsError>` for error handling
- No panics in production code
- Minimize unsafe blocks
- Comprehensive rustdoc comments
- Format with `cargo fmt`
- Lint with `cargo clippy`

### 3. Testing Standards
- 80%+ test coverage
- Unit tests in `tests/unit/`
- Integration tests in `tests/integration/`
- Contract tests in `tests/contracts/`

### 4. File Structure
```
src/
  lib.rs          # Public API exports
  value.rs        # Value enum and operations
  error.rs        # JsError and ErrorKind
  source.rs       # SourcePosition and StackFrame
tests/
  unit/
  integration/
  contracts/
```

## Key Implementation Details

### Tagged Pointers
Use least significant bits for type identification:
- Bit 0 = 1: Smi (small integer)
- Bit 0 = 0: Heap object pointer

### Memory Safety
All operations must be safe Rust except when explicitly working with raw pointers (which will be used by memory_manager).

### No Dependencies
This component has ZERO external dependencies. It uses only Rust standard library.

## Git Commit Format
```
[core_types] <type>: <description>
```
Types: feat, fix, test, docs, refactor

## Definition of Done
- [ ] All TDD cycles in git history
- [ ] 80%+ test coverage
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` warnings resolved
- [ ] All public APIs documented
- [ ] README.md updated
- [ ] Contract tests passing
