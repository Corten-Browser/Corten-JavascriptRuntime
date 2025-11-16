//! JavaScript Parser Component
//!
//! Provides lexer, parser, AST construction, scope analysis, and bytecode generation
//! for ES2024 JavaScript syntax.
//!
//! # Overview
//!
//! - [`Lexer`] - Tokenizes JavaScript source code
//! - [`Token`] - Token types including identifiers, literals, keywords
//! - [`Parser`] - Recursive descent parser producing AST
//! - [`ASTNode`] - Abstract Syntax Tree node types
//! - [`BytecodeGenerator`] - Converts AST to bytecode
//! - [`ScopeAnalyzer`] - Resolves variable scopes and references
//!
//! # Example
//!
//! ```
//! use parser::{Parser, BytecodeGenerator};
//!
//! let source = "let x = 42;";
//! let mut parser = Parser::new(source);
//! let ast = parser.parse().unwrap();
//!
//! let mut gen = BytecodeGenerator::new();
//! let bytecode = gen.generate(&ast).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod bytecode_gen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod scope;

pub use ast::{ASTNode, Expression, Statement};
pub use bytecode_gen::BytecodeGenerator;
pub use lexer::{Keyword, Lexer, Punctuator, Token};
pub use parser::{LazyAST, Parser};
pub use scope::{ScopeAnalyzer, ScopeInfo};
