use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Test262 negative test expectation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NegativeExpectation {
    /// Phase where error is expected: "parse", "resolution", or "runtime"
    pub phase: String,
    /// Error type expected (e.g., "SyntaxError", "ReferenceError")
    #[serde(rename = "type")]
    pub error_type: String,
}

/// Test262 test metadata parsed from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TestMetadata {
    /// Human-readable description of what the test verifies
    pub description: String,
    /// Additional information about the test
    pub info: Option<String>,
    /// Expected error for negative tests
    pub negative: Option<NegativeExpectation>,
    /// Helper files that must be loaded before the test
    pub includes: Vec<String>,
    /// Test execution flags (e.g., "onlyStrict", "noStrict", "module", "raw", "async")
    pub flags: HashSet<String>,
    /// ECMAScript features required by this test
    pub features: Vec<String>,
    /// ES5.1 section identifier
    pub es5id: Option<String>,
    /// ES6 section identifier
    pub es6id: Option<String>,
    /// ES section identifier
    pub esid: Option<String>,
    /// Author of the test
    pub author: Option<String>,
}

impl Default for TestMetadata {
    fn default() -> Self {
        Self {
            description: String::new(),
            info: None,
            negative: None,
            includes: Vec::new(),
            flags: HashSet::new(),
            features: Vec::new(),
            es5id: None,
            es6id: None,
            esid: None,
            author: None,
        }
    }
}

impl TestMetadata {
    /// Parse YAML frontmatter from test file source
    ///
    /// Test262 files contain metadata in a YAML block between `/*---` and `---*/`
    pub fn parse(source: &str) -> Result<Self, String> {
        let re = Regex::new(r"(?s)/\*---\n(.+?)\n---\*/")
            .map_err(|e| format!("Failed to compile regex: {}", e))?;

        let yaml = re
            .captures(source)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .ok_or("No YAML frontmatter found in test file")?;

        serde_yaml::from_str(yaml)
            .map_err(|e| format!("Failed to parse YAML frontmatter: {}", e))
    }

    /// Check if test should be skipped due to unsupported features
    pub fn should_skip(&self, supported_features: &HashSet<String>) -> bool {
        self.features
            .iter()
            .any(|f| !supported_features.contains(f))
    }

    /// Get list of unsupported features required by this test
    pub fn unsupported_features(&self, supported_features: &HashSet<String>) -> Vec<String> {
        self.features
            .iter()
            .filter(|f| !supported_features.contains(*f))
            .cloned()
            .collect()
    }

    /// Check if test expects a parse-phase error
    pub fn expects_parse_error(&self) -> bool {
        self.negative
            .as_ref()
            .map(|n| n.phase == "parse")
            .unwrap_or(false)
    }

    /// Check if test expects a runtime-phase error
    pub fn expects_runtime_error(&self) -> bool {
        self.negative
            .as_ref()
            .map(|n| n.phase == "runtime")
            .unwrap_or(false)
    }

    /// Check if test expects a resolution-phase error (ES modules)
    pub fn expects_resolution_error(&self) -> bool {
        self.negative
            .as_ref()
            .map(|n| n.phase == "resolution")
            .unwrap_or(false)
    }

    /// Get the expected error type for negative tests
    pub fn expected_error_type(&self) -> Option<&str> {
        self.negative.as_ref().map(|n| n.error_type.as_str())
    }

    /// Check if test requires strict mode only
    pub fn is_strict_only(&self) -> bool {
        self.flags.contains("onlyStrict")
    }

    /// Check if test requires non-strict mode only
    pub fn is_no_strict(&self) -> bool {
        self.flags.contains("noStrict")
    }

    /// Check if test is an ES module test
    pub fn is_module(&self) -> bool {
        self.flags.contains("module")
    }

    /// Check if test is asynchronous
    pub fn is_async(&self) -> bool {
        self.flags.contains("async")
    }

    /// Check if test should use raw interpretation (no harness setup)
    pub fn is_raw(&self) -> bool {
        self.flags.contains("raw")
    }
}

/// Test262 test file with source and parsed metadata
#[derive(Debug, Clone)]
pub struct TestFile {
    /// Path to the test file
    pub path: String,
    /// Source code of the test
    pub source: String,
    /// Parsed metadata from YAML frontmatter
    pub metadata: TestMetadata,
}

impl TestFile {
    /// Load a test file from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or("Invalid path encoding")?
            .to_string();

        let source =
            std::fs::read_to_string(&path_str).map_err(|e| format!("Failed to read file: {}", e))?;

        let metadata = TestMetadata::parse(&source)?;

        Ok(Self {
            path: path_str,
            source,
            metadata,
        })
    }

    /// Get the test name (file name without extension)
    pub fn name(&self) -> &str {
        Path::new(&self.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.path)
    }

    /// Get the test code (source without metadata block)
    pub fn code(&self) -> String {
        let re = Regex::new(r"(?s)/\*---\n.+?\n---\*/\s*").unwrap();
        re.replace(&self.source, "").to_string()
    }
}
