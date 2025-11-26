//! Benchmark infrastructure for Corten JavaScript Runtime
//!
//! This crate provides benchmarking tools to measure the performance
//! of the JavaScript interpreter. It includes:
//!
//! - Micro-benchmarks for fundamental operations
//! - SunSpider benchmark suite (simplified)
//! - Benchmark runner with timing and result formatting
//!
//! # Examples
//!
//! ```rust,no_run
//! use benchmarks::micro;
//!
//! let results = micro::run_all();
//! for result in results {
//!     println!("{}: {:.2}ms", result.name, result.duration_ms);
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod micro;
pub mod runner;
pub mod sunspider;

pub use runner::{Benchmark, BenchmarkResult, BenchmarkSuite};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Ensure public API is accessible
        let _suite = BenchmarkSuite::new("test".to_string());
        let _bench = Benchmark {
            name: "test".to_string(),
            description: "test".to_string(),
            code: "1+1".to_string(),
        };
    }
}
