use crate::harness::TestResult;
use serde::{Deserialize, Serialize};

/// Test run report with statistics and failure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Total number of tests run
    pub total: usize,
    /// Number of tests that passed
    pub passed: usize,
    /// Number of tests that failed
    pub failed: usize,
    /// Number of tests that were skipped
    pub skipped: usize,
    /// Number of tests that timed out
    pub timeout: usize,
    /// List of failures with (path, reason)
    pub failures: Vec<(String, String)>,
    /// List of skipped tests with (path, reason)
    pub skips: Vec<(String, String)>,
}

impl TestReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            timeout: 0,
            failures: Vec::new(),
            skips: Vec::new(),
        }
    }

    /// Add a test result to the report
    pub fn add_result(&mut self, path: &str, result: TestResult) {
        self.total += 1;
        match result {
            TestResult::Pass => self.passed += 1,
            TestResult::Fail(reason) => {
                self.failed += 1;
                self.failures.push((path.to_string(), reason));
            }
            TestResult::Skip(reason) => {
                self.skipped += 1;
                self.skips.push((path.to_string(), reason));
            }
            TestResult::Timeout => self.timeout += 1,
        }
    }

    /// Calculate the pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    /// Calculate the effective pass rate (excluding skips)
    pub fn effective_pass_rate(&self) -> f64 {
        let executed = self.total - self.skipped;
        if executed == 0 {
            0.0
        } else {
            (self.passed as f64 / executed as f64) * 100.0
        }
    }

    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Test262 Results:\n\
             Total: {}\n\
             Passed: {} ({:.1}%)\n\
             Failed: {}\n\
             Skipped: {}\n\
             Timeout: {}\n\
             Effective Pass Rate: {:.1}%",
            self.total,
            self.passed,
            self.pass_rate(),
            self.failed,
            self.skipped,
            self.timeout,
            self.effective_pass_rate()
        )
    }

    /// Generate a detailed report including failures
    pub fn detailed_summary(&self) -> String {
        let mut output = self.summary();

        if !self.failures.is_empty() {
            output.push_str("\n\nFailures:\n");
            for (path, reason) in &self.failures {
                output.push_str(&format!("  - {}\n    Reason: {}\n", path, reason));
            }
        }

        output
    }

    /// Merge another report into this one
    pub fn merge(&mut self, other: &TestReport) {
        self.total += other.total;
        self.passed += other.passed;
        self.failed += other.failed;
        self.skipped += other.skipped;
        self.timeout += other.timeout;
        self.failures.extend(other.failures.clone());
        self.skips.extend(other.skips.clone());
    }

    /// Check if all tests passed (no failures or timeouts)
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.timeout == 0
    }

    /// Get the first N failures
    pub fn top_failures(&self, n: usize) -> Vec<&(String, String)> {
        self.failures.iter().take(n).collect()
    }

    /// Export report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import report from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.failed as f64 / self.total as f64) * 100.0
        }
    }

    /// Get skip rate as percentage
    pub fn skip_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.skipped as f64 / self.total as f64) * 100.0
        }
    }
}

impl Default for TestReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Report builder for aggregating multiple test runs
pub struct ReportBuilder {
    reports: Vec<TestReport>,
}

impl ReportBuilder {
    /// Create a new report builder
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    /// Add a report to be aggregated
    pub fn add_report(&mut self, report: TestReport) -> &mut Self {
        self.reports.push(report);
        self
    }

    /// Build an aggregated report from all added reports
    pub fn build(&self) -> TestReport {
        let mut combined = TestReport::new();
        for report in &self.reports {
            combined.merge(report);
        }
        combined
    }

    /// Get number of reports added
    pub fn count(&self) -> usize {
        self.reports.len()
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}
