# Benchmarks Component

Performance benchmarking infrastructure for the Corten JavaScript Runtime.

## Overview

This component provides tools to measure and track the performance of the JavaScript interpreter. It includes micro-benchmarks for fundamental operations and adapted versions of industry-standard benchmark suites.

## Quick Start

```bash
# Run all benchmarks
cargo run --bin corten-bench all

# Run specific suite
cargo run --bin corten-bench micro
cargo run --bin corten-bench sunspider

# JSON output
cargo run --bin corten-bench --json micro
```

## Benchmark Suites

### Micro-benchmarks (`micro`)

Test fundamental JavaScript operations:
- Arithmetic operations (addition, multiplication)
- Variable access (local, global)
- Function calls
- Array operations (push, indexing)
- Loop constructs (for, while)
- Object property access
- String operations

### SunSpider (`sunspider`)

Adapted classic JavaScript benchmarks:
- **3d-cube**: Floating-point math and 3D transformations
- **access-binary-trees**: Object creation and tree traversal
- **math-fibonacci**: Recursive function calls

## Implementation

### Core Modules

- **`runner.rs`**: Benchmark execution, timing, and result formatting
- **`micro.rs`**: Micro-benchmark definitions
- **`sunspider.rs`**: SunSpider suite loader
- **`bin/bench.rs`**: CLI interface

### Adding Benchmarks

#### Micro-benchmark

Edit `src/micro.rs`:

```rust
Benchmark {
    name: "my_test".to_string(),
    description: "Description".to_string(),
    code: r#"
        // JavaScript code
        let x = 0;
        for (let i = 0; i < 1000; i++) {
            x = x + 1;
        }
        x
    "#.to_string(),
}
```

#### SunSpider Test

1. Create `suites/sunspider/my-test.js` with JavaScript code
2. Edit `src/sunspider.rs`:

```rust
suite.add(Benchmark {
    name: "my-test".to_string(),
    description: "Description".to_string(),
    code: include_str!("../suites/sunspider/my-test.js").to_string(),
});
```

## API

### Benchmark Struct

```rust
pub struct Benchmark {
    pub name: String,
    pub description: String,
    pub code: String,
}
```

### BenchmarkResult

```rust
pub struct BenchmarkResult {
    pub name: String,
    pub description: String,
    pub duration_ms: f64,
    pub ops_per_sec: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}
```

### BenchmarkSuite

```rust
pub struct BenchmarkSuite {
    pub name: String,
    pub benchmarks: Vec<Benchmark>,
}

impl BenchmarkSuite {
    pub fn new(name: String) -> Self;
    pub fn add(&mut self, benchmark: Benchmark);
    pub fn run(&self, runtime: &mut Runtime) -> Vec<BenchmarkResult>;
}
```

## Testing

```bash
# Run unit tests
cargo test --package benchmarks

# Run with output
cargo test --package benchmarks -- --nocapture
```

## Performance Notes

- Benchmarks use interpreter-only mode (JIT disabled)
- Each benchmark gets timing from `std::time::Instant`
- Results include parsing + bytecode generation + execution time
- Debug builds are much slower than release builds

## Dependencies

- `js_cli`: JavaScript runtime
- `core_types`: Value types
- `serde`, `serde_json`: Result serialization

## Future Enhancements

- [ ] Memory usage tracking
- [ ] Warmup iterations before timing
- [ ] Statistical analysis (mean, median, stddev)
- [ ] Comparison with baseline results
- [ ] Performance regression detection
- [ ] More benchmark suites (Octane, Kraken, JetStream)
- [ ] Benchmark result database/history
- [ ] Graphical output of trends

## License

MIT
