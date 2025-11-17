//! Unit tests for report generation

use test262_harness::{TestReport, TestResult};

#[test]
fn test_new_report() {
    let report = TestReport::new();
    assert_eq!(report.total, 0);
    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 0);
    assert_eq!(report.skipped, 0);
    assert_eq!(report.timeout, 0);
    assert!(report.failures.is_empty());
    assert!(report.skips.is_empty());
}

#[test]
fn test_add_pass_result() {
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Pass);
    assert_eq!(report.total, 1);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
}

#[test]
fn test_add_fail_result() {
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Fail("error".to_string()));
    assert_eq!(report.total, 1);
    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].0, "test.js");
    assert_eq!(report.failures[0].1, "error");
}

#[test]
fn test_add_skip_result() {
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Skip("reason".to_string()));
    assert_eq!(report.total, 1);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.skips.len(), 1);
    assert_eq!(report.skips[0].0, "test.js");
    assert_eq!(report.skips[0].1, "reason");
}

#[test]
fn test_add_timeout_result() {
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Timeout);
    assert_eq!(report.total, 1);
    assert_eq!(report.timeout, 1);
}

#[test]
fn test_pass_rate_empty() {
    let report = TestReport::new();
    assert_eq!(report.pass_rate(), 0.0);
}

#[test]
fn test_pass_rate_all_pass() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Pass);
    assert_eq!(report.pass_rate(), 100.0);
}

#[test]
fn test_pass_rate_half_pass() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Fail("error".to_string()));
    assert_eq!(report.pass_rate(), 50.0);
}

#[test]
fn test_pass_rate_with_skips() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Skip("reason".to_string()));
    report.add_result("test3.js", TestResult::Pass);
    // 2/3 = 66.67%
    let rate = report.pass_rate();
    assert!((rate - 66.666).abs() < 0.01);
}

#[test]
fn test_effective_pass_rate_empty() {
    let report = TestReport::new();
    assert_eq!(report.effective_pass_rate(), 0.0);
}

#[test]
fn test_effective_pass_rate_excludes_skips() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Skip("reason".to_string()));
    report.add_result("test3.js", TestResult::Fail("error".to_string()));
    // 1/2 = 50% (skips excluded)
    assert_eq!(report.effective_pass_rate(), 50.0);
}

#[test]
fn test_effective_pass_rate_all_skipped() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Skip("reason".to_string()));
    report.add_result("test2.js", TestResult::Skip("reason".to_string()));
    // 0/0 = 0%
    assert_eq!(report.effective_pass_rate(), 0.0);
}

#[test]
fn test_summary_format() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Fail("error".to_string()));
    report.add_result("test3.js", TestResult::Skip("reason".to_string()));
    report.add_result("test4.js", TestResult::Timeout);

    let summary = report.summary();
    assert!(summary.contains("Total: 4"));
    assert!(summary.contains("Passed: 1"));
    assert!(summary.contains("Failed: 1"));
    assert!(summary.contains("Skipped: 1"));
    assert!(summary.contains("Timeout: 1"));
}

#[test]
fn test_detailed_summary_includes_failures() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Fail("syntax error".to_string()));
    report.add_result("test2.js", TestResult::Fail("type error".to_string()));

    let detailed = report.detailed_summary();
    assert!(detailed.contains("Failures:"));
    assert!(detailed.contains("test1.js"));
    assert!(detailed.contains("syntax error"));
    assert!(detailed.contains("test2.js"));
    assert!(detailed.contains("type error"));
}

#[test]
fn test_merge_reports() {
    let mut report1 = TestReport::new();
    report1.add_result("test1.js", TestResult::Pass);
    report1.add_result("test2.js", TestResult::Fail("error".to_string()));

    let mut report2 = TestReport::new();
    report2.add_result("test3.js", TestResult::Pass);
    report2.add_result("test4.js", TestResult::Skip("reason".to_string()));

    report1.merge(&report2);

    assert_eq!(report1.total, 4);
    assert_eq!(report1.passed, 2);
    assert_eq!(report1.failed, 1);
    assert_eq!(report1.skipped, 1);
    assert_eq!(report1.failures.len(), 1);
    assert_eq!(report1.skips.len(), 1);
}

#[test]
fn test_is_success_all_pass() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Pass);
    assert!(report.is_success());
}

#[test]
fn test_is_success_with_failure() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Fail("error".to_string()));
    assert!(!report.is_success());
}

#[test]
fn test_is_success_with_timeout() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Timeout);
    assert!(!report.is_success());
}

#[test]
fn test_is_success_with_skips_only() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Skip("reason".to_string()));
    assert!(report.is_success());
}

#[test]
fn test_top_failures() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Fail("error1".to_string()));
    report.add_result("test2.js", TestResult::Fail("error2".to_string()));
    report.add_result("test3.js", TestResult::Fail("error3".to_string()));

    let top2 = report.top_failures(2);
    assert_eq!(top2.len(), 2);
    assert_eq!(top2[0].0, "test1.js");
    assert_eq!(top2[1].0, "test2.js");
}

#[test]
fn test_top_failures_more_than_available() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Fail("error1".to_string()));

    let top5 = report.top_failures(5);
    assert_eq!(top5.len(), 1);
}

#[test]
fn test_to_json() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Fail("error".to_string()));

    let json = report.to_json().unwrap();
    assert!(json.contains("\"total\": 2"));
    assert!(json.contains("\"passed\": 1"));
    assert!(json.contains("\"failed\": 1"));
}

#[test]
fn test_from_json() {
    let json = r#"{
        "total": 10,
        "passed": 7,
        "failed": 2,
        "skipped": 1,
        "timeout": 0,
        "failures": [["test1.js", "error1"], ["test2.js", "error2"]],
        "skips": [["test3.js", "reason"]]
    }"#;

    let report = TestReport::from_json(json).unwrap();
    assert_eq!(report.total, 10);
    assert_eq!(report.passed, 7);
    assert_eq!(report.failed, 2);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.failures.len(), 2);
}

#[test]
fn test_failure_rate() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Fail("error".to_string()));
    report.add_result("test3.js", TestResult::Pass);
    report.add_result("test4.js", TestResult::Fail("error".to_string()));
    // 2/4 = 50%
    assert_eq!(report.failure_rate(), 50.0);
}

#[test]
fn test_skip_rate() {
    let mut report = TestReport::new();
    report.add_result("test1.js", TestResult::Pass);
    report.add_result("test2.js", TestResult::Skip("reason".to_string()));
    report.add_result("test3.js", TestResult::Pass);
    report.add_result("test4.js", TestResult::Skip("reason".to_string()));
    // 2/4 = 50%
    assert_eq!(report.skip_rate(), 50.0);
}

#[test]
fn test_default_report() {
    let report = TestReport::default();
    assert_eq!(report.total, 0);
    assert!(report.is_success());
}
