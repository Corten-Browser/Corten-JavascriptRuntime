//! Source map support
//!
//! Provides source map parsing and mapping for debugging transpiled
//! or minified JavaScript code back to original source.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source map v3 format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMap {
    pub version: u32,
    pub file: Option<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Option<Vec<String>>,
    pub names: Vec<String>,
    pub mappings: String,

    #[serde(skip)]
    decoded_mappings: Vec<SourceMapping>,
}

/// Individual source mapping entry
#[derive(Debug, Clone, Default)]
pub struct SourceMapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub source_index: Option<u32>,
    pub original_line: Option<u32>,
    pub original_column: Option<u32>,
    pub name_index: Option<u32>,
}

/// Original position in source file
#[derive(Debug, Clone, PartialEq)]
pub struct OriginalPosition {
    pub source: String,
    pub line: u32,
    pub column: u32,
    pub name: Option<String>,
}

/// Generated position in output file
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedPosition {
    pub line: u32,
    pub column: u32,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self {
            version: 3,
            file: None,
            source_root: None,
            sources: vec![],
            sources_content: None,
            names: vec![],
            mappings: String::new(),
            decoded_mappings: vec![],
        }
    }

    /// Parse source map from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        let mut map: SourceMap =
            serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
        map.decode_mappings()?;
        Ok(map)
    }

    /// Convert source map to JSON
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Decode VLQ mappings
    fn decode_mappings(&mut self) -> Result<(), String> {
        self.decoded_mappings.clear();

        let mut gen_line = 0u32;
        let mut gen_col = 0u32;
        let mut src_idx = 0i32;
        let mut orig_line = 0i32;
        let mut orig_col = 0i32;
        let mut name_idx = 0i32;

        for line in self.mappings.split(';') {
            gen_col = 0;

            for segment in line.split(',') {
                if segment.is_empty() {
                    continue;
                }

                let values = Self::decode_vlq(segment)?;
                if values.is_empty() {
                    continue;
                }

                gen_col = (gen_col as i32 + values[0]) as u32;

                let mapping = if values.len() >= 4 {
                    src_idx += values[1];
                    orig_line += values[2];
                    orig_col += values[3];

                    let name = if values.len() >= 5 {
                        name_idx += values[4];
                        Some(name_idx as u32)
                    } else {
                        None
                    };

                    SourceMapping {
                        generated_line: gen_line,
                        generated_column: gen_col,
                        source_index: Some(src_idx as u32),
                        original_line: Some(orig_line as u32),
                        original_column: Some(orig_col as u32),
                        name_index: name,
                    }
                } else {
                    SourceMapping {
                        generated_line: gen_line,
                        generated_column: gen_col,
                        ..Default::default()
                    }
                };

                self.decoded_mappings.push(mapping);
            }

            gen_line += 1;
        }

        Ok(())
    }

    /// Decode VLQ string to values
    fn decode_vlq(encoded: &str) -> Result<Vec<i32>, String> {
        const BASE64_CHARS: &str =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut values = vec![];
        let mut shift = 0;
        let mut value = 0;

        for c in encoded.chars() {
            let digit = BASE64_CHARS
                .find(c)
                .ok_or_else(|| format!("Invalid VLQ character: {}", c))?
                as i32;

            let has_continuation = digit & 32 != 0;
            let digit_value = digit & 31;

            value += digit_value << shift;

            if has_continuation {
                shift += 5;
            } else {
                // Convert to signed
                let is_negative = value & 1 != 0;
                value >>= 1;
                if is_negative {
                    value = -value;
                }
                values.push(value);
                shift = 0;
                value = 0;
            }
        }

        Ok(values)
    }

    /// Encode values to VLQ string
    pub fn encode_vlq(values: &[i32]) -> String {
        const BASE64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::new();

        for &value in values {
            // Convert to VLQ signed representation
            let mut vlq = if value < 0 {
                ((-value) << 1) | 1
            } else {
                value << 1
            };

            // Encode in base64
            loop {
                let mut digit = vlq & 31;
                vlq >>= 5;
                if vlq > 0 {
                    digit |= 32; // Set continuation bit
                }
                result.push(BASE64_CHARS[digit as usize] as char);
                if vlq == 0 {
                    break;
                }
            }
        }

        result
    }

    /// Map generated position to original
    pub fn original_position_for(&self, line: u32, column: u32) -> Option<(String, u32, u32)> {
        // Find the mapping for this generated position
        let mapping = self
            .decoded_mappings
            .iter()
            .filter(|m| m.generated_line == line && m.generated_column <= column)
            .last()?;

        let source_index = mapping.source_index? as usize;
        let source = self.sources.get(source_index)?.clone();
        let orig_line = mapping.original_line?;
        let orig_col = mapping.original_column?;

        Some((source, orig_line, orig_col))
    }

    /// Get original position with full details
    pub fn original_position_for_detailed(
        &self,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let mapping = self
            .decoded_mappings
            .iter()
            .filter(|m| m.generated_line == line && m.generated_column <= column)
            .last()?;

        let source_index = mapping.source_index? as usize;
        let source = self.sources.get(source_index)?.clone();
        let orig_line = mapping.original_line?;
        let orig_col = mapping.original_column?;

        let name = mapping.name_index.and_then(|idx| {
            self.names.get(idx as usize).cloned()
        });

        Some(OriginalPosition {
            source,
            line: orig_line,
            column: orig_col,
            name,
        })
    }

    /// Get generated position from original position
    pub fn generated_position_for(
        &self,
        source: &str,
        line: u32,
        column: u32,
    ) -> Option<GeneratedPosition> {
        let source_index = self.sources.iter().position(|s| s == source)? as u32;

        let mapping = self.decoded_mappings.iter().find(|m| {
            m.source_index == Some(source_index)
                && m.original_line == Some(line)
                && m.original_column.map(|c| c <= column).unwrap_or(false)
        })?;

        Some(GeneratedPosition {
            line: mapping.generated_line,
            column: mapping.generated_column,
        })
    }

    /// Add a mapping
    pub fn add_mapping(&mut self, mapping: SourceMapping) {
        self.decoded_mappings.push(mapping);
    }

    /// Get number of mappings
    pub fn mappings_count(&self) -> usize {
        self.decoded_mappings.len()
    }

    /// Get all mappings
    pub fn get_mappings(&self) -> &[SourceMapping] {
        &self.decoded_mappings
    }

    /// Add a source file
    pub fn add_source(&mut self, source: String) -> u32 {
        let index = self.sources.len() as u32;
        self.sources.push(source);
        index
    }

    /// Add a name
    pub fn add_name(&mut self, name: String) -> u32 {
        let index = self.names.len() as u32;
        self.names.push(name);
        index
    }

    /// Regenerate mappings string from decoded mappings
    pub fn regenerate_mappings(&mut self) {
        let mut lines: HashMap<u32, Vec<&SourceMapping>> = HashMap::new();

        for mapping in &self.decoded_mappings {
            lines
                .entry(mapping.generated_line)
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        let max_line = lines.keys().max().copied().unwrap_or(0);
        let mut result = Vec::new();

        let mut prev_src_idx = 0i32;
        let mut prev_orig_line = 0i32;
        let mut prev_orig_col = 0i32;
        let mut prev_name_idx = 0i32;

        for line_num in 0..=max_line {
            let mut segments = Vec::new();

            if let Some(mappings) = lines.get(&line_num) {
                let mut sorted_mappings = mappings.clone();
                sorted_mappings.sort_by_key(|m| m.generated_column);

                let mut prev_gen_col = 0i32;

                for mapping in sorted_mappings {
                    let mut values = vec![(mapping.generated_column as i32) - prev_gen_col];
                    prev_gen_col = mapping.generated_column as i32;

                    if let Some(src_idx) = mapping.source_index {
                        let src_delta = (src_idx as i32) - prev_src_idx;
                        prev_src_idx = src_idx as i32;

                        let orig_line = mapping.original_line.unwrap_or(0) as i32;
                        let line_delta = orig_line - prev_orig_line;
                        prev_orig_line = orig_line;

                        let orig_col = mapping.original_column.unwrap_or(0) as i32;
                        let col_delta = orig_col - prev_orig_col;
                        prev_orig_col = orig_col;

                        values.push(src_delta);
                        values.push(line_delta);
                        values.push(col_delta);

                        if let Some(name_idx) = mapping.name_index {
                            let name_delta = (name_idx as i32) - prev_name_idx;
                            prev_name_idx = name_idx as i32;
                            values.push(name_delta);
                        }
                    }

                    segments.push(Self::encode_vlq(&values));
                }
            }

            result.push(segments.join(","));
        }

        self.mappings = result.join(";");
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}
