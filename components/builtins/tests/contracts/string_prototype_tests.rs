//! Contract tests for StringPrototype

use builtins::{StringPrototype, JsValue, JsResult};

#[test]
fn test_string_substring() {
    let s = "hello world";
    let result = StringPrototype::substring(s, 0, Some(5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_string_slice() {
    let s = "hello world";
    let result = StringPrototype::slice(s, 6, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "world");
}

#[test]
fn test_string_split() {
    let s = "a,b,c";
    let result = StringPrototype::split(s, ",");
    assert!(result.is_ok());
    let parts = result.unwrap();
    assert_eq!(parts, vec!["a", "b", "c"]);
}

#[test]
fn test_string_replace() {
    let s = "hello world";
    let result = StringPrototype::replace(s, "world", "rust");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello rust");
}

#[test]
fn test_string_match() {
    let s = "test123";
    let result = StringPrototype::match_str(s, r"\d+");
    assert!(result.is_ok());
    let matches = result.unwrap();
    assert!(!matches.is_empty());
}

#[test]
fn test_string_trim() {
    let s = "  hello  ";
    let result = StringPrototype::trim(s);
    assert_eq!(result, "hello");
}

#[test]
fn test_string_to_lower_case() {
    let s = "HELLO";
    let result = StringPrototype::to_lower_case(s);
    assert_eq!(result, "hello");
}

#[test]
fn test_string_to_upper_case() {
    let s = "hello";
    let result = StringPrototype::to_upper_case(s);
    assert_eq!(result, "HELLO");
}

#[test]
fn test_string_char_at() {
    let s = "hello";
    let result = StringPrototype::char_at(s, 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "e");
}

#[test]
fn test_string_char_code_at() {
    let s = "hello";
    let result = StringPrototype::char_code_at(s, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 104); // 'h' = 104
}

#[test]
fn test_string_pad_start() {
    let s = "5";
    let result = StringPrototype::pad_start(s, 3, "0");
    assert_eq!(result, "005");
}

#[test]
fn test_string_pad_end() {
    let s = "5";
    let result = StringPrototype::pad_end(s, 3, "0");
    assert_eq!(result, "500");
}

#[test]
fn test_string_starts_with() {
    let s = "hello world";
    assert!(StringPrototype::starts_with(s, "hello"));
    assert!(!StringPrototype::starts_with(s, "world"));
}

#[test]
fn test_string_ends_with() {
    let s = "hello world";
    assert!(StringPrototype::ends_with(s, "world"));
    assert!(!StringPrototype::ends_with(s, "hello"));
}

#[test]
fn test_string_includes() {
    let s = "hello world";
    assert!(StringPrototype::includes(s, "lo wo"));
    assert!(!StringPrototype::includes(s, "foo"));
}
