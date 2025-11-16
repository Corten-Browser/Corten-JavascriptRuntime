# bytecode_system

**Type**: Core Library (Level 1)
**Tech Stack**: Rust
**Version**: 0.1.0

## Overview

Bytecode instruction set and optimization system for the JavaScript runtime. This component provides:

- **Register-based bytecode architecture** for efficient execution
- **Complete opcode set** covering JavaScript operations
- **Binary serialization** for bytecode caching
- **Optimization passes** including dead code elimination and constant folding

## Features

### Opcodes
- **Literals**: LoadConstant, LoadUndefined, LoadNull, LoadTrue, LoadFalse
- **Variables**: LoadGlobal, StoreGlobal, LoadLocal, StoreLocal
- **Arithmetic**: Add, Sub, Mul, Div, Mod, Neg
- **Comparison**: Equal, StrictEqual, NotEqual, StrictNotEqual, LessThan, LessThanEqual, GreaterThan, GreaterThanEqual
- **Control Flow**: Jump, JumpIfTrue, JumpIfFalse, Return
- **Objects**: CreateObject, LoadProperty, StoreProperty
- **Functions**: CreateClosure, Call

### Optimization Passes
- **Dead Code Elimination**: Removes unreachable code after unconditional jumps
- **Constant Folding**: Evaluates constant expressions at compile time
- **Peephole Optimization**: Eliminates redundant operations (e.g., double negation)

## Usage

```rust
use bytecode_system::{BytecodeChunk, Opcode, Value};

// Create a new bytecode chunk
let mut chunk = BytecodeChunk::new();

// Add constants
let idx = chunk.add_constant(Value::Number(42.0));

// Emit instructions
chunk.emit(Opcode::LoadConstant(idx));
chunk.emit(Opcode::Return);

// Optimize the bytecode
chunk.optimize();

// Serialize for caching
let bytes = chunk.to_bytes();
let restored = BytecodeChunk::from_bytes(&bytes).unwrap();
```

## Structure

```
src/
├── lib.rs           # Public exports and documentation
├── opcode.rs        # Opcode enum and RegisterId
├── instruction.rs   # Instruction and SourcePosition
├── chunk.rs         # BytecodeChunk container
├── optimizer.rs     # Optimization passes
└── value.rs         # Value types (placeholder for core_types)

tests/
├── unit/            # Unit tests for each module
│   ├── test_opcode.rs
│   ├── test_instruction.rs
│   ├── test_chunk.rs
│   └── test_optimizer.rs
└── contracts/       # Contract compliance tests
    └── test_contract_compliance.rs
```

## Test Results

- **Total Tests**: 110 passing
  - Library tests: 21
  - Unit tests: 73
  - Contract tests: 15
  - Doc tests: 1
- **Coverage**: Comprehensive (all opcodes, instructions, chunks, and optimizations)
- **Quality**: Zero clippy warnings, properly formatted

## Dependencies

- `core_types`: Value and SourcePosition (currently using placeholder implementations)

## Contract Compliance

This component fully implements the contract defined in `contracts/bytecode_system.yaml`:

- All opcode variants
- RegisterId structure
- Instruction with optional source position
- BytecodeChunk with new(), emit(), add_constant(), and optimize() methods
- Binary serialization support
- Basic optimization passes

## Development

```bash
# Build
cargo build

# Run all tests
cargo test

# Check code quality
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## License

Part of the Corten JavaScript Runtime project.
