# parser Component

**Type**: Feature Library (Level 2)
**Tech Stack**: Rust, nom (optional)
**Version**: 0.1.0

## Purpose
JavaScript lexer, parser (recursive descent), scope analysis, AST construction, and bytecode generation. Supports ES2024 syntax with lazy parsing for performance.

## Dependencies
- `core_types`: Value, JsError, SourcePosition
- `bytecode_system`: Opcode, BytecodeChunk, Instruction

## Token Budget
- Optimal: 70,000 tokens
- Warning: 90,000 tokens
- Critical: 110,000 tokens

## Exported Types

```rust
// Lexer
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

pub enum Token {
    Identifier(String),
    Number(f64),
    String(String),
    Keyword(Keyword),
    Punctuator(Punctuator),
    // ...
}

// Parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

// AST nodes
pub enum ASTNode {
    Program(Vec<Statement>),
    FunctionDecl { name: String, params: Vec<String>, body: Vec<Statement> },
    VariableDecl { name: String, init: Option<Expression> },
    // ...
}

// Scope analysis
pub struct ScopeAnalyzer {
    scopes: Vec<Scope>,
}

// Bytecode generation
pub struct BytecodeGenerator {
    chunk: BytecodeChunk,
    scope_info: ScopeInfo,
}

impl BytecodeGenerator {
    pub fn generate(&mut self, ast: &ASTNode) -> Result<BytecodeChunk, JsError>;
}
```

## Key Implementation Requirements

### Lazy Parsing
1. First pass (preparser): Validate syntax without full AST
2. Second pass (full parser): Generate AST when function executes
3. Store function metadata for delazification

### Scope Analysis
- Identify lexical scopes
- Resolve variable references
- Determine heap-allocated variables (closures)
- Annotate AST with scope info

### ES2024 Features
- Classes, arrow functions, destructuring
- async/await, generators
- Template literals, spread operator
- Optional chaining, nullish coalescing

### Error Reporting
- Precise source locations
- Helpful error messages
- Early error detection per spec

## Mandatory Requirements

### 1. Test-Driven Development
- Test each grammar rule
- 80%+ coverage
- TDD pattern in commits

### 2. File Structure
```
src/
  lib.rs              # Public exports
  lexer.rs            # Tokenizer
  parser.rs           # Recursive descent parser
  ast.rs              # AST node definitions
  scope.rs            # Scope analysis
  bytecode_gen.rs     # AST to bytecode
  error.rs            # Parse errors
tests/
  unit/
  integration/
  contracts/
```

## Git Commit Format
```
[parser] <type>: <description>
```

## Definition of Done
- [ ] Complete ES2024 grammar
- [ ] TDD cycles in git history
- [ ] 80%+ coverage
- [ ] Lazy parsing working
- [ ] Scope analysis complete
- [ ] Bytecode generation functional
- [ ] Contract tests passing
