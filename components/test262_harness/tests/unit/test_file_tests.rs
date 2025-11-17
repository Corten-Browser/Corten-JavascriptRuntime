//! Unit tests for test file and metadata parsing

use std::collections::HashSet;
use test262_harness::TestMetadata;

#[test]
fn test_parse_basic_metadata() {
    let source = r#"/*---
description: Test for addition operator
info: |
  The addition operator either performs string concatenation
  or numeric addition.
---*/
1 + 1;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.description, "Test for addition operator");
    assert!(metadata.info.is_some());
    assert!(metadata.info.unwrap().contains("string concatenation"));
}

#[test]
fn test_parse_negative_expectation_parse_phase() {
    let source = r#"/*---
description: Early error for reserved word
negative:
  phase: parse
  type: SyntaxError
---*/
var class = 1;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.negative.is_some());
    let neg = metadata.negative.as_ref().unwrap();
    assert_eq!(neg.phase, "parse");
    assert_eq!(neg.error_type, "SyntaxError");
    assert!(metadata.expects_parse_error());
    assert!(!metadata.expects_runtime_error());
}

#[test]
fn test_parse_negative_expectation_runtime_phase() {
    let source = r#"/*---
description: Runtime error for undefined variable
negative:
  phase: runtime
  type: ReferenceError
---*/
undefinedVar;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.negative.is_some());
    let neg = metadata.negative.as_ref().unwrap();
    assert_eq!(neg.phase, "runtime");
    assert_eq!(neg.error_type, "ReferenceError");
    assert!(!metadata.expects_parse_error());
    assert!(metadata.expects_runtime_error());
}

#[test]
fn test_parse_negative_expectation_resolution_phase() {
    let source = r#"/*---
description: Module resolution error
negative:
  phase: resolution
  type: SyntaxError
---*/
import { x } from './nonexistent.js';
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.expects_resolution_error());
    assert!(!metadata.expects_parse_error());
    assert!(!metadata.expects_runtime_error());
}

#[test]
fn test_parse_includes() {
    let source = r#"/*---
description: Test with helper includes
includes: [propertyHelper.js, testTypedArray.js]
---*/
test();
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.includes.len(), 2);
    assert!(metadata.includes.contains(&"propertyHelper.js".to_string()));
    assert!(metadata.includes.contains(&"testTypedArray.js".to_string()));
}

#[test]
fn test_parse_flags() {
    let source = r#"/*---
description: Strict mode only test
flags: [onlyStrict, async]
---*/
"use strict";
await Promise.resolve();
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.flags.contains("onlyStrict"));
    assert!(metadata.flags.contains("async"));
    assert!(metadata.is_strict_only());
    assert!(metadata.is_async());
    assert!(!metadata.is_no_strict());
    assert!(!metadata.is_module());
}

#[test]
fn test_parse_no_strict_flag() {
    let source = r#"/*---
description: Non-strict mode only test
flags: [noStrict]
---*/
test();
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.is_no_strict());
    assert!(!metadata.is_strict_only());
}

#[test]
fn test_parse_module_flag() {
    let source = r#"/*---
description: ES module test
flags: [module]
---*/
export default 42;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.is_module());
}

#[test]
fn test_parse_raw_flag() {
    let source = r#"/*---
description: Raw test without harness
flags: [raw]
---*/
// Test runs without harness setup
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert!(metadata.is_raw());
}

#[test]
fn test_parse_features() {
    let source = r#"/*---
description: BigInt and Symbol test
features: [BigInt, Symbol, optional-chaining]
---*/
let x = 42n;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.features.len(), 3);
    assert!(metadata.features.contains(&"BigInt".to_string()));
    assert!(metadata.features.contains(&"Symbol".to_string()));
    assert!(metadata.features.contains(&"optional-chaining".to_string()));
}

#[test]
fn test_parse_es_identifiers() {
    let source = r#"/*---
description: Test with ES identifiers
es5id: 12.3.4
es6id: 23.4.5.6
esid: sec-addition-operator-plus
---*/
1 + 1;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.es5id, Some("12.3.4".to_string()));
    assert_eq!(metadata.es6id, Some("23.4.5.6".to_string()));
    assert_eq!(metadata.esid, Some("sec-addition-operator-plus".to_string()));
}

#[test]
fn test_parse_author() {
    let source = r#"/*---
description: Test with author
author: John Doe
---*/
test();
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.author, Some("John Doe".to_string()));
}

#[test]
fn test_should_skip_missing_features() {
    let source = r#"/*---
description: Test requiring unsupported feature
features: [unsupported-feature, BigInt]
---*/
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    let mut supported = HashSet::new();
    supported.insert("BigInt".to_string());

    assert!(metadata.should_skip(&supported));
}

#[test]
fn test_should_not_skip_supported_features() {
    let source = r#"/*---
description: Test with supported features
features: [BigInt, Symbol]
---*/
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    let mut supported = HashSet::new();
    supported.insert("BigInt".to_string());
    supported.insert("Symbol".to_string());

    assert!(!metadata.should_skip(&supported));
}

#[test]
fn test_unsupported_features_list() {
    let source = r#"/*---
description: Test requiring multiple unsupported features
features: [BigInt, unsupported-1, Symbol, unsupported-2]
---*/
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    let mut supported = HashSet::new();
    supported.insert("BigInt".to_string());
    supported.insert("Symbol".to_string());

    let unsupported = metadata.unsupported_features(&supported);
    assert_eq!(unsupported.len(), 2);
    assert!(unsupported.contains(&"unsupported-1".to_string()));
    assert!(unsupported.contains(&"unsupported-2".to_string()));
}

#[test]
fn test_expected_error_type() {
    let source = r#"/*---
description: Test expecting TypeError
negative:
  phase: runtime
  type: TypeError
---*/
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.expected_error_type(), Some("TypeError"));
}

#[test]
fn test_no_expected_error_type() {
    let source = r#"/*---
description: Positive test
---*/
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.expected_error_type(), None);
}

#[test]
fn test_parse_empty_metadata() {
    let source = r#"/*---
description: Minimal test
---*/
1;
"#;

    let metadata = TestMetadata::parse(source).unwrap();
    assert_eq!(metadata.description, "Minimal test");
    assert!(metadata.includes.is_empty());
    assert!(metadata.flags.is_empty());
    assert!(metadata.features.is_empty());
    assert!(metadata.negative.is_none());
}

#[test]
fn test_parse_missing_frontmatter_error() {
    let source = r#"
// No frontmatter here
1 + 1;
"#;

    let result = TestMetadata::parse(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No YAML frontmatter found"));
}

#[test]
fn test_parse_invalid_yaml_error() {
    let source = r#"/*---
description: [invalid yaml
  not closed
---*/
"#;

    let result = TestMetadata::parse(source);
    assert!(result.is_err());
}

#[test]
fn test_default_metadata() {
    let metadata = TestMetadata::default();
    assert_eq!(metadata.description, "");
    assert!(metadata.info.is_none());
    assert!(metadata.negative.is_none());
    assert!(metadata.includes.is_empty());
    assert!(metadata.flags.is_empty());
    assert!(metadata.features.is_empty());
}
