# core_types

Core JavaScript value types and error handling for the Corten JavaScript Runtime.

**Type**: Base Library (Level 0)
**Tech Stack**: Rust (no external dependencies)
**Version**: 0.1.0

## Overview

This crate provides the foundational types for a JavaScript runtime:

- **`Value`** - Tagged representation of JavaScript values (undefined, null, boolean, number, object)
- **`JsError`** - JavaScript errors with stack traces
- **`ErrorKind`** - All JavaScript error types (SyntaxError, TypeError, etc.)
- **`SourcePosition`** - Source code location tracking
- **`StackFrame`** - Call stack frame information

## Features

- **Safe Rust Only** - No unsafe code (`#![deny(unsafe_code)]`)
- **Zero External Dependencies** - Uses only Rust standard library
- **Comprehensive Testing** - 137 tests including unit, contract compliance, and documentation tests
- **Full Documentation** - All public APIs have rustdoc comments with examples
- **JavaScript Semantics** - Correctly implements JavaScript truthiness, typeof behavior, and string conversion

## Quick Start

```rust
use core_types::{Value, JsError, ErrorKind, SourcePosition, StackFrame};

// Create JavaScript values
let undefined = Value::Undefined;
let number = Value::Smi(42);
let float = Value::Double(3.14);

// Check truthiness (JavaScript semantics)
assert!(!undefined.is_truthy());
assert!(number.is_truthy());
assert!(!Value::Double(f64::NAN).is_truthy());

// Get JavaScript typeof
assert_eq!(number.type_of(), "number");
assert_eq!(Value::Null.type_of(), "object"); // JS quirk

// Convert to string
assert_eq!(number.to_string(), "42");
assert_eq!(Value::Double(f64::INFINITY).to_string(), "Infinity");

// Create errors with stack traces
let error = JsError {
    kind: ErrorKind::TypeError,
    message: "undefined is not a function".to_string(),
    stack: vec![
        StackFrame {
            function_name: Some("myFunction".to_string()),
            source_url: Some("app.js".to_string()),
            line: 25,
            column: 10,
        }
    ],
    source_position: Some(SourcePosition {
        line: 25,
        column: 10,
        offset: 450,
    }),
};
```

## API Reference

### Value

The core type representing any JavaScript value:

```rust
pub enum Value {
    Undefined,           // undefined
    Null,                // null
    Boolean(bool),       // true or false
    Smi(i32),           // Small integer (tagged)
    HeapObject(usize),  // Object ID (safe abstraction)
    Double(f64),        // IEEE 754 double
}
```

**Methods:**
- `is_truthy() -> bool` - JavaScript truthiness check
- `type_of() -> String` - Returns JavaScript typeof result
- `to_string() -> String` - JavaScript string conversion (via Display trait)

### Error Types

```rust
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    ReferenceError,
    RangeError,
    EvalError,
    URIError,
    InternalError,
}

pub struct JsError {
    pub kind: ErrorKind,
    pub message: String,
    pub stack: Vec<StackFrame>,
    pub source_position: Option<SourcePosition>,
}
```

### Source Tracking

```rust
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

## Build

```bash
cargo build
cargo test
cargo clippy
cargo fmt --check
```

## Testing

Run all tests:
```bash
cargo test
```

Test categories:
- **Unit Tests** (83 tests) - Core functionality tests
- **Contract Tests** (38 tests) - API contract compliance
- **Inline Tests** (8 tests) - Module-level tests
- **Doc Tests** (8 tests) - Documentation examples

Total: **137 tests** with estimated **90%+ code coverage**

## Quality Standards

- All clippy lints pass (`cargo clippy -- -D warnings`)
- Code formatted with `cargo fmt`
- Comprehensive rustdoc documentation
- No unsafe code
- TDD development pattern followed

## Design Decisions

### HeapObject Representation

The contract specifies `HeapObject(*mut Object)` but also requires safe Rust. This implementation uses `HeapObject(usize)` as a safe object ID abstraction. The actual memory management will be handled by the separate `memory_manager` component.

### JavaScript Semantics

- **typeof null** returns "object" (historical JavaScript quirk)
- **-0.0** is falsy
- **NaN** is falsy
- All objects (HeapObject) are truthy

## Dependencies

**None** - This is a Level 0 base library with zero external dependencies.

## Contract Compliance

This component fully implements the contract defined in `contracts/core_types.yaml`:
- All specified types and variants
- All required methods with correct signatures
- Safe Rust requirement satisfied
- No external dependencies

## License

Part of the Corten JavaScript Runtime project.
