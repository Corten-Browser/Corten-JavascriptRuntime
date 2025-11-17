# Test262 Harness

**Type**: Test Infrastructure (Level 3)
**Tech Stack**: Rust, serde_yaml, walkdir, regex
**Version**: 0.1.0

## Purpose

Test262 conformance test harness for validating ECMAScript 2024 compliance of the Corten JavaScript runtime. Test262 is the official ECMAScript conformance test suite with 35,000+ tests.

## Features

- Parse Test262 YAML frontmatter metadata
- Feature flag management for test filtering
- Support for negative tests (expected parse/runtime errors)
- Directory traversal for batch test execution
- Comprehensive report generation with pass rates

## Structure

```
├── src/
│   ├── lib.rs          # Public exports
│   ├── test_file.rs    # Test file and metadata parsing
│   ├── harness.rs      # Main test runner
│   └── report.rs       # Report generation
├── tests/
│   ├── unit/           # Unit tests
│   └── integration/    # Integration tests
├── Cargo.toml          # Dependencies
├── CLAUDE.md           # Component instructions
└── README.md           # This file
```

## Usage

```rust
use test262_harness::{Test262Harness, TestFile};

// Create harness with supported features
let mut harness = Test262Harness::new();

// Add additional features as they're implemented
harness.add_feature("optional-chaining");

// Run single test
let test = TestFile::load("test262/test/language/expressions/addition.js")?;
let result = harness.run_test(&test);

// Run all tests in directory
let report = harness.run_directory("test262/test/language/");
println!("{}", report.summary());
```

## Test262 Metadata Format

Test262 tests include YAML frontmatter:

```javascript
/*---
description: Addition operator
info: |
  The addition operator either performs string concatenation
  or numeric addition.
negative:
  phase: parse
  type: SyntaxError
includes: [propertyHelper.js]
flags: [onlyStrict]
features: [BigInt, Symbol]
---*/
```

## Supported Features

The harness tracks which ES2024 features are implemented:
- Symbol, Map, Set, WeakMap, WeakSet
- BigInt, Proxy, Reflect
- Promise, generators, async-functions
- TypedArray, ArrayBuffer, DataView
- WeakRef, FinalizationRegistry

## Report Output

```
Test262 Results:
Total: 1000
Passed: 850 (85.0%)
Failed: 100
Skipped: 45
Timeout: 5
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

## Getting Test262

```bash
git clone https://github.com/tc39/test262.git
```

## Contributing

1. Follow TDD practices
2. Maintain 80%+ test coverage
3. Use component commit format: `[test262_harness] type: description`
