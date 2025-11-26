# Test262 Compliance Testing

This document describes how to run Test262 conformance tests against the Corten JavaScript Runtime and tracks our current compliance status.

## Overview

[Test262](https://github.com/tc39/test262) is the official ECMAScript conformance test suite. It contains over 50,000 tests covering all aspects of the JavaScript language specification.

The Corten JavaScript Runtime uses the `test262_harness` component to run these tests and track compliance progress.

## Quick Start

### Prerequisites

1. **Clone Test262 repository** (if not already done):
   ```bash
   git clone --depth 1 https://github.com/tc39/test262.git test262
   ```

2. **Build the test runner**:
   ```bash
   cargo build --release --bin run_test262
   ```

### Running Tests

#### Using the Shell Script (Recommended)

The easiest way to run tests is using the provided shell script:

```bash
# Run parse-only tests on expressions (default)
./scripts/run-test262.sh

# Run parse + execute tests
./scripts/run-test262.sh --execute

# Run tests on a specific directory
./scripts/run-test262.sh test262/test/language/statements

# Run with a limit
./scripts/run-test262.sh --limit 100 test262/test/language/expressions

# Run specific feature tests
./scripts/run-test262.sh test262/test/language/expressions/addition
```

#### Using the Binary Directly

```bash
# Parse-only mode (faster, tests parsing correctness)
./target/release/run_test262 test262/test/language/expressions

# Parse + execute mode (slower, tests runtime behavior)
./target/release/run_test262 --execute test262/test/language/expressions

# Limit number of tests
./target/release/run_test262 --limit 100 test262/test/language/statements

# Get help
./target/release/run_test262 --help
```

## Test Modes

### Parse-Only Mode (Default)

Tests whether the parser can correctly parse JavaScript syntax. This mode:
- ✓ Fast (processes 1000+ tests per second)
- ✓ Good for testing parser correctness
- ✓ Detects syntax error handling
- ✗ Does not test runtime behavior

### Parse + Execute Mode

Tests both parsing and runtime execution. This mode:
- ✓ Tests complete JavaScript behavior
- ✓ Validates bytecode generation
- ✓ Tests runtime semantics
- ✗ Slower (~100-200 tests per second)
- ✗ May fail due to missing runtime features

## Test262 Harness Implementation

### $262 Global Object

The Test262 test suite requires a special `$262` global object with helper functions. Our implementation provides:

**Location**: `components/test262_harness/src/harness.rs` (HARNESS_PRELUDE constant)

**Available functions**:
- `$262.createRealm()` - Create isolated global environment (stub)
- `$262.evalScript(code)` - Evaluate code in current realm (stub)
- `$262.gc()` - Trigger garbage collection (no-op)
- `$262.detachArrayBuffer(buffer)` - Detach ArrayBuffer (stub)
- `$262.global` - Reference to global object
- `$262.agent.*` - Agent-related functions for concurrent tests (stubs)

**Native implementations**: `components/builtins/src/test262.rs`

Provides Rust-native implementations for future integration:
- `Test262Object::create_realm()` - Native realm creation
- `Test262Object::eval_script()` - Native eval integration
- `Test262Object::gc()` - GC trigger integration
- `Assert::assert()`, `Assert::same_value()`, `Assert::not_same_value()`, `Assert::throws()` - Native assertion functions

### Test262Error Constructor

A custom error type used by Test262 tests for assertion failures. Defined in HARNESS_PRELUDE and automatically available to all tests.

### Assert Functions

Assertion helpers provided to tests:
- `assert(condition, message)` - Throw error if condition is false
- `assert.sameValue(actual, expected, message)` - Strict equality check (===)
- `assert.notSameValue(actual, unexpected, message)` - Strict inequality check (!==)
- `assert.throws(ErrorType, fn, message)` - Expect exception from function

### Harness Injection

The harness prelude is automatically executed before each test in execute mode:

1. Test262 harness creates fresh VM
2. HARNESS_PRELUDE is parsed and executed
3. Test code is executed with $262 and assert functions available
4. Test results are collected

This ensures all tests have access to the required Test262-specific globals without modifying the core runtime.

## Current Baseline Results

Results captured on: November 26, 2025
Test262 commit: Latest (shallow clone)
Corten Runtime version: 0.1.0

### Expressions (Parse-Only)

**Addition Expressions** (`test262/test/language/expressions/addition`):
- Total: 48 tests
- Passed: 37 (77.1%)
- Failed: 3 (6.2%)
- Skipped: 8 (16.7%)
- **Status**: Good progress, some work remaining

**Sample of 100 Expression Tests**:
- Total: 100 tests
- Passed: 26 (26.0%)
- Failed: 28 (28.0%)
- Skipped: 46 (46.0%)
- **Status**: Low pass rate, major implementation work required

### Statements (Parse-Only)

**Sample of 100 Statement Tests**:
- Total: 100 tests
- Passed: 66 (66.0%)
- Failed: 28 (28.0%)
- Skipped: 6 (6.0%)
- **Status**: Moderate compliance, significant work needed

### Execution Mode

**Addition Expressions (Parse + Execute)**:
- Total: 48 tests
- Passed: 0 (0.0%)
- Failed: 40 (83.3%)
- Skipped: 8 (16.7%)
- **Status**: Runtime implementation in early stages

## Common Failure Patterns

Based on baseline testing, the most common failures are:

### Parse Failures

1. **BigInt Literal Parsing**
   - Error: `Expected RParen, got Identifier`
   - Example: `test262/test/language/expressions/addition/bigint-arithmetic.js`
   - Cause: BigInt literals (`123n`) not fully supported in parser

2. **Complex Expression Parsing**
   - Error: `Expected semicolon`
   - Various expression tests
   - Cause: Parser strictness or edge cases in expression handling

3. **Negative Test Failures**
   - Error: `Expected parse error but parsed successfully`
   - Cause: Parser too lenient, accepting invalid syntax

### Runtime Failures (Execute Mode)

1. **Missing Built-in Functions**
   - Error: `Undefined is not a function`
   - Cause: Test262 harness functions not fully implemented

2. **Type Errors**
   - Error: `Undefined is not a constructor`
   - Cause: Missing constructor support for built-in types

3. **Bytecode Generation**
   - Error: `Unsupported AST node`
   - Cause: Some syntax constructs not yet supported in bytecode generator

## Known Limitations

### Unsupported Features

The following ES2024+ features are not yet supported and tests requiring them are skipped:

- **Regular Expression Features**:
  - Unicode property escapes
  - Match indices
  - Named groups
  - Lookbehind assertions
  - Dotall flag

- **Class Features**:
  - Private fields and methods
  - Private static fields and methods
  - Decorators

- **Module Features**:
  - Import assertions
  - Import attributes
  - JSON modules
  - Top-level await

- **Advanced Features**:
  - ShadowRealm
  - Temporal API
  - Resizable ArrayBuffer
  - Array find-from-last
  - Iterator helpers
  - Explicit resource management
  - Float16Array
  - Set methods
  - Uint8Array base64
  - Promise.try
  - RegExp.escape

### Module Tests

Module tests are currently skipped as they require:
- Module resolution system
- Import/export statement support
- Separate compilation context

## Test Organization

Test262 tests are organized by language feature:

```
test262/test/language/
├── expressions/          # Expression tests (11,093 tests)
│   ├── addition/        # Addition operator
│   ├── assignment/      # Assignment expressions
│   ├── call/            # Function calls
│   └── ...
├── statements/          # Statement tests (9,236 tests)
│   ├── for/             # For loops
│   ├── if/              # If statements
│   ├── while/           # While loops
│   └── ...
├── literals/            # Literal values
├── types/               # Type system tests
└── ...
```

## Interpreting Results

### Pass Rates

- **90-100%**: Excellent - Feature fully implemented
- **75-89%**: Good - Minor issues or edge cases remaining
- **50-74%**: Moderate - Significant work needed
- **25-49%**: Low - Major implementation gaps
- **0-24%**: Very Low - Feature in early development

### Skip Rates

High skip rates (>30%) indicate:
- Tests depend on unsupported features
- May need to implement dependencies first
- Not necessarily a problem for current development stage

### Failure Types

- **Parse errors**: Parser needs improvement
- **Expected parse error**: Parser too lenient (security concern)
- **Runtime errors**: Interpreter/VM needs work
- **Bytecode generation errors**: Codegen incomplete

## Improving Compliance

### Workflow

1. **Run baseline tests** to identify failure patterns
2. **Group failures** by error type
3. **Prioritize** based on:
   - Number of affected tests
   - Importance to core functionality
   - Dependencies on other features
4. **Fix and re-test** specific test categories
5. **Track progress** by re-running baseline periodically

### Example: Fixing BigInt Support

```bash
# 1. Identify BigInt failures
./target/release/run_test262 test262/test/language/expressions/addition | grep -i bigint

# 2. Fix parser to support BigInt literals

# 3. Re-test
./target/release/run_test262 test262/test/language/expressions/addition

# 4. Check improvement
# Before: 77.1% pass rate
# After: (expected improvement)
```

## Continuous Testing

### Integration with CI/CD

The Test262 runner can be integrated into continuous integration:

```bash
# Run smoke test (100 tests from each category)
./scripts/run-test262.sh --limit 100 test262/test/language/expressions
./scripts/run-test262.sh --limit 100 test262/test/language/statements

# Exit code 1 if any failures (for CI failure detection)
```

### Regression Testing

Track pass rates over time to detect regressions:

```bash
# Run full suite and save results
./target/release/run_test262 test262/test/language > baseline-$(date +%Y%m%d).txt

# Compare against previous baseline
diff baseline-20251125.txt baseline-20251126.txt
```

## Contributing

When implementing new features:

1. **Run relevant Test262 tests** before and after
2. **Document pass rate improvements** in commit messages
3. **Add new supported features** to the harness feature list
4. **Update this document** with new baseline results

## Resources

- [Test262 Repository](https://github.com/tc39/test262)
- [ECMAScript Specification](https://tc39.es/ecma262/)
- [Test262 Documentation](https://github.com/tc39/test262/blob/main/INTERPRETING.md)
- [Corten test262_harness Component](/components/test262_harness/CLAUDE.md)

## Appendix: Full Test Suite Statistics

Total Test262 tests: ~54,000 files
- Language tests: ~20,000
- Built-in object tests: ~25,000
- Annex B tests: ~5,000
- Intl tests: ~4,000

**Note**: Running the complete suite takes significant time. Start with focused subsets and gradually expand coverage as implementation matures.
