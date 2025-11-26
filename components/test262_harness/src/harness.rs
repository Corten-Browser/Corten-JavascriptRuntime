use crate::report::TestReport;
use crate::test_file::TestFile;
use core_types::{JsError, ErrorKind, Value};
use interpreter::VM;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// Result of running a single test
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed successfully
    Pass,
    /// Test failed with reason
    Fail(String),
    /// Test was skipped with reason
    Skip(String),
    /// Test timed out
    Timeout,
}

/// Test262 harness prelude that provides assert functions
pub const HARNESS_PRELUDE: &str = r#"
// Test262Error constructor
function Test262Error(message) {
    this.message = message || '';
}
Test262Error.prototype = new Error();
Test262Error.prototype.constructor = Test262Error;
Test262Error.prototype.toString = function() {
    return 'Test262Error: ' + this.message;
};

// Test262 $262 object and assert functions
var $262 = {
    createRealm: function() { return {}; },
    detachArrayBuffer: function(ab) { },
    evalScript: function(code) { return eval(code); },
    gc: function() { },
    global: this,
    IsHTMLDDA: { toString: function() { return ''; } },
    agent: {
        start: function() {},
        broadcast: function() {},
        getReport: function() { return null; },
        sleep: function() {},
        monotonicNow: function() { return 0; }
    }
};

function assert(condition, message) {
    if (!condition) {
        throw new Error("Assertion failed: " + (message || ""));
    }
}

assert.sameValue = function(actual, expected, message) {
    if (actual !== expected) {
        throw new Error("Expected " + expected + " but got " + actual + (message ? ": " + message : ""));
    }
};

assert.notSameValue = function(actual, unexpected, message) {
    if (actual === unexpected) {
        throw new Error("Value should not be " + unexpected + (message ? ": " + message : ""));
    }
};

assert.throws = function(expectedErrorType, fn, message) {
    var thrown = false;
    try {
        fn();
    } catch (e) {
        thrown = true;
        if (expectedErrorType && !(e instanceof expectedErrorType)) {
            throw new Error("Expected " + expectedErrorType.name + " but got " + e.name);
        }
    }
    if (!thrown) {
        throw new Error("Expected exception but none was thrown" + (message ? ": " + message : ""));
    }
};

// print function for compatibility
if (typeof print === 'undefined') {
    var print = function() {};
}
"#;

impl TestResult {
    /// Check if the result is a pass
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    /// Check if the result is a failure
    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail(_))
    }

    /// Check if the result is a skip
    pub fn is_skip(&self) -> bool {
        matches!(self, TestResult::Skip(_))
    }

    /// Check if the result is a timeout
    pub fn is_timeout(&self) -> bool {
        matches!(self, TestResult::Timeout)
    }
}

/// Test262 conformance test harness
pub struct Test262Harness {
    /// Set of ES2024 features supported by the runtime
    supported_features: HashSet<String>,
    /// Timeout in milliseconds for each test
    timeout_ms: u64,
    /// Results of executed tests
    results: HashMap<String, TestResult>,
    /// Whether to execute tests (vs just parse)
    execute_tests: bool,
}

impl Test262Harness {
    /// Create a new Test262 harness with default supported features
    pub fn new() -> Self {
        let mut features = HashSet::new();

        // Core ES2024 features we support
        features.insert("Symbol".to_string());
        features.insert("Symbol.species".to_string());
        features.insert("Symbol.iterator".to_string());
        features.insert("Symbol.toStringTag".to_string());
        features.insert("Map".to_string());
        features.insert("Set".to_string());
        features.insert("WeakMap".to_string());
        features.insert("WeakSet".to_string());
        features.insert("BigInt".to_string());
        features.insert("Proxy".to_string());
        features.insert("Reflect".to_string());
        features.insert("Promise".to_string());
        features.insert("TypedArray".to_string());
        features.insert("ArrayBuffer".to_string());
        features.insert("DataView".to_string());
        features.insert("generators".to_string());
        features.insert("async-functions".to_string());
        features.insert("WeakRef".to_string());
        features.insert("FinalizationRegistry".to_string());
        features.insert("arrow-function".to_string());
        features.insert("let".to_string());
        features.insert("const".to_string());
        features.insert("class".to_string());
        features.insert("template".to_string());
        features.insert("destructuring-binding".to_string());
        features.insert("destructuring-assignment".to_string());
        features.insert("default-parameters".to_string());
        features.insert("rest-parameters".to_string());
        features.insert("spread".to_string());
        features.insert("object-spread".to_string());
        features.insert("for-of".to_string());
        features.insert("computed-property-names".to_string());
        features.insert("Atomics".to_string());
        features.insert("SharedArrayBuffer".to_string());

        Self {
            supported_features: features,
            timeout_ms: 10000,
            results: HashMap::new(),
            execute_tests: true,
        }
    }

    /// Create a harness with custom features
    pub fn with_features(features: HashSet<String>) -> Self {
        Self {
            supported_features: features,
            timeout_ms: 10000,
            results: HashMap::new(),
            execute_tests: true,
        }
    }

    /// Enable or disable test execution (vs parse-only)
    pub fn set_execute(&mut self, execute: bool) {
        self.execute_tests = execute;
    }

    /// Check if test execution is enabled
    pub fn is_execute_enabled(&self) -> bool {
        self.execute_tests
    }

    /// Add a supported feature
    pub fn add_feature(&mut self, feature: &str) {
        self.supported_features.insert(feature.to_string());
    }

    /// Remove a supported feature
    pub fn remove_feature(&mut self, feature: &str) {
        self.supported_features.remove(feature);
    }

    /// Set the timeout for test execution
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }

    /// Get the current timeout setting
    pub fn timeout(&self) -> u64 {
        self.timeout_ms
    }

    /// Get the set of supported features
    pub fn supported_features(&self) -> &HashSet<String> {
        &self.supported_features
    }

    /// Run a single test
    pub fn run_test(&mut self, test: &TestFile) -> TestResult {
        // Check if we should skip due to missing features
        if test.metadata.should_skip(&self.supported_features) {
            let missing = test.metadata.unsupported_features(&self.supported_features);
            return TestResult::Skip(format!("Missing features: {:?}", missing));
        }

        // Try to parse the test source
        let parse_result = parser::Parser::new(&test.source).parse();

        if test.metadata.expects_parse_error() {
            // Negative parse test: expect parsing to fail
            match parse_result {
                Err(e) => {
                    // Check if error type matches expected
                    if let Some(expected_type) = test.metadata.expected_error_type() {
                        let error_str = format!("{:?}", e);
                        if error_str.contains(expected_type)
                            || self.is_matching_error_type(&error_str, expected_type)
                        {
                            TestResult::Pass
                        } else {
                            TestResult::Fail(format!(
                                "Expected {} but got: {}",
                                expected_type, error_str
                            ))
                        }
                    } else {
                        TestResult::Pass
                    }
                }
                Ok(_) => {
                    TestResult::Fail("Expected parse error but parsed successfully".to_string())
                }
            }
        } else {
            // Positive test: expect parsing to succeed
            match parse_result {
                Err(e) => TestResult::Fail(format!("Parse error: {:?}", e)),
                Ok(ast) => {
                    // If execution is disabled, pass on successful parse
                    if !self.execute_tests {
                        return TestResult::Pass;
                    }

                    // Generate bytecode from AST
                    let mut generator = parser::BytecodeGenerator::new();
                    let bytecode = match generator.generate(&ast) {
                        Ok(bc) => bc,
                        Err(e) => return TestResult::Fail(format!("Bytecode generation error: {:?}", e)),
                    };

                    // Create a fresh VM for each test
                    let mut vm = VM::new();

                    // Execute harness prelude to set up $262 and assert functions
                    if let Err(e) = Self::setup_harness_prelude(&mut vm) {
                        return TestResult::Fail(format!("Harness setup failed: {:?}", e));
                    }

                    // Register nested functions
                    let nested_functions = generator.take_nested_functions();
                    for func_bytecode in nested_functions {
                        vm.register_function(func_bytecode);
                    }

                    // Execute the bytecode
                    let exec_result = vm.execute(&bytecode);

                    if test.metadata.expects_runtime_error() {
                        // Expect runtime error
                        match exec_result {
                            Err(e) => {
                                // Check if error type matches expected
                                if let Some(expected_type) = test.metadata.expected_error_type() {
                                    let error_str = format!("{:?}", e.kind);
                                    if self.is_matching_error_type(&error_str, expected_type) {
                                        TestResult::Pass
                                    } else {
                                        TestResult::Fail(format!(
                                            "Expected {} but got: {:?}",
                                            expected_type, e
                                        ))
                                    }
                                } else {
                                    TestResult::Pass
                                }
                            }
                            Ok(_) => {
                                TestResult::Fail("Expected runtime error but execution succeeded".to_string())
                            }
                        }
                    } else if test.metadata.expects_resolution_error() {
                        // Would need to resolve modules and verify error
                        // For now, skip module resolution tests
                        TestResult::Skip("Module resolution not implemented".to_string())
                    } else {
                        // Expect success
                        match exec_result {
                            Ok(_) => TestResult::Pass,
                            Err(e) => TestResult::Fail(format!("Runtime error: {:?}", e)),
                        }
                    }
                }
            }
        }
    }

    /// Check if an error message matches the expected error type
    fn is_matching_error_type(&self, error_msg: &str, expected: &str) -> bool {
        match expected {
            "SyntaxError" => {
                error_msg.contains("Syntax")
                    || error_msg.contains("syntax")
                    || error_msg.contains("Unexpected")
                    || error_msg.contains("Invalid")
            }
            "ReferenceError" => error_msg.contains("Reference") || error_msg.contains("undefined"),
            "TypeError" => error_msg.contains("Type") || error_msg.contains("not a function"),
            "RangeError" => error_msg.contains("Range") || error_msg.contains("out of range"),
            _ => error_msg.contains(expected),
        }
    }

    /// Run all tests in a directory
    pub fn run_directory<P: AsRef<Path>>(&mut self, dir: P) -> TestReport {
        let mut report = TestReport::new();

        let walker = WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(
            |e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "js")
                    .unwrap_or(false)
            },
        );

        for entry in walker {
            let path = entry.path().to_string_lossy().to_string();
            match TestFile::load(&path) {
                Ok(test) => {
                    let result = self.run_test(&test);
                    report.add_result(&path, result.clone());
                    self.results.insert(path, result);
                }
                Err(e) => {
                    let result = TestResult::Skip(format!("Could not load test: {}", e));
                    report.add_result(&path, result.clone());
                    self.results.insert(path, result);
                }
            }
        }

        report
    }

    /// Get all test results
    pub fn results(&self) -> &HashMap<String, TestResult> {
        &self.results
    }

    /// Clear all test results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Get number of tests run
    pub fn test_count(&self) -> usize {
        self.results.len()
    }

    /// Get number of passing tests
    pub fn pass_count(&self) -> usize {
        self.results.values().filter(|r| r.is_pass()).count()
    }

    /// Get number of failing tests
    pub fn fail_count(&self) -> usize {
        self.results.values().filter(|r| r.is_fail()).count()
    }

    /// Get number of skipped tests
    pub fn skip_count(&self) -> usize {
        self.results.values().filter(|r| r.is_skip()).count()
    }

    /// Set up the test262 harness prelude in the VM
    /// This creates the $262 object and assert functions in the global scope
    fn setup_harness_prelude(vm: &mut VM) -> Result<(), JsError> {
        // Parse the harness prelude
        let ast = parser::Parser::new(HARNESS_PRELUDE)
            .parse()
            .map_err(|e| JsError {
                kind: ErrorKind::SyntaxError,
                message: format!("Failed to parse harness prelude: {:?}", e),
                stack: vec![],
                source_position: None,
            })?;

        // Generate bytecode
        let mut generator = parser::BytecodeGenerator::new();
        let bytecode = generator.generate(&ast)
            .map_err(|e| JsError {
                kind: ErrorKind::InternalError,
                message: format!("Failed to generate bytecode for harness prelude: {:?}", e),
                stack: vec![],
                source_position: None,
            })?;

        // Register any nested functions from the prelude
        let nested_functions = generator.take_nested_functions();
        for func_bytecode in nested_functions {
            vm.register_function(func_bytecode);
        }

        // Execute the prelude to set up global functions
        vm.execute(&bytecode)?;

        Ok(())
    }
}

impl Default for Test262Harness {
    fn default() -> Self {
        Self::new()
    }
}
