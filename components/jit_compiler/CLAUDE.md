# jit_compiler Component

**Type**: Integration Library (Level 3)
**Tech Stack**: Rust, cranelift
**Version**: 0.1.0

## Purpose
Multi-tier JIT compilation with baseline (template) JIT for quick speedup and optimizing JIT with speculation, OSR support, and deoptimization infrastructure.

## Dependencies
- `bytecode_system`: Opcode, BytecodeChunk
- `interpreter`: ProfileData, InlineCache

## Token Budget
- Optimal: 80,000 tokens
- Warning: 100,000 tokens
- Critical: 120,000 tokens

## Exported Types

```rust
// Baseline JIT (template compiler)
pub struct BaselineJIT {
    // Fast compilation, modest speedup
}

impl BaselineJIT {
    pub fn compile(&mut self, chunk: &BytecodeChunk) -> Result<CompiledCode, JsError>;
}

// Optimizing JIT (speculative compiler)
pub struct OptimizingJIT {
    // Slow compilation, maximum performance
}

impl OptimizingJIT {
    pub fn compile(&mut self, chunk: &BytecodeChunk, profile: &ProfileData) -> Result<CompiledCode, JsError>;
}

// Compiled native code
pub struct CompiledCode {
    code: *const u8,
    size: usize,
    entry_point: *const (),
    osr_entries: Vec<OSREntry>,
}

// On-Stack Replacement entry
pub struct OSREntry {
    bytecode_offset: usize,
    native_offset: usize,
    frame_mapping: FrameMapping,
}

// Deoptimization
pub struct Deoptimizer {
    // Maps optimized frame to interpreter frame
}

impl Deoptimizer {
    pub fn deoptimize(&self, compiled: &CompiledCode) -> ExecutionContext;
}
```

## Key Implementation Requirements

### Baseline JIT
- Template-based: bytecode → fixed machine code sequence
- Preserves interpreter ICs
- 10x faster compilation than optimizing
- 2-3x speedup over interpreter

### Optimizing JIT (Cranelift)
```rust
use cranelift::prelude::*;

impl OptimizingJIT {
    fn type_specialize(&mut self, op: Opcode, feedback: TypeInfo) -> Value {
        // Specialize based on profiling
    }

    fn insert_guard(&mut self, assumption: Assumption) {
        // Guard with deoptimization
    }
}
```

### OSR (On-Stack Replacement)
- Enter compiled code from running interpreter
- Exit compiled code back to interpreter
- Frame reconstruction at transition

### Deoptimization
- Bail out when speculation fails
- Map optimized frame → interpreter frame
- Continue execution in interpreter

### Optimizations
- Type specialization from profile
- Escape analysis for allocation elimination
- Function inlining
- Loop-invariant code motion
- Dead code elimination
- Bounds check elimination

## Mandatory Requirements

### 1. Test-Driven Development
- Test each optimization pass
- 80%+ coverage
- TDD pattern in commits

### 2. Safety
- No undefined behavior in generated code
- Proper memory barriers
- Safe deoptimization

### 3. File Structure
```
src/
  lib.rs             # Public exports
  baseline.rs        # Baseline JIT
  optimizing.rs      # Optimizing JIT
  compiled_code.rs   # CompiledCode management
  osr.rs             # On-Stack Replacement
  deopt.rs           # Deoptimization
  ir.rs              # Intermediate representation
  codegen.rs         # Cranelift code generation
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[jit_compiler] <type>: <description>
```

## Definition of Done
- [ ] Baseline JIT functional
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] OSR working
- [ ] Deoptimization safe
- [ ] Contract tests passing
