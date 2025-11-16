# bytecode_system Component

**Type**: Core Library (Level 1)
**Tech Stack**: Rust
**Version**: 0.1.0

## Purpose
Define bytecode instruction set for JavaScript execution, including opcodes, bytecode chunks, and basic optimization passes (dead code elimination, constant folding).

## Dependencies
- `core_types`: Value, SourcePosition

## Token Budget
- Optimal: 50,000 tokens
- Warning: 70,000 tokens
- Critical: 90,000 tokens

## Exported Types

```rust
// Bytecode opcodes
pub enum Opcode {
    // Literals
    LoadConstant(usize),
    LoadUndefined,
    LoadNull,
    LoadTrue,
    LoadFalse,

    // Variables
    LoadGlobal(String),
    StoreGlobal(String),
    LoadLocal(RegisterId),
    StoreLocal(RegisterId),

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Equal,
    StrictEqual,
    LessThan,
    GreaterThan,

    // Control flow
    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),
    Return,

    // Objects
    CreateObject,
    LoadProperty(String),
    StoreProperty(String),

    // Functions
    CreateClosure(usize),
    Call(u8),  // arg count
}

// Register identifier
pub struct RegisterId(pub u32);

// Single instruction
pub struct Instruction {
    pub opcode: Opcode,
    pub source_position: Option<SourcePosition>,
}

// Compiled bytecode
pub struct BytecodeChunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub register_count: u32,
}
```

## Key Implementation Requirements

### Register-Based Architecture
- Accumulator register for expression results
- Explicit register operands
- Compact instruction encoding

### Bytecode Optimization
1. **Dead Code Elimination**: Remove unreachable statements
2. **Constant Folding**: Evaluate constant expressions at compile time
3. **Peephole Optimization**: Replace sequences with equivalent shorter forms

### Instruction Encoding
- Variable-length encoding for compactness
- Fast decode for interpreter performance
- Debug info (source positions) optional

## Mandatory Requirements

### 1. Test-Driven Development
- Tests FIRST for every opcode
- 80%+ coverage
- TDD pattern in commits

### 2. Serialization
- Binary serialization for bytecode caching
- Debug-friendly text representation

### 3. File Structure
```
src/
  lib.rs           # Public exports
  opcode.rs        # Opcode enum
  instruction.rs   # Instruction and RegisterId
  chunk.rs         # BytecodeChunk
  optimizer.rs     # Optimization passes
  encoding.rs      # Binary encoding/decoding
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[bytecode_system] <type>: <description>
```

## Definition of Done
- [ ] All opcodes implemented and tested
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Optimization passes working
- [ ] Binary encoding/decoding
- [ ] `cargo fmt` && `cargo clippy`
- [ ] Contract tests passing
