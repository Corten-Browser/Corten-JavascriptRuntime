//! SunSpider benchmark suite
//!
//! Simplified versions of classic SunSpider JavaScript benchmarks.
//! These test more realistic JavaScript patterns than micro-benchmarks.

use crate::runner::{Benchmark, BenchmarkSuite};

/// Create the SunSpider benchmark suite
pub fn create_suite() -> BenchmarkSuite {
    let mut suite = BenchmarkSuite::new("SunSpider".to_string());

    // 3D Cube Rotation
    suite.add(Benchmark {
        name: "3d-cube".to_string(),
        description: "3D cube rotation math".to_string(),
        code: include_str!("../suites/sunspider/3d-cube.js").to_string(),
    });

    // Access Binary Trees
    suite.add(Benchmark {
        name: "access-binary-trees".to_string(),
        description: "Binary tree traversal".to_string(),
        code: include_str!("../suites/sunspider/access-binary-trees.js").to_string(),
    });

    // Fibonacci
    suite.add(Benchmark {
        name: "math-fibonacci".to_string(),
        description: "Fibonacci sequence calculation".to_string(),
        code: include_str!("../suites/sunspider/math-fibonacci.js").to_string(),
    });

    suite
}

#[cfg(test)]
mod tests {
    use super::*;
    use js_cli::Runtime;

    #[test]
    fn test_create_sunspider_suite() {
        let suite = create_suite();
        assert_eq!(suite.name, "SunSpider");
        assert_eq!(suite.benchmarks.len(), 3);
    }

    #[test]
    fn test_sunspider_benchmarks_run() {
        let suite = create_suite();
        let mut runtime = Runtime::new(false);
        let results = suite.run(&mut runtime);

        assert_eq!(results.len(), 3);

        for result in results {
            if !result.success {
                eprintln!("Benchmark {} failed: {:?}", result.name, result.error);
            }
        }
    }
}
