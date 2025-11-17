# builtins

ECMAScript 2024 standard library implementation for the Corten JavaScript Runtime.

## Overview

This component provides built-in objects and prototypes for JavaScript execution:

- **ObjectPrototype** - hasOwnProperty, isPrototypeOf, toString, valueOf
- **ArrayPrototype** - push, pop, shift, unshift, slice, splice, map, filter, reduce, forEach, find, findIndex, includes, indexOf, join, sort, reverse
- **StringPrototype** - substring, slice, split, replace, match, trim, toLowerCase, toUpperCase, charAt, charCodeAt, padStart, padEnd, startsWith, endsWith, includes
- **NumberPrototype** - toString (with radix), toFixed, toPrecision, valueOf
- **MathObject** - abs, ceil, floor, round, sqrt, pow, sin, cos, tan, random, max, min, log, exp, PI, E, etc.
- **JSONObject** - parse, stringify
- **ConsoleObject** - log, error, warn, info, debug, assert, time, timeEnd

## Quick Start

```rust
use builtins::{JsValue, ArrayPrototype, MathObject};

// Create and manipulate arrays
let arr = JsValue::array_from(vec![
    JsValue::number(1.0),
    JsValue::number(2.0),
    JsValue::number(3.0),
]);

let sum = ArrayPrototype::reduce(&arr, JsValue::number(0.0), |acc, v| {
    Ok(JsValue::number(acc.as_number().unwrap() + v.as_number().unwrap()))
}).unwrap();

assert_eq!(sum.as_number().unwrap(), 6.0);

// Use Math methods
assert_eq!(MathObject::sqrt(16.0), 4.0);
assert_eq!(MathObject::pow(2.0, 10.0), 1024.0);
```

## Architecture

The implementation uses:
- `JsValue` enum for JavaScript value representation (Undefined, Null, Boolean, Number, String, Object, Array)
- `Rc<RefCell>` pattern for mutable shared objects
- Comprehensive error handling with `JsResult<T>` and `JsError`

## Build

```bash
cargo build
cargo test
```

## Test Coverage

- 86 unit tests covering all public APIs
- Integration tests for cross-module functionality
- Contract test skeletons for external validation
- 100% test pass rate

## Dependencies

- `regex` - Regular expression support for String.prototype.match
- `serde_json` - JSON parsing and stringification

See `CLAUDE.md` for detailed implementation requirements and `component.yaml` for component configuration.

## Status

**Implemented** - All contract-required methods are implemented following TDD workflow with comprehensive test coverage.
