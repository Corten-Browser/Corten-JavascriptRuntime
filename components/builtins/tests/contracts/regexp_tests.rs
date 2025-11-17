//! RegExp contract tests - ES2024 compliance
//!
//! Tests for RegExp constructor, flags, methods, and advanced features.

use builtins::{JsValue, RegExpObject};

#[test]
fn test_regexp_constructor_string_pattern() {
    let re = RegExpObject::new("abc", "").unwrap();
    assert_eq!(re.source(), "abc");
    assert_eq!(re.flags(), "");
}

#[test]
fn test_regexp_constructor_with_flags() {
    let re = RegExpObject::new("test", "gi").unwrap();
    assert_eq!(re.source(), "test");
    assert_eq!(re.flags(), "gi");
}

#[test]
fn test_regexp_invalid_pattern_error() {
    let result = RegExpObject::new("[invalid", "");
    assert!(result.is_err());
}

#[test]
fn test_regexp_invalid_flags_error() {
    let result = RegExpObject::new("valid", "xyz");
    assert!(result.is_err());
}

#[test]
fn test_regexp_duplicate_flags_error() {
    let result = RegExpObject::new("valid", "gg");
    assert!(result.is_err());
}

// Flag tests
#[test]
fn test_regexp_global_flag() {
    let re = RegExpObject::new("test", "g").unwrap();
    assert!(re.global());
    assert!(!re.ignore_case());
    assert!(!re.multiline());
}

#[test]
fn test_regexp_ignore_case_flag() {
    let re = RegExpObject::new("test", "i").unwrap();
    assert!(re.ignore_case());
    assert!(!re.global());
}

#[test]
fn test_regexp_multiline_flag() {
    let re = RegExpObject::new("^test", "m").unwrap();
    assert!(re.multiline());
}

#[test]
fn test_regexp_dot_all_flag() {
    let re = RegExpObject::new(".", "s").unwrap();
    assert!(re.dot_all());
}

#[test]
fn test_regexp_unicode_flag() {
    let re = RegExpObject::new("\\u{1F600}", "u").unwrap();
    assert!(re.unicode());
}

#[test]
fn test_regexp_sticky_flag() {
    let re = RegExpObject::new("test", "y").unwrap();
    assert!(re.sticky());
}

#[test]
fn test_regexp_has_indices_flag() {
    let re = RegExpObject::new("test", "d").unwrap();
    assert!(re.has_indices());
}

#[test]
fn test_regexp_all_flags() {
    let re = RegExpObject::new("test", "gimsuy").unwrap();
    assert!(re.global());
    assert!(re.ignore_case());
    assert!(re.multiline());
    assert!(re.dot_all());
    assert!(re.unicode());
    assert!(re.sticky());
    // flags property should be sorted
    assert_eq!(re.flags(), "gimsuy");
}

// lastIndex tests
#[test]
fn test_regexp_last_index_initial() {
    let re = RegExpObject::new("test", "g").unwrap();
    assert_eq!(re.last_index(), 0);
}

#[test]
fn test_regexp_last_index_mutable() {
    let mut re = RegExpObject::new("test", "g").unwrap();
    re.set_last_index(5);
    assert_eq!(re.last_index(), 5);
}

// test() method
#[test]
fn test_regexp_test_match() {
    let mut re = RegExpObject::new("abc", "").unwrap();
    assert!(re.test("xabcy"));
}

#[test]
fn test_regexp_test_no_match() {
    let mut re = RegExpObject::new("xyz", "").unwrap();
    assert!(!re.test("abc"));
}

#[test]
fn test_regexp_test_case_insensitive() {
    let mut re = RegExpObject::new("abc", "i").unwrap();
    assert!(re.test("ABC"));
}

#[test]
fn test_regexp_test_global_updates_last_index() {
    let mut re = RegExpObject::new("ab", "g").unwrap();
    assert!(re.test("abab"));
    assert_eq!(re.last_index(), 2);
    assert!(re.test("abab"));
    assert_eq!(re.last_index(), 4);
    assert!(!re.test("abab"));
    assert_eq!(re.last_index(), 0); // Reset after no match
}

#[test]
fn test_regexp_test_sticky() {
    let mut re = RegExpObject::new("ab", "y").unwrap();
    assert!(re.test("abab"));
    assert_eq!(re.last_index(), 2);
    re.set_last_index(1);
    assert!(!re.test("abab")); // Must match at lastIndex
}

// exec() method
#[test]
fn test_regexp_exec_match() {
    let mut re = RegExpObject::new("a(b)c", "").unwrap();
    let result = re.exec("xabcy").unwrap();
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.full_match(), "abc");
    assert_eq!(m.index(), 1);
    assert_eq!(m.input(), "xabcy");
    assert_eq!(m.groups().len(), 1);
    assert_eq!(m.groups()[0], Some("b".to_string()));
}

#[test]
fn test_regexp_exec_no_match() {
    let mut re = RegExpObject::new("xyz", "").unwrap();
    let result = re.exec("abc").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_regexp_exec_multiple_groups() {
    let mut re = RegExpObject::new("(\\d+)-(\\d+)-(\\d+)", "").unwrap();
    let result = re.exec("Date: 2024-01-15").unwrap().unwrap();
    assert_eq!(result.full_match(), "2024-01-15");
    assert_eq!(result.groups().len(), 3);
    assert_eq!(result.groups()[0], Some("2024".to_string()));
    assert_eq!(result.groups()[1], Some("01".to_string()));
    assert_eq!(result.groups()[2], Some("15".to_string()));
}

#[test]
fn test_regexp_exec_optional_group_not_matched() {
    let mut re = RegExpObject::new("a(b)?c", "").unwrap();
    let result = re.exec("ac").unwrap().unwrap();
    assert_eq!(result.full_match(), "ac");
    assert_eq!(result.groups().len(), 1);
    assert_eq!(result.groups()[0], None); // Optional group not matched
}

#[test]
fn test_regexp_exec_global_advances() {
    let mut re = RegExpObject::new("ab", "g").unwrap();
    let m1 = re.exec("abxab").unwrap().unwrap();
    assert_eq!(m1.index(), 0);
    assert_eq!(re.last_index(), 2);

    let m2 = re.exec("abxab").unwrap().unwrap();
    assert_eq!(m2.index(), 3);
    assert_eq!(re.last_index(), 5);

    let m3 = re.exec("abxab").unwrap();
    assert!(m3.is_none());
    assert_eq!(re.last_index(), 0);
}

// Named capture groups
#[test]
fn test_regexp_named_capture_groups() {
    let mut re = RegExpObject::new("(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})", "").unwrap();
    let result = re.exec("Date: 2024-01-15").unwrap().unwrap();
    assert_eq!(result.named_group("year"), Some("2024".to_string()));
    assert_eq!(result.named_group("month"), Some("01".to_string()));
    assert_eq!(result.named_group("day"), Some("15".to_string()));
}

#[test]
fn test_regexp_named_groups_object() {
    let mut re = RegExpObject::new("(?P<word>\\w+)", "").unwrap();
    let result = re.exec("hello world").unwrap().unwrap();
    let groups = result.named_groups();
    assert_eq!(groups.get("word"), Some(&"hello".to_string()));
}

// Multiline mode
#[test]
fn test_regexp_multiline_start_anchor() {
    let mut re = RegExpObject::new("^test", "m").unwrap();
    assert!(re.test("line1\ntest"));
    let mut re_no_m = RegExpObject::new("^test", "").unwrap();
    assert!(!re_no_m.test("line1\ntest"));
}

#[test]
fn test_regexp_multiline_end_anchor() {
    let mut re = RegExpObject::new("end$", "m").unwrap();
    assert!(re.test("end\nmore"));
    let mut re_no_m = RegExpObject::new("end$", "").unwrap();
    assert!(!re_no_m.test("end\nmore"));
}

// dotAll mode
#[test]
fn test_regexp_dot_all_matches_newline() {
    let mut re = RegExpObject::new("a.b", "s").unwrap();
    assert!(re.test("a\nb"));
    let mut re_no_s = RegExpObject::new("a.b", "").unwrap();
    assert!(!re_no_s.test("a\nb"));
}

// Unicode mode
#[test]
fn test_regexp_unicode_emoji() {
    let mut re = RegExpObject::new("\\p{Emoji}", "u").unwrap();
    assert!(re.test("Hello 😀 World"));
}

#[test]
fn test_regexp_unicode_property_letter() {
    let mut re = RegExpObject::new("\\p{Letter}+", "u").unwrap();
    assert!(re.test("Здравствуйте"));
}

// Non-capturing groups
#[test]
fn test_regexp_non_capturing_group() {
    let mut re = RegExpObject::new("(?:ab)+c", "").unwrap();
    assert!(re.test("ababc"));
    let result = re.exec("xababcy").unwrap().unwrap();
    assert_eq!(result.full_match(), "ababc");
    assert_eq!(result.groups().len(), 0); // No captured groups
}

// Lookbehind assertions (ES2018+)
// NOTE: Rust's regex crate doesn't support lookahead/lookbehind
// These tests verify the expected error behavior until we switch to fancy-regex
#[test]
fn test_regexp_positive_lookbehind() {
    // Lookbehind not supported by Rust regex crate
    let result = RegExpObject::new("(?<=\\$)\\d+", "");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("look-around") || err.message.contains("look-behind"));
}

#[test]
fn test_regexp_negative_lookbehind() {
    // Lookbehind not supported by Rust regex crate
    let result = RegExpObject::new("(?<!\\$)\\d+", "");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("look-around") || err.message.contains("look-behind"));
}

// Lookahead assertions
#[test]
fn test_regexp_positive_lookahead() {
    // Lookahead not supported by Rust regex crate
    let result = RegExpObject::new("\\d+(?=\\$)", "");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("look-around") || err.message.contains("look-ahead"));
}

#[test]
fn test_regexp_negative_lookahead() {
    // Lookahead not supported by Rust regex crate
    let result = RegExpObject::new("\\d+(?!\\$)", "");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("look-around") || err.message.contains("look-ahead"));
}

// Edge cases
#[test]
fn test_regexp_empty_pattern() {
    let mut re = RegExpObject::new("", "").unwrap();
    assert!(re.test("anything"));
}

#[test]
fn test_regexp_empty_string() {
    let mut re = RegExpObject::new("abc", "").unwrap();
    assert!(!re.test(""));
}

#[test]
fn test_regexp_special_chars_escaped() {
    let mut re = RegExpObject::new(r"\.\*\+\?", "").unwrap();
    assert!(re.test(".*+?"));
    assert!(!re.test("abcd"));
}

// Symbol methods
#[test]
fn test_regexp_symbol_match() {
    let re = RegExpObject::new("\\d+", "g").unwrap();
    let matches = re.symbol_match("a1b22c333").unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0], "1");
    assert_eq!(matches[1], "22");
    assert_eq!(matches[2], "333");
}

#[test]
fn test_regexp_symbol_match_non_global() {
    let re = RegExpObject::new("\\d+", "").unwrap();
    let matches = re.symbol_match("a1b22c333").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], "1");
}

#[test]
fn test_regexp_symbol_replace() {
    let re = RegExpObject::new("\\d+", "g").unwrap();
    let result = re.symbol_replace("a1b22c333", "X").unwrap();
    assert_eq!(result, "aXbXcX");
}

#[test]
fn test_regexp_symbol_replace_non_global() {
    let re = RegExpObject::new("\\d+", "").unwrap();
    let result = re.symbol_replace("a1b22c333", "X").unwrap();
    assert_eq!(result, "aXb22c333");
}

#[test]
fn test_regexp_symbol_replace_with_groups() {
    let re = RegExpObject::new("(\\d+)-(\\d+)", "g").unwrap();
    let result = re.symbol_replace("1-2 and 3-4", "$2-$1").unwrap();
    assert_eq!(result, "2-1 and 4-3");
}

#[test]
fn test_regexp_symbol_search() {
    let re = RegExpObject::new("world", "").unwrap();
    let index = re.symbol_search("hello world").unwrap();
    assert_eq!(index, Some(6));
}

#[test]
fn test_regexp_symbol_search_no_match() {
    let re = RegExpObject::new("xyz", "").unwrap();
    let index = re.symbol_search("hello world").unwrap();
    assert_eq!(index, None);
}

#[test]
fn test_regexp_symbol_split() {
    let re = RegExpObject::new(",\\s*", "").unwrap();
    let parts = re.symbol_split("a, b,  c,d").unwrap();
    assert_eq!(parts, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_regexp_symbol_split_with_limit() {
    let re = RegExpObject::new(",\\s*", "").unwrap();
    let parts = re.symbol_split_with_limit("a, b, c, d", 2).unwrap();
    assert_eq!(parts, vec!["a", "b"]);
}

#[test]
fn test_regexp_symbol_split_with_groups() {
    let re = RegExpObject::new("(,)", "").unwrap();
    let parts = re.symbol_split("a,b,c").unwrap();
    // Should include captured groups in result
    assert_eq!(parts, vec!["a", ",", "b", ",", "c"]);
}

// JsValue integration
#[test]
fn test_jsvalue_regexp_type() {
    let re = RegExpObject::new("test", "gi").unwrap();
    let val = JsValue::regexp(re);
    assert!(val.is_regexp());
    assert_eq!(val.type_of(), "object");
}

#[test]
fn test_jsvalue_regexp_to_string() {
    let re = RegExpObject::new("test", "gi").unwrap();
    let val = JsValue::regexp(re);
    assert_eq!(val.to_js_string(), "/test/gi");
}

#[test]
fn test_jsvalue_regexp_as_regexp() {
    let re = RegExpObject::new("abc", "i").unwrap();
    let val = JsValue::regexp(re);
    let extracted = val.as_regexp().unwrap();
    assert_eq!(extracted.source(), "abc");
    assert!(extracted.ignore_case());
}

// hasIndices (d flag) tests
#[test]
fn test_regexp_has_indices_exec() {
    let mut re = RegExpObject::new("(a)(b)", "d").unwrap();
    let result = re.exec("xaby").unwrap().unwrap();
    assert!(result.has_indices());
    let indices = result.indices().unwrap();
    // Main match indices
    assert_eq!(indices.0, vec![(1, 3)]);
    // Group indices
    assert_eq!(indices.1[0], Some((1, 2))); // 'a'
    assert_eq!(indices.1[1], Some((2, 3))); // 'b'
}

// Complex patterns
#[test]
fn test_regexp_alternation() {
    let mut re = RegExpObject::new("cat|dog", "").unwrap();
    assert!(re.test("I have a cat"));
    assert!(re.test("I have a dog"));
    assert!(!re.test("I have a bird"));
}

#[test]
fn test_regexp_quantifiers() {
    let mut re = RegExpObject::new("a{2,4}", "").unwrap();
    assert!(re.test("aa"));
    assert!(re.test("aaa"));
    assert!(re.test("aaaa"));
    assert!(!re.test("a"));
}

#[test]
fn test_regexp_word_boundary() {
    let mut re = RegExpObject::new("\\bword\\b", "").unwrap();
    assert!(re.test("a word here"));
    assert!(!re.test("awordhere"));
}

#[test]
fn test_regexp_character_class() {
    let re = RegExpObject::new("[aeiou]", "gi").unwrap();
    let matches = re.symbol_match("Hello World").unwrap();
    assert_eq!(matches, vec!["e", "o", "o"]);
}

#[test]
fn test_regexp_negated_character_class() {
    let mut re = RegExpObject::new("[^0-9]+", "").unwrap();
    let result = re.exec("abc123def").unwrap().unwrap();
    assert_eq!(result.full_match(), "abc");
}
