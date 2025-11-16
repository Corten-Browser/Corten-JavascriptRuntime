//! String.prototype methods

use crate::value::{JsError, JsResult};
use regex::Regex;

/// String.prototype methods
pub struct StringPrototype;

impl StringPrototype {
    /// String.prototype.substring(start, end)
    pub fn substring(s: &str, start: usize, end: Option<usize>) -> JsResult<String> {
        let len = s.chars().count();
        let start_idx = start.min(len);
        let end_idx = end.unwrap_or(len).min(len);

        let (actual_start, actual_end) = if start_idx <= end_idx {
            (start_idx, end_idx)
        } else {
            (end_idx, start_idx)
        };

        Ok(s.chars()
            .skip(actual_start)
            .take(actual_end - actual_start)
            .collect())
    }

    /// String.prototype.slice(start, end)
    pub fn slice(s: &str, start: i32, end: Option<i32>) -> JsResult<String> {
        let len = s.chars().count() as i32;

        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let end_idx = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => e.min(len) as usize,
            None => len as usize,
        };

        if start_idx >= end_idx {
            Ok(String::new())
        } else {
            Ok(s.chars()
                .skip(start_idx)
                .take(end_idx - start_idx)
                .collect())
        }
    }

    /// String.prototype.split(separator)
    pub fn split(s: &str, separator: &str) -> JsResult<Vec<String>> {
        if separator.is_empty() {
            // Split into individual characters
            Ok(s.chars().map(|c| c.to_string()).collect())
        } else {
            Ok(s.split(separator).map(|part| part.to_string()).collect())
        }
    }

    /// String.prototype.replace(search, replacement)
    pub fn replace(s: &str, search: &str, replacement: &str) -> JsResult<String> {
        Ok(s.replacen(search, replacement, 1))
    }

    /// String.prototype.match(regexp)
    pub fn match_str(s: &str, pattern: &str) -> JsResult<Vec<String>> {
        let re = Regex::new(pattern)
            .map_err(|e| JsError::syntax_error(format!("Invalid regex: {}", e)))?;

        let matches: Vec<String> = re
            .find_iter(s)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(matches)
    }

    /// String.prototype.trim()
    pub fn trim(s: &str) -> String {
        s.trim().to_string()
    }

    /// String.prototype.toLowerCase()
    pub fn to_lower_case(s: &str) -> String {
        s.to_lowercase()
    }

    /// String.prototype.toUpperCase()
    pub fn to_upper_case(s: &str) -> String {
        s.to_uppercase()
    }

    /// String.prototype.charAt(index)
    pub fn char_at(s: &str, index: usize) -> JsResult<String> {
        Ok(s.chars()
            .nth(index)
            .map(|c| c.to_string())
            .unwrap_or_default())
    }

    /// String.prototype.charCodeAt(index)
    pub fn char_code_at(s: &str, index: usize) -> JsResult<u32> {
        s.chars()
            .nth(index)
            .map(|c| c as u32)
            .ok_or_else(|| JsError::range_error("Index out of bounds"))
    }

    /// String.prototype.padStart(targetLength, padString)
    pub fn pad_start(s: &str, target_length: usize, pad_string: &str) -> String {
        let current_len = s.chars().count();
        if current_len >= target_length {
            return s.to_string();
        }

        let pad_len = target_length - current_len;
        let pad_chars: Vec<char> = pad_string.chars().collect();

        if pad_chars.is_empty() {
            return s.to_string();
        }

        let mut padding = String::new();
        let mut i = 0;
        while padding.chars().count() < pad_len {
            padding.push(pad_chars[i % pad_chars.len()]);
            i += 1;
        }

        // Truncate padding if it's too long
        let padding: String = padding.chars().take(pad_len).collect();
        format!("{}{}", padding, s)
    }

    /// String.prototype.padEnd(targetLength, padString)
    pub fn pad_end(s: &str, target_length: usize, pad_string: &str) -> String {
        let current_len = s.chars().count();
        if current_len >= target_length {
            return s.to_string();
        }

        let pad_len = target_length - current_len;
        let pad_chars: Vec<char> = pad_string.chars().collect();

        if pad_chars.is_empty() {
            return s.to_string();
        }

        let mut padding = String::new();
        let mut i = 0;
        while padding.chars().count() < pad_len {
            padding.push(pad_chars[i % pad_chars.len()]);
            i += 1;
        }

        // Truncate padding if it's too long
        let padding: String = padding.chars().take(pad_len).collect();
        format!("{}{}", s, padding)
    }

    /// String.prototype.startsWith(searchString)
    pub fn starts_with(s: &str, search_string: &str) -> bool {
        s.starts_with(search_string)
    }

    /// String.prototype.endsWith(searchString)
    pub fn ends_with(s: &str, search_string: &str) -> bool {
        s.ends_with(search_string)
    }

    /// String.prototype.includes(searchString)
    pub fn includes(s: &str, search_string: &str) -> bool {
        s.contains(search_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring() {
        assert_eq!(
            StringPrototype::substring("hello world", 0, Some(5)).unwrap(),
            "hello"
        );
        assert_eq!(
            StringPrototype::substring("hello", 2, None).unwrap(),
            "llo"
        );
    }

    #[test]
    fn test_slice() {
        assert_eq!(
            StringPrototype::slice("hello world", 6, None).unwrap(),
            "world"
        );
        assert_eq!(
            StringPrototype::slice("hello", -3, None).unwrap(),
            "llo"
        );
    }

    #[test]
    fn test_split() {
        let parts = StringPrototype::split("a,b,c", ",").unwrap();
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            StringPrototype::replace("hello world", "world", "rust").unwrap(),
            "hello rust"
        );
    }

    #[test]
    fn test_match_str() {
        let matches = StringPrototype::match_str("test123abc456", r"\d+").unwrap();
        assert_eq!(matches, vec!["123", "456"]);
    }

    #[test]
    fn test_trim() {
        assert_eq!(StringPrototype::trim("  hello  "), "hello");
    }

    #[test]
    fn test_to_lower_case() {
        assert_eq!(StringPrototype::to_lower_case("HELLO"), "hello");
    }

    #[test]
    fn test_to_upper_case() {
        assert_eq!(StringPrototype::to_upper_case("hello"), "HELLO");
    }

    #[test]
    fn test_char_at() {
        assert_eq!(StringPrototype::char_at("hello", 1).unwrap(), "e");
        assert_eq!(StringPrototype::char_at("hello", 10).unwrap(), "");
    }

    #[test]
    fn test_char_code_at() {
        assert_eq!(StringPrototype::char_code_at("hello", 0).unwrap(), 104);
    }

    #[test]
    fn test_pad_start() {
        assert_eq!(StringPrototype::pad_start("5", 3, "0"), "005");
        assert_eq!(StringPrototype::pad_start("hello", 3, "x"), "hello");
    }

    #[test]
    fn test_pad_end() {
        assert_eq!(StringPrototype::pad_end("5", 3, "0"), "500");
    }

    #[test]
    fn test_starts_with() {
        assert!(StringPrototype::starts_with("hello world", "hello"));
        assert!(!StringPrototype::starts_with("hello world", "world"));
    }

    #[test]
    fn test_ends_with() {
        assert!(StringPrototype::ends_with("hello world", "world"));
        assert!(!StringPrototype::ends_with("hello world", "hello"));
    }

    #[test]
    fn test_includes() {
        assert!(StringPrototype::includes("hello world", "lo wo"));
        assert!(!StringPrototype::includes("hello world", "foo"));
    }
}
