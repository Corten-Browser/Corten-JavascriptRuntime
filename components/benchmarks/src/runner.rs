//! Benchmark runner and result types
//!
//! Provides infrastructure for running benchmarks and collecting results.

use js_cli::Runtime;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// A single benchmark test
#[derive(Debug, Clone)]
pub struct Benchmark {
    /// Name of the benchmark
    pub name: String,
    /// Description of what the benchmark tests
    pub description: String,
    /// JavaScript code to execute
    pub code: String,
}

/// Result of running a benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Description of the benchmark
    pub description: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Operations per second (if applicable)
    pub ops_per_sec: Option<f64>,
    /// Whether the benchmark completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl Benchmark {
    /// Run this benchmark using the provided runtime
    ///
    /// # Arguments
    /// * `runtime` - The JavaScript runtime to use for execution
    ///
    /// # Returns
    /// A `BenchmarkResult` containing timing and success information
    pub fn run(&self, runtime: &mut Runtime) -> BenchmarkResult {
        let start = Instant::now();

        let result = runtime.execute_string(&self.code);

        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        match result {
            Ok(_) => BenchmarkResult {
                name: self.name.clone(),
                description: self.description.clone(),
                duration_ms,
                ops_per_sec: None, // Can be calculated based on known iteration count
                success: true,
                error: None,
            },
            Err(e) => BenchmarkResult {
                name: self.name.clone(),
                description: self.description.clone(),
                duration_ms,
                ops_per_sec: None,
                success: false,
                error: Some(format!("{:?}", e)),
            },
        }
    }

    /// Run this benchmark multiple times and return average
    ///
    /// # Arguments
    /// * `runtime` - The JavaScript runtime to use
    /// * `iterations` - Number of times to run the benchmark
    ///
    /// # Returns
    /// A `BenchmarkResult` with averaged timing information
    pub fn run_multiple(&self, runtime: &mut Runtime, iterations: usize) -> BenchmarkResult {
        let mut total_duration_ms = 0.0;
        let mut last_result = None;

        for _ in 0..iterations {
            let result = self.run(runtime);
            if !result.success {
                return result; // Return error immediately
            }
            total_duration_ms += result.duration_ms;
            last_result = Some(result);
        }

        let mut final_result = last_result.unwrap();
        final_result.duration_ms = total_duration_ms / iterations as f64;
        final_result
    }
}

/// Suite of benchmarks
pub struct BenchmarkSuite {
    /// Name of the suite
    pub name: String,
    /// Benchmarks in this suite
    pub benchmarks: Vec<Benchmark>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: String) -> Self {
        Self {
            name,
            benchmarks: Vec::new(),
        }
    }

    /// Add a benchmark to this suite
    pub fn add(&mut self, benchmark: Benchmark) {
        self.benchmarks.push(benchmark);
    }

    /// Run all benchmarks in this suite
    ///
    /// # Arguments
    /// * `runtime` - The JavaScript runtime to use
    ///
    /// # Returns
    /// Vector of results for each benchmark
    pub fn run(&self, runtime: &mut Runtime) -> Vec<BenchmarkResult> {
        self.benchmarks.iter().map(|b| b.run(runtime)).collect()
    }

    /// Run all benchmarks multiple times and average
    pub fn run_multiple(&self, runtime: &mut Runtime, iterations: usize) -> Vec<BenchmarkResult> {
        self.benchmarks
            .iter()
            .map(|b| b.run_multiple(runtime, iterations))
            .collect()
    }
}

/// Format benchmark results as a human-readable table
pub fn format_results(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "\n{:<35} {:<15} {:<10}\n",
        "Benchmark", "Duration (ms)", "Status"
    ));
    output.push_str(&format!("{}\n", "=".repeat(65)));

    for result in results {
        let status = if result.success { "✓ PASS" } else { "✗ FAIL" };
        output.push_str(&format!(
            "{:<35} {:>13.2} ms  {:<10}\n",
            result.name, result.duration_ms, status
        ));

        if let Some(error) = &result.error {
            output.push_str(&format!("  Error: {}\n", error));
        }
    }

    output
}

/// Format benchmark results as JSON
pub fn format_results_json(results: &[BenchmarkResult]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_creation() {
        let bench = Benchmark {
            name: "test".to_string(),
            description: "Test benchmark".to_string(),
            code: "1 + 1".to_string(),
        };

        assert_eq!(bench.name, "test");
        assert_eq!(bench.description, "Test benchmark");
    }

    #[test]
    fn test_benchmark_run_success() {
        let mut runtime = Runtime::new(false);
        let bench = Benchmark {
            name: "simple_math".to_string(),
            description: "Simple math".to_string(),
            code: "2 + 2".to_string(),
        };

        let result = bench.run(&mut runtime);
        assert!(result.success);
        assert!(result.duration_ms > 0.0);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_benchmark_run_error() {
        let mut runtime = Runtime::new(false);
        let bench = Benchmark {
            name: "syntax_error".to_string(),
            description: "Invalid syntax".to_string(),
            code: "this is not valid javascript !!!".to_string(),
        };

        let result = bench.run(&mut runtime);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new("Test Suite".to_string());

        suite.add(Benchmark {
            name: "bench1".to_string(),
            description: "First".to_string(),
            code: "1 + 1".to_string(),
        });

        suite.add(Benchmark {
            name: "bench2".to_string(),
            description: "Second".to_string(),
            code: "2 * 2".to_string(),
        });

        assert_eq!(suite.benchmarks.len(), 2);

        let mut runtime = Runtime::new(false);
        let results = suite.run(&mut runtime);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_format_results() {
        let results = vec![
            BenchmarkResult {
                name: "test1".to_string(),
                description: "Test 1".to_string(),
                duration_ms: 123.45,
                ops_per_sec: None,
                success: true,
                error: None,
            },
            BenchmarkResult {
                name: "test2".to_string(),
                description: "Test 2".to_string(),
                duration_ms: 67.89,
                ops_per_sec: None,
                success: false,
                error: Some("Error message".to_string()),
            },
        ];

        let output = format_results(&results);
        assert!(output.contains("test1"));
        assert!(output.contains("test2"));
        assert!(output.contains("123.45"));
        assert!(output.contains("67.89"));
        assert!(output.contains("PASS"));
        assert!(output.contains("FAIL"));
    }

    #[test]
    fn test_format_results_json() {
        let results = vec![BenchmarkResult {
            name: "test".to_string(),
            description: "Test".to_string(),
            duration_ms: 100.0,
            ops_per_sec: None,
            success: true,
            error: None,
        }];

        let json = format_results_json(&results).unwrap();
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"duration_ms\": 100.0"));
        assert!(json.contains("\"success\": true"));
    }

    #[test]
    fn test_benchmark_run_multiple() {
        let mut runtime = Runtime::new(false);
        let bench = Benchmark {
            name: "multi_run".to_string(),
            description: "Multiple runs".to_string(),
            code: "1 + 1".to_string(),
        };

        let result = bench.run_multiple(&mut runtime, 3);
        assert!(result.success);
        assert!(result.duration_ms > 0.0);
    }
}
