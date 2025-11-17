//! Unit tests for report builder

use test262_harness::{TestReport, TestResult};
use test262_harness::report::ReportBuilder;

#[test]
fn test_new_builder() {
    let builder = ReportBuilder::new();
    assert_eq!(builder.count(), 0);
}

#[test]
fn test_add_report() {
    let mut builder = ReportBuilder::new();
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Pass);

    builder.add_report(report);
    assert_eq!(builder.count(), 1);
}

#[test]
fn test_add_multiple_reports() {
    let mut builder = ReportBuilder::new();

    let mut report1 = TestReport::new();
    report1.add_result("test1.js", TestResult::Pass);

    let mut report2 = TestReport::new();
    report2.add_result("test2.js", TestResult::Pass);

    builder.add_report(report1).add_report(report2);
    assert_eq!(builder.count(), 2);
}

#[test]
fn test_build_empty() {
    let builder = ReportBuilder::new();
    let combined = builder.build();
    assert_eq!(combined.total, 0);
}

#[test]
fn test_build_single_report() {
    let mut builder = ReportBuilder::new();
    let mut report = TestReport::new();
    report.add_result("test.js", TestResult::Pass);

    builder.add_report(report);
    let combined = builder.build();

    assert_eq!(combined.total, 1);
    assert_eq!(combined.passed, 1);
}

#[test]
fn test_build_multiple_reports() {
    let mut builder = ReportBuilder::new();

    let mut report1 = TestReport::new();
    report1.add_result("test1.js", TestResult::Pass);
    report1.add_result("test2.js", TestResult::Fail("error".to_string()));

    let mut report2 = TestReport::new();
    report2.add_result("test3.js", TestResult::Pass);
    report2.add_result("test4.js", TestResult::Skip("reason".to_string()));

    builder.add_report(report1).add_report(report2);
    let combined = builder.build();

    assert_eq!(combined.total, 4);
    assert_eq!(combined.passed, 2);
    assert_eq!(combined.failed, 1);
    assert_eq!(combined.skipped, 1);
}

#[test]
fn test_build_aggregates_failures() {
    let mut builder = ReportBuilder::new();

    let mut report1 = TestReport::new();
    report1.add_result("test1.js", TestResult::Fail("error1".to_string()));

    let mut report2 = TestReport::new();
    report2.add_result("test2.js", TestResult::Fail("error2".to_string()));

    builder.add_report(report1).add_report(report2);
    let combined = builder.build();

    assert_eq!(combined.failures.len(), 2);
}

#[test]
fn test_default_builder() {
    let builder = ReportBuilder::default();
    assert_eq!(builder.count(), 0);
}
