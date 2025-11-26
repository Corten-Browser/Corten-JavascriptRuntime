//! Test262 Conformance Test Harness
//!
//! This crate provides a harness for running Test262, the official ECMAScript
//! conformance test suite, against the Corten JavaScript runtime.

pub mod harness;
pub mod report;
pub mod test_file;

pub use harness::{Test262Harness, TestResult, HARNESS_PRELUDE};
pub use report::TestReport;
pub use test_file::{NegativeExpectation, TestFile, TestMetadata};
