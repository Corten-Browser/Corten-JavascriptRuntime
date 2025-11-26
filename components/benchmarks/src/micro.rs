//! Micro-benchmarks for fundamental JavaScript operations
//!
//! These benchmarks test basic operations to establish baseline performance
//! for the Corten JavaScript Runtime.

use crate::runner::{Benchmark, BenchmarkResult};
use js_cli::Runtime;

/// Create all micro-benchmarks
pub fn create_benchmarks() -> Vec<Benchmark> {
    vec![
        Benchmark {
            name: "arithmetic_addition".to_string(),
            description: "1M iterations of addition".to_string(),
            code: r#"
                let sum = 0;
                for (let i = 0; i < 1000000; i++) {
                    sum = sum + 1;
                }
                sum
            "#
            .to_string(),
        },
        Benchmark {
            name: "arithmetic_multiplication".to_string(),
            description: "1M iterations of multiplication".to_string(),
            code: r#"
                let product = 1;
                for (let i = 0; i < 1000000; i++) {
                    product = product * 1;
                }
                product
            "#
            .to_string(),
        },
        Benchmark {
            name: "variable_access_local".to_string(),
            description: "1M iterations of local variable access".to_string(),
            code: r#"
                let x = 42;
                let count = 0;
                for (let i = 0; i < 1000000; i++) {
                    count = count + x;
                }
                count
            "#
            .to_string(),
        },
        Benchmark {
            name: "function_call_overhead".to_string(),
            description: "100K function calls".to_string(),
            code: r#"
                function add(a, b) {
                    return a + b;
                }
                let sum = 0;
                for (let i = 0; i < 100000; i++) {
                    sum = add(sum, 1);
                }
                sum
            "#
            .to_string(),
        },
        Benchmark {
            name: "array_push".to_string(),
            description: "10K array push operations".to_string(),
            code: r#"
                let arr = [];
                for (let i = 0; i < 10000; i++) {
                    arr.push(i);
                }
                arr.length
            "#
            .to_string(),
        },
        Benchmark {
            name: "array_indexing".to_string(),
            description: "100K array index reads".to_string(),
            code: r#"
                let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
                let sum = 0;
                for (let i = 0; i < 100000; i++) {
                    sum = sum + arr[i % 10];
                }
                sum
            "#
            .to_string(),
        },
        Benchmark {
            name: "loop_for".to_string(),
            description: "1M iterations of for loop".to_string(),
            code: r#"
                let count = 0;
                for (let i = 0; i < 1000000; i++) {
                    count = count + 1;
                }
                count
            "#
            .to_string(),
        },
        Benchmark {
            name: "loop_while".to_string(),
            description: "1M iterations of while loop".to_string(),
            code: r#"
                let count = 0;
                let i = 0;
                while (i < 1000000) {
                    count = count + 1;
                    i = i + 1;
                }
                count
            "#
            .to_string(),
        },
        Benchmark {
            name: "object_property_access".to_string(),
            description: "100K object property reads".to_string(),
            code: r#"
                let obj = { x: 1, y: 2, z: 3 };
                let sum = 0;
                for (let i = 0; i < 100000; i++) {
                    sum = sum + obj.x;
                }
                sum
            "#
            .to_string(),
        },
        Benchmark {
            name: "string_concatenation".to_string(),
            description: "10K string concatenations".to_string(),
            code: r#"
                let str = "";
                for (let i = 0; i < 10000; i++) {
                    str = str + "a";
                }
                str.length
            "#
            .to_string(),
        },
    ]
}

/// Run all micro-benchmarks
pub fn run_all() -> Vec<BenchmarkResult> {
    let benchmarks = create_benchmarks();
    let mut runtime = Runtime::new(false); // Disable JIT for interpreter baseline
    let mut results = Vec::new();

    for bench in benchmarks {
        results.push(bench.run(&mut runtime));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_benchmarks() {
        let benchmarks = create_benchmarks();
        assert!(!benchmarks.is_empty());
        assert!(benchmarks.len() >= 10);
    }

    #[test]
    fn test_arithmetic_addition_benchmark() {
        let mut runtime = Runtime::new(false);
        let bench = Benchmark {
            name: "test_addition".to_string(),
            description: "Test addition".to_string(),
            code: "let sum = 0; for (let i = 0; i < 100; i++) { sum = sum + 1; } sum".to_string(),
        };

        let result = bench.run(&mut runtime);
        assert!(result.success);
        assert!(result.duration_ms > 0.0);
    }

    #[test]
    fn test_all_micro_benchmarks_run() {
        let results = run_all();
        assert_eq!(results.len(), 10);

        for result in results {
            assert!(result.success, "Benchmark {} failed: {:?}", result.name, result.error);
        }
    }
}
