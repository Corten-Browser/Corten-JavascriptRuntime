# builtins Component

**Type**: Feature Library (Level 2)
**Tech Stack**: Rust, num-bigint, regex, ryu
**Version**: 0.1.0

## Purpose
ECMAScript 2024 standard library implementation including all built-in objects (Object, Array, String, etc.), prototypes, and constructors.

## Dependencies
- `memory_manager`: Heap, JSObject
- `interpreter`: VM, ExecutionContext

## Token Budget
- Optimal: 70,000 tokens
- Warning: 90,000 tokens
- Critical: 110,000 tokens

## Exported Types

```rust
// Core prototypes
pub struct ObjectPrototype;
pub struct ArrayPrototype;
pub struct StringPrototype;
pub struct NumberPrototype;
pub struct BooleanPrototype;
pub struct FunctionPrototype;

// Built-in objects
pub struct MathObject;
pub struct JSONObject;
pub struct ConsoleObject;

// Constructors
pub struct RegExpConstructor;
pub struct BigIntConstructor;
pub struct DateConstructor;
pub struct ErrorConstructor;
pub struct ArrayBufferConstructor;
pub struct TypedArrayConstructor;
pub struct MapConstructor;
pub struct SetConstructor;
pub struct WeakMapConstructor;
pub struct WeakSetConstructor;
pub struct PromiseConstructor;
```

## Key Implementation Requirements

### Object Prototype Methods
- `hasOwnProperty()`, `isPrototypeOf()`
- `toString()`, `valueOf()`
- `getPrototypeOf()`, `setPrototypeOf()`
- Property descriptors

### Array Methods
- Mutating: `push()`, `pop()`, `shift()`, `unshift()`, `splice()`
- Non-mutating: `map()`, `filter()`, `reduce()`, `forEach()`
- Search: `find()`, `findIndex()`, `includes()`, `indexOf()`
- ES2024: `toSorted()`, `toReversed()`, `toSpliced()`

### String Methods
- `substring()`, `slice()`, `split()`, `join()`
- `replace()`, `match()` (RegExp integration)
- `trim()`, `padStart()`, `padEnd()`
- Unicode support

### Math Object
- `abs()`, `ceil()`, `floor()`, `round()`
- `sin()`, `cos()`, `tan()`, `sqrt()`, `pow()`
- `random()`, `max()`, `min()`

### BigInt Support
- Arbitrary precision integers
- Integration with `num-bigint` crate
- Type coercion rules

### RegExp Engine
- Full Unicode support
- Named capture groups
- Lookbehind assertions
- Use Rust `regex` crate or custom

## Mandatory Requirements

### 1. Test-Driven Development
- Test each built-in method
- 80%+ coverage
- TDD pattern in commits

### 2. Spec Compliance
- Follow ECMAScript 2024 exactly
- Handle edge cases per spec
- Type coercion rules correct

### 3. File Structure
```
src/
  lib.rs             # Public exports
  object.rs          # Object prototype
  array.rs           # Array prototype
  string.rs          # String prototype
  number.rs          # Number prototype
  math.rs            # Math object
  json.rs            # JSON object
  regexp.rs          # RegExp
  bigint.rs          # BigInt
  date.rs            # Date
  collections.rs     # Map, Set, WeakMap, WeakSet
  typed_arrays.rs    # TypedArray family
  console.rs         # Console API
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[builtins] <type>: <description>
```

## Definition of Done
- [ ] Core prototypes implemented
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Spec compliance verified
- [ ] Console API complete
- [ ] Contract tests passing
