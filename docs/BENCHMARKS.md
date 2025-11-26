# Corten JavaScript Runtime Benchmarks

This document describes the benchmarking infrastructure and provides baseline performance measurements for the Corten JavaScript Runtime.

## Overview

The Corten runtime is an **interpreter-only** JavaScript engine (no JIT compilation active in these benchmarks). As such, it is expected to be significantly slower than production JavaScript engines like V8 or SpiderMonkey, which use sophisticated JIT compilation. These benchmarks serve to:

1. **Establish baseline performance** for the interpreter
2. **Track performance regressions** during development
3. **Identify optimization opportunities**
4. **Compare different implementation strategies**

## Running Benchmarks

The benchmark suite is available through the `corten-bench` CLI tool.

### Install

```bash
cargo build --release --bin corten-bench
```

### Run All Benchmarks

```bash
cargo run --release --bin corten-bench all
```

### Run Specific Suite

```bash
# Micro-benchmarks only
cargo run --release --bin corten-bench micro

# SunSpider suite only
cargo run --release --bin corten-bench sunspider
```

### JSON Output

For programmatic analysis:

```bash
cargo run --release --bin corten-bench --json micro > results.json
```

## Benchmark Suites

### Micro-benchmarks

Small, focused tests that measure fundamental operations:

- **arithmetic_addition**: 1M iterations of addition
- **arithmetic_multiplication**: 1M iterations of multiplication
- **variable_access_local**: 1M local variable reads
- **function_call_overhead**: 100K function calls
- **array_push**: 10K array push operations
- **array_indexing**: 100K array index reads
- **loop_for**: 1M iterations of for loop
- **loop_while**: 1M iterations of while loop
- **object_property_access**: 100K object property reads
- **string_concatenation**: 10K string concatenations

### SunSpider Suite

Adapted from the classic SunSpider JavaScript benchmark suite:

- **3d-cube**: 3D cube rotation using floating-point math
- **access-binary-trees**: Binary tree creation and traversal
- **math-fibonacci**: Recursive fibonacci calculation

Note: These are simplified versions adapted for an early-stage interpreter.

## Baseline Results

Measured on a development machine (debug build):

### Micro-benchmarks (Debug Build)

```
Benchmark                           Duration (ms)   Status
=================================================================
arithmetic_addition                       1049.05 ms  ✓ PASS
arithmetic_multiplication                 1093.51 ms  ✓ PASS
variable_access_local                     1120.44 ms  ✓ PASS
function_call_overhead                     268.35 ms  ✓ PASS
array_push                                4994.82 ms  ✓ PASS
array_indexing                             198.28 ms  ✓ PASS
loop_for                                  1029.17 ms  ✓ PASS
loop_while                                 923.14 ms  ✓ PASS
object_property_access                     205.44 ms  ✓ PASS
string_concatenation                        15.05 ms  ✓ PASS
```

**Total**: 10,897 ms (10.9 seconds)

### SunSpider Suite (Debug Build)

```
Benchmark                           Duration (ms)   Status
=================================================================
3d-cube                                    234.13 ms  ✓ PASS
access-binary-trees                         17.47 ms  ✓ PASS
math-fibonacci                             604.75 ms  ✓ PASS
```

**Total**: 856 ms (0.86 seconds)

### Performance Analysis

**Operations per Second (estimated):**

- Arithmetic operations: ~970K ops/sec
- Function calls: ~373K calls/sec
- Array operations: ~2-50K ops/sec (varies by operation)
- Object property access: ~487K accesses/sec

**Note**: These are debug build measurements. Release builds with optimizations (`--release`) will be significantly faster.

## Performance Expectations

### Comparison to Production Engines

**V8 (Chrome/Node.js)** and **SpiderMonkey (Firefox)** use:
- Multi-tier JIT compilation (interpreter → baseline JIT → optimizing JIT)
- Inline caching
- Hidden classes for objects
- Sophisticated garbage collection
- Type feedback and speculation

**Corten (interpreter-only)** currently has:
- Bytecode interpreter
- No JIT compilation (in these benchmarks)
- Basic garbage collection
- No speculative optimizations

**Expected performance ratio**: 10-100x slower than V8/SpiderMonkey

This is **normal and expected** for interpreter-only execution. The goal is correctness first, then optimization.

## Future Improvements

Potential optimization opportunities:

1. **JIT Compilation**: Enable baseline and optimizing JIT tiers
2. **Inline Caching**: Cache property lookups and method calls
3. **Bytecode Optimization**: Peephole optimizations, constant folding
4. **Better GC**: Generational garbage collection
5. **Type Specialization**: Use type feedback to specialize operations
6. **Register Allocation**: Better register usage in bytecode
7. **Native Functions**: Implement hot builtins in native code

## Adding New Benchmarks

### Micro-benchmark

Add to `components/benchmarks/src/micro.rs`:

```rust
Benchmark {
    name: "my_benchmark".to_string(),
    description: "Description of what it tests".to_string(),
    code: r#"
        // JavaScript code here
        let result = 0;
        for (let i = 0; i < 1000; i++) {
            result = result + i;
        }
        result
    "#.to_string(),
}
```

### SunSpider Test

1. Create `components/benchmarks/suites/sunspider/my-test.js`
2. Add to `components/benchmarks/src/sunspider.rs`:

```rust
suite.add(Benchmark {
    name: "my-test".to_string(),
    description: "What it tests".to_string(),
    code: include_str!("../suites/sunspider/my-test.js").to_string(),
});
```

## Development Notes

- Benchmarks run with JIT **disabled** (`Runtime::new(false)`)
- Each benchmark gets a fresh runtime instance (via the suite runner)
- Results include compilation time (parsing + bytecode generation)
- Debug builds include assertion overhead
- Use `--release` for more realistic performance measurements

## Benchmark Implementation

Location: `components/benchmarks/`

```
benchmarks/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── runner.rs       # Benchmark execution and timing
│   ├── micro.rs        # Micro-benchmark definitions
│   ├── sunspider.rs    # SunSpider suite
│   └── bin/
│       └── bench.rs    # CLI interface
└── suites/
    └── sunspider/      # SunSpider JavaScript files
        ├── 3d-cube.js
        ├── access-binary-trees.js
        └── math-fibonacci.js
```

## CI Integration

To run benchmarks in CI and track performance over time:

```bash
# Run benchmarks and save results
cargo run --release --bin corten-bench --json all > baseline.json

# Compare with previous baseline
# (requires custom comparison tool)
```

## License

Same as the main Corten project (MIT).
