//! Unit tests for the test harness

use std::collections::HashSet;
use test262_harness::{Test262Harness, TestResult};

#[test]
fn test_harness_creation() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("BigInt"));
    assert!(harness.supported_features().contains("Symbol"));
    assert!(harness.supported_features().contains("Promise"));
    assert!(harness.supported_features().contains("Map"));
    assert!(harness.supported_features().contains("Set"));
}

#[test]
fn test_add_feature() {
    let mut harness = Test262Harness::new();
    harness.add_feature("optional-chaining");
    assert!(harness.supported_features().contains("optional-chaining"));
}

#[test]
fn test_remove_feature() {
    let mut harness = Test262Harness::new();
    assert!(harness.supported_features().contains("BigInt"));
    harness.remove_feature("BigInt");
    assert!(!harness.supported_features().contains("BigInt"));
}

#[test]
fn test_set_timeout() {
    let mut harness = Test262Harness::new();
    assert_eq!(harness.timeout(), 10000);
    harness.set_timeout(5000);
    assert_eq!(harness.timeout(), 5000);
}

#[test]
fn test_with_custom_features() {
    let mut features = HashSet::new();
    features.insert("CustomFeature".to_string());
    features.insert("AnotherFeature".to_string());

    let harness = Test262Harness::with_features(features);
    assert!(harness.supported_features().contains("CustomFeature"));
    assert!(harness.supported_features().contains("AnotherFeature"));
    assert!(!harness.supported_features().contains("BigInt")); // Not included
}

#[test]
fn test_result_is_pass() {
    assert!(TestResult::Pass.is_pass());
    assert!(!TestResult::Fail("error".to_string()).is_pass());
    assert!(!TestResult::Skip("reason".to_string()).is_pass());
    assert!(!TestResult::Timeout.is_pass());
}

#[test]
fn test_result_is_fail() {
    assert!(!TestResult::Pass.is_fail());
    assert!(TestResult::Fail("error".to_string()).is_fail());
    assert!(!TestResult::Skip("reason".to_string()).is_fail());
    assert!(!TestResult::Timeout.is_fail());
}

#[test]
fn test_result_is_skip() {
    assert!(!TestResult::Pass.is_skip());
    assert!(!TestResult::Fail("error".to_string()).is_skip());
    assert!(TestResult::Skip("reason".to_string()).is_skip());
    assert!(!TestResult::Timeout.is_skip());
}

#[test]
fn test_result_is_timeout() {
    assert!(!TestResult::Pass.is_timeout());
    assert!(!TestResult::Fail("error".to_string()).is_timeout());
    assert!(!TestResult::Skip("reason".to_string()).is_timeout());
    assert!(TestResult::Timeout.is_timeout());
}

#[test]
fn test_clear_results() {
    let mut harness = Test262Harness::new();
    // Simulate adding some results
    assert_eq!(harness.test_count(), 0);
    harness.clear_results();
    assert_eq!(harness.test_count(), 0);
}

#[test]
fn test_default_harness() {
    let harness = Test262Harness::default();
    assert!(harness.supported_features().len() > 0);
    assert_eq!(harness.timeout(), 10000);
}

#[test]
fn test_pass_count_initially_zero() {
    let harness = Test262Harness::new();
    assert_eq!(harness.pass_count(), 0);
}

#[test]
fn test_fail_count_initially_zero() {
    let harness = Test262Harness::new();
    assert_eq!(harness.fail_count(), 0);
}

#[test]
fn test_skip_count_initially_zero() {
    let harness = Test262Harness::new();
    assert_eq!(harness.skip_count(), 0);
}

#[test]
fn test_harness_contains_arrow_function_feature() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("arrow-function"));
}

#[test]
fn test_harness_contains_let_const() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("let"));
    assert!(harness.supported_features().contains("const"));
}

#[test]
fn test_harness_contains_class() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("class"));
}

#[test]
fn test_harness_contains_generators() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("generators"));
}

#[test]
fn test_harness_contains_async_functions() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("async-functions"));
}

#[test]
fn test_harness_contains_destructuring() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("destructuring-binding"));
    assert!(harness.supported_features().contains("destructuring-assignment"));
}

#[test]
fn test_harness_contains_spread() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("spread"));
    assert!(harness.supported_features().contains("object-spread"));
}

#[test]
fn test_harness_contains_for_of() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("for-of"));
}

#[test]
fn test_harness_contains_template_literals() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("template"));
}

#[test]
fn test_harness_contains_atomics() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("Atomics"));
    assert!(harness.supported_features().contains("SharedArrayBuffer"));
}

#[test]
fn test_harness_contains_weakref() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("WeakRef"));
    assert!(harness.supported_features().contains("FinalizationRegistry"));
}

#[test]
fn test_harness_contains_symbol_variants() {
    let harness = Test262Harness::new();
    assert!(harness.supported_features().contains("Symbol"));
    assert!(harness.supported_features().contains("Symbol.species"));
    assert!(harness.supported_features().contains("Symbol.iterator"));
    assert!(harness.supported_features().contains("Symbol.toStringTag"));
}
