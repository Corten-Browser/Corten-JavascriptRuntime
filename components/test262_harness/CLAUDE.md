# test262_harness Component

**Type**: Test Infrastructure (Level 3)
**Tech Stack**: Rust, serde_yaml, walkdir, regex
**Version**: 0.1.0

## Purpose
Test262 conformance test harness for running the official ECMAScript test suite against the Corten JavaScript runtime. Parses test metadata, manages feature flags, executes tests, and generates conformance reports.

## Dependencies
- `parser`: JavaScript parser for syntax tests
- `interpreter`: VM execution for runtime tests
- `builtins`: Standard library for feature support

## Token Budget
- Optimal: 50,000 tokens
- Warning: 70,000 tokens
- Critical: 90,000 tokens

## Exported Types

```rust
// Test metadata parsing
pub struct TestMetadata {
    pub description: String,
    pub info: Option<String>,
    pub negative: Option<NegativeExpectation>,
    pub includes: Vec<String>,
    pub flags: HashSet<String>,
    pub features: Vec<String>,
}

pub struct NegativeExpectation {
    pub phase: String,  // "parse", "resolution", "runtime"
    pub r#type: String, // Error type expected
}

pub struct TestFile {
    pub path: String,
    pub source: String,
    pub metadata: TestMetadata,
}

// Test execution
pub enum TestResult {
    Pass,
    Fail(String),
    Skip(String),
    Timeout,
}

pub struct Test262Harness {
    supported_features: HashSet<String>,
    timeout_ms: u64,
    results: HashMap<String, TestResult>,
}

// Reporting
pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub timeout: usize,
    pub failures: Vec<(String, String)>,
}
```

## Key Implementation Requirements

### Test Metadata Parsing
1. Extract YAML frontmatter from test files
2. Parse negative expectations (parse/runtime errors)
3. Extract feature requirements
4. Handle includes and flags

### Feature Management
- Track supported ES2024 features
- Skip tests requiring unsupported features
- Report feature coverage gaps

### Test Execution
- Parse phase: Check syntax errors
- Runtime phase: Execute and verify behavior
- Negative tests: Expect specific errors
- Timeout handling for infinite loops

### Report Generation
- Pass/fail/skip/timeout counts
- Pass rate calculation
- Failure details with paths
- Feature coverage summary

## Mandatory Requirements

### 1. Test-Driven Development
- Test metadata parsing thoroughly
- Test report calculations
- 80%+ coverage
- TDD pattern in commits

### 2. File Structure
```
src/
  lib.rs              # Public exports
  test_file.rs        # Test metadata parsing
  harness.rs          # Test runner
  report.rs           # Report generation
tests/
  unit/
  integration/
```

## Git Commit Format
```
[test262_harness] <type>: <description>
```

## Definition of Done
- [ ] YAML frontmatter parsing complete
- [ ] Feature flag management working
- [ ] Negative test expectations handled
- [ ] Report generation accurate
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Contract tests passing
