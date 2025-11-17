//! RegExp implementation for ECMAScript 2024
//!
//! Provides full regular expression support including:
//! - Pattern compilation with flags
//! - Named capture groups
//! - Lookbehind assertions
//! - Unicode property escapes
//! - Symbol methods for string integration

use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

use crate::value::{JsError, JsResult};

/// Match result from exec()
#[derive(Debug, Clone)]
pub struct RegExpMatch {
    /// The full matched string
    full: String,
    /// Index where match was found
    idx: usize,
    /// The input string
    inp: String,
    /// Captured groups (None if group didn't participate)
    captured_groups: Vec<Option<String>>,
    /// Named groups mapping
    named: HashMap<String, String>,
    /// Whether hasIndices flag was set
    has_indices_flag: bool,
    /// Match indices (start, end) for main match and groups
    match_indices: Option<(Vec<(usize, usize)>, Vec<Option<(usize, usize)>>)>,
}

impl RegExpMatch {
    /// Get the full matched string
    pub fn full_match(&self) -> &str {
        &self.full
    }

    /// Get the index where match was found
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Get the input string
    pub fn input(&self) -> &str {
        &self.inp
    }

    /// Get captured groups (positional)
    pub fn groups(&self) -> &[Option<String>] {
        &self.captured_groups
    }

    /// Get a named capture group
    pub fn named_group(&self, name: &str) -> Option<String> {
        self.named.get(name).cloned()
    }

    /// Get all named groups
    pub fn named_groups(&self) -> &HashMap<String, String> {
        &self.named
    }

    /// Check if hasIndices flag was set
    pub fn has_indices(&self) -> bool {
        self.has_indices_flag
    }

    /// Get match indices (requires 'd' flag)
    pub fn indices(&self) -> Option<&(Vec<(usize, usize)>, Vec<Option<(usize, usize)>>)> {
        self.match_indices.as_ref()
    }
}

/// RegExp object
#[derive(Debug, Clone)]
pub struct RegExpObject {
    /// Compiled regex
    regex: Regex,
    /// Original source pattern
    source_pattern: String,
    /// Flags string (sorted)
    flags_str: String,
    /// Individual flags
    flag_global: bool,
    flag_ignore_case: bool,
    flag_multiline: bool,
    flag_dot_all: bool,
    flag_unicode: bool,
    flag_sticky: bool,
    flag_has_indices: bool,
    /// lastIndex property (mutable)
    last_index_value: usize,
    /// Named group names in order
    group_names: Vec<Option<String>>,
}

impl RegExpObject {
    /// Create a new RegExp from pattern and flags
    pub fn new(pattern: &str, flags: &str) -> JsResult<Self> {
        // Validate flags
        let (sorted_flags, flag_g, flag_i, flag_m, flag_s, flag_u, flag_y, flag_d) =
            Self::parse_flags(flags)?;

        // Convert JavaScript pattern to Rust regex pattern
        let (rust_pattern, group_names) = Self::convert_pattern(pattern, flag_u)?;

        // Build regex with appropriate flags
        let regex = Self::build_regex(&rust_pattern, flag_i, flag_m, flag_s, flag_u)?;

        Ok(RegExpObject {
            regex,
            source_pattern: pattern.to_string(),
            flags_str: sorted_flags,
            flag_global: flag_g,
            flag_ignore_case: flag_i,
            flag_multiline: flag_m,
            flag_dot_all: flag_s,
            flag_unicode: flag_u,
            flag_sticky: flag_y,
            flag_has_indices: flag_d,
            last_index_value: 0,
            group_names,
        })
    }

    fn parse_flags(flags: &str) -> JsResult<(String, bool, bool, bool, bool, bool, bool, bool)> {
        let mut g = false;
        let mut i = false;
        let mut m = false;
        let mut s = false;
        let mut u = false;
        let mut y = false;
        let mut d = false;

        for ch in flags.chars() {
            match ch {
                'g' => {
                    if g {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'g'"));
                    }
                    g = true;
                }
                'i' => {
                    if i {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'i'"));
                    }
                    i = true;
                }
                'm' => {
                    if m {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'm'"));
                    }
                    m = true;
                }
                's' => {
                    if s {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 's'"));
                    }
                    s = true;
                }
                'u' => {
                    if u {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'u'"));
                    }
                    u = true;
                }
                'y' => {
                    if y {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'y'"));
                    }
                    y = true;
                }
                'd' => {
                    if d {
                        return Err(JsError::syntax_error("Invalid flags: duplicate 'd'"));
                    }
                    d = true;
                }
                'v' => {
                    // unicodeSets flag - treat as unicode for now
                    if u {
                        return Err(JsError::syntax_error(
                            "Invalid flags: 'v' incompatible with 'u'",
                        ));
                    }
                    u = true;
                }
                _ => {
                    return Err(JsError::syntax_error(format!("Invalid flag: '{}'", ch)));
                }
            }
        }

        // Build sorted flags string
        let mut sorted = String::new();
        if d {
            sorted.push('d');
        }
        if g {
            sorted.push('g');
        }
        if i {
            sorted.push('i');
        }
        if m {
            sorted.push('m');
        }
        if s {
            sorted.push('s');
        }
        if u {
            sorted.push('u');
        }
        if y {
            sorted.push('y');
        }

        Ok((sorted, g, i, m, s, u, y, d))
    }

    fn convert_pattern(
        pattern: &str,
        _unicode: bool,
    ) -> JsResult<(String, Vec<Option<String>>)> {
        // JavaScript uses (?<name>...) but Rust regex uses (?P<name>...)
        // However, Rust regex also supports (?<name>...) in newer versions
        // For now, convert (?<name>...) to (?P<name>...) for compatibility
        let mut result = String::new();
        let mut group_names = Vec::new();
        let mut chars = pattern.chars().peekable();
        let mut _group_count = 0;

        while let Some(ch) = chars.next() {
            if ch == '(' {
                if chars.peek() == Some(&'?') {
                    chars.next(); // consume '?'
                    if chars.peek() == Some(&'<') {
                        chars.next(); // consume '<'
                        // Check if it's a named group or lookbehind
                        if chars.peek() == Some(&'=') || chars.peek() == Some(&'!') {
                            // Lookbehind assertion - pass through
                            result.push_str("(?<");
                        } else {
                            // Named capture group (?<name>...)
                            let mut name = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '>' {
                                    chars.next();
                                    break;
                                }
                                name.push(c);
                                chars.next();
                            }
                            _group_count += 1;
                            group_names.push(Some(name.clone()));
                            result.push_str(&format!("(?P<{}>", name));
                        }
                    } else if chars.peek() == Some(&':') {
                        // Non-capturing group
                        chars.next();
                        result.push_str("(?:");
                    } else if chars.peek() == Some(&'=') {
                        // Positive lookahead
                        chars.next();
                        result.push_str("(?=");
                    } else if chars.peek() == Some(&'!') {
                        // Negative lookahead
                        chars.next();
                        result.push_str("(?!");
                    } else if chars.peek() == Some(&'P') {
                        // Already in Rust syntax (?P<name>...)
                        chars.next(); // consume 'P'
                        if chars.peek() == Some(&'<') {
                            chars.next(); // consume '<'
                            let mut name = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '>' {
                                    chars.next();
                                    break;
                                }
                                name.push(c);
                                chars.next();
                            }
                            _group_count += 1;
                            group_names.push(Some(name.clone()));
                            result.push_str(&format!("(?P<{}>", name));
                        } else {
                            result.push_str("(?P");
                        }
                    } else {
                        result.push_str("(?");
                    }
                } else {
                    // Regular capturing group
                    _group_count += 1;
                    group_names.push(None);
                    result.push('(');
                }
            } else {
                result.push(ch);
            }
        }

        Ok((result, group_names))
    }

    fn build_regex(
        pattern: &str,
        case_insensitive: bool,
        multiline: bool,
        dot_all: bool,
        _unicode: bool,
    ) -> JsResult<Regex> {
        let mut builder = RegexBuilder::new(pattern);

        builder.case_insensitive(case_insensitive);
        builder.multi_line(multiline);
        builder.dot_matches_new_line(dot_all);
        // Always use unicode mode for proper UTF-8 handling
        builder.unicode(true);

        builder
            .build()
            .map_err(|e| JsError::syntax_error(format!("Invalid regular expression: {}", e)))
    }

    /// Get the source pattern
    pub fn source(&self) -> &str {
        &self.source_pattern
    }

    /// Get the flags string
    pub fn flags(&self) -> &str {
        &self.flags_str
    }

    /// Check if global flag is set
    pub fn global(&self) -> bool {
        self.flag_global
    }

    /// Check if ignoreCase flag is set
    pub fn ignore_case(&self) -> bool {
        self.flag_ignore_case
    }

    /// Check if multiline flag is set
    pub fn multiline(&self) -> bool {
        self.flag_multiline
    }

    /// Check if dotAll flag is set
    pub fn dot_all(&self) -> bool {
        self.flag_dot_all
    }

    /// Check if unicode flag is set
    pub fn unicode(&self) -> bool {
        self.flag_unicode
    }

    /// Check if sticky flag is set
    pub fn sticky(&self) -> bool {
        self.flag_sticky
    }

    /// Check if hasIndices flag is set
    pub fn has_indices(&self) -> bool {
        self.flag_has_indices
    }

    /// Get lastIndex
    pub fn last_index(&self) -> usize {
        self.last_index_value
    }

    /// Set lastIndex
    pub fn set_last_index(&mut self, value: usize) {
        self.last_index_value = value;
    }

    /// Test if pattern matches string
    pub fn test(&mut self, string: &str) -> bool {
        if self.flag_global || self.flag_sticky {
            let start_index = self.last_index_value;
            if start_index > string.len() {
                self.last_index_value = 0;
                return false;
            }

            let search_string = &string[start_index..];

            if self.flag_sticky {
                // Must match at the start of search_string
                if let Some(mat) = self.regex.find(search_string) {
                    if mat.start() == 0 {
                        self.last_index_value = start_index + mat.end();
                        return true;
                    }
                }
                self.last_index_value = 0;
                false
            } else {
                // Global: find next match
                if let Some(mat) = self.regex.find(search_string) {
                    self.last_index_value = start_index + mat.end();
                    true
                } else {
                    self.last_index_value = 0;
                    false
                }
            }
        } else {
            self.regex.is_match(string)
        }
    }

    /// Execute pattern on string, returning match result
    pub fn exec(&mut self, string: &str) -> JsResult<Option<RegExpMatch>> {
        let start_index = if self.flag_global || self.flag_sticky {
            self.last_index_value
        } else {
            0
        };

        if start_index > string.len() {
            if self.flag_global || self.flag_sticky {
                self.last_index_value = 0;
            }
            return Ok(None);
        }

        let search_string = &string[start_index..];

        let captures = if self.flag_sticky {
            // Must match at start
            match self.regex.captures(search_string) {
                Some(caps) => {
                    if caps.get(0).map(|m| m.start()) == Some(0) {
                        Some(caps)
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            self.regex.captures(search_string)
        };

        match captures {
            Some(caps) => {
                let full_match = caps.get(0).unwrap();
                let match_start = start_index + full_match.start();
                let match_end = start_index + full_match.end();

                // Update lastIndex for global/sticky
                if self.flag_global || self.flag_sticky {
                    self.last_index_value = match_end;
                }

                // Extract captured groups
                let mut captured_groups = Vec::new();
                let mut named_groups = HashMap::new();

                for i in 1..caps.len() {
                    let group_value = caps.get(i).map(|m| m.as_str().to_string());
                    captured_groups.push(group_value.clone());

                    // Check if this group has a name
                    if let Some(Some(name)) = self.group_names.get(i - 1) {
                        if let Some(value) = group_value {
                            named_groups.insert(name.clone(), value);
                        }
                    }
                }

                // Calculate indices if 'd' flag is set
                let match_indices = if self.flag_has_indices {
                    let main_indices = vec![(match_start, match_end)];
                    let mut group_indices = Vec::new();

                    for i in 1..caps.len() {
                        let idx = caps.get(i).map(|m| {
                            (start_index + m.start(), start_index + m.end())
                        });
                        group_indices.push(idx);
                    }

                    Some((main_indices, group_indices))
                } else {
                    None
                };

                Ok(Some(RegExpMatch {
                    full: full_match.as_str().to_string(),
                    idx: match_start,
                    inp: string.to_string(),
                    captured_groups,
                    named: named_groups,
                    has_indices_flag: self.flag_has_indices,
                    match_indices,
                }))
            }
            None => {
                if self.flag_global || self.flag_sticky {
                    self.last_index_value = 0;
                }
                Ok(None)
            }
        }
    }

    /// Symbol.match implementation - find all matches
    pub fn symbol_match(&self, string: &str) -> JsResult<Vec<String>> {
        if self.flag_global {
            let matches: Vec<String> = self
                .regex
                .find_iter(string)
                .map(|m| m.as_str().to_string())
                .collect();
            Ok(matches)
        } else {
            // Non-global: return first match only
            match self.regex.find(string) {
                Some(m) => Ok(vec![m.as_str().to_string()]),
                None => Ok(vec![]),
            }
        }
    }

    /// Symbol.replace implementation - replace matches
    pub fn symbol_replace(&self, string: &str, replacement: &str) -> JsResult<String> {
        // Convert $1, $2 to ${1}, ${2} for Rust regex
        let rust_replacement = Self::convert_replacement(replacement);

        if self.flag_global {
            Ok(self.regex.replace_all(string, rust_replacement.as_str()).into_owned())
        } else {
            Ok(self.regex.replace(string, rust_replacement.as_str()).into_owned())
        }
    }

    fn convert_replacement(replacement: &str) -> String {
        let mut result = String::new();
        let mut chars = replacement.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        let mut num = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                num.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        result.push_str(&format!("${{{}}}", num));
                    } else if next == '$' {
                        result.push('$');
                        chars.next();
                    } else if next == '&' {
                        result.push_str("${0}");
                        chars.next();
                    } else if next == '`' {
                        // $` - portion before match (not easily supported, skip)
                        result.push_str("$`");
                        chars.next();
                    } else if next == '\'' {
                        // $' - portion after match (not easily supported, skip)
                        result.push_str("$'");
                        chars.next();
                    } else {
                        result.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Symbol.search implementation - find index of first match
    pub fn symbol_search(&self, string: &str) -> JsResult<Option<usize>> {
        Ok(self.regex.find(string).map(|m| m.start()))
    }

    /// Symbol.split implementation - split string by pattern
    pub fn symbol_split(&self, string: &str) -> JsResult<Vec<String>> {
        self.symbol_split_with_limit(string, usize::MAX)
    }

    /// Symbol.split with limit
    pub fn symbol_split_with_limit(&self, string: &str, limit: usize) -> JsResult<Vec<String>> {
        if limit == 0 {
            return Ok(vec![]);
        }

        let mut result = Vec::new();
        let mut last_end = 0;

        for caps in self.regex.captures_iter(string) {
            if result.len() >= limit {
                break;
            }

            let mat = caps.get(0).unwrap();

            // Add portion before match
            if result.len() < limit {
                result.push(string[last_end..mat.start()].to_string());
            }

            // Add captured groups
            for i in 1..caps.len() {
                if result.len() >= limit {
                    break;
                }
                if let Some(group) = caps.get(i) {
                    result.push(group.as_str().to_string());
                }
            }

            last_end = mat.end();
        }

        // Add remaining portion
        if result.len() < limit {
            result.push(string[last_end..].to_string());
        }

        // Trim to limit
        result.truncate(limit);

        Ok(result)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("/{}/{}", self.source_pattern, self.flags_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pattern() {
        let re = RegExpObject::new("abc", "").unwrap();
        assert_eq!(re.source(), "abc");
        assert_eq!(re.flags(), "");
    }

    #[test]
    fn test_flag_parsing() {
        let re = RegExpObject::new("test", "gi").unwrap();
        assert!(re.global());
        assert!(re.ignore_case());
        assert!(!re.multiline());
    }

    #[test]
    fn test_duplicate_flag_error() {
        let result = RegExpObject::new("test", "gg");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_pattern() {
        let result = RegExpObject::new("[invalid", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_string() {
        let re = RegExpObject::new("abc", "gi").unwrap();
        assert_eq!(re.to_string(), "/abc/gi");
    }
}
