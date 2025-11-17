//! Integration tests for test execution

use std::fs;
use tempfile::TempDir;
use test262_harness::{Test262Harness, TestFile};

#[test]
fn test_run_simple_passing_test() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("simple_pass.js");

    let test_content = r#"/*---
description: Simple passing test
---*/
var x = 1 + 1;
"#;

    fs::write(&test_path, test_content).unwrap();

    let test_file = TestFile::load(&test_path).unwrap();
    let mut harness = Test262Harness::new();
    let result = harness.run_test(&test_file);

    assert!(result.is_pass());
}

#[test]
fn test_run_test_with_missing_feature() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("missing_feature.js");

    let test_content = r#"/*---
description: Test requiring unsupported feature
features: [unsupported-feature]
---*/
test();
"#;

    fs::write(&test_path, test_content).unwrap();

    let test_file = TestFile::load(&test_path).unwrap();
    let mut harness = Test262Harness::new();
    let result = harness.run_test(&test_file);

    assert!(result.is_skip());
}

#[test]
fn test_file_name_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("my_test_file.js");

    let test_content = r#"/*---
description: Test for name extraction
---*/
1;
"#;

    fs::write(&test_path, test_content).unwrap();

    let test_file = TestFile::load(&test_path).unwrap();
    assert_eq!(test_file.name(), "my_test_file");
}

#[test]
fn test_file_code_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("code_test.js");

    let test_content = r#"/*---
description: Test for code extraction
---*/
var x = 42;
console.log(x);
"#;

    fs::write(&test_path, test_content).unwrap();

    let test_file = TestFile::load(&test_path).unwrap();
    let code = test_file.code();

    assert!(code.contains("var x = 42;"));
    assert!(code.contains("console.log(x);"));
    assert!(!code.contains("/*---"));
    assert!(!code.contains("---*/"));
}

#[test]
fn test_run_directory_empty() {
    let temp_dir = TempDir::new().unwrap();
    let mut harness = Test262Harness::new();
    let report = harness.run_directory(temp_dir.path());

    assert_eq!(report.total, 0);
}

#[test]
fn test_run_directory_with_tests() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    let test1 = r#"/*---
description: Test 1
---*/
var a = 1;
"#;

    let test2 = r#"/*---
description: Test 2
---*/
var b = 2;
"#;

    let test3 = r#"/*---
description: Test 3 with missing feature
features: [unsupported-xyz]
---*/
xyz();
"#;

    fs::write(temp_dir.path().join("test1.js"), test1).unwrap();
    fs::write(temp_dir.path().join("test2.js"), test2).unwrap();
    fs::write(temp_dir.path().join("test3.js"), test3).unwrap();

    let mut harness = Test262Harness::new();
    let report = harness.run_directory(temp_dir.path());

    assert_eq!(report.total, 3);
    assert_eq!(report.passed, 2);
    assert_eq!(report.skipped, 1);
}

#[test]
fn test_run_directory_skips_non_js_files() {
    let temp_dir = TempDir::new().unwrap();

    let test1 = r#"/*---
description: Test 1
---*/
var a = 1;
"#;

    fs::write(temp_dir.path().join("test1.js"), test1).unwrap();
    fs::write(temp_dir.path().join("readme.md"), "# README").unwrap();
    fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

    let mut harness = Test262Harness::new();
    let report = harness.run_directory(temp_dir.path());

    // Should only run the .js file
    assert_eq!(report.total, 1);
}

#[test]
fn test_harness_results_tracking() {
    let temp_dir = TempDir::new().unwrap();

    let test1 = r#"/*---
description: Test 1
---*/
var a = 1;
"#;

    let test_path = temp_dir.path().join("test1.js");
    fs::write(&test_path, test1).unwrap();

    let mut harness = Test262Harness::new();
    harness.run_directory(temp_dir.path());

    assert_eq!(harness.test_count(), 1);
    assert_eq!(harness.pass_count(), 1);
    assert_eq!(harness.fail_count(), 0);
    assert_eq!(harness.skip_count(), 0);
}

#[test]
fn test_harness_clears_results() {
    let temp_dir = TempDir::new().unwrap();

    let test1 = r#"/*---
description: Test 1
---*/
var a = 1;
"#;

    fs::write(temp_dir.path().join("test1.js"), test1).unwrap();

    let mut harness = Test262Harness::new();
    harness.run_directory(temp_dir.path());
    assert_eq!(harness.test_count(), 1);

    harness.clear_results();
    assert_eq!(harness.test_count(), 0);
}

#[test]
fn test_load_nonexistent_file() {
    let result = TestFile::load("/nonexistent/path/test.js");
    assert!(result.is_err());
}

#[test]
fn test_load_file_without_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("no_frontmatter.js");

    let test_content = r#"// No frontmatter here
var x = 1;
"#;

    fs::write(&test_path, test_content).unwrap();

    let result = TestFile::load(&test_path);
    assert!(result.is_err());
}

#[test]
fn test_report_json_roundtrip() {
    let mut harness = Test262Harness::new();
    let temp_dir = TempDir::new().unwrap();

    let test1 = r#"/*---
description: Test 1
---*/
var a = 1;
"#;

    fs::write(temp_dir.path().join("test1.js"), test1).unwrap();

    let report = harness.run_directory(temp_dir.path());
    let json = report.to_json().unwrap();
    let restored = test262_harness::TestReport::from_json(&json).unwrap();

    assert_eq!(report.total, restored.total);
    assert_eq!(report.passed, restored.passed);
    assert_eq!(report.failed, restored.failed);
    assert_eq!(report.skipped, restored.skipped);
}
