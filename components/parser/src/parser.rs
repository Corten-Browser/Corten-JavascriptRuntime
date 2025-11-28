//! Recursive descent parser for JavaScript

use crate::ast::*;
use crate::error::*;
use crate::lexer::{Keyword, Lexer, Punctuator, Token};
use core_types::JsError;

/// Lazy AST representation for deferred parsing
#[derive(Debug, Clone)]
pub struct LazyAST {
    /// Source code
    pub source: String,
    /// Pre-parsed function metadata
    pub functions: Vec<LazyFunction>,
}

/// Lazy function metadata
#[derive(Debug, Clone)]
pub struct LazyFunction {
    /// Function name
    pub name: Option<String>,
    /// Start offset in source
    pub start: usize,
    /// End offset in source
    pub end: usize,
}

/// JavaScript parser
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    source: &'a str,
    last_position: Option<core_types::SourcePosition>,
    /// Track if we're in strict mode
    strict_mode: bool,
    /// Track loop depth for break/continue validation
    loop_depth: usize,
    /// Track function depth for return validation
    function_depth: usize,
    /// Track if we're inside a generator function (for yield expressions)
    in_generator: bool,
    /// Track if we're inside an async function (for await expressions)
    in_async: bool,
    /// Track if we're inside a class method (allows super.property)
    in_class_method: bool,
    /// Track if we're inside a class constructor (allows super())
    in_constructor: bool,
    /// Track if the current class has heritage (extends clause) - super() requires heritage
    has_class_heritage: bool,
    /// Track if we're inside any method (class or object literal - allows super.property)
    in_method: bool,
    /// Track active labels for break/continue validation
    active_labels: std::collections::HashSet<String>,
    /// Track labels that are iteration statements (for continue validation)
    iteration_labels: std::collections::HashSet<String>,
    /// Track if we're in for loop init (disallows 'in' as relational operator)
    in_for_init: bool,
    /// Track if we're inside a class static block (await and arguments are reserved)
    in_static_block: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            source,
            last_position: None,
            strict_mode: false,
            loop_depth: 0,
            function_depth: 0,
            in_generator: false,
            in_async: false,
            in_class_method: false,
            in_constructor: false,
            has_class_heritage: false,
            in_method: false,
            active_labels: std::collections::HashSet::new(),
            iteration_labels: std::collections::HashSet::new(),
            in_for_init: false,
            in_static_block: false,
        }
    }

    /// Parse the source into an AST
    pub fn parse(&mut self) -> Result<ASTNode, JsError> {
        let mut statements = Vec::new();

        // Check for "use strict" directive at the start
        self.check_directive_prologue()?;

        while !self.is_at_end()? {
            statements.push(self.parse_statement()?);
        }

        Ok(ASTNode::Program(statements))
    }

    /// Check for directive prologue (e.g., "use strict")
    fn check_directive_prologue(&mut self) -> Result<(), JsError> {
        // Look for string literal expression statements at the start
        while !self.is_at_end()? {
            // Peek at the token
            let token = self.lexer.peek_token()?.clone();

            // Check if it's a string literal that could be a directive
            if let Token::String(ref s) = token {
                if s == "use strict" {
                    self.strict_mode = true;
                }
                // Consume the string and check for semicolon
                self.lexer.next_token()?;
                if self.check_punctuator(Punctuator::Semicolon)? {
                    self.lexer.next_token()?;
                } else if !self.lexer.line_terminator_before_token {
                    // Not a directive - put the string back by returning
                    // In real implementation we'd need proper lookahead
                    // For now, just continue - the string is consumed
                    break;
                }
            } else {
                // No more potential directives
                break;
            }
        }
        Ok(())
    }

    /// Parse with lazy function bodies (for performance)
    pub fn parse_lazy(&mut self) -> Result<LazyAST, JsError> {
        let mut functions = Vec::new();

        // Simple preparse - identify function boundaries
        while !self.is_at_end()? {
            let token = self.lexer.peek_token()?;
            match token {
                Token::Keyword(Keyword::Function) | Token::Keyword(Keyword::Async) => {
                    let start = self.current_offset();
                    self.skip_function()?;
                    let end = self.current_offset();
                    functions.push(LazyFunction {
                        name: None,
                        start,
                        end,
                    });
                }
                _ => {
                    self.lexer.next_token()?;
                }
            }
        }

        Ok(LazyAST {
            source: self.source.to_string(),
            functions,
        })
    }

    fn skip_function(&mut self) -> Result<(), JsError> {
        // Skip async if present
        if matches!(self.lexer.peek_token()?, Token::Keyword(Keyword::Async)) {
            self.lexer.next_token()?;
        }

        // Skip function keyword
        self.expect_keyword(Keyword::Function)?;

        // Skip optional name
        if let Token::Identifier(_, _) = self.lexer.peek_token()? {
            self.lexer.next_token()?;
        }

        // Skip parameters
        self.expect_punctuator(Punctuator::LParen)?;
        self.skip_until_matching(Punctuator::LParen, Punctuator::RParen)?;

        // Skip body
        self.expect_punctuator(Punctuator::LBrace)?;
        self.skip_until_matching(Punctuator::LBrace, Punctuator::RBrace)?;

        Ok(())
    }

    fn skip_until_matching(&mut self, open: Punctuator, close: Punctuator) -> Result<(), JsError> {
        let mut depth = 1;
        while depth > 0 && !self.is_at_end()? {
            let token = self.lexer.next_token()?;
            if let Token::Punctuator(p) = token {
                if p == open {
                    depth += 1;
                } else if p == close {
                    depth -= 1;
                }
            }
        }
        Ok(())
    }

    fn current_offset(&self) -> usize {
        0 // Simplified - would track actual position
    }

    fn is_at_end(&mut self) -> Result<bool, JsError> {
        Ok(matches!(self.lexer.peek_token()?, Token::EOF))
    }

    fn parse_statement(&mut self) -> Result<Statement, JsError> {
        let token = self.lexer.peek_token()?.clone();

        // Handle 'let' specially - in non-strict mode it can be an identifier
        if matches!(token, Token::Keyword(Keyword::Let)) && !self.strict_mode {
            // Check if 'let' is used as keyword (followed by identifier, [, or {)
            let is_let_declaration = self.is_let_declaration()?;
            if !is_let_declaration {
                // 'let' is an identifier, parse as expression statement
                return self.parse_expression_statement();
            }
        }

        match token {
            Token::Keyword(Keyword::Let) // Only reached if strict mode or confirmed declaration
            | Token::Keyword(Keyword::Const)
            | Token::Keyword(Keyword::Var) => self.parse_variable_declaration(),
            Token::Keyword(Keyword::Function) => self.parse_function_declaration(),
            Token::Keyword(Keyword::Async) => self.parse_async_statement(),
            Token::Keyword(Keyword::Class) => self.parse_class_declaration(),
            Token::Keyword(Keyword::Return) => self.parse_return_statement(),
            Token::Keyword(Keyword::If) => self.parse_if_statement(),
            Token::Keyword(Keyword::While) => self.parse_while_statement(),
            Token::Keyword(Keyword::Do) => self.parse_do_while_statement(),
            Token::Keyword(Keyword::For) => self.parse_for_statement(),
            Token::Keyword(Keyword::Switch) => self.parse_switch_statement(),
            Token::Keyword(Keyword::Break) => self.parse_break_statement(),
            Token::Keyword(Keyword::Continue) => self.parse_continue_statement(),
            Token::Keyword(Keyword::Throw) => self.parse_throw_statement(),
            Token::Keyword(Keyword::Try) => self.parse_try_statement(),
            Token::Keyword(Keyword::With) => self.parse_with_statement(),
            Token::Keyword(Keyword::Debugger) => self.parse_debugger_statement(),
            Token::Punctuator(Punctuator::LBrace) => self.parse_block_statement(),
            Token::Punctuator(Punctuator::Semicolon) => {
                self.lexer.next_token()?;
                Ok(Statement::EmptyStatement { position: None })
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, JsError> {
        let kind = match self.lexer.next_token()? {
            Token::Keyword(Keyword::Let) => VariableKind::Let,
            Token::Keyword(Keyword::Const) => VariableKind::Const,
            Token::Keyword(Keyword::Var) => VariableKind::Var,
            _ => unreachable!(),
        };

        let mut declarations = Vec::new();

        loop {
            let id = self.parse_pattern()?;
            let init = if self.check_punctuator(Punctuator::Assign)? {
                self.lexer.next_token()?;
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.consume_semicolon()?;

        Ok(Statement::VariableDeclaration {
            kind,
            declarations,
            position: None,
        })
    }

    /// Check if 'let' is being used as a keyword (declaration) or identifier
    /// Returns true if 'let' is followed by: identifier, [, or {
    /// Returns false if 'let' is followed by: =, ;, ,, ), or other operators
    fn is_let_declaration(&mut self) -> Result<bool, JsError> {
        // Save lexer state
        let saved_pos = self.lexer.position;
        let saved_line = self.lexer.line;
        let saved_column = self.lexer.column;
        let saved_previous_line = self.lexer.previous_line;
        let saved_line_term = self.lexer.line_terminator_before_token;
        let saved_token = self.lexer.current_token.clone();

        self.lexer.next_token()?; // consume "let"
        let next = self.lexer.peek_token()?.clone();
        // Check for line terminator after peek so the flag is updated
        let has_line_terminator = self.lexer.line_terminator_before_token;

        // "let" is a keyword (declaration) if followed by (without line terminator):
        // - identifier (but not 'in' or 'of' which could be for-in/for-of with let as LHS)
        // - [ (array destructuring)
        // - { (object destructuring)
        // If there's a line terminator, `let` is an identifier expression with ASI after it
        let is_declaration = if has_line_terminator {
            // Line terminator after 'let' means it's an identifier, not a declaration
            false
        } else {
            match next {
                Token::Identifier(_, _) => true,
                Token::Punctuator(Punctuator::LBracket) => true,
                Token::Punctuator(Punctuator::LBrace) => true,
                // Keywords that can be identifiers in certain contexts
                Token::Keyword(Keyword::Yield) if !self.in_generator => true,
                Token::Keyword(Keyword::Await) if !self.in_async && !self.in_static_block => true,
                Token::Keyword(Keyword::Static) if !self.strict_mode => true,
                // 'async' can be used as an identifier
                Token::Keyword(Keyword::Async) => true,
                _ => false,
            }
        };

        // Restore lexer state
        self.lexer.position = saved_pos;
        self.lexer.line = saved_line;
        self.lexer.column = saved_column;
        self.lexer.previous_line = saved_previous_line;
        self.lexer.line_terminator_before_token = saved_line_term;
        self.lexer.current_token = saved_token;

        Ok(is_declaration)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, JsError> {
        self.update_position()?;
        let token = self.lexer.peek_token()?.clone();

        match token {
            Token::Identifier(name, _has_escapes) => {
                self.lexer.next_token()?;
                // Validate the identifier is not a reserved word and is valid as a binding
                // Per ES spec, even escaped reserved words are invalid as identifiers
                self.validate_binding_identifier(&name)?;
                Ok(Pattern::Identifier(name))
            }
            // yield is a valid identifier in non-strict mode (outside generators)
            Token::Keyword(Keyword::Yield) => {
                self.lexer.next_token()?;
                // In generator context, 'yield' cannot be used as identifier
                if self.in_generator {
                    return Err(syntax_error(
                        "'yield' is not allowed as an identifier in generator functions",
                        self.last_position.clone(),
                    ));
                }
                // In strict mode, 'yield' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'yield' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok(Pattern::Identifier("yield".to_string()))
            }
            Token::Keyword(Keyword::Let) => {
                self.lexer.next_token()?;
                // In strict mode, 'let' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'let' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok(Pattern::Identifier("let".to_string()))
            }
            Token::Keyword(Keyword::Static) => {
                self.lexer.next_token()?;
                // In strict mode, 'static' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'static' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok(Pattern::Identifier("static".to_string()))
            }
            // 'await' can be used as identifier outside async functions
            Token::Keyword(Keyword::Await) => {
                self.lexer.next_token()?;
                // In async function context, 'await' cannot be used as identifier
                if self.in_async {
                    return Err(syntax_error(
                        "'await' is not allowed as an identifier in async functions",
                        self.last_position.clone(),
                    ));
                }
                Ok(Pattern::Identifier("await".to_string()))
            }
            // 'async' can be used as identifier in non-strict mode
            Token::Keyword(Keyword::Async) => {
                self.lexer.next_token()?;
                // In strict mode, 'async' is technically still allowed as an identifier
                // but it's considered bad practice. For now, we allow it.
                Ok(Pattern::Identifier("async".to_string()))
            }
            Token::Punctuator(Punctuator::LBrace) => self.parse_object_pattern(),
            Token::Punctuator(Punctuator::LBracket) => self.parse_array_pattern(),
            _ => Err(syntax_error("Expected pattern", self.last_position.clone())),
        }
    }

    fn parse_object_pattern(&mut self) -> Result<Pattern, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut properties = Vec::new();
        let mut has_rest = false;

        while !self.check_punctuator(Punctuator::RBrace)? {
            if self.check_punctuator(Punctuator::Spread)? {
                // Rest element must be last - can't have multiple rest elements
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in object pattern",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                properties.push(ObjectPatternProperty {
                    key: crate::ast::PatternKey::Literal(String::new()),
                    value: Pattern::RestElement(Box::new(pattern)),
                    shorthand: false,
                });
                has_rest = true;
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Rest element must be last - can't have elements after it
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in object pattern",
                        self.last_position.clone(),
                    ));
                }
                // Computed property key: { [expr]: pattern }
                self.lexer.next_token()?;
                let key_expr = self.parse_assignment_expression()?;
                self.expect_punctuator(Punctuator::RBracket)?;
                self.expect_punctuator(Punctuator::Colon)?;
                let value = self.parse_pattern()?;

                // Check for default value
                let final_value = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    let default_value = self.parse_assignment_expression()?;
                    Pattern::AssignmentPattern {
                        left: Box::new(value),
                        right: Box::new(default_value),
                    }
                } else {
                    value
                };

                properties.push(ObjectPatternProperty {
                    key: crate::ast::PatternKey::Computed(key_expr),
                    value: final_value,
                    shorthand: false,
                });
            } else {
                // Rest element must be last - can't have elements after it
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in object pattern",
                        self.last_position.clone(),
                    ));
                }
                // Regular key: identifier, string, or number
                let key = self.expect_property_name()?;
                let (value, shorthand) = if self.check_punctuator(Punctuator::Colon)? {
                    self.lexer.next_token()?;
                    (self.parse_pattern()?, false)
                } else {
                    // Shorthand only valid for identifiers
                    (Pattern::Identifier(key.clone()), true)
                };

                // Check for default value
                let final_value = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    let default_value = self.parse_assignment_expression()?;
                    Pattern::AssignmentPattern {
                        left: Box::new(value),
                        right: Box::new(default_value),
                    }
                } else {
                    value
                };

                properties.push(ObjectPatternProperty {
                    key: crate::ast::PatternKey::Literal(key),
                    value: final_value,
                    shorthand,
                });
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(Pattern::ObjectPattern(properties))
    }

    fn parse_array_pattern(&mut self) -> Result<Pattern, JsError> {
        self.expect_punctuator(Punctuator::LBracket)?;
        let mut elements = Vec::new();
        let mut has_rest = false;

        while !self.check_punctuator(Punctuator::RBracket)? {
            if self.check_punctuator(Punctuator::Comma)? {
                // Rest element must be last - can't have elements after it
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in array pattern",
                        self.last_position.clone(),
                    ));
                }
                elements.push(None);
            } else if self.check_punctuator(Punctuator::Spread)? {
                // Rest element must be last - can't have multiple rest elements
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in array pattern",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                elements.push(Some(Pattern::RestElement(Box::new(pattern))));
                has_rest = true;
            } else {
                // Rest element must be last - can't have elements after it
                if has_rest {
                    return Err(syntax_error(
                        "Rest element must be last in array pattern",
                        self.last_position.clone(),
                    ));
                }
                let pattern = self.parse_pattern()?;
                // Check for default value
                let final_pattern = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    let default_value = self.parse_assignment_expression()?;
                    Pattern::AssignmentPattern {
                        left: Box::new(pattern),
                        right: Box::new(default_value),
                    }
                } else {
                    pattern
                };
                elements.push(Some(final_pattern));
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RBracket)?;
        Ok(Pattern::ArrayPattern(elements))
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Function)?;

        let is_generator = self.check_punctuator(Punctuator::Star)?;
        if is_generator {
            self.lexer.next_token()?;
        }

        let name = self.expect_identifier()?;

        // Save context flags
        let prev_generator = self.in_generator;
        let prev_async = self.in_async;
        // Clear class context - regular functions cannot use super
        let prev_in_class_method = self.in_class_method;
        let prev_in_constructor = self.in_constructor;

        // Set context for this function - regular functions are NOT async
        // so 'await' is allowed as an identifier inside them
        self.in_generator = is_generator;
        self.in_async = false;  // Regular function declarations are not async
        self.in_class_method = false;
        self.in_constructor = false;

        let params = self.parse_parameters()?;
        // Validate for duplicate parameters
        self.validate_parameters(&params)?;
        // Validate no yield expressions in formal parameters (early error for generators)
        if is_generator {
            self.validate_params_no_yield(&params)?;
        }
        let body = self.parse_function_body()?;
        // Validate "use strict" with non-simple params
        self.validate_params_with_body(&params, &body)?;
        // Validate parameter names don't conflict with lexical declarations in body
        self.validate_params_body_lexical(&params, &body)?;

        // Restore context
        self.in_generator = prev_generator;
        self.in_async = prev_async;
        self.in_class_method = prev_in_class_method;
        self.in_constructor = prev_in_constructor;

        Ok(Statement::FunctionDeclaration {
            name,
            params,
            body,
            is_async: false,
            is_generator,
            position: None,
        })
    }

    /// Parse statement starting with 'async' keyword
    /// Handles: async function declaration, async arrow expression statement, or async as identifier
    fn parse_async_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Async)?;

        // Peek next token to update line_terminator_before_token
        // (after consuming async, we need to check what comes next)
        self.lexer.peek_token()?;

        // Check for line terminator after async (before the next token)
        if self.lexer.line_terminator_before_token {
            // Check if this looks like an async arrow function with invalid line terminator
            // The grammar says: async [no LineTerminator here] ArrowFormalParameters
            if self.check_punctuator(Punctuator::LParen)? {
                // Look ahead to see if `=>` follows the parenthesized expression
                // Save the full lexer state including cached token
                let start_pos = self.lexer.position;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let start_token = self.lexer.current_token.clone();

                // Clear token cache so next_token actually scans
                self.lexer.current_token = None;

                // Scan `(` fresh
                let _ = self.lexer.next_token()?;
                let mut depth = 1;

                // Find matching `)` by scanning tokens
                while depth > 0 {
                    let tok = self.lexer.next_token()?;
                    match tok {
                        Token::Punctuator(Punctuator::LParen) => depth += 1,
                        Token::Punctuator(Punctuator::RParen) => depth -= 1,
                        Token::EOF => break,
                        _ => {}
                    }
                }

                // Check if `=>` follows
                let next = self.lexer.next_token()?;
                let is_arrow = matches!(next, Token::Punctuator(Punctuator::Arrow));

                // Restore lexer state
                self.lexer.position = start_pos;
                self.lexer.line = start_line;
                self.lexer.column = start_col;
                self.lexer.current_token = start_token;
                self.lexer.line_terminator_before_token = true;

                if is_arrow {
                    // This is an async arrow function with an invalid line terminator
                    return Err(syntax_error(
                        "Line terminator not allowed between 'async' and arrow function parameters",
                        self.last_position.clone(),
                    ));
                }
            }

            // Line terminator after async - async is just an identifier
            self.consume_semicolon()?;
            return Ok(Statement::ExpressionStatement {
                expression: Expression::Identifier {
                    name: "async".to_string(),
                    position: None,
                },
                position: None,
            });
        }

        // Check for async function declaration
        if self.check_keyword(Keyword::Function)? {
            return self.parse_async_function_declaration_after_async();
        }

        // Otherwise it's an expression statement with async arrow or async as identifier
        // Parse the async expression part
        let expr = self.parse_async_expression_after_async()?;
        self.consume_semicolon()?;
        Ok(Statement::ExpressionStatement {
            expression: expr,
            position: None,
        })
    }

    fn parse_async_function_or_expression(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Async)?;
        self.parse_async_function_declaration_after_async()
    }

    /// Parse async expression after 'async' keyword has been consumed (for expression statements)
    /// Handles async arrow functions and async as identifier
    fn parse_async_expression_after_async(&mut self) -> Result<Expression, JsError> {
        if self.check_punctuator(Punctuator::LParen)? {
            // Async arrow function with parens: async (params) => body
            // Set async context for parameter parsing - 'await' should be reserved
            // in default parameter expressions of async functions
            let prev_async = self.in_async;
            self.in_async = true;
            let params = self.parse_parameters()?;
            // Validate arrow parameters (duplicates and yield expressions)
            self.validate_arrow_parameters(&params)?;
            self.validate_arrow_params_no_yield(&params)?;
            // Check for line terminator before =>
            if self.check_punctuator(Punctuator::Arrow)? {
                if self.lexer.line_terminator_before_token {
                    self.in_async = prev_async;
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
            }
            self.expect_punctuator(Punctuator::Arrow)?;
            let body = self.parse_arrow_body_with_context(true)?;
            self.in_async = prev_async;
            // Validate "use strict" with non-simple parameters
            self.validate_arrow_params_with_body(&params, &body)?;

            Ok(Expression::ArrowFunctionExpression {
                params,
                body,
                is_async: true,
                position: None,
            })
        } else if let Token::Identifier(name, _) = self.lexer.peek_token()?.clone() {
            // Could be async arrow function without parens: async x => body
            // Or async followed by identifier as call target: async(x)
            self.validate_binding_identifier(&name)?;
            self.lexer.next_token()?;
            // Check for arrow
            if self.check_punctuator(Punctuator::Arrow)? {
                if self.lexer.line_terminator_before_token {
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?; // consume =>
                let body = self.parse_arrow_body_with_context(true)?;

                Ok(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::Identifier(name)],
                    body,
                    is_async: true,
                    position: None,
                })
            } else {
                // No arrow - this is 'async' as identifier followed by something else
                // But we've consumed the identifier after async, so this is ambiguous
                // For statement context, this is likely an error
                Err(syntax_error("Expected '=>' after async arrow function parameter", None))
            }
        } else {
            // Just 'async' followed by something that isn't ( or identifier
            // Return async as standalone identifier
            Ok(Expression::Identifier {
                name: "async".to_string(),
                position: None,
            })
        }
    }

    /// Parse async function declaration after 'async' keyword has been consumed
    fn parse_async_function_declaration_after_async(&mut self) -> Result<Statement, JsError> {
        if self.check_keyword(Keyword::Function)? {
            self.lexer.next_token()?;

            // Check for async generator: async function *name()
            let is_generator = self.check_punctuator(Punctuator::Star)?;
            if is_generator {
                self.lexer.next_token()?;
            }

            let name = self.expect_identifier()?;

            // Set async context before parsing parameters so 'await' is properly rejected
            let prev_async = self.in_async;
            let prev_generator = self.in_generator;
            // Clear class context - regular async functions cannot use super
            let prev_in_class_method = self.in_class_method;
            let prev_in_constructor = self.in_constructor;

            self.in_async = true;
            self.in_generator = is_generator;
            self.in_class_method = false;
            self.in_constructor = false;

            let params = self.parse_parameters()?;
            // Validate for duplicate parameters
            self.validate_parameters(&params)?;
            // Validate no await expressions in formal parameters (early error for async functions)
            self.validate_params_no_await(&params)?;
            // Validate no yield expressions in formal parameters (early error for generators)
            if is_generator {
                self.validate_params_no_yield(&params)?;
            }
            let body = self.parse_function_body()?;
            // Validate "use strict" with non-simple params
            self.validate_params_with_body(&params, &body)?;
            // Validate parameter names don't conflict with lexical declarations in body
            self.validate_params_body_lexical(&params, &body)?;

            self.in_async = prev_async;
            self.in_generator = prev_generator;
            self.in_class_method = prev_in_class_method;
            self.in_constructor = prev_in_constructor;

            Ok(Statement::FunctionDeclaration {
                name,
                params,
                body,
                is_async: true,
                is_generator,
                position: None,
            })
        } else {
            Err(syntax_error("Expected function after async", None))
        }
    }

    fn parse_class_declaration(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Class)?;

        // All parts of a class are strict mode code, including the class name
        let prev_strict = self.strict_mode;
        self.strict_mode = true;

        let name = self.expect_identifier()?;

        let super_class = if self.check_keyword(Keyword::Extends)? {
            self.lexer.next_token()?;
            let heritage = self.parse_left_hand_side_expression()?;
            // ArrowFunctionExpression is not a valid LeftHandSideExpression
            // It's an AssignmentExpression, so reject it here
            if matches!(heritage, Expression::ArrowFunctionExpression { .. }) {
                return Err(syntax_error(
                    "Arrow function is not allowed as class heritage",
                    self.last_position.clone(),
                ));
            }
            Some(Box::new(heritage))
        } else {
            None
        };

        // Set heritage flag for super() validation in constructor
        let prev_has_heritage = self.has_class_heritage;
        self.has_class_heritage = super_class.is_some();

        let body = self.parse_class_body()?;

        self.has_class_heritage = prev_has_heritage;
        self.strict_mode = prev_strict;

        Ok(Statement::ClassDeclaration {
            name,
            super_class,
            body,
            position: None,
        })
    }

    fn parse_class_body(&mut self) -> Result<Vec<ClassElement>, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut elements = Vec::new();
        let mut has_constructor = false;
        // Track private names to detect duplicates
        // Fields and methods conflict with everything
        // Getters and setters can coexist with each other but not duplicate
        let mut private_fields_methods: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut private_getters: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut private_setters: std::collections::HashSet<String> = std::collections::HashSet::new();

        while !self.check_punctuator(Punctuator::RBrace)? {
            // Skip extra semicolons (allowed in class bodies)
            while self.check_punctuator(Punctuator::Semicolon)? {
                self.lexer.next_token()?;
            }
            // Check if we hit the closing brace after skipping semicolons
            if self.check_punctuator(Punctuator::RBrace)? {
                break;
            }

            // Check for static
            // Note: "static" can be a field name if followed by = ; , or }
            // Note: "static {" is a static initialization block
            let is_static = if self.check_keyword(Keyword::Static)? {
                // Peek ahead to see if this is the static keyword or a field named "static"
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_line_term = self.lexer.line_terminator_before_token;
                let saved_token = self.lexer.current_token.clone();

                self.lexer.next_token()?;
                let next = self.lexer.peek_token()?;

                // Check for static initialization block: static { ... }
                if matches!(next, Token::Punctuator(Punctuator::LBrace)) {
                    // Parse static block
                    self.lexer.next_token()?; // consume {

                    // Save context and set static block context
                    // In static blocks: 'await' is reserved, 'arguments' is reserved,
                    // and super.prop is allowed (like methods)
                    let prev_in_static_block = self.in_static_block;
                    let prev_in_method = self.in_method;
                    self.in_static_block = true;
                    self.in_method = true;  // Allow super.prop in static blocks

                    let mut body = Vec::new();
                    while !self.check_punctuator(Punctuator::RBrace)? {
                        body.push(self.parse_statement()?);
                    }

                    // Restore context
                    self.in_static_block = prev_in_static_block;
                    self.in_method = prev_in_method;

                    self.expect_punctuator(Punctuator::RBrace)?;
                    elements.push(ClassElement::StaticBlock { body });
                    continue;
                }

                // If followed by = ; , or } then "static" is a field name, not keyword
                let is_field_name = matches!(
                    next,
                    Token::Punctuator(Punctuator::Assign)
                        | Token::Punctuator(Punctuator::Semicolon)
                        | Token::Punctuator(Punctuator::Comma)
                        | Token::Punctuator(Punctuator::RBrace)
                );

                if is_field_name {
                    // Restore - treat "static" as field name
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = saved_token;
                    false
                } else {
                    // It's the static keyword
                    true
                }
            } else {
                false
            };

            // Check for async
            let is_async = if self.check_keyword(Keyword::Async)? {
                // Peek ahead to see if this is actually an async method
                // vs a method named "async"
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_line_term = self.lexer.line_terminator_before_token;

                self.lexer.next_token()?;
                let next = self.lexer.peek_token()?;
                let is_method = matches!(next, Token::Punctuator(Punctuator::Star))
                    || matches!(next, Token::Identifier(_, _))
                    || matches!(next, Token::Keyword(_))
                    || matches!(next, Token::Punctuator(Punctuator::LBracket))
                    || matches!(next, Token::PrivateIdentifier(_));

                if is_method && !self.lexer.line_terminator_before_token {
                    // It's an async method
                    true
                } else {
                    // Restore - it's a method named "async"
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = None;
                    false
                }
            } else {
                false
            };

            // Check for generator
            let is_generator = if self.check_punctuator(Punctuator::Star)? {
                self.lexer.next_token()?;
                true
            } else {
                false
            };

            // Check for private identifier
            let mut is_private = self.check_private_identifier()?;

            // Check for get/set and parse key
            let mut kind = MethodKind::Method;
            let (key, computed) = if is_private {
                // Private field/method
                let name = self.expect_private_identifier()?;
                (PropertyKey::Identifier(name), false)
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Computed property name: [expr] - 'in' is always allowed inside
                self.lexer.next_token()?;
                let prev_in_for_init = self.in_for_init;
                self.in_for_init = false;
                let key_expr = self.parse_assignment_expression()?;
                self.in_for_init = prev_in_for_init;
                self.expect_punctuator(Punctuator::RBracket)?;
                (PropertyKey::Computed(key_expr), true)
            } else if !is_generator && self.check_identifier("get")? {
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_previous_line = self.lexer.previous_line;
                let saved_line_term = self.lexer.line_terminator_before_token;
                let saved_token = self.lexer.current_token.clone();

                self.lexer.next_token()?;
                // Check if followed by a valid property name
                if self.is_property_name()? {
                    kind = MethodKind::Get;
                    // Now parse the actual key - check for private identifier
                    if self.check_punctuator(Punctuator::LBracket)? {
                        // Computed property name - 'in' is always allowed
                        self.lexer.next_token()?;
                        let prev_in_for_init = self.in_for_init;
                        self.in_for_init = false;
                        let key_expr = self.parse_assignment_expression()?;
                        self.in_for_init = prev_in_for_init;
                        self.expect_punctuator(Punctuator::RBracket)?;
                        (PropertyKey::Computed(key_expr), true)
                    } else if self.check_private_identifier()? {
                        is_private = true;
                        let name = self.expect_private_identifier()?;
                        (PropertyKey::Identifier(name), false)
                    } else {
                        (self.parse_property_name()?, false)
                    }
                } else {
                    // It's a method named "get"
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.previous_line = saved_previous_line;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = saved_token;
                    (self.parse_property_name()?, false)
                }
            } else if !is_generator && self.check_identifier("set")? {
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_previous_line = self.lexer.previous_line;
                let saved_line_term = self.lexer.line_terminator_before_token;
                let saved_token = self.lexer.current_token.clone();

                self.lexer.next_token()?;
                // Check if followed by a valid property name
                if self.is_property_name()? {
                    kind = MethodKind::Set;
                    // Now parse the actual key - check for private identifier
                    if self.check_punctuator(Punctuator::LBracket)? {
                        // Computed property name - 'in' is always allowed
                        self.lexer.next_token()?;
                        let prev_in_for_init = self.in_for_init;
                        self.in_for_init = false;
                        let key_expr = self.parse_assignment_expression()?;
                        self.in_for_init = prev_in_for_init;
                        self.expect_punctuator(Punctuator::RBracket)?;
                        (PropertyKey::Computed(key_expr), true)
                    } else if self.check_private_identifier()? {
                        is_private = true;
                        let name = self.expect_private_identifier()?;
                        (PropertyKey::Identifier(name), false)
                    } else {
                        (self.parse_property_name()?, false)
                    }
                } else {
                    // It's a method named "set"
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.previous_line = saved_previous_line;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = saved_token;
                    (self.parse_property_name()?, false)
                }
            } else {
                (self.parse_property_name()?, false)
            };

            // Check for constructor (both identifier and string literal "constructor")
            let constructor_name = match &key {
                PropertyKey::Identifier(ref name) => Some(name.as_str()),
                PropertyKey::String(ref s) => Some(s.as_str()),
                _ => None,
            };
            if let Some(name) = constructor_name {
                if name == "constructor" && !is_static {
                    kind = MethodKind::Constructor;
                }
            }

            if self.check_punctuator(Punctuator::LParen)? {
                // Check for duplicate constructor (only applies to methods)
                if kind == MethodKind::Constructor {
                    if has_constructor {
                        return Err(syntax_error(
                            "A class may only have one constructor",
                            self.last_position.clone(),
                        ));
                    }
                    has_constructor = true;
                }
                // Check for duplicate private names (methods)
                if is_private {
                    if let PropertyKey::Identifier(ref name) = key {
                        if kind == MethodKind::Get {
                            // Getter: conflicts with fields/methods and duplicate getters
                            if private_fields_methods.contains(name) || private_getters.contains(name) {
                                return Err(syntax_error(
                                    "Duplicate private name",
                                    self.last_position.clone(),
                                ));
                            }
                            private_getters.insert(name.clone());
                        } else if kind == MethodKind::Set {
                            // Setter: conflicts with fields/methods and duplicate setters
                            if private_fields_methods.contains(name) || private_setters.contains(name) {
                                return Err(syntax_error(
                                    "Duplicate private name",
                                    self.last_position.clone(),
                                ));
                            }
                            private_setters.insert(name.clone());
                        } else {
                            // Method: conflicts with everything
                            if private_fields_methods.contains(name) || private_getters.contains(name) || private_setters.contains(name) {
                                return Err(syntax_error(
                                    "Duplicate private name",
                                    self.last_position.clone(),
                                ));
                            }
                            private_fields_methods.insert(name.clone());
                        }
                    }
                }
                // Method - set class context for super validation
                let prev_in_class_method = self.in_class_method;
                let prev_in_constructor = self.in_constructor;
                let prev_static_block = self.in_static_block;
                let prev_in_async = self.in_async;
                let prev_in_generator = self.in_generator;
                self.in_class_method = true;
                self.in_constructor = kind == MethodKind::Constructor;
                // Reset static block context - method parameters and body have their own scope
                // 'await' should be allowed as identifier in non-async methods
                self.in_static_block = false;
                // Set async/generator context for parameter parsing
                self.in_async = is_async;
                self.in_generator = is_generator;

                let params = self.parse_parameters()?;
                // Validate for duplicate parameters in non-simple params
                self.validate_parameters(&params)?;
                // Validate no await expressions in formal parameters (early error for async methods)
                if is_async {
                    self.validate_params_no_await(&params)?;
                }
                // Validate no yield expressions in formal parameters (early error for generator methods)
                if is_generator {
                    self.validate_params_no_yield(&params)?;
                }
                let body = self.parse_function_body_with_context(is_async, is_generator)?;
                // Validate use strict with non-simple parameters
                self.validate_params_with_body(&params, &body)?;

                self.in_class_method = prev_in_class_method;
                self.in_constructor = prev_in_constructor;
                self.in_static_block = prev_static_block;
                self.in_async = prev_in_async;
                self.in_generator = prev_in_generator;

                elements.push(ClassElement::MethodDefinition {
                    key,
                    kind,
                    value: Expression::FunctionExpression {
                        name: None,
                        params,
                        body,
                        is_async,
                        is_generator,
                        position: None,
                    },
                    is_static,
                    is_private,
                    computed,
                });
            } else {
                // Property (class field)
                // Early error: Field named "constructor" is forbidden
                // Static field named "prototype" or "constructor" is forbidden
                if !computed {
                    let field_name = match &key {
                        PropertyKey::Identifier(name) => Some(name.as_str()),
                        PropertyKey::String(s) => Some(s.as_str()),
                        _ => None,
                    };
                    if let Some(name) = field_name {
                        if name == "constructor" {
                            return Err(syntax_error(
                                "Class fields cannot be named 'constructor'",
                                self.last_position.clone(),
                            ));
                        }
                        if is_static && name == "prototype" {
                            return Err(syntax_error(
                                "Static class fields cannot be named 'prototype'",
                                self.last_position.clone(),
                            ));
                        }
                    }
                }
                // Check for duplicate private names (fields)
                if is_private {
                    if let PropertyKey::Identifier(ref name) = key {
                        // Field: conflicts with everything
                        if private_fields_methods.contains(name) || private_getters.contains(name) || private_setters.contains(name) {
                            return Err(syntax_error(
                                "Duplicate private name",
                                self.last_position.clone(),
                            ));
                        }
                        private_fields_methods.insert(name.clone());
                    }
                }
                let value = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    // Class field initializers allow super.property access (like methods)
                    // Arrow functions in field initializers inherit this super binding
                    let prev_in_method = self.in_method;
                    self.in_method = true;
                    let init_expr = self.parse_assignment_expression()?;
                    self.in_method = prev_in_method;
                    // Early error: ContainsArguments of Initializer is true
                    // This checks that `arguments` is not used in the field initializer
                    // (except inside regular functions, which have their own arguments binding)
                    if Self::expression_contains_arguments(&init_expr) {
                        return Err(syntax_error(
                            "'arguments' is not allowed in class field initializer",
                            self.last_position.clone(),
                        ));
                    }
                    Some(init_expr)
                } else {
                    None
                };
                elements.push(ClassElement::PropertyDefinition {
                    key,
                    value,
                    is_static,
                    is_private,
                    computed,
                });
                self.consume_semicolon()?;
            }
        }

        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(elements)
    }

    fn parse_class_expression(&mut self) -> Result<Expression, JsError> {
        self.expect_keyword(Keyword::Class)?;

        // All parts of a class are strict mode code, including the class name
        let prev_strict = self.strict_mode;
        self.strict_mode = true;

        // Name is optional for class expressions
        // await/yield can be class names when not in async/generator/static block context
        let name = match self.lexer.peek_token()? {
            Token::Identifier(_, _) => Some(self.expect_identifier()?),
            Token::Keyword(Keyword::Await) if !self.in_async && !self.in_static_block => Some(self.expect_identifier()?),
            Token::Keyword(Keyword::Yield) if !self.in_generator => Some(self.expect_identifier()?),
            _ => None,
        };

        let super_class = if self.check_keyword(Keyword::Extends)? {
            self.lexer.next_token()?;
            let heritage = self.parse_left_hand_side_expression()?;
            // ArrowFunctionExpression is not a valid LeftHandSideExpression
            // It's an AssignmentExpression, so reject it here
            if matches!(heritage, Expression::ArrowFunctionExpression { .. }) {
                return Err(syntax_error(
                    "Arrow function is not allowed as class heritage",
                    self.last_position.clone(),
                ));
            }
            Some(Box::new(heritage))
        } else {
            None
        };

        // Set heritage flag for super() validation in constructor
        let prev_has_heritage = self.has_class_heritage;
        self.has_class_heritage = super_class.is_some();

        let body = self.parse_class_body()?;

        self.has_class_heritage = prev_has_heritage;
        self.strict_mode = prev_strict;

        Ok(Expression::ClassExpression {
            name,
            super_class,
            body,
            position: None,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Pattern>, JsError> {
        self.expect_punctuator(Punctuator::LParen)?;
        let mut params = Vec::new();

        while !self.check_punctuator(Punctuator::RParen)? {
            if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                // Rest parameters cannot have default values
                if self.check_punctuator(Punctuator::Assign)? {
                    return Err(syntax_error(
                        "Rest parameter may not have a default initializer",
                        self.last_position.clone(),
                    ));
                }
                params.push(Pattern::RestElement(Box::new(pattern)));
                break;
            }

            let pattern = self.parse_pattern()?;
            // Check for default value
            let final_pattern = if self.check_punctuator(Punctuator::Assign)? {
                self.lexer.next_token()?;
                let default_value = self.parse_assignment_expression()?;
                Pattern::AssignmentPattern {
                    left: Box::new(pattern),
                    right: Box::new(default_value),
                }
            } else {
                pattern
            };
            params.push(final_pattern);

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RParen)?;
        Ok(params)
    }

    fn parse_function_body(&mut self) -> Result<Vec<Statement>, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut statements = Vec::new();

        // Check for directive prologue at the start of function body
        let prev_strict = self.strict_mode;
        self.check_function_directive_prologue()?;

        // Save and clear label sets - labels don't cross function boundaries
        let prev_active_labels = std::mem::take(&mut self.active_labels);
        let prev_iteration_labels = std::mem::take(&mut self.iteration_labels);

        // Save and reset loop depth - loops don't cross function boundaries
        let prev_loop_depth = self.loop_depth;
        self.loop_depth = 0;

        while !self.check_punctuator(Punctuator::RBrace)? {
            statements.push(self.parse_statement()?);
        }

        self.expect_punctuator(Punctuator::RBrace)?;

        // Restore strict mode and labels after function body
        self.strict_mode = prev_strict;
        self.active_labels = prev_active_labels;
        self.iteration_labels = prev_iteration_labels;
        self.loop_depth = prev_loop_depth;

        Ok(statements)
    }

    /// Check for directive prologue in function body
    fn check_function_directive_prologue(&mut self) -> Result<(), JsError> {
        // Look for string literal expression statements at the start
        while !self.check_punctuator(Punctuator::RBrace)? {
            let token = self.lexer.peek_token()?.clone();

            // Check if it's a string literal that could be a directive
            if let Token::String(ref s) = token {
                let is_use_strict = s == "use strict";

                // Peek ahead to see if this is a statement (followed by ; or newline)
                // Save position for potential rollback
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_line_term = self.lexer.line_terminator_before_token;
                let saved_token = self.lexer.current_token.clone();

                self.lexer.next_token()?; // consume string

                // Check if it's a directive (ends with semicolon or has ASI)
                let is_directive = self.check_punctuator(Punctuator::Semicolon)?
                    || self.check_punctuator(Punctuator::RBrace)?
                    || self.lexer.line_terminator_before_token;

                // Restore position - we'll parse normally
                self.lexer.position = saved_pos;
                self.lexer.line = saved_line;
                self.lexer.column = saved_column;
                self.lexer.line_terminator_before_token = saved_line_term;
                self.lexer.current_token = saved_token;

                if is_directive && is_use_strict {
                    self.strict_mode = true;
                }

                if !is_directive {
                    // Not a directive, stop looking
                    break;
                }

                // Parse the directive as a normal statement and continue checking
                // Actually, let the main loop handle it - just break after setting strict mode
                break;
            } else {
                // No more potential directives
                break;
            }
        }
        Ok(())
    }

    /// Parse function body with async/generator context tracking
    fn parse_function_body_with_context(
        &mut self,
        is_async: bool,
        is_generator: bool,
    ) -> Result<Vec<Statement>, JsError> {
        let prev_async = self.in_async;
        let prev_generator = self.in_generator;
        let prev_static_block = self.in_static_block;

        self.in_async = is_async;
        self.in_generator = is_generator;
        // Reset static block context - nested functions have their own scope
        // and 'await' should be allowed as identifier in non-async nested functions
        self.in_static_block = false;

        let body = self.parse_function_body()?;

        self.in_async = prev_async;
        self.in_generator = prev_generator;
        self.in_static_block = prev_static_block;

        Ok(body)
    }

    /// Parse method body (sets in_method flag for super access)
    fn parse_method_body(&mut self) -> Result<Vec<Statement>, JsError> {
        let prev_method = self.in_method;
        let prev_static_block = self.in_static_block;
        self.in_method = true;
        // Reset static block context - nested methods have their own scope
        self.in_static_block = false;

        let body = self.parse_function_body()?;

        self.in_method = prev_method;
        self.in_static_block = prev_static_block;
        Ok(body)
    }

    /// Parse method body with async/generator context tracking
    fn parse_method_body_with_context(
        &mut self,
        is_async: bool,
        is_generator: bool,
    ) -> Result<Vec<Statement>, JsError> {
        let prev_async = self.in_async;
        let prev_generator = self.in_generator;
        let prev_method = self.in_method;
        let prev_static_block = self.in_static_block;

        self.in_async = is_async;
        self.in_generator = is_generator;
        self.in_method = true;
        // Reset static block context - nested methods have their own scope
        self.in_static_block = false;

        let body = self.parse_function_body()?;

        self.in_async = prev_async;
        self.in_generator = prev_generator;
        self.in_method = prev_method;
        self.in_static_block = prev_static_block;

        Ok(body)
    }

    fn parse_return_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Return)?;

        // ASI Restricted Production: If there's a line terminator after 'return',
        // treat it as 'return;' with no expression (per ECMAScript 12.9.1)
        let argument = if self.check_punctuator(Punctuator::Semicolon)?
            || self.is_at_end()?
            || self.check_punctuator(Punctuator::RBrace)?
            || self.check_restricted_production()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume_semicolon()?;

        Ok(Statement::ReturnStatement {
            argument,
            position: None,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::If)?;
        self.expect_punctuator(Punctuator::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punctuator(Punctuator::RParen)?;

        let consequent = Box::new(self.parse_substatement()?);

        let alternate = if self.check_keyword(Keyword::Else)? {
            self.lexer.next_token()?;
            Some(Box::new(self.parse_substatement()?))
        } else {
            None
        };

        Ok(Statement::IfStatement {
            test,
            consequent,
            alternate,
            position: None,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator(Punctuator::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punctuator(Punctuator::RParen)?;

        self.loop_depth += 1;
        let body = Box::new(self.parse_substatement()?);
        self.loop_depth -= 1;

        Ok(Statement::WhileStatement {
            test,
            body,
            position: None,
        })
    }

    fn parse_do_while_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Do)?;
        self.loop_depth += 1;
        let body = Box::new(self.parse_substatement()?);
        self.loop_depth -= 1;
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator(Punctuator::LParen)?;
        let test = self.parse_expression()?;
        self.expect_punctuator(Punctuator::RParen)?;
        // Use special ASI handling for do-while per ECMAScript 12.9.1
        self.consume_semicolon_do_while()?;
        Ok(Statement::DoWhileStatement {
            body,
            test,
            position: None,
        })
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, JsError> {
        use crate::ast::SwitchCase;
        self.expect_keyword(Keyword::Switch)?;
        self.expect_punctuator(Punctuator::LParen)?;
        let discriminant = self.parse_expression()?;
        self.expect_punctuator(Punctuator::RParen)?;
        self.expect_punctuator(Punctuator::LBrace)?;

        let mut cases = Vec::new();
        self.loop_depth += 1; // Allow break inside switch

        while !self.check_punctuator(Punctuator::RBrace)? {
            let test = if self.check_keyword(Keyword::Case)? {
                self.lexer.next_token()?;
                Some(self.parse_expression()?)
            } else if self.check_keyword(Keyword::Default)? {
                self.lexer.next_token()?;
                None
            } else {
                return Err(syntax_error(
                    "Expected 'case' or 'default'",
                    self.last_position.clone(),
                ));
            };
            self.expect_punctuator(Punctuator::Colon)?;

            let mut consequent = Vec::new();
            while !self.check_punctuator(Punctuator::RBrace)?
                && !self.check_keyword(Keyword::Case)?
                && !self.check_keyword(Keyword::Default)?
            {
                consequent.push(self.parse_statement()?);
            }

            cases.push(SwitchCase { test, consequent });
        }

        self.loop_depth -= 1;
        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(Statement::SwitchStatement {
            discriminant,
            cases,
            position: None,
        })
    }

    fn parse_with_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::With)?;
        self.expect_punctuator(Punctuator::LParen)?;
        let object = self.parse_expression()?;
        self.expect_punctuator(Punctuator::RParen)?;
        let body = Box::new(self.parse_substatement()?);
        Ok(Statement::WithStatement {
            object,
            body,
            position: None,
        })
    }

    fn parse_debugger_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Debugger)?;
        self.consume_semicolon()?;
        Ok(Statement::DebuggerStatement { position: None })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::For)?;

        // Check for for-await-of: for await (... of ...)
        let is_await = if self.check_keyword(Keyword::Await)? {
            if !self.in_async {
                return Err(syntax_error(
                    "for await can only be used in async functions",
                    self.last_position.clone(),
                ));
            }
            self.lexer.next_token()?; // consume await
            true
        } else {
            false
        };

        self.expect_punctuator(Punctuator::LParen)?;

        // Check for for-in/for-of first
        if self.check_punctuator(Punctuator::Semicolon)? {
            if is_await {
                return Err(syntax_error(
                    "for await must be used with for-of, not regular for loop",
                    self.last_position.clone(),
                ));
            }
            // Empty init - regular for loop
            return self.parse_regular_for(None);
        }

        // Special case: for (let ...) - "let" can be an identifier or keyword
        // Per spec: for ( [lookahead ∉ { let [ }] LeftHandSideExpression in Expression )
        // "let" is only a keyword if followed by: identifier (not 'in'/'of'), [, or {
        // "let" is an identifier if followed by: ;, =, ,, in, of, ), or other operators
        let is_let_as_keyword = if self.check_keyword(Keyword::Let)? && !self.strict_mode {
            // Peek ahead to see what follows "let"
            let saved_pos = self.lexer.position;
            let saved_line = self.lexer.line;
            let saved_column = self.lexer.column;
            let saved_previous_line = self.lexer.previous_line;
            let saved_line_term = self.lexer.line_terminator_before_token;
            let saved_token = self.lexer.current_token.clone();

            self.lexer.next_token()?; // consume "let"
            let next = self.lexer.peek_token()?;
            // "let" is a keyword if followed by identifier (not in/of), [, or {
            let is_keyword = matches!(
                next,
                Token::Identifier(_, _)
                    | Token::Punctuator(Punctuator::LBracket)
                    | Token::Punctuator(Punctuator::LBrace)
            );

            // Restore lexer state
            self.lexer.position = saved_pos;
            self.lexer.line = saved_line;
            self.lexer.column = saved_column;
            self.lexer.previous_line = saved_previous_line;
            self.lexer.line_terminator_before_token = saved_line_term;
            self.lexer.current_token = saved_token;

            is_keyword
        } else if self.check_keyword(Keyword::Let)? && self.strict_mode {
            // In strict mode, 'let' is always a keyword
            true
        } else {
            false
        };

        // Parse left side
        if (self.check_keyword(Keyword::Let)? && is_let_as_keyword)
            || self.check_keyword(Keyword::Const)?
            || self.check_keyword(Keyword::Var)?
        {
            let kind = match self.lexer.next_token()? {
                Token::Keyword(Keyword::Let) => VariableKind::Let,
                Token::Keyword(Keyword::Const) => VariableKind::Const,
                Token::Keyword(Keyword::Var) => VariableKind::Var,
                _ => unreachable!(),
            };
            let id = self.parse_pattern()?;

            // Check for in/of (for-in/for-of) vs semicolon (regular for)
            if self.check_keyword(Keyword::In)? {
                if is_await {
                    return Err(syntax_error(
                        "for await must be used with for-of, not for-in",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?; // consume 'in'
                let right = self.parse_expression()?;
                self.expect_punctuator(Punctuator::RParen)?;
                self.loop_depth += 1;
                let body = Box::new(self.parse_substatement()?);
                self.loop_depth -= 1;
                return Ok(Statement::ForInStatement {
                    left: ForInOfLeft::VariableDeclaration { kind, id },
                    right,
                    body,
                    position: None,
                });
            }

            if self.check_identifier("of")? {
                self.lexer.next_token()?; // consume 'of'
                let right = self.parse_assignment_expression()?;
                self.expect_punctuator(Punctuator::RParen)?;
                self.loop_depth += 1;
                let body = Box::new(self.parse_substatement()?);
                self.loop_depth -= 1;
                return Ok(Statement::ForOfStatement {
                    left: ForInOfLeft::VariableDeclaration { kind, id },
                    right,
                    body,
                    r#await: is_await,
                    position: None,
                });
            }

            // Regular for loop with variable declaration
            if is_await {
                return Err(syntax_error(
                    "for await must be used with for-of, not regular for loop",
                    self.last_position.clone(),
                ));
            }
            // Parse all declarators (there may be multiple: var i = 0, j = 1)
            let mut declarations = Vec::new();

            // Set flag to disallow 'in' as relational operator in for loop init
            let prev_in_for_init = self.in_for_init;
            self.in_for_init = true;

            let init_expr = if self.check_punctuator(Punctuator::Assign)? {
                self.lexer.next_token()?;
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };
            declarations.push(VariableDeclarator { id, init: init_expr });

            // Parse additional declarators
            while self.check_punctuator(Punctuator::Comma)? {
                self.lexer.next_token()?;
                let id = self.parse_pattern()?;
                let init_expr = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    Some(self.parse_assignment_expression()?)
                } else {
                    None
                };
                declarations.push(VariableDeclarator { id, init: init_expr });
            }

            self.in_for_init = prev_in_for_init;

            let init = Some(ForInit::VariableDeclaration { kind, declarations });
            return self.parse_regular_for(init);
        }

        // Expression as left side - could be for-in/for-of or regular for
        let left_expr = self.parse_left_hand_side_expression()?;

        // Check for in/of
        if self.check_keyword(Keyword::In)? {
            if is_await {
                return Err(syntax_error(
                    "for await must be used with for-of, not for-in",
                    self.last_position.clone(),
                ));
            }
            // In strict mode, reject invalid assignment targets like call expressions
            if self.strict_mode {
                self.validate_for_in_of_left(&left_expr)?;
            }
            self.lexer.next_token()?; // consume 'in'
            let right = self.parse_expression()?;
            self.expect_punctuator(Punctuator::RParen)?;
            self.loop_depth += 1;
            let body = Box::new(self.parse_substatement()?);
            self.loop_depth -= 1;

            // Convert expression to pattern if possible, otherwise keep as expression
            let left = match self.expression_to_pattern(left_expr.clone()) {
                Ok(pattern) => ForInOfLeft::Pattern(pattern),
                Err(_) => ForInOfLeft::Expression(left_expr),
            };
            return Ok(Statement::ForInStatement {
                left,
                right,
                body,
                position: None,
            });
        }

        if self.check_identifier("of")? {
            // In strict mode, reject invalid assignment targets like call expressions
            if self.strict_mode {
                self.validate_for_in_of_left(&left_expr)?;
            }
            self.lexer.next_token()?; // consume 'of'
            let right = self.parse_assignment_expression()?;
            self.expect_punctuator(Punctuator::RParen)?;
            self.loop_depth += 1;
            let body = Box::new(self.parse_substatement()?);
            self.loop_depth -= 1;

            // Convert expression to pattern if possible, otherwise keep as expression
            let left = match self.expression_to_pattern(left_expr.clone()) {
                Ok(pattern) => ForInOfLeft::Pattern(pattern),
                Err(_) => ForInOfLeft::Expression(left_expr),
            };
            return Ok(Statement::ForOfStatement {
                left,
                right,
                body,
                r#await: is_await,
                position: None,
            });
        }

        // Regular for loop with expression init
        if is_await {
            return Err(syntax_error(
                "for await must be used with for-of, not regular for loop",
                self.last_position.clone(),
            ));
        }

        // The left_expr may just be the LHS of an assignment expression
        // We need to finish parsing the full expression
        let init_expr = self.finish_expression_from_lhs(left_expr)?;
        self.parse_regular_for(Some(ForInit::Expression(init_expr)))
    }

    /// Parse the rest of a regular for loop after init is determined
    fn parse_regular_for(&mut self, init: Option<ForInit>) -> Result<Statement, JsError> {
        self.expect_punctuator(Punctuator::Semicolon)?;

        let test = if self.check_punctuator(Punctuator::Semicolon)? {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.expect_punctuator(Punctuator::Semicolon)?;

        let update = if self.check_punctuator(Punctuator::RParen)? {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.expect_punctuator(Punctuator::RParen)?;

        self.loop_depth += 1;
        let body = Box::new(self.parse_substatement()?);
        self.loop_depth -= 1;

        Ok(Statement::ForStatement {
            init,
            test,
            update,
            body,
            position: None,
        })
    }

    fn parse_break_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Break)?;

        // Peek to update line_terminator_before_token
        let next_token = self.lexer.peek_token()?.clone();

        // ASI restricted production: break can have an optional label on same line
        // If there's a line terminator, treat as break with no label
        let label = if !self.lexer.line_terminator_before_token {
            // Check for optional label
            if let Token::Identifier(name, _) = next_token {
                self.lexer.next_token()?;
                Some(name)
            } else {
                None
            }
        } else {
            None
        };

        // Break without label must be inside a loop or switch
        if label.is_none() && self.loop_depth == 0 {
            return Err(syntax_error(
                "Illegal break statement",
                self.last_position.clone(),
            ));
        }

        // Break with label must target an enclosing label
        if let Some(ref label_name) = label {
            if !self.active_labels.contains(label_name) {
                return Err(syntax_error(
                    &format!("Undefined label '{}'", label_name),
                    self.last_position.clone(),
                ));
            }
        }

        self.consume_semicolon()?;
        Ok(Statement::BreakStatement {
            label,
            position: None,
        })
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Continue)?;

        // Peek to update line_terminator_before_token
        let next_token = self.lexer.peek_token()?.clone();

        // ASI restricted production: continue can have an optional label on same line
        // If there's a line terminator, treat as continue with no label
        let label = if !self.lexer.line_terminator_before_token {
            // Check for optional label
            if let Token::Identifier(name, _) = next_token {
                self.lexer.next_token()?;
                Some(name)
            } else {
                None
            }
        } else {
            None
        };

        // Continue must be inside a loop
        if self.loop_depth == 0 {
            return Err(syntax_error(
                "Illegal continue statement",
                self.last_position.clone(),
            ));
        }

        // Continue with label must target an enclosing iteration label
        if let Some(ref label_name) = label {
            if !self.iteration_labels.contains(label_name) {
                // Label exists but is not an iteration statement
                if self.active_labels.contains(label_name) {
                    return Err(syntax_error(
                        &format!("Label '{}' is not an iteration statement", label_name),
                        self.last_position.clone(),
                    ));
                } else {
                    return Err(syntax_error(
                        &format!("Undefined label '{}'", label_name),
                        self.last_position.clone(),
                    ));
                }
            }
        }

        self.consume_semicolon()?;
        Ok(Statement::ContinueStatement {
            label,
            position: None,
        })
    }

    fn parse_throw_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Throw)?;

        // Peek the next token to ensure line terminator state is updated
        let _ = self.lexer.peek_token()?;

        // ASI Restricted Production: Throw MUST have an expression on the same line
        // A line terminator between 'throw' and expression is a syntax error
        if self.lexer.line_terminator_before_token {
            return Err(JsError {
                kind: core_types::ErrorKind::SyntaxError,
                message: "Illegal newline after throw".to_string(),
                stack: vec![],
                source_position: self.last_position.clone(),
            });
        }

        let argument = self.parse_expression()?;
        self.consume_semicolon()?;
        Ok(Statement::ThrowStatement {
            argument,
            position: None,
        })
    }

    fn parse_try_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Try)?;
        let block = self.parse_block_body()?;

        let handler = if self.check_keyword(Keyword::Catch)? {
            self.lexer.next_token()?;
            let param = if self.check_punctuator(Punctuator::LParen)? {
                self.lexer.next_token()?;
                let p = self.parse_pattern()?;
                self.expect_punctuator(Punctuator::RParen)?;
                Some(p)
            } else {
                None
            };
            let body = self.parse_block_body()?;
            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.check_keyword(Keyword::Finally)? {
            self.lexer.next_token()?;
            Some(self.parse_block_body()?)
        } else {
            None
        };

        Ok(Statement::TryStatement {
            block,
            handler,
            finalizer,
            position: None,
        })
    }

    fn parse_block_statement(&mut self) -> Result<Statement, JsError> {
        let body = self.parse_block_body()?;
        Ok(Statement::BlockStatement {
            body,
            position: None,
        })
    }

    fn parse_block_body(&mut self) -> Result<Vec<Statement>, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut statements = Vec::new();
        let mut lexical_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        // First pass: parse all statements and collect lexical names (but not var names yet)
        while !self.check_punctuator(Punctuator::RBrace)? {
            let stmt = self.parse_statement()?;

            // Check for duplicate lexical declarations (let/const/function/class at this level)
            Self::check_lexical_declaration(&stmt, &mut lexical_names, &self.last_position)?;

            statements.push(stmt);
        }

        self.expect_punctuator(Punctuator::RBrace)?;

        // Second pass: collect ALL var names (including from nested blocks) and check against lexical names
        let mut var_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        Self::collect_var_declared_names(&statements, &mut var_names);

        // Check var names against lexical names
        for var_name in &var_names {
            if lexical_names.contains(var_name) {
                return Err(syntax_error(
                    &format!("Identifier '{}' has already been declared", var_name),
                    None,
                ));
            }
        }

        Ok(statements)
    }

    /// Check a statement for lexical declarations and detect duplicates
    fn check_lexical_declaration(
        stmt: &Statement,
        lexical_names: &mut std::collections::HashSet<String>,
        position: &Option<core_types::SourcePosition>,
    ) -> Result<(), JsError> {
        match stmt {
            // const and let declarations
            Statement::VariableDeclaration { kind, declarations, .. } => {
                if matches!(kind, crate::ast::VariableKind::Let | crate::ast::VariableKind::Const) {
                    for decl in declarations {
                        let mut names = Vec::new();
                        Self::collect_bound_names(&decl.id, &mut names);
                        for name in names {
                            // Check against existing lexical names
                            if lexical_names.contains(&name) {
                                return Err(syntax_error(
                                    &format!("Identifier '{}' has already been declared", name),
                                    position.clone(),
                                ));
                            }
                            lexical_names.insert(name);
                        }
                    }
                }
            }
            // Function declarations are lexically scoped in blocks
            Statement::FunctionDeclaration { name, .. } => {
                if lexical_names.contains(name) {
                    return Err(syntax_error(
                        &format!("Identifier '{}' has already been declared", name),
                        position.clone(),
                    ));
                }
                lexical_names.insert(name.clone());
            }
            // Class declarations
            Statement::ClassDeclaration { name, .. } => {
                if lexical_names.contains(name) {
                    return Err(syntax_error(
                        &format!("Identifier '{}' has already been declared", name),
                        position.clone(),
                    ));
                }
                lexical_names.insert(name.clone());
            }
            _ => {}
        }
        Ok(())
    }

    /// Recursively collect VarDeclaredNames from statements (including nested blocks)
    fn collect_var_declared_names(statements: &[Statement], var_names: &mut std::collections::HashSet<String>) {
        for stmt in statements {
            Self::collect_var_declared_names_stmt(stmt, var_names);
        }
    }

    /// Collect VarDeclaredNames from a single statement (including nested blocks)
    fn collect_var_declared_names_stmt(stmt: &Statement, var_names: &mut std::collections::HashSet<String>) {
        match stmt {
            // Var declarations contribute their names
            Statement::VariableDeclaration { kind, declarations, .. } => {
                if matches!(kind, crate::ast::VariableKind::Var) {
                    for decl in declarations {
                        let mut names = Vec::new();
                        Self::collect_bound_names(&decl.id, &mut names);
                        for name in names {
                            var_names.insert(name);
                        }
                    }
                }
            }
            // Block statements - var names from inside hoist out
            Statement::BlockStatement { body, .. } => {
                Self::collect_var_declared_names(body, var_names);
            }
            // If statements - check both branches
            Statement::IfStatement { consequent, alternate, .. } => {
                Self::collect_var_declared_names_stmt(consequent, var_names);
                if let Some(alt) = alternate {
                    Self::collect_var_declared_names_stmt(alt, var_names);
                }
            }
            // While/DoWhile - check body
            Statement::WhileStatement { body, .. } => {
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            Statement::DoWhileStatement { body, .. } => {
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            // For statement - check init and body
            Statement::ForStatement { init, body, .. } => {
                if let Some(init_val) = init {
                    Self::collect_var_declared_names_forinit(init_val, var_names);
                }
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            // ForIn/ForOf - check left and body
            Statement::ForInStatement { left, body, .. } => {
                Self::collect_var_declared_names_forinof_left(left, var_names);
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            Statement::ForOfStatement { left, body, .. } => {
                Self::collect_var_declared_names_forinof_left(left, var_names);
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            // Switch - check all case bodies
            Statement::SwitchStatement { cases, .. } => {
                for case in cases {
                    Self::collect_var_declared_names(&case.consequent, var_names);
                }
            }
            // Try - check all parts
            Statement::TryStatement { block, handler, finalizer, .. } => {
                Self::collect_var_declared_names(block, var_names);
                if let Some(h) = handler {
                    Self::collect_var_declared_names(&h.body, var_names);
                }
                if let Some(f) = finalizer {
                    Self::collect_var_declared_names(f, var_names);
                }
            }
            // With - check body
            Statement::WithStatement { body, .. } => {
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            // Labeled - check body
            Statement::LabeledStatement { body, .. } => {
                Self::collect_var_declared_names_stmt(body, var_names);
            }
            // Function declarations do NOT contribute var names (they're lexical in blocks)
            // NOTE: In function body context (not block), function declarations ARE var-scoped
            // But this function is for block-level VarDeclaredNames collection
            _ => {}
        }
    }

    /// Collect VarDeclaredNames from ForInOfLeft
    fn collect_var_declared_names_forinof_left(left: &crate::ast::ForInOfLeft, var_names: &mut std::collections::HashSet<String>) {
        if let crate::ast::ForInOfLeft::VariableDeclaration { kind, id } = left {
            if matches!(kind, crate::ast::VariableKind::Var) {
                let mut names = Vec::new();
                Self::collect_bound_names(id, &mut names);
                for name in names {
                    var_names.insert(name);
                }
            }
        }
    }

    /// Collect VarDeclaredNames from ForInit
    fn collect_var_declared_names_forinit(init: &crate::ast::ForInit, var_names: &mut std::collections::HashSet<String>) {
        if let crate::ast::ForInit::VariableDeclaration { kind, declarations } = init {
            if matches!(kind, crate::ast::VariableKind::Var) {
                for decl in declarations {
                    let mut names = Vec::new();
                    Self::collect_bound_names(&decl.id, &mut names);
                    for name in names {
                        var_names.insert(name);
                    }
                }
            }
        }
    }

    /// Check if a statement is an iteration statement or a labeled statement
    /// that eventually wraps an iteration statement
    fn is_iteration_labeled(stmt: &Statement) -> bool {
        match stmt {
            Statement::WhileStatement { .. }
            | Statement::DoWhileStatement { .. }
            | Statement::ForStatement { .. }
            | Statement::ForInStatement { .. }
            | Statement::ForOfStatement { .. } => true,
            Statement::LabeledStatement { body, .. } => Self::is_iteration_labeled(body),
            _ => false,
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, JsError> {
        // Check for labeled statement: identifier followed by colon
        if let Token::Identifier(name, _) = self.lexer.peek_token()?.clone() {
            // Save lexer state (same pattern as look_ahead_for_arrow in lexer)
            let saved_position = self.lexer.position;
            let saved_line = self.lexer.line;
            let saved_column = self.lexer.column;
            let saved_previous_line = self.lexer.previous_line;
            let saved_line_term = self.lexer.line_terminator_before_token;
            let saved_token = self.lexer.current_token.clone();

            self.lexer.next_token()?; // consume identifier

            if self.check_punctuator(Punctuator::Colon)? {
                // This is a labeled statement
                // Validate the label identifier (await/yield restrictions apply)
                self.validate_identifier(&name)?;

                // Check for duplicate label in current label set
                if self.active_labels.contains(&name) {
                    return Err(syntax_error(
                        &format!("Label '{}' has already been declared", name),
                        self.last_position.clone(),
                    ));
                }

                self.lexer.next_token()?; // consume ':'

                // Add label to active labels
                self.active_labels.insert(name.clone());

                // Check if the body starts an iteration statement
                // (while, do, for) - then this label is also an iteration label
                let is_iteration = self.check_keyword(Keyword::While)?
                    || self.check_keyword(Keyword::Do)?
                    || self.check_keyword(Keyword::For)?;

                if is_iteration {
                    self.iteration_labels.insert(name.clone());
                }

                // Check for disallowed declarations after labels:
                // - async function declarations
                // - generator function declarations
                // - class declarations
                // - lexical declarations (let, const)
                if self.check_keyword(Keyword::Class)? {
                    return Err(syntax_error(
                        "Class declaration is not allowed in statement position",
                        self.last_position.clone(),
                    ));
                }

                // Check for async function declarations after label
                if self.check_keyword(Keyword::Async)? {
                    // Peek to see if it's followed by 'function' (without line terminator)
                    let saved_pos = self.lexer.position;
                    let saved_line = self.lexer.line;
                    let saved_col = self.lexer.column;
                    let saved_line_term = self.lexer.line_terminator_before_token;
                    let saved_tok = self.lexer.current_token.clone();

                    self.lexer.next_token()?; // consume async
                    if !self.lexer.line_terminator_before_token && self.check_keyword(Keyword::Function)? {
                        return Err(syntax_error(
                            "Async function declaration is not allowed in statement position",
                            self.last_position.clone(),
                        ));
                    }

                    // Restore
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_col;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = saved_tok;
                }

                // Also check for nested labels that might wrap an iteration
                // e.g., label1: label2: while(...)
                // We'll determine this after parsing the body
                let body = Box::new(self.parse_statement()?);

                // Check if the parsed body is a disallowed declaration type
                // (catches generator functions which start with 'function *')
                match body.as_ref() {
                    Statement::FunctionDeclaration { is_generator: true, .. }
                    | Statement::FunctionDeclaration { is_async: true, .. } => {
                        return Err(syntax_error(
                            "Generator/async function declaration is not allowed in statement position",
                            self.last_position.clone(),
                        ));
                    }
                    // In strict mode, function declarations are not allowed after labels
                    Statement::FunctionDeclaration { .. } if self.strict_mode => {
                        return Err(syntax_error(
                            "Function declaration is not allowed in statement position in strict mode",
                            self.last_position.clone(),
                        ));
                    }
                    Statement::ClassDeclaration { .. } => {
                        return Err(syntax_error(
                            "Class declaration is not allowed in statement position",
                            self.last_position.clone(),
                        ));
                    }
                    Statement::VariableDeclaration { kind, .. }
                        if matches!(kind, crate::ast::VariableKind::Let | crate::ast::VariableKind::Const) =>
                    {
                        return Err(syntax_error(
                            "Lexical declaration is not allowed in statement position",
                            self.last_position.clone(),
                        ));
                    }
                    _ => {}
                }

                // If body is a labeled statement wrapping an iteration,
                // this label is also an iteration label
                if !is_iteration {
                    if Self::is_iteration_labeled(&body) {
                        self.iteration_labels.insert(name.clone());
                    }
                }

                // Remove labels after parsing
                self.active_labels.remove(&name);
                self.iteration_labels.remove(&name);

                return Ok(Statement::LabeledStatement {
                    label: name,
                    body,
                    position: None,
                });
            }

            // Not a labeled statement, restore lexer state
            self.lexer.position = saved_position;
            self.lexer.line = saved_line;
            self.lexer.column = saved_column;
            self.lexer.previous_line = saved_previous_line;
            self.lexer.line_terminator_before_token = saved_line_term;
            self.lexer.current_token = saved_token;
        }

        let expression = self.parse_expression()?;
        self.consume_semicolon()?;

        Ok(Statement::ExpressionStatement {
            expression,
            position: None,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, JsError> {
        // Expression can be comma-separated (SequenceExpression)
        let mut expr = self.parse_assignment_expression()?;

        if self.check_punctuator(Punctuator::Comma)? {
            let mut expressions = vec![expr];
            while self.check_punctuator(Punctuator::Comma)? {
                self.lexer.next_token()?;
                expressions.push(self.parse_assignment_expression()?);
            }
            expr = Expression::SequenceExpression {
                expressions,
                position: None,
            };
        }

        Ok(expr)
    }

    fn parse_assignment_expression(&mut self) -> Result<Expression, JsError> {
        // YieldExpression is at AssignmentExpression level, NOT at UnaryExpression level
        // This ensures `void yield` fails (yield cannot be identifier in generator)
        if self.in_generator && self.check_keyword(Keyword::Yield)? {
            return self.parse_yield_expression();
        }

        let expr = self.parse_conditional_expression()?;

        // Check for single-parameter arrow function: identifier => expr
        // After parsing an identifier, if next token is =>, this is an arrow function
        // Note: Line terminator before => is not allowed
        if let Expression::Identifier { ref name, .. } = expr {
            if self.check_punctuator(Punctuator::Arrow)? && !self.lexer.line_terminator_before_token {
                // Validate the parameter identifier (e.g., no 'arguments'/'eval' in strict mode)
                self.validate_binding_identifier(name)?;
                self.lexer.next_token()?; // consume =>
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::Identifier(name.clone())],
                    body,
                    is_async: false,
                    position: None,
                });
            }
        }

        if let Some(op) = self.check_assignment_operator()? {
            self.lexer.next_token()?;
            let right = Box::new(self.parse_assignment_expression()?);
            let left = self.expression_to_assignment_target(expr)?;
            return Ok(Expression::AssignmentExpression {
                left,
                operator: op,
                right,
                position: None,
            });
        }

        Ok(expr)
    }

    /// Parse a YieldExpression
    /// This is at AssignmentExpression level, NOT UnaryExpression level
    fn parse_yield_expression(&mut self) -> Result<Expression, JsError> {
        self.expect_keyword(Keyword::Yield)?;

        // Check for yield* (delegate)
        // The grammar is: yield [no LineTerminator here] * AssignmentExpression
        // So if there's a line terminator before *, it's NOT yield*
        let delegate = if self.check_punctuator(Punctuator::Star)?
            && !self.lexer.line_terminator_before_token
        {
            self.lexer.next_token()?;
            true
        } else {
            false
        };

        // Check if there's an argument (yield can be used without argument)
        // If there's a line terminator or the next token can't start an expression, no argument
        // Note: Colon is added for conditional expressions: `a ? yield : b` where yield has no argument
        let argument = if delegate {
            // yield* requires an argument
            Some(Box::new(self.parse_assignment_expression()?))
        } else if self.lexer.line_terminator_before_token
            || self.check_punctuator(Punctuator::Semicolon)?
            || self.check_punctuator(Punctuator::RBrace)?
            || self.check_punctuator(Punctuator::RParen)?
            || self.check_punctuator(Punctuator::RBracket)?
            || self.check_punctuator(Punctuator::Comma)?
            || self.check_punctuator(Punctuator::Colon)?
            || self.is_at_end()?
        {
            None
        } else {
            Some(Box::new(self.parse_assignment_expression()?))
        };

        Ok(Expression::YieldExpression {
            argument,
            delegate,
            position: None,
        })
    }

    /// Given a left-hand-side expression, finish parsing the full expression.
    /// This handles cases like `for (x = 1; ...)` where we've already parsed `x`
    /// but need to continue parsing `= 1` and any subsequent parts.
    /// Also handles binary operators and conditional expressions.
    fn finish_expression_from_lhs(&mut self, lhs: Expression) -> Result<Expression, JsError> {
        // We're in for loop init, so set the flag to disallow 'in' as relational operator
        let prev_in_for_init = self.in_for_init;
        self.in_for_init = true;

        // Continue parsing binary operators from the LHS
        // Start from lowest precedence (exponentiation) and work up
        let expr = self.continue_exponentiation_from_lhs(lhs)?;
        let expr = self.continue_multiplicative_from_lhs(expr)?;
        let expr = self.continue_additive_from_lhs(expr)?;
        let expr = self.continue_shift_from_lhs(expr)?;
        let expr = self.continue_relational_from_lhs(expr)?;
        let expr = self.continue_equality_from_lhs(expr)?;
        let expr = self.continue_bitwise_and_from_lhs(expr)?;
        let expr = self.continue_bitwise_xor_from_lhs(expr)?;
        let expr = self.continue_bitwise_or_from_lhs(expr)?;
        let expr = self.continue_logical_and_from_lhs(expr)?;
        let expr = self.continue_logical_or_from_lhs(expr)?;
        let expr = self.continue_nullish_from_lhs(expr)?;

        // Handle conditional expression
        let expr = if self.check_punctuator(Punctuator::Question)? {
            self.lexer.next_token()?;
            // In conditional consequent, 'in' IS allowed
            self.in_for_init = false;
            let consequent = Box::new(self.parse_assignment_expression()?);
            self.in_for_init = true;
            self.expect_punctuator(Punctuator::Colon)?;
            let alternate = Box::new(self.parse_assignment_expression()?);
            Expression::ConditionalExpression {
                test: Box::new(expr),
                consequent,
                alternate,
                position: None,
            }
        } else {
            expr
        };

        // Handle assignment operator
        let expr = if let Some(op) = self.check_assignment_operator()? {
            self.lexer.next_token()?;
            let right = Box::new(self.parse_assignment_expression()?);
            let left = self.expression_to_assignment_target(expr)?;
            Expression::AssignmentExpression {
                left,
                operator: op,
                right,
                position: None,
            }
        } else {
            expr
        };

        self.in_for_init = prev_in_for_init;

        // Then handle comma operator (sequence expression)
        if self.check_punctuator(Punctuator::Comma)? {
            self.lexer.next_token()?;
            let rest = self.parse_expression()?;
            // Flatten nested sequence expressions
            let mut expressions = vec![expr];
            match rest {
                Expression::SequenceExpression { expressions: mut es, .. } => {
                    expressions.append(&mut es);
                }
                e => expressions.push(e),
            }
            Ok(Expression::SequenceExpression {
                expressions,
                position: None,
            })
        } else {
            Ok(expr)
        }
    }

    // Helper functions to continue parsing binary operators from an existing LHS

    fn continue_exponentiation_from_lhs(&mut self, lhs: Expression) -> Result<Expression, JsError> {
        if self.check_punctuator(Punctuator::StarStar)? {
            self.lexer.next_token()?;
            let right = self.parse_exponentiation_expression()?;
            Ok(Expression::BinaryExpression {
                left: Box::new(lhs),
                operator: BinaryOperator::Exp,
                right: Box::new(right),
                position: None,
            })
        } else {
            Ok(lhs)
        }
    }

    fn continue_multiplicative_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Star) => BinaryOperator::Mul,
                Token::Punctuator(Punctuator::Slash) => BinaryOperator::Div,
                Token::Punctuator(Punctuator::Percent) => BinaryOperator::Mod,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_exponentiation_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_additive_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Plus) => BinaryOperator::Add,
                Token::Punctuator(Punctuator::Minus) => BinaryOperator::Sub,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_multiplicative_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_shift_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::LtLt) => BinaryOperator::LeftShift,
                Token::Punctuator(Punctuator::GtGt) => BinaryOperator::RightShift,
                Token::Punctuator(Punctuator::GtGtGt) => BinaryOperator::UnsignedRightShift,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_additive_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_relational_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Lt) => BinaryOperator::Lt,
                Token::Punctuator(Punctuator::LtEq) => BinaryOperator::LtEq,
                Token::Punctuator(Punctuator::Gt) => BinaryOperator::Gt,
                Token::Punctuator(Punctuator::GtEq) => BinaryOperator::GtEq,
                Token::Keyword(Keyword::Instanceof) => BinaryOperator::Instanceof,
                // In for loop init context, 'in' is not allowed as relational operator
                Token::Keyword(Keyword::In) if !self.in_for_init => BinaryOperator::In,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_shift_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_equality_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::EqEq) => BinaryOperator::Eq,
                Token::Punctuator(Punctuator::NotEq) => BinaryOperator::NotEq,
                Token::Punctuator(Punctuator::EqEqEq) => BinaryOperator::StrictEq,
                Token::Punctuator(Punctuator::NotEqEq) => BinaryOperator::StrictNotEq,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_relational_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_bitwise_and_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::And)? {
            self.lexer.next_token()?;
            let right = self.parse_equality_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseAnd,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_bitwise_xor_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::Xor)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_and_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseXor,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_bitwise_or_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::Or)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_xor_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseOr,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_logical_and_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::AndAnd)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_or_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::And,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_logical_or_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::OrOr)? {
            self.lexer.next_token()?;
            let right = self.parse_logical_and_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::Or,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn continue_nullish_from_lhs(&mut self, mut left: Expression) -> Result<Expression, JsError> {
        while self.check_punctuator(Punctuator::NullishCoalesce)? {
            self.lexer.next_token()?;
            let right = self.parse_logical_or_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::NullishCoalesce,
                right: Box::new(right),
                position: None,
            };
        }
        Ok(left)
    }

    fn expression_to_assignment_target(
        &self,
        expr: Expression,
    ) -> Result<AssignmentTarget, JsError> {
        match expr {
            Expression::Identifier { ref name, .. } => {
                // In strict mode, `arguments` and `eval` cannot be assignment targets
                if self.strict_mode && (name == "arguments" || name == "eval") {
                    return Err(syntax_error(
                        &format!("'{}' cannot be assigned in strict mode", name),
                        self.last_position.clone(),
                    ));
                }
                Ok(AssignmentTarget::Identifier(name.clone()))
            }
            Expression::MemberExpression { optional, .. } => {
                // Optional chaining expressions cannot be assignment targets
                if optional {
                    return Err(syntax_error(
                        "Invalid left-hand side in assignment: optional chaining not allowed",
                        self.last_position.clone(),
                    ));
                }
                Ok(AssignmentTarget::Member(Box::new(expr)))
            }
            Expression::CallExpression { .. } => {
                // Call expressions are never valid assignment targets
                Err(syntax_error("Call expression cannot be assigned", None))
            }
            // Handle array destructuring: [a, b] = value
            Expression::ArrayExpression { .. } => {
                let pattern = self.expression_to_pattern(expr)?;
                Ok(AssignmentTarget::Pattern(pattern))
            }
            // Handle object destructuring: {a, b} = value
            Expression::ObjectExpression { .. } => {
                let pattern = self.expression_to_pattern(expr)?;
                Ok(AssignmentTarget::Pattern(pattern))
            }
            // Handle parenthesized expressions
            // (x) = value is valid, but ({x}) = value and ([x]) = value are not
            Expression::ParenthesizedExpression { expression, .. } => {
                match *expression {
                    // Parenthesized identifiers are valid: (x) = 1
                    Expression::Identifier { ref name, .. } => {
                        if self.strict_mode && (name == "arguments" || name == "eval") {
                            return Err(syntax_error(
                                &format!("'{}' cannot be assigned in strict mode", name),
                                self.last_position.clone(),
                            ));
                        }
                        Ok(AssignmentTarget::Identifier(name.clone()))
                    }
                    // Parenthesized member expressions are valid: (a.b) = 1
                    // But not optional chaining: (a?.b) = 1 is invalid
                    Expression::MemberExpression { optional, .. } => {
                        if optional {
                            return Err(syntax_error(
                                "Invalid left-hand side in assignment: optional chaining not allowed",
                                self.last_position.clone(),
                            ));
                        }
                        Ok(AssignmentTarget::Member(expression))
                    }
                    // Parenthesized object/array literals cannot be destructuring targets
                    // ({x}) = y and ([x]) = y are invalid
                    Expression::ObjectExpression { .. } | Expression::ArrayExpression { .. } => {
                        Err(syntax_error("Invalid left-hand side in assignment", None))
                    }
                    // Nested parentheses: recurse
                    Expression::ParenthesizedExpression { .. } => {
                        self.expression_to_assignment_target(*expression)
                    }
                    // Sequence expressions in parentheses can be valid if last is assignable
                    Expression::SequenceExpression { expressions, .. } => {
                        if let Some(last) = expressions.into_iter().last() {
                            self.expression_to_assignment_target(last)
                        } else {
                            Err(syntax_error("Invalid assignment target", None))
                        }
                    }
                    // All other parenthesized expressions are invalid
                    _ => Err(syntax_error("Invalid left-hand side in assignment", None)),
                }
            }
            _ => Err(syntax_error("Invalid assignment target", None)),
        }
    }

    fn check_assignment_operator(&mut self) -> Result<Option<AssignmentOperator>, JsError> {
        let op = match self.lexer.peek_token()? {
            Token::Punctuator(Punctuator::Assign) => Some(AssignmentOperator::Assign),
            Token::Punctuator(Punctuator::PlusEq) => Some(AssignmentOperator::AddAssign),
            Token::Punctuator(Punctuator::MinusEq) => Some(AssignmentOperator::SubAssign),
            Token::Punctuator(Punctuator::StarEq) => Some(AssignmentOperator::MulAssign),
            Token::Punctuator(Punctuator::SlashEq) => Some(AssignmentOperator::DivAssign),
            Token::Punctuator(Punctuator::PercentEq) => Some(AssignmentOperator::ModAssign),
            Token::Punctuator(Punctuator::StarStarEq) => Some(AssignmentOperator::ExpAssign),
            Token::Punctuator(Punctuator::AndEq) => Some(AssignmentOperator::BitAndAssign),
            Token::Punctuator(Punctuator::OrEq) => Some(AssignmentOperator::BitOrAssign),
            Token::Punctuator(Punctuator::XorEq) => Some(AssignmentOperator::BitXorAssign),
            Token::Punctuator(Punctuator::LtLtEq) => Some(AssignmentOperator::LeftShiftAssign),
            Token::Punctuator(Punctuator::GtGtEq) => Some(AssignmentOperator::RightShiftAssign),
            Token::Punctuator(Punctuator::GtGtGtEq) => Some(AssignmentOperator::UnsignedRightShiftAssign),
            Token::Punctuator(Punctuator::AndAndEq) => Some(AssignmentOperator::LogicalAndAssign),
            Token::Punctuator(Punctuator::OrOrEq) => Some(AssignmentOperator::LogicalOrAssign),
            Token::Punctuator(Punctuator::NullishCoalesceEq) => Some(AssignmentOperator::NullishCoalesceAssign),
            _ => None,
        };
        Ok(op)
    }

    fn parse_conditional_expression(&mut self) -> Result<Expression, JsError> {
        let test = self.parse_nullish_coalescing_expression()?;

        if self.check_punctuator(Punctuator::Question)? {
            self.lexer.next_token()?;
            // Per ECMAScript grammar, consequent allows 'in' operator ([+In])
            let prev_in_for_init = self.in_for_init;
            self.in_for_init = false;
            let consequent = Box::new(self.parse_assignment_expression()?);
            self.in_for_init = prev_in_for_init;
            self.expect_punctuator(Punctuator::Colon)?;
            let alternate = Box::new(self.parse_assignment_expression()?);

            return Ok(Expression::ConditionalExpression {
                test: Box::new(test),
                consequent,
                alternate,
                position: None,
            });
        }

        Ok(test)
    }

    fn parse_nullish_coalescing_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_logical_or_expression()?;

        while self.check_punctuator(Punctuator::NullishCoalesce)? {
            self.lexer.next_token()?;
            let right = self.parse_logical_or_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::NullishCoalesce,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_logical_or_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_logical_and_expression()?;

        while self.check_punctuator(Punctuator::OrOr)? {
            self.lexer.next_token()?;
            let right = self.parse_logical_and_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::Or,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_logical_and_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_bitwise_or_expression()?;

        while self.check_punctuator(Punctuator::AndAnd)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_or_expression()?;
            left = Expression::LogicalExpression {
                left: Box::new(left),
                operator: LogicalOperator::And,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_or_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_bitwise_xor_expression()?;

        while self.check_punctuator(Punctuator::Or)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_xor_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseOr,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_xor_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_bitwise_and_expression()?;

        while self.check_punctuator(Punctuator::Xor)? {
            self.lexer.next_token()?;
            let right = self.parse_bitwise_and_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseXor,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_bitwise_and_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_equality_expression()?;

        while self.check_punctuator(Punctuator::And)? {
            self.lexer.next_token()?;
            let right = self.parse_equality_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::BitwiseAnd,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_equality_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_relational_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::EqEq) => BinaryOperator::Eq,
                Token::Punctuator(Punctuator::NotEq) => BinaryOperator::NotEq,
                Token::Punctuator(Punctuator::EqEqEq) => BinaryOperator::StrictEq,
                Token::Punctuator(Punctuator::NotEqEq) => BinaryOperator::StrictNotEq,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_relational_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_relational_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_shift_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Lt) => BinaryOperator::Lt,
                Token::Punctuator(Punctuator::LtEq) => BinaryOperator::LtEq,
                Token::Punctuator(Punctuator::Gt) => BinaryOperator::Gt,
                Token::Punctuator(Punctuator::GtEq) => BinaryOperator::GtEq,
                Token::Keyword(Keyword::Instanceof) => BinaryOperator::Instanceof,
                // In for loop init context, 'in' is not allowed as relational operator
                Token::Keyword(Keyword::In) if !self.in_for_init => BinaryOperator::In,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_shift_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_shift_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_additive_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::LtLt) => BinaryOperator::LeftShift,
                Token::Punctuator(Punctuator::GtGt) => BinaryOperator::RightShift,
                Token::Punctuator(Punctuator::GtGtGt) => BinaryOperator::UnsignedRightShift,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_additive_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_multiplicative_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Plus) => BinaryOperator::Add,
                Token::Punctuator(Punctuator::Minus) => BinaryOperator::Sub,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_multiplicative_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, JsError> {
        let mut left = self.parse_exponentiation_expression()?;

        loop {
            // Check for line terminator before operator
            // This is needed for restricted productions like yield:
            // yield
            // * 1   <-- * should not be a binary operator here (ASI terminates yield)
            if self.lexer.line_terminator_before_token {
                // After yield/await with no argument (due to line terminator),
                // a * on the next line is NOT a multiplication operator
                // It would be an invalid statement start
                if matches!(&left, Expression::YieldExpression { argument: None, .. }) {
                    break;
                }
            }

            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Star) => BinaryOperator::Mul,
                Token::Punctuator(Punctuator::Slash) => BinaryOperator::Div,
                Token::Punctuator(Punctuator::Percent) => BinaryOperator::Mod,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_exponentiation_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
        }

        Ok(left)
    }

    /// Parse exponentiation expression (**) which is right-associative
    fn parse_exponentiation_expression(&mut self) -> Result<Expression, JsError> {
        let left = self.parse_unary_expression()?;

        // Exponentiation is right-associative: a ** b ** c = a ** (b ** c)
        if self.check_punctuator(Punctuator::StarStar)? {
            self.lexer.next_token()?;
            let right = self.parse_exponentiation_expression()?;
            return Ok(Expression::BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::Exp,
                right: Box::new(right),
                position: None,
            });
        }

        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, JsError> {
        let op = match self.lexer.peek_token()? {
            Token::Punctuator(Punctuator::Not) => Some(UnaryOperator::Not),
            Token::Punctuator(Punctuator::Minus) => Some(UnaryOperator::Minus),
            Token::Punctuator(Punctuator::Plus) => Some(UnaryOperator::Plus),
            Token::Punctuator(Punctuator::Tilde) => Some(UnaryOperator::BitwiseNot),
            Token::Keyword(Keyword::Typeof) => Some(UnaryOperator::Typeof),
            Token::Keyword(Keyword::Void) => Some(UnaryOperator::Void),
            Token::Keyword(Keyword::Delete) => Some(UnaryOperator::Delete),
            _ => None,
        };

        if let Some(operator) = op {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_unary_expression()?);

            // Early error: In strict mode, delete on private name is forbidden
            // Also check the covered form (parenthesized expression)
            if matches!(operator, UnaryOperator::Delete) && self.strict_mode {
                if Self::expression_has_private_name_access(&argument) {
                    return Err(syntax_error(
                        "Deleting a private field is a syntax error",
                        self.last_position.clone(),
                    ));
                }
            }

            return Ok(Expression::UnaryExpression {
                operator,
                argument,
                prefix: true,
                position: None,
            });
        }

        // await is only an AwaitExpression when inside an async function
        // Otherwise it should be treated as an identifier
        if self.in_async && self.check_keyword(Keyword::Await)? {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_unary_expression()?);
            return Ok(Expression::AwaitExpression {
                argument,
                position: None,
            });
        }

        // Note: YieldExpression is NOT parsed here - it's at AssignmentExpression level
        // This ensures `void yield` fails in generators (yield cannot be identifier)

        self.parse_update_expression()
    }

    fn parse_update_expression(&mut self) -> Result<Expression, JsError> {
        // Prefix ++/--
        if self.check_punctuator(Punctuator::PlusPlus)? {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_left_hand_side_expression()?);
            // Check for strict mode invalid assignment targets
            self.validate_update_target(&argument)?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Increment,
                argument,
                prefix: true,
                position: None,
            });
        }

        if self.check_punctuator(Punctuator::MinusMinus)? {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_left_hand_side_expression()?);
            // Check for strict mode invalid assignment targets
            self.validate_update_target(&argument)?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument,
                prefix: true,
                position: None,
            });
        }

        let expr = self.parse_left_hand_side_expression()?;

        // Postfix ++/-- (restricted production: no line terminator allowed before)
        // If there's a line terminator, don't parse as postfix - let ASI handle it
        if !self.lexer.line_terminator_before_token && self.check_punctuator(Punctuator::PlusPlus)? {
            self.lexer.next_token()?;
            // Check for strict mode invalid assignment targets
            self.validate_update_target(&expr)?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Box::new(expr),
                prefix: false,
                position: None,
            });
        }

        if !self.lexer.line_terminator_before_token && self.check_punctuator(Punctuator::MinusMinus)? {
            self.lexer.next_token()?;
            // Check for strict mode invalid assignment targets
            self.validate_update_target(&expr)?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Box::new(expr),
                prefix: false,
                position: None,
            });
        }

        Ok(expr)
    }

    /// Validate that an update expression target is valid
    /// - `this` is never a valid update target
    /// - In strict mode, call expressions and other non-simple expressions are not valid
    /// - In strict mode, `arguments` and `eval` cannot be update targets
    fn validate_update_target(&self, expr: &Expression) -> Result<(), JsError> {
        match expr {
            // `this` is never a valid assignment target
            Expression::ThisExpression { .. } => {
                Err(syntax_error(
                    "Invalid update target: 'this' is not assignable",
                    self.last_position.clone(),
                ))
            }
            // Identifiers are valid, but check for eval/arguments in strict mode
            Expression::Identifier { name, .. } => {
                if self.strict_mode && (name == "arguments" || name == "eval") {
                    return Err(syntax_error(
                        &format!("'{}' cannot be used as an update target in strict mode", name),
                        self.last_position.clone(),
                    ));
                }
                Ok(())
            }
            // Member expressions are valid, but not optional chaining
            Expression::MemberExpression { optional, .. } => {
                if *optional {
                    Err(syntax_error(
                        "Invalid left-hand side in prefix/postfix expression: optional chaining not allowed",
                        self.last_position.clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            // In strict mode, call expressions are not valid update targets
            Expression::CallExpression { .. } => {
                if self.strict_mode {
                    Err(syntax_error(
                        "Invalid left-hand side in prefix/postfix expression",
                        self.last_position.clone(),
                    ))
                } else {
                    // In non-strict mode, allow for web compatibility (runtime error)
                    Ok(())
                }
            }
            // Handle parenthesized expressions
            Expression::ParenthesizedExpression { expression, .. } => {
                self.validate_update_target(expression)
            }
            // All other expressions are invalid
            _ => {
                if self.strict_mode {
                    Err(syntax_error(
                        "Invalid left-hand side in prefix/postfix expression",
                        self.last_position.clone(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn parse_left_hand_side_expression(&mut self) -> Result<Expression, JsError> {
        let mut expr = if self.check_keyword(Keyword::New)? {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };

        loop {
            if self.check_punctuator(Punctuator::Dot)? {
                self.lexer.next_token()?;
                // Check for private identifier after dot
                let property = if self.check_private_identifier()? {
                    let name = self.expect_private_identifier()?;
                    Expression::Identifier {
                        name: format!("#{}", name),
                        position: None,
                    }
                } else {
                    // Allow keywords as property names after dot
                    let name = self.expect_property_name()?;
                    Expression::Identifier {
                        name,
                        position: None,
                    }
                };
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: false,
                    optional: false,
                    position: None,
                };
            } else if self.check_punctuator(Punctuator::OptionalChain)? {
                self.lexer.next_token()?;
                // Check for private identifier after optional chain
                let property = if self.check_private_identifier()? {
                    let name = self.expect_private_identifier()?;
                    Expression::Identifier {
                        name: format!("#{}", name),
                        position: None,
                    }
                } else {
                    // Allow keywords as property names after optional chain
                    let name = self.expect_property_name()?;
                    Expression::Identifier {
                        name,
                        position: None,
                    }
                };
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: false,
                    optional: true,
                    position: None,
                };
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Computed property access: obj[expr] - 'in' is always allowed
                self.lexer.next_token()?;
                let prev_in_for_init = self.in_for_init;
                self.in_for_init = false;
                let property = Box::new(self.parse_expression()?);
                self.in_for_init = prev_in_for_init;
                self.expect_punctuator(Punctuator::RBracket)?;
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property,
                    computed: true,
                    optional: false,
                    position: None,
                };
            } else if self.check_punctuator(Punctuator::LParen)? {
                // super() calls are only allowed in constructors
                if matches!(expr, Expression::SuperExpression { .. }) && !self.in_constructor {
                    return Err(syntax_error(
                        "'super' call is not allowed outside of class constructor",
                        self.last_position.clone(),
                    ));
                }
                // super() calls require class heritage (extends clause)
                if matches!(expr, Expression::SuperExpression { .. }) && !self.has_class_heritage {
                    return Err(syntax_error(
                        "'super' call is only valid in derived class constructor",
                        self.last_position.clone(),
                    ));
                }
                let arguments = self.parse_arguments()?;
                expr = Expression::CallExpression {
                    callee: Box::new(expr),
                    arguments,
                    optional: false,
                    position: None,
                };
            } else if matches!(self.lexer.peek_token()?, Token::TemplateLiteral(_)) {
                // Tagged template literal: tag`template`
                if let Token::TemplateLiteral(s) = self.lexer.next_token()? {
                    let quasi = Expression::TemplateLiteral {
                        quasis: vec![TemplateElement {
                            raw: s.clone(),
                            cooked: s,
                            tail: true,
                        }],
                        expressions: vec![],
                        position: None,
                    };
                    expr = Expression::TaggedTemplateExpression {
                        tag: Box::new(expr),
                        quasi: Box::new(quasi),
                        position: None,
                    };
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_new_expression(&mut self) -> Result<Expression, JsError> {
        self.expect_keyword(Keyword::New)?;

        // Check for new.target meta property
        if self.check_punctuator(Punctuator::Dot)? {
            self.lexer.next_token()?;
            let property = self.expect_identifier()?;
            if property == "target" {
                return Ok(Expression::MetaProperty {
                    meta: "new".to_string(),
                    property: "target".to_string(),
                    position: None,
                });
            } else {
                return Err(syntax_error("Expected 'target' after 'new.'", None));
            }
        }

        // Parse callee without consuming call expressions - those belong to the NewExpression
        let callee = Box::new(self.parse_member_expression_without_call()?);
        let arguments = if self.check_punctuator(Punctuator::LParen)? {
            self.parse_arguments()?
        } else {
            vec![]
        };

        Ok(Expression::NewExpression {
            callee,
            arguments,
            position: None,
        })
    }

    fn parse_member_expression_without_call(&mut self) -> Result<Expression, JsError> {
        let mut expr = if self.check_keyword(Keyword::New)? {
            self.parse_new_expression()?
        } else {
            self.parse_primary_expression()?
        };

        // Parse member access but NOT call expressions
        loop {
            if self.check_punctuator(Punctuator::Dot)? {
                self.lexer.next_token()?;
                // Check for private identifier after dot
                let property = if self.check_private_identifier()? {
                    let name = self.expect_private_identifier()?;
                    Expression::Identifier {
                        name: format!("#{}", name),
                        position: None,
                    }
                } else {
                    // Allow keywords as property names after dot
                    let name = self.expect_property_name()?;
                    Expression::Identifier {
                        name,
                        position: None,
                    }
                };
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: false,
                    optional: false,
                    position: None,
                };
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Computed property access: obj[expr] - 'in' is always allowed
                self.lexer.next_token()?;
                let prev_in_for_init = self.in_for_init;
                self.in_for_init = false;
                let property = Box::new(self.parse_expression()?);
                self.in_for_init = prev_in_for_init;
                self.expect_punctuator(Punctuator::RBracket)?;
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property,
                    computed: true,
                    optional: false,
                    position: None,
                };
            } else {
                // Do NOT parse LParen here - that's for the NewExpression's arguments
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, JsError> {
        self.expect_punctuator(Punctuator::LParen)?;
        let mut args = Vec::new();

        while !self.check_punctuator(Punctuator::RParen)? {
            if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let expr = self.parse_assignment_expression()?;
                args.push(Expression::SpreadElement {
                    argument: Box::new(expr),
                    position: None,
                });
            } else {
                args.push(self.parse_assignment_expression()?);
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RParen)?;
        Ok(args)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, JsError> {
        let token = self.lexer.peek_token()?.clone();

        match token {
            Token::Identifier(name, _has_escapes) => {
                self.lexer.next_token()?;
                // Validate identifier references in async/generator context
                // This catches escaped reserved words like \u0061wait (await)
                self.validate_identifier(&name)?;
                Ok(Expression::Identifier {
                    name,
                    position: None,
                })
            }
            Token::Number(n) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::Number(n),
                    position: None,
                })
            }
            Token::String(s) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::String(s),
                    position: None,
                })
            }
            Token::TemplateLiteral(s) => {
                self.lexer.next_token()?;
                Ok(Expression::TemplateLiteral {
                    quasis: vec![TemplateElement {
                        raw: s.clone(),
                        cooked: s,
                        tail: true,
                    }],
                    expressions: vec![],
                    position: None,
                })
            }
            Token::Keyword(Keyword::True) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::Boolean(true),
                    position: None,
                })
            }
            Token::Keyword(Keyword::False) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::Boolean(false),
                    position: None,
                })
            }
            Token::Keyword(Keyword::Null) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::Null,
                    position: None,
                })
            }
            // Note: 'undefined' is NOT a keyword - it's handled as an identifier
            // and resolved at runtime from the global scope
            Token::Keyword(Keyword::This) => {
                self.lexer.next_token()?;
                Ok(Expression::ThisExpression { position: None })
            }
            Token::Keyword(Keyword::Super) => {
                self.lexer.next_token()?;
                // super is valid inside any method (class or object literal) or constructor
                if !self.in_class_method && !self.in_constructor && !self.in_method {
                    return Err(syntax_error(
                        "'super' keyword is unexpected here",
                        self.last_position.clone(),
                    ));
                }
                Ok(Expression::SuperExpression { position: None })
            }
            Token::Keyword(Keyword::Function) => self.parse_function_expression(),
            Token::Keyword(Keyword::Async) => self.parse_async_function_expression(),
            Token::Keyword(Keyword::Class) => self.parse_class_expression(),
            // yield is valid as identifier only in non-strict mode AND outside generators
            Token::Keyword(Keyword::Yield) => {
                self.lexer.next_token()?;
                // In strict mode, 'yield' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'yield' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                // In generator context, 'yield' is reserved (cannot be identifier reference)
                if self.in_generator {
                    return Err(syntax_error(
                        "'yield' is a reserved word in generator functions",
                        self.last_position.clone(),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "yield".to_string(),
                    position: None,
                })
            }
            Token::Keyword(Keyword::Let) => {
                self.lexer.next_token()?;
                // In strict mode, 'let' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'let' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "let".to_string(),
                    position: None,
                })
            }
            Token::Keyword(Keyword::Static) => {
                self.lexer.next_token()?;
                // In strict mode, 'static' is a reserved word
                if self.strict_mode {
                    return Err(syntax_error(
                        "'static' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "static".to_string(),
                    position: None,
                })
            }
            // 'await' is an identifier outside async functions and static blocks
            Token::Keyword(Keyword::Await) if !self.in_async && !self.in_static_block => {
                self.lexer.next_token()?;
                Ok(Expression::Identifier {
                    name: "await".to_string(),
                    position: None,
                })
            }
            Token::BigIntLiteral(s) => {
                self.lexer.next_token()?;
                Ok(Expression::Literal {
                    value: Literal::BigInt(s),
                    position: None,
                })
            }
            Token::Punctuator(Punctuator::LParen) => self.parse_parenthesized_or_arrow(),
            Token::Punctuator(Punctuator::LBracket) => self.parse_array_literal(),
            Token::Punctuator(Punctuator::LBrace) => self.parse_object_literal(),
            // Dynamic import: import(specifier), import.defer(specifier), import.source(specifier)
            Token::Keyword(Keyword::Import) => {
                self.lexer.next_token()?; // consume 'import'

                // Check for import.defer or import.source (import phases)
                if self.check_punctuator(Punctuator::Dot)? {
                    self.lexer.next_token()?; // consume '.'
                    // Expect 'defer' or 'source' or 'meta'
                    let phase = self.expect_identifier()?;
                    if phase == "meta" {
                        // import.meta - handled specially
                        return Ok(Expression::MetaProperty {
                            meta: "import".to_string(),
                            property: "meta".to_string(),
                            position: None,
                        });
                    }
                    // For defer/source, continue to parse the call
                    self.expect_punctuator(Punctuator::LParen)?;
                    let source = self.parse_assignment_expression()?;
                    self.expect_punctuator(Punctuator::RParen)?;
                    // Return ImportExpression with phase marker (for now, same as regular import)
                    return Ok(Expression::ImportExpression {
                        source: Box::new(source),
                        position: None,
                    });
                }

                // Regular import() - expect opening paren
                if !self.check_punctuator(Punctuator::LParen)? {
                    return Err(syntax_error(
                        "Expected '(' after 'import' for dynamic import",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?; // consume '('
                let source = self.parse_assignment_expression()?;
                self.expect_punctuator(Punctuator::RParen)?;
                Ok(Expression::ImportExpression {
                    source: Box::new(source),
                    position: None,
                })
            }
            _ => Err(syntax_error("Unexpected token", None)),
        }
    }

    fn parse_function_expression(&mut self) -> Result<Expression, JsError> {
        self.expect_keyword(Keyword::Function)?;

        // Check for generator: function *name() or function *()
        let is_generator = self.check_punctuator(Punctuator::Star)?;
        if is_generator {
            self.lexer.next_token()?;
        }

        let name = if let Token::Identifier(_, _) = self.lexer.peek_token()? {
            Some(self.expect_identifier()?)
        } else if !is_generator
            && !self.strict_mode
            && matches!(self.lexer.peek_token()?, Token::Keyword(Keyword::Yield))
        {
            // For non-generator function expressions, 'yield' can be used as a binding identifier
            // (in non-strict mode) even when we're inside an outer generator context
            self.lexer.next_token()?;
            Some("yield".to_string())
        } else if matches!(self.lexer.peek_token()?, Token::Keyword(Keyword::Await)) {
            // For non-async function expressions, 'await' can be used as a binding identifier
            // even when we're inside a static block or class (which is strict mode).
            // 'await' is only reserved in async contexts and modules.
            // Note: We don't check strict_mode here because 'await' is allowed in strict mode
            // when not in an async context.
            self.lexer.next_token()?;
            Some("await".to_string())
        } else {
            None
        };

        // Save context and set up for this function
        let prev_in_class_method = self.in_class_method;
        let prev_in_constructor = self.in_constructor;
        let prev_in_generator = self.in_generator;
        let prev_in_async = self.in_async;
        let prev_in_static_block = self.in_static_block;
        // Clear class context - function expressions cannot use super
        self.in_class_method = false;
        self.in_constructor = false;
        // Set generator context for parameter parsing
        self.in_generator = is_generator;
        self.in_async = false;
        // Function expressions have their own 'arguments' binding, so clear static block context
        // This allows 'arguments' to be used inside function expressions in static blocks
        self.in_static_block = false;

        let params = self.parse_parameters()?;
        // Validate for duplicate parameters
        self.validate_parameters(&params)?;
        // Validate no yield expressions in formal parameters (early error for generators)
        if is_generator {
            self.validate_params_no_yield(&params)?;
        }
        let body = self.parse_function_body_with_context(false, is_generator)?;

        // Validate use strict with non-simple parameters
        self.validate_params_with_body(&params, &body)?;
        // Validate parameter names don't conflict with lexical declarations in body
        self.validate_params_body_lexical(&params, &body)?;

        self.in_class_method = prev_in_class_method;
        self.in_constructor = prev_in_constructor;
        self.in_generator = prev_in_generator;
        self.in_async = prev_in_async;
        self.in_static_block = prev_in_static_block;

        Ok(Expression::FunctionExpression {
            name,
            params,
            body,
            is_async: false,
            is_generator,
            position: None,
        })
    }

    fn parse_async_function_expression(&mut self) -> Result<Expression, JsError> {
        self.expect_keyword(Keyword::Async)?;

        // Peek next token to update line_terminator_before_token
        self.lexer.peek_token()?;

        // If there's a line terminator after async, we need to be careful
        // about the "async [no LineTerminator here] ArrowFormalParameters" restriction.
        // If the next token is `(`, we need to check if this looks like an async arrow function.
        if self.lexer.line_terminator_before_token {
            // Check if this could be an async arrow function with invalid line terminator
            if self.check_punctuator(Punctuator::LParen)? {
                // Look ahead to see if `=>` follows the parenthesized expression
                // Save position for potential lookahead
                // Save the full lexer state including cached token
                let start_pos = self.lexer.position;
                let start_line = self.lexer.line;
                let start_col = self.lexer.column;
                let start_token = self.lexer.current_token.clone();

                // Clear token cache so next_token actually scans
                self.lexer.current_token = None;

                // Scan `(` fresh
                let _ = self.lexer.next_token()?;
                let mut depth = 1;

                // Find matching `)` by scanning tokens
                while depth > 0 {
                    let tok = self.lexer.next_token()?;
                    match tok {
                        Token::Punctuator(Punctuator::LParen) => depth += 1,
                        Token::Punctuator(Punctuator::RParen) => depth -= 1,
                        Token::EOF => break,
                        _ => {}
                    }
                }

                // Check if `=>` follows
                let next = self.lexer.next_token()?;
                let is_arrow = matches!(next, Token::Punctuator(Punctuator::Arrow));

                // Restore lexer state
                self.lexer.position = start_pos;
                self.lexer.line = start_line;
                self.lexer.column = start_col;
                self.lexer.current_token = start_token;
                self.lexer.line_terminator_before_token = true;

                if is_arrow {
                    // This is an async arrow function with an invalid line terminator
                    return Err(syntax_error(
                        "Line terminator not allowed between 'async' and arrow function parameters",
                        self.last_position.clone(),
                    ));
                }
            }

            // Not an async arrow function, treat `async` as an identifier
            return Ok(Expression::Identifier {
                name: "async".to_string(),
                position: None,
            });
        }

        if self.check_keyword(Keyword::Function)? {
            self.lexer.next_token()?;

            // Check for async generator: async function *name() or async function *()
            let is_generator = self.check_punctuator(Punctuator::Star)?;
            if is_generator {
                self.lexer.next_token()?;
            }

            let name = if let Token::Identifier(_, _) = self.lexer.peek_token()? {
                Some(self.expect_identifier()?)
            } else if !is_generator
                && !self.strict_mode
                && matches!(self.lexer.peek_token()?, Token::Keyword(Keyword::Yield))
            {
                // For non-generator async function expressions, 'yield' can be used as a binding identifier
                // (in non-strict mode) even when we're inside an outer generator context
                self.lexer.next_token()?;
                Some("yield".to_string())
            } else {
                None
            };

            // Set up context for async function expression
            let prev_async = self.in_async;
            let prev_generator = self.in_generator;
            let prev_in_class_method = self.in_class_method;
            let prev_in_constructor = self.in_constructor;
            let prev_in_static_block = self.in_static_block;
            self.in_async = true;
            self.in_generator = is_generator;
            self.in_class_method = false;
            self.in_constructor = false;
            // Function expressions have their own 'arguments' binding, so clear static block context
            self.in_static_block = false;

            let params = self.parse_parameters()?;
            // Validate for duplicate parameters - must reject duplicates in async functions
            // and when there are non-simple parameters (defaults, destructuring, rest)
            self.validate_parameters(&params)?;
            // Validate no await expressions in formal parameters (early error for async functions)
            self.validate_params_no_await(&params)?;
            // Validate no yield expressions in formal parameters (early error for generators)
            if is_generator {
                self.validate_params_no_yield(&params)?;
            }
            let body = self.parse_function_body_with_context(true, is_generator)?;

            // Validate use strict with non-simple parameters
            self.validate_params_with_body(&params, &body)?;
            // Validate parameter names don't conflict with lexical declarations in body
            self.validate_params_body_lexical(&params, &body)?;

            self.in_async = prev_async;
            self.in_generator = prev_generator;
            self.in_class_method = prev_in_class_method;
            self.in_constructor = prev_in_constructor;
            self.in_static_block = prev_in_static_block;

            Ok(Expression::FunctionExpression {
                name,
                params,
                body,
                is_async: true,
                is_generator,
                position: None,
            })
        } else if self.check_punctuator(Punctuator::LParen)? {
            // Async arrow function with parens: async (params) => body
            // Set async context for parameter parsing - 'await' should be reserved
            // in default parameter expressions of async functions
            let prev_async = self.in_async;
            self.in_async = true;
            let params = self.parse_parameters()?;
            // Validate arrow parameters (duplicates and yield expressions)
            self.validate_arrow_parameters(&params)?;
            self.validate_arrow_params_no_yield(&params)?;
            // Check for line terminator before =>
            if self.check_punctuator(Punctuator::Arrow)? {
                if self.lexer.line_terminator_before_token {
                    self.in_async = prev_async;
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
            }
            self.expect_punctuator(Punctuator::Arrow)?;
            let body = self.parse_arrow_body_with_context(true)?;
            // Restore async context (though parse_arrow_body_with_context already handles this)
            self.in_async = prev_async;
            // Validate "use strict" with non-simple parameters
            self.validate_arrow_params_with_body(&params, &body)?;

            Ok(Expression::ArrowFunctionExpression {
                params,
                body,
                is_async: true,
                position: None,
            })
        } else if let Token::Identifier(name, _) = self.lexer.peek_token()?.clone() {
            // Async arrow function without parens: async x => body
            // But first, look ahead to see if there's actually an arrow after the identifier
            // If not, 'async' is just a standalone identifier

            // Save lexer state to look ahead
            let saved_pos = self.lexer.position;
            let saved_line = self.lexer.line;
            let saved_column = self.lexer.column;
            let saved_previous_line = self.lexer.previous_line;
            let saved_line_term = self.lexer.line_terminator_before_token;
            let saved_token = self.lexer.current_token.clone();

            self.lexer.next_token()?; // consume identifier
            let has_arrow = self.check_punctuator(Punctuator::Arrow)?;
            let has_line_term = self.lexer.line_terminator_before_token;

            if has_arrow && !has_line_term {
                // This is an async arrow function: async x => body
                // Validate the parameter identifier (e.g., no 'arguments'/'eval' in strict mode)
                self.validate_binding_identifier(&name)?;
                self.lexer.next_token()?; // consume =>
                let body = self.parse_arrow_body_with_context(true)?;

                Ok(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::Identifier(name)],
                    body,
                    is_async: true,
                    position: None,
                })
            } else {
                // No arrow - restore lexer state and treat 'async' as an identifier
                self.lexer.position = saved_pos;
                self.lexer.line = saved_line;
                self.lexer.column = saved_column;
                self.lexer.previous_line = saved_previous_line;
                self.lexer.line_terminator_before_token = saved_line_term;
                self.lexer.current_token = saved_token;

                Ok(Expression::Identifier {
                    name: "async".to_string(),
                    position: None,
                })
            }
        } else {
            // No function, no paren, no identifier after async
            // This means 'async' is a standalone identifier
            Ok(Expression::Identifier {
                name: "async".to_string(),
                position: None,
            })
        }
    }

    fn parse_parenthesized_or_arrow(&mut self) -> Result<Expression, JsError> {
        self.lexer.next_token()?; // consume (

        // Check for empty params ()
        if self.check_punctuator(Punctuator::RParen)? {
            self.lexer.next_token()?;
            if self.check_punctuator(Punctuator::Arrow)? {
                // Line terminator before => is not allowed
                if self.lexer.line_terminator_before_token {
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params: vec![],
                    body,
                    is_async: false,
                    position: None,
                });
            }
            return Err(syntax_error("Unexpected )", None));
        }

        // Check for rest parameter as first param: (...args) or (...[x])
        if self.check_punctuator(Punctuator::Spread)? {
            self.lexer.next_token()?;
            let rest_pattern = self.parse_pattern()?;
            self.expect_punctuator(Punctuator::RParen)?;
            if self.check_punctuator(Punctuator::Arrow)? {
                // Line terminator before => is not allowed
                if self.lexer.line_terminator_before_token {
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::RestElement(Box::new(rest_pattern))],
                    body,
                    is_async: false,
                    position: None,
                });
            }
            return Err(syntax_error("Rest parameter must be in arrow function", None));
        }

        // Parse first expression/pattern
        let first = self.parse_assignment_expression()?;

        // Check for arrow with single param
        if self.check_punctuator(Punctuator::RParen)? {
            self.lexer.next_token()?;
            if self.check_punctuator(Punctuator::Arrow)? {
                // Line terminator before => is not allowed
                if self.lexer.line_terminator_before_token {
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let param = self.expression_to_pattern(first)?;
                let params = vec![param];
                // Validate arrow parameters (duplicates and yield expressions)
                self.validate_arrow_parameters(&params)?;
                self.validate_arrow_params_no_yield(&params)?;
                let body = self.parse_arrow_body()?;
                // Validate "use strict" with non-simple parameters
                self.validate_arrow_params_with_body(&params, &body)?;
                return Ok(Expression::ArrowFunctionExpression {
                    params,
                    body,
                    is_async: false,
                    position: None,
                });
            }
            // Single expression in parentheses - wrap in ParenthesizedExpression
            // to track that it cannot be used as a destructuring assignment target
            return Ok(Expression::ParenthesizedExpression {
                expression: Box::new(first),
                position: None,
            });
        }

        // Multiple params or expressions
        if self.check_punctuator(Punctuator::Comma)? {
            let mut exprs = vec![first];
            let mut has_rest = false;
            let mut rest_param: Option<Pattern> = None;
            let mut has_trailing_comma = false;

            while self.check_punctuator(Punctuator::Comma)? {
                self.lexer.next_token()?;
                // Check for trailing comma: (a, b,)
                if self.check_punctuator(Punctuator::RParen)? {
                    has_trailing_comma = true;
                    break;
                }
                // Check for rest parameter: (a, b, ...c) or (a, b, ...[c])
                if self.check_punctuator(Punctuator::Spread)? {
                    self.lexer.next_token()?;
                    let rest_pattern = self.parse_pattern()?;
                    has_rest = true;
                    rest_param = Some(Pattern::RestElement(Box::new(rest_pattern)));
                    break; // Rest must be last
                }
                exprs.push(self.parse_assignment_expression()?);
            }
            self.expect_punctuator(Punctuator::RParen)?;

            if self.check_punctuator(Punctuator::Arrow)? {
                // Line terminator before => is not allowed
                if self.lexer.line_terminator_before_token {
                    return Err(syntax_error(
                        "Line terminator not allowed before '=>'",
                        self.last_position.clone(),
                    ));
                }
                self.lexer.next_token()?;
                let mut params: Vec<Pattern> = exprs
                    .into_iter()
                    .map(|e| self.expression_to_pattern(e))
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(rest) = rest_param {
                    params.push(rest);
                }
                // Validate arrow parameters (duplicates always rejected, yield expressions checked)
                self.validate_arrow_parameters(&params)?;
                self.validate_arrow_params_no_yield(&params)?;
                let body = self.parse_arrow_body()?;
                // Validate "use strict" with non-simple parameters
                self.validate_arrow_params_with_body(&params, &body)?;
                return Ok(Expression::ArrowFunctionExpression {
                    params,
                    body,
                    is_async: false,
                    position: None,
                });
            }

            if has_rest || has_trailing_comma {
                return Err(syntax_error("Rest parameter or trailing comma must be in arrow function", None));
            }

            // Sequence expression in parentheses - wrap in ParenthesizedExpression
            return Ok(Expression::ParenthesizedExpression {
                expression: Box::new(Expression::SequenceExpression {
                    expressions: exprs,
                    position: None,
                }),
                position: None,
            });
        }

        self.expect_punctuator(Punctuator::RParen)?;
        // Single expression after comma check - wrap in ParenthesizedExpression
        Ok(Expression::ParenthesizedExpression {
            expression: Box::new(first),
            position: None,
        })
    }

    fn expression_to_pattern(&self, expr: Expression) -> Result<Pattern, JsError> {
        match expr {
            Expression::Identifier { name, .. } => {
                // Validate the identifier is not a reserved word
                // This catches escaped reserved words like bre\u0061k -> break
                self.validate_binding_identifier(&name)?;
                Ok(Pattern::Identifier(name))
            }
            Expression::SpreadElement { argument, .. } => {
                let inner = self.expression_to_pattern(*argument)?;
                Ok(Pattern::RestElement(Box::new(inner)))
            }
            // Handle assignment expressions for default parameters: x = value
            Expression::AssignmentExpression { left, right, operator, .. } => {
                if matches!(operator, crate::ast::AssignmentOperator::Assign) {
                    let left_pattern = self.assignment_target_to_pattern(left)?;
                    Ok(Pattern::AssignmentPattern {
                        left: Box::new(left_pattern),
                        right: right,
                    })
                } else {
                    Err(syntax_error("Invalid parameter", None))
                }
            }
            // Handle array destructuring
            Expression::ArrayExpression { elements, .. } => {
                let mut patterns: Vec<Option<Pattern>> = Vec::new();
                let mut seen_rest = false;

                for (i, e) in elements.into_iter().enumerate() {
                    if seen_rest {
                        // Rest element must be last
                        return Err(syntax_error(
                            "Rest element must be last element",
                            self.last_position.clone(),
                        ));
                    }

                    match e {
                        Some(crate::ast::ArrayElement::Expression(expr)) => {
                            patterns.push(Some(self.expression_to_pattern(expr)?));
                        }
                        Some(crate::ast::ArrayElement::Spread(expr)) => {
                            // Check if the spread argument has an initializer (rest can't have default)
                            if matches!(expr, Expression::AssignmentExpression { .. }) {
                                return Err(syntax_error(
                                    "Rest element may not have a default initializer",
                                    self.last_position.clone(),
                                ));
                            }
                            let inner = self.expression_to_pattern(expr)?;
                            // Also check if the inner pattern is an AssignmentPattern
                            if matches!(inner, Pattern::AssignmentPattern { .. }) {
                                return Err(syntax_error(
                                    "Rest element may not have a default initializer",
                                    self.last_position.clone(),
                                ));
                            }
                            patterns.push(Some(Pattern::RestElement(Box::new(inner))));
                            seen_rest = true;
                        }
                        None => {
                            patterns.push(None);
                        }
                    }
                }
                Ok(Pattern::ArrayPattern(patterns))
            }
            // Handle object destructuring
            Expression::ObjectExpression { properties, .. } => {
                let mut patterns: Vec<ObjectPatternProperty> = Vec::new();
                let mut seen_rest = false;

                for prop in properties.into_iter() {
                    if seen_rest {
                        // Rest element must be last
                        return Err(syntax_error(
                            "Rest element must be last in object pattern",
                            self.last_position.clone(),
                        ));
                    }

                    match prop {
                        ObjectProperty::Property { key, value, shorthand, .. } => {
                            let value_pattern = self.expression_to_pattern(value)?;
                            let pattern_key = match key {
                                PropertyKey::Identifier(s) => crate::ast::PatternKey::Literal(s),
                                PropertyKey::String(s) => crate::ast::PatternKey::Literal(s),
                                PropertyKey::Number(n) => crate::ast::PatternKey::Literal(n.to_string()),
                                PropertyKey::Computed(expr) => crate::ast::PatternKey::Computed(expr),
                            };
                            patterns.push(ObjectPatternProperty {
                                key: pattern_key,
                                value: value_pattern,
                                shorthand,
                            });
                        }
                        ObjectProperty::SpreadElement(expr) => {
                            let pattern = self.expression_to_pattern(expr)?;
                            patterns.push(ObjectPatternProperty {
                                key: crate::ast::PatternKey::Literal(String::new()),
                                value: Pattern::RestElement(Box::new(pattern)),
                                shorthand: false,
                            });
                            seen_rest = true;
                        }
                    }
                }
                Ok(Pattern::ObjectPattern(patterns))
            }
            // Handle member expressions as destructuring targets
            // These are valid in destructuring assignment but not in function parameters
            Expression::MemberExpression { .. } => {
                Ok(Pattern::MemberExpression(Box::new(expr)))
            }
            _ => Err(syntax_error("Invalid parameter", None)),
        }
    }

    /// Convert an AssignmentTarget to a Pattern
    fn assignment_target_to_pattern(&self, target: crate::ast::AssignmentTarget) -> Result<Pattern, JsError> {
        match target {
            crate::ast::AssignmentTarget::Identifier(name) => {
                // Validate that the identifier can be used as a binding
                // (e.g., eval/arguments are forbidden in strict mode)
                self.validate_binding_identifier(&name)?;
                Ok(Pattern::Identifier(name))
            }
            crate::ast::AssignmentTarget::Member(expr) => {
                // Member expressions are valid as destructuring assignment targets
                // e.g., [{y: 1}.y = 42] = vals; or [obj.x = 42] = arr;
                Ok(Pattern::MemberExpression(expr))
            }
            crate::ast::AssignmentTarget::Pattern(pattern) => Ok(pattern),
        }
    }

    fn parse_arrow_body(&mut self) -> Result<ArrowFunctionBody, JsError> {
        // Reset static block context - arrow function bodies have their own scope
        let prev_static_block = self.in_static_block;
        self.in_static_block = false;

        let result = if self.check_punctuator(Punctuator::LBrace)? {
            let body = self.parse_function_body()?;
            Ok(ArrowFunctionBody::Block(body))
        } else {
            let expr = self.parse_assignment_expression()?;
            Ok(ArrowFunctionBody::Expression(Box::new(expr)))
        };

        self.in_static_block = prev_static_block;
        result
    }

    /// Parse arrow function body with async context tracking
    fn parse_arrow_body_with_context(&mut self, is_async: bool) -> Result<ArrowFunctionBody, JsError> {
        let prev_async = self.in_async;
        let prev_static_block = self.in_static_block;
        self.in_async = is_async;
        // Reset static block context - arrow function bodies have their own scope
        self.in_static_block = false;

        let result = if self.check_punctuator(Punctuator::LBrace)? {
            let body = self.parse_function_body()?;
            Ok(ArrowFunctionBody::Block(body))
        } else {
            let expr = self.parse_assignment_expression()?;
            Ok(ArrowFunctionBody::Expression(Box::new(expr)))
        };

        self.in_async = prev_async;
        self.in_static_block = prev_static_block;
        result
    }

    fn parse_array_literal(&mut self) -> Result<Expression, JsError> {
        self.expect_punctuator(Punctuator::LBracket)?;
        let mut elements = Vec::new();
        let mut last_was_spread = false;

        while !self.check_punctuator(Punctuator::RBracket)? {
            if self.check_punctuator(Punctuator::Comma)? {
                elements.push(None);
                last_was_spread = false;
            } else if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let expr = self.parse_assignment_expression()?;
                elements.push(Some(ArrayElement::Spread(expr)));
                last_was_spread = true;
            } else {
                let expr = self.parse_assignment_expression()?;
                elements.push(Some(ArrayElement::Expression(expr)));
                last_was_spread = false;
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            // If last element was a spread and we're about to see a trailing comma
            // followed by ], add a None marker to indicate invalid rest position
            if last_was_spread {
                self.lexer.next_token()?; // consume comma
                // Check if this is a trailing comma (followed by ])
                if self.check_punctuator(Punctuator::RBracket)? {
                    // Add a None to mark that there was a comma after spread
                    // This will be detected during pattern conversion
                    elements.push(None);
                    break;
                }
                // Otherwise continue normally (next element will be added)
                continue;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RBracket)?;

        Ok(Expression::ArrayExpression {
            elements,
            position: None,
        })
    }

    fn parse_object_literal(&mut self) -> Result<Expression, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut properties = Vec::new();

        while !self.check_punctuator(Punctuator::RBrace)? {
            if self.check_punctuator(Punctuator::Spread)? {
                // Spread property: ...expr
                self.lexer.next_token()?;
                let expr = self.parse_assignment_expression()?;
                properties.push(ObjectProperty::SpreadElement(expr));
            } else if self.check_punctuator(Punctuator::Star)? {
                // Generator method: *name() {} or *[expr]() {}
                self.lexer.next_token()?;

                let (key, computed) = if self.check_punctuator(Punctuator::LBracket)? {
                    // Computed generator method: *[expr]() {} - 'in' allowed
                    self.lexer.next_token()?;
                    let prev_in_for_init = self.in_for_init;
                    self.in_for_init = false;
                    let key_expr = self.parse_assignment_expression()?;
                    self.in_for_init = prev_in_for_init;
                    self.expect_punctuator(Punctuator::RBracket)?;
                    (PropertyKey::Computed(key_expr), true)
                } else {
                    // Regular generator method: *name() {}
                    let key = self.expect_identifier_or_keyword()?;
                    (PropertyKey::Identifier(key), false)
                };

                // Set generator context before parsing params
                let prev_generator = self.in_generator;
                let prev_method = self.in_method;
                self.in_generator = true;
                self.in_method = true;

                let params = self.parse_parameters()?;
                // Validate parameters
                self.validate_parameters(&params)?;
                self.validate_params_no_yield(&params)?;
                let body = self.parse_method_body_with_context(false, true)?;
                // Validate use strict with non-simple parameters
                self.validate_params_with_body(&params, &body)?;

                self.in_generator = prev_generator;
                self.in_method = prev_method;

                let func_name = match &key {
                    PropertyKey::Identifier(s) => Some(s.clone()),
                    _ => None,
                };
                let func = Expression::FunctionExpression {
                    name: func_name,
                    params,
                    body,
                    is_async: false,
                    is_generator: true,
                    position: None,
                };

                properties.push(ObjectProperty::Property {
                    key,
                    value: func,
                    shorthand: false,
                    computed,
                });
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Computed property: [expr]: value or [expr]() {} - 'in' allowed
                self.lexer.next_token()?;
                let prev_in_for_init = self.in_for_init;
                self.in_for_init = false;
                let key_expr = self.parse_assignment_expression()?;
                self.in_for_init = prev_in_for_init;
                self.expect_punctuator(Punctuator::RBracket)?;

                if self.check_punctuator(Punctuator::LParen)? {
                    // Computed method: [expr]() {}
                    // Set method context for super access in parameter defaults
                    let prev_method = self.in_method;
                    self.in_method = true;
                    let params = self.parse_parameters()?;
                    let body = self.parse_method_body()?;
                    self.in_method = prev_method;

                    let func = Expression::FunctionExpression {
                        name: None,
                        params,
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Computed(key_expr),
                        value: func,
                        shorthand: false,
                        computed: true,
                    });
                } else {
                    // Computed property: [expr]: value
                    self.expect_punctuator(Punctuator::Colon)?;
                    let value = self.parse_assignment_expression()?;

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Computed(key_expr),
                        value,
                        shorthand: false,
                        computed: true,
                    });
                }
            } else if self.check_identifier("get")? {
                // Could be: get prop() {}, { get }, { get: value }, or { get() {} }
                self.lexer.next_token()?;
                // Check if this is shorthand property named "get" or method shorthand
                if self.check_punctuator(Punctuator::Colon)?
                    || self.check_punctuator(Punctuator::Comma)?
                    || self.check_punctuator(Punctuator::RBrace)?
                {
                    // Shorthand property: { get } or { get: value }
                    if self.check_punctuator(Punctuator::Colon)? {
                        self.lexer.next_token()?;
                        let value = self.parse_assignment_expression()?;
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("get".to_string()),
                            value,
                            shorthand: false,
                            computed: false,
                        });
                    } else {
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("get".to_string()),
                            value: Expression::Identifier {
                                name: "get".to_string(),
                                position: None,
                            },
                            shorthand: true,
                            computed: false,
                        });
                    }
                } else if self.check_punctuator(Punctuator::LParen)? {
                    // Method shorthand: { get() {} } - "get" is the method name
                    // Set method context for super access in parameter defaults
                    let prev_method = self.in_method;
                    self.in_method = true;
                    let params = self.parse_parameters()?;
                    let body = self.parse_method_body()?;
                    self.in_method = prev_method;

                    let func = Expression::FunctionExpression {
                        name: Some("get".to_string()),
                        params,
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier("get".to_string()),
                        value: func,
                        shorthand: false,
                        computed: false,
                    });
                } else {
                    // Getter accessor: get prop() {} or get [expr]() {}
                    let (key, computed) = if self.check_punctuator(Punctuator::LBracket)? {
                        // Computed property name: get [expr]() {} - 'in' allowed
                        self.lexer.next_token()?;
                        let prev_in_for_init = self.in_for_init;
                        self.in_for_init = false;
                        let key_expr = self.parse_assignment_expression()?;
                        self.in_for_init = prev_in_for_init;
                        self.expect_punctuator(Punctuator::RBracket)?;
                        (PropertyKey::Computed(key_expr), true)
                    } else {
                        // Regular property name (identifier, keyword, string, number)
                        let key = self.expect_property_name()?;
                        (PropertyKey::Identifier(key), false)
                    };
                    self.expect_punctuator(Punctuator::LParen)?;
                    self.expect_punctuator(Punctuator::RParen)?;
                    let body = self.parse_method_body()?;

                    let func_name = match &key {
                        PropertyKey::Identifier(s) => Some(s.clone()),
                        _ => None,
                    };
                    let func = Expression::FunctionExpression {
                        name: func_name,
                        params: vec![],
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key,
                        value: func,
                        shorthand: false,
                        computed,
                    });
                }
            } else if self.check_identifier("set")? {
                // Could be: set prop(v) {}, { set }, { set: value }, or { set() {} }
                self.lexer.next_token()?;
                // Check if this is shorthand property named "set" or method shorthand
                if self.check_punctuator(Punctuator::Colon)?
                    || self.check_punctuator(Punctuator::Comma)?
                    || self.check_punctuator(Punctuator::RBrace)?
                {
                    // Shorthand property: { set } or { set: value }
                    if self.check_punctuator(Punctuator::Colon)? {
                        self.lexer.next_token()?;
                        let value = self.parse_assignment_expression()?;
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("set".to_string()),
                            value,
                            shorthand: false,
                            computed: false,
                        });
                    } else {
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("set".to_string()),
                            value: Expression::Identifier {
                                name: "set".to_string(),
                                position: None,
                            },
                            shorthand: true,
                            computed: false,
                        });
                    }
                } else if self.check_punctuator(Punctuator::LParen)? {
                    // Method shorthand: { set(v) {} } - "set" is the method name
                    // Set method context for super access in parameter defaults
                    let prev_method = self.in_method;
                    self.in_method = true;
                    let params = self.parse_parameters()?;
                    let body = self.parse_method_body()?;
                    self.in_method = prev_method;

                    let func = Expression::FunctionExpression {
                        name: Some("set".to_string()),
                        params,
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier("set".to_string()),
                        value: func,
                        shorthand: false,
                        computed: false,
                    });
                } else {
                    // Setter accessor: set prop(v) {} or set [expr](v) {}
                    let (key, computed) = if self.check_punctuator(Punctuator::LBracket)? {
                        // Computed property name: set [expr](v) {} - 'in' allowed
                        self.lexer.next_token()?;
                        let prev_in_for_init = self.in_for_init;
                        self.in_for_init = false;
                        let key_expr = self.parse_assignment_expression()?;
                        self.in_for_init = prev_in_for_init;
                        self.expect_punctuator(Punctuator::RBracket)?;
                        (PropertyKey::Computed(key_expr), true)
                    } else {
                        // Regular property name (identifier, keyword, string, number)
                        let key = self.expect_property_name()?;
                        (PropertyKey::Identifier(key), false)
                    };
                    // Set method context for super access in parameter defaults
                    let prev_method = self.in_method;
                    self.in_method = true;
                    let params = self.parse_parameters()?;
                    let body = self.parse_method_body()?;
                    self.in_method = prev_method;

                    let func_name = match &key {
                        PropertyKey::Identifier(s) => Some(s.clone()),
                        _ => None,
                    };
                    let func = Expression::FunctionExpression {
                        name: func_name,
                        params,
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key,
                        value: func,
                        shorthand: false,
                        computed,
                    });
                }
            } else if self.check_keyword(Keyword::Async)? {
                // Async method: async name() {} or async *name() {}
                self.lexer.next_token()?;
                // Check if this is shorthand property named "async"
                if self.check_punctuator(Punctuator::Colon)?
                    || self.check_punctuator(Punctuator::Comma)?
                    || self.check_punctuator(Punctuator::RBrace)?
                {
                    if self.check_punctuator(Punctuator::Colon)? {
                        self.lexer.next_token()?;
                        let value = self.parse_assignment_expression()?;
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("async".to_string()),
                            value,
                            shorthand: false,
                            computed: false,
                        });
                    } else {
                        properties.push(ObjectProperty::Property {
                            key: PropertyKey::Identifier("async".to_string()),
                            value: Expression::Identifier {
                                name: "async".to_string(),
                                position: None,
                            },
                            shorthand: true,
                            computed: false,
                        });
                    }
                } else {
                    // Check for async generator: async *name() {}
                    let is_generator = self.check_punctuator(Punctuator::Star)?;
                    if is_generator {
                        self.lexer.next_token()?;
                    }

                    let key = self.expect_identifier()?;

                    // Set async/generator context before parsing params
                    let prev_async = self.in_async;
                    let prev_generator = self.in_generator;
                    let prev_method = self.in_method;
                    self.in_async = true;
                    self.in_generator = is_generator;
                    self.in_method = true;

                    let params = self.parse_parameters()?;
                    // Validate parameters
                    self.validate_parameters(&params)?;
                    self.validate_params_no_await(&params)?;
                    if is_generator {
                        self.validate_params_no_yield(&params)?;
                    }
                    let body = self.parse_method_body_with_context(true, is_generator)?;
                    // Validate use strict with non-simple parameters
                    self.validate_params_with_body(&params, &body)?;

                    self.in_async = prev_async;
                    self.in_generator = prev_generator;
                    self.in_method = prev_method;

                    let func = Expression::FunctionExpression {
                        name: Some(key.clone()),
                        params,
                        body,
                        is_async: true,
                        is_generator,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier(key),
                        value: func,
                        shorthand: false,
                        computed: false,
                    });
                }
            } else if self.check_punctuator(Punctuator::Star)? {
                // Generator method: *name() {}
                self.lexer.next_token()?;
                let key = self.expect_identifier()?;

                // Set generator context before parsing params
                let prev_generator = self.in_generator;
                let prev_method = self.in_method;
                self.in_generator = true;
                self.in_method = true;

                let params = self.parse_parameters()?;
                // Validate parameters
                self.validate_parameters(&params)?;
                self.validate_params_no_yield(&params)?;
                let body = self.parse_method_body_with_context(false, true)?;
                // Validate use strict with non-simple parameters
                self.validate_params_with_body(&params, &body)?;

                self.in_generator = prev_generator;
                self.in_method = prev_method;

                let func = Expression::FunctionExpression {
                    name: Some(key.clone()),
                    params,
                    body,
                    is_async: false,
                    is_generator: true,
                    position: None,
                };

                properties.push(ObjectProperty::Property {
                    key: PropertyKey::Identifier(key),
                    value: func,
                    shorthand: false,
                    computed: false,
                });
            } else {
                // Regular property or method shorthand
                // Use expect_property_name to allow keywords as property names
                let key = self.expect_property_name()?;

                if self.check_punctuator(Punctuator::LParen)? {
                    // Method shorthand: name() {}
                    // Set method context for super access in parameter defaults
                    let prev_method = self.in_method;
                    self.in_method = true;
                    let params = self.parse_parameters()?;
                    let body = self.parse_method_body()?;
                    self.in_method = prev_method;

                    let func = Expression::FunctionExpression {
                        name: Some(key.clone()),
                        params,
                        body,
                        is_async: false,
                        is_generator: false,
                        position: None,
                    };

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier(key),
                        value: func,
                        shorthand: false,
                        computed: false,
                    });
                } else if self.check_punctuator(Punctuator::Colon)? {
                    // Regular property: key: value
                    self.lexer.next_token()?;
                    let value = self.parse_assignment_expression()?;

                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier(key),
                        value,
                        shorthand: false,
                        computed: false,
                    });
                } else if self.check_punctuator(Punctuator::Assign)? {
                    // CoverInitializedName: { key = defaultValue }
                    // This is only valid when re-interpreted as a pattern
                    self.lexer.next_token()?; // consume =
                    let default_value = self.parse_assignment_expression()?;
                    // Create an AssignmentExpression that will be converted to AssignmentPattern later
                    let value = Expression::AssignmentExpression {
                        left: AssignmentTarget::Identifier(key.clone()),
                        operator: AssignmentOperator::Assign,
                        right: Box::new(default_value),
                        position: None,
                    };
                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier(key.clone()),
                        value,
                        shorthand: true,
                        computed: false,
                    });
                } else {
                    // Shorthand property: { key }
                    properties.push(ObjectProperty::Property {
                        key: PropertyKey::Identifier(key.clone()),
                        value: Expression::Identifier {
                            name: key,
                            position: None,
                        },
                        shorthand: true,
                        computed: false,
                    });
                }
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
            }
            self.lexer.next_token()?;
        }

        self.expect_punctuator(Punctuator::RBrace)?;

        Ok(Expression::ObjectExpression {
            properties,
            position: None,
        })
    }

    // Helper methods

    fn check_punctuator(&mut self, p: Punctuator) -> Result<bool, JsError> {
        Ok(matches!(self.lexer.peek_token()?, Token::Punctuator(ref x) if *x == p))
    }

    fn check_keyword(&mut self, k: Keyword) -> Result<bool, JsError> {
        Ok(matches!(self.lexer.peek_token()?, Token::Keyword(ref x) if *x == k))
    }

    fn check_identifier(&mut self, name: &str) -> Result<bool, JsError> {
        Ok(matches!(self.lexer.peek_token()?, Token::Identifier(ref x, _) if x == name))
    }

    fn check_private_identifier(&mut self) -> Result<bool, JsError> {
        Ok(matches!(self.lexer.peek_token()?, Token::PrivateIdentifier(_)))
    }

    fn expect_private_identifier(&mut self) -> Result<String, JsError> {
        let token = self.lexer.next_token()?;
        if let Token::PrivateIdentifier(name) = token {
            return Ok(name);
        }
        Err(unexpected_token(
            "private identifier",
            &format!("{:?}", token),
            None,
        ))
    }

    fn expect_punctuator(&mut self, p: Punctuator) -> Result<(), JsError> {
        let token = self.lexer.next_token()?;
        if let Token::Punctuator(ref x) = token {
            if *x == p {
                return Ok(());
            }
        }
        Err(unexpected_token(
            &format!("{:?}", p),
            &format!("{:?}", token),
            None,
        ))
    }

    fn expect_keyword(&mut self, k: Keyword) -> Result<(), JsError> {
        let token = self.lexer.next_token()?;
        if let Token::Keyword(ref x) = token {
            if *x == k {
                return Ok(());
            }
        }
        Err(unexpected_token(
            &format!("{:?}", k),
            &format!("{:?}", token),
            None,
        ))
    }

    fn expect_identifier(&mut self) -> Result<String, JsError> {
        self.update_position()?;
        let token = self.lexer.next_token()?;
        match token {
            Token::Identifier(name, _has_escapes) => {
                // Validate the identifier is not a reserved word and is valid as a binding
                // Per ES spec, even escaped reserved words are invalid as identifiers
                self.validate_binding_identifier(&name)?;
                Ok(name)
            }
            // 'await' is a keyword only in async contexts or static blocks, otherwise it's a valid identifier
            // But 'await' cannot be an identifier in module code (handled elsewhere)
            Token::Keyword(Keyword::Await) if !self.in_async && !self.in_static_block => {
                Ok("await".to_string())
            }
            // 'yield' is a keyword only in generator contexts, otherwise it's a valid identifier
            // But 'yield' is a strict-mode reserved word, so reject in strict mode
            Token::Keyword(Keyword::Yield) if !self.in_generator => {
                if self.strict_mode {
                    return Err(syntax_error(
                        "'yield' is a reserved word in strict mode",
                        self.last_position.clone(),
                    ));
                }
                Ok("yield".to_string())
            }
            _ => Err(unexpected_token(
                "identifier",
                &format!("{:?}", token),
                self.last_position.clone(),
            ))
        }
    }

    /// Expect a property name (identifier or keyword) for object properties
    /// In JS, reserved words can be used as property names without quotes
    fn expect_property_name(&mut self) -> Result<String, JsError> {
        self.update_position()?;
        let token = self.lexer.next_token()?;
        match token {
            Token::Identifier(name, _) => Ok(name),
            Token::Keyword(k) => Ok(keyword_to_string(k)),
            Token::String(s) => Ok(s),
            Token::Number(n) => Ok(n.to_string()),
            _ => Err(unexpected_token(
                "property name",
                &format!("{:?}", token),
                self.last_position.clone(),
            )),
        }
    }

    fn update_position(&mut self) -> Result<(), JsError> {
        // Get position from lexer's current state
        // This is a simplified approach - in a full implementation, the lexer would expose position
        self.last_position = Some(core_types::SourcePosition {
            line: 1,
            column: 1,
            offset: 0,
        });
        Ok(())
    }

    /// Parse a property key (identifier, keyword, string, or number)
    fn parse_property_name(&mut self) -> Result<PropertyKey, JsError> {
        let token = self.lexer.next_token()?;
        match token {
            Token::Identifier(name, _) => Ok(PropertyKey::Identifier(name)),
            Token::String(s) => Ok(PropertyKey::String(s)),
            Token::Number(n) => Ok(PropertyKey::Number(n)),
            Token::Keyword(kw) => {
                let name = self.keyword_to_string(kw);
                Ok(PropertyKey::Identifier(name))
            }
            Token::PrivateIdentifier(name) => Ok(PropertyKey::Identifier(name)),
            _ => Err(unexpected_token(
                "property name",
                &format!("{:?}", token),
                None,
            )),
        }
    }

    /// Check if next token is a valid property name
    fn is_property_name(&mut self) -> Result<bool, JsError> {
        let token = self.lexer.peek_token()?;
        Ok(matches!(
            token,
            Token::Identifier(_, _)
                | Token::Keyword(_)
                | Token::String(_)
                | Token::Number(_)
                | Token::Punctuator(Punctuator::LBracket)
                | Token::PrivateIdentifier(_)
        ))
    }

    /// Convert keyword to string
    fn keyword_to_string(&self, kw: Keyword) -> String {
        match kw {
            Keyword::Let => "let".to_string(),
            Keyword::Const => "const".to_string(),
            Keyword::Var => "var".to_string(),
            Keyword::Function => "function".to_string(),
            Keyword::Return => "return".to_string(),
            Keyword::If => "if".to_string(),
            Keyword::Else => "else".to_string(),
            Keyword::While => "while".to_string(),
            Keyword::For => "for".to_string(),
            Keyword::Break => "break".to_string(),
            Keyword::Continue => "continue".to_string(),
            Keyword::Class => "class".to_string(),
            Keyword::Extends => "extends".to_string(),
            Keyword::New => "new".to_string(),
            Keyword::This => "this".to_string(),
            Keyword::Super => "super".to_string(),
            Keyword::Async => "async".to_string(),
            Keyword::Await => "await".to_string(),
            Keyword::True => "true".to_string(),
            Keyword::False => "false".to_string(),
            Keyword::Null => "null".to_string(),
            Keyword::Typeof => "typeof".to_string(),
            Keyword::Void => "void".to_string(),
            Keyword::Instanceof => "instanceof".to_string(),
            Keyword::In => "in".to_string(),
            Keyword::Try => "try".to_string(),
            Keyword::Catch => "catch".to_string(),
            Keyword::Finally => "finally".to_string(),
            Keyword::Throw => "throw".to_string(),
            Keyword::Yield => "yield".to_string(),
            Keyword::Import => "import".to_string(),
            Keyword::Export => "export".to_string(),
            Keyword::Default => "default".to_string(),
            Keyword::Delete => "delete".to_string(),
            Keyword::With => "with".to_string(),
            Keyword::Switch => "switch".to_string(),
            Keyword::Case => "case".to_string(),
            Keyword::Do => "do".to_string(),
            Keyword::Debugger => "debugger".to_string(),
            Keyword::Static => "static".to_string(),
        }
    }

    fn expect_identifier_or_keyword(&mut self) -> Result<String, JsError> {
        let token = self.lexer.next_token()?;
        match token {
            Token::Identifier(name, _) => Ok(name),
            Token::Keyword(kw) => {
                // Allow keywords to be used as property/method names
                // Note: 'constructor' is now an identifier, not a keyword
                Ok(match kw {
                    Keyword::Let => "let".to_string(),
                    Keyword::Const => "const".to_string(),
                    Keyword::Var => "var".to_string(),
                    Keyword::Function => "function".to_string(),
                    Keyword::Return => "return".to_string(),
                    Keyword::If => "if".to_string(),
                    Keyword::Else => "else".to_string(),
                    Keyword::While => "while".to_string(),
                    Keyword::For => "for".to_string(),
                    Keyword::Break => "break".to_string(),
                    Keyword::Continue => "continue".to_string(),
                    Keyword::Class => "class".to_string(),
                    Keyword::Extends => "extends".to_string(),
                    Keyword::New => "new".to_string(),
                    Keyword::This => "this".to_string(),
                    Keyword::Super => "super".to_string(),
                    Keyword::Async => "async".to_string(),
                    Keyword::Await => "await".to_string(),
                    Keyword::True => "true".to_string(),
                    Keyword::False => "false".to_string(),
                    Keyword::Null => "null".to_string(),
                    // Note: 'undefined' is NOT a keyword
                    Keyword::Typeof => "typeof".to_string(),
                    Keyword::Void => "void".to_string(),
                    Keyword::Instanceof => "instanceof".to_string(),
                    Keyword::In => "in".to_string(),
                    Keyword::Try => "try".to_string(),
                    Keyword::Catch => "catch".to_string(),
                    Keyword::Finally => "finally".to_string(),
                    Keyword::Throw => "throw".to_string(),
                    Keyword::Yield => "yield".to_string(),
                    Keyword::Import => "import".to_string(),
                    Keyword::Export => "export".to_string(),
                    Keyword::Default => "default".to_string(),
                    Keyword::Delete => "delete".to_string(),
                    Keyword::With => "with".to_string(),
                    Keyword::Switch => "switch".to_string(),
                    Keyword::Case => "case".to_string(),
                    Keyword::Do => "do".to_string(),
                    Keyword::Debugger => "debugger".to_string(),
                    Keyword::Static => "static".to_string(),
                })
            }
            _ => Err(unexpected_token(
                "identifier or keyword",
                &format!("{:?}", token),
                None,
            )),
        }
    }

    /// Consume a semicolon, implementing Automatic Semicolon Insertion (ASI)
    /// per ECMAScript specification section 12.9
    fn consume_semicolon(&mut self) -> Result<(), JsError> {
        // If there's an explicit semicolon, consume it
        if self.check_punctuator(Punctuator::Semicolon)? {
            self.lexer.next_token()?;
            return Ok(());
        }

        // ASI Rule 1: Insert semicolon if the next token is preceded by
        // a line terminator and cannot legally follow the previous token
        if self.lexer.line_terminator_before_token {
            return Ok(());
        }

        // ASI Rule 2: Insert semicolon at end of file
        if self.is_at_end()? {
            return Ok(());
        }

        // ASI Rule 3: Insert semicolon before closing brace
        if self.check_punctuator(Punctuator::RBrace)? {
            return Ok(());
        }

        // If none of the ASI rules apply and there's no semicolon, it's an error
        Err(JsError {
            kind: core_types::ErrorKind::SyntaxError,
            message: "Expected semicolon".to_string(),
            stack: vec![],
            source_position: self.last_position.clone(),
        })
    }

    /// Consume a semicolon for do-while statements with special ASI handling.
    /// Per ECMAScript 12.9.1, ASI applies after `)` in do-while even without
    /// a line terminator: "The previous token is ) and the inserted semicolon
    /// would then be parsed as the terminating semicolon of a do-while statement."
    fn consume_semicolon_do_while(&mut self) -> Result<(), JsError> {
        // If there's an explicit semicolon, consume it
        if self.check_punctuator(Punctuator::Semicolon)? {
            self.lexer.next_token()?;
            return Ok(());
        }

        // For do-while, ASI always applies after `)` - no line terminator required
        // This is a special case in ECMAScript 12.9.1
        Ok(())
    }

    /// Check if ASI should apply for restricted productions
    /// (return, break, continue, throw must not have line terminator before operand)
    fn check_restricted_production(&self) -> bool {
        self.lexer.line_terminator_before_token
    }

    /// Check if an identifier name is a reserved word
    fn is_reserved_word(&self, name: &str) -> bool {
        matches!(
            name,
            "break" | "case" | "catch" | "continue" | "debugger"
            | "default" | "delete" | "do" | "else" | "finally"
            | "for" | "function" | "if" | "in" | "instanceof"
            | "new" | "return" | "switch" | "this" | "throw"
            | "try" | "typeof" | "var" | "void" | "while"
            | "with" | "class" | "const" | "enum" | "export"
            | "extends" | "import" | "super"
            // Literals that are also reserved words
            | "null" | "true" | "false"
        )
    }

    /// Check if an identifier name is a strict mode reserved word
    fn is_strict_reserved_word(&self, name: &str) -> bool {
        matches!(
            name,
            "implements" | "interface" | "let" | "package" | "private"
            | "protected" | "public" | "static" | "yield"
        )
    }

    /// Validate that a for-in/for-of left-hand side expression is a valid assignment target
    /// In strict mode, call expressions and other non-simple expressions are not allowed
    fn validate_for_in_of_left(&self, expr: &Expression) -> Result<(), JsError> {
        match expr {
            // Valid: identifiers
            Expression::Identifier { name, .. } => {
                // 'arguments' and 'eval' cannot be assigned in strict mode
                if self.strict_mode && (name == "arguments" || name == "eval") {
                    return Err(syntax_error(
                        &format!("'{}' cannot be assigned in strict mode", name),
                        self.last_position.clone(),
                    ));
                }
                Ok(())
            }
            // Valid: member expressions, but not optional chaining
            Expression::MemberExpression { optional, .. } => {
                if *optional {
                    Err(syntax_error(
                        "Invalid left-hand side in for-in/for-of: optional chaining not allowed",
                        self.last_position.clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            // Valid: object/array destructuring patterns - but need to check elements recursively
            Expression::ArrayExpression { elements, .. } => {
                for elem in elements {
                    if let Some(e) = elem {
                        match e {
                            ArrayElement::Expression(expr) => self.validate_for_in_of_left(expr)?,
                            ArrayElement::Spread(expr) => self.validate_for_in_of_left(expr)?,
                        }
                    }
                }
                Ok(())
            }
            Expression::ObjectExpression { properties, .. } => {
                for prop in properties {
                    match prop {
                        ObjectProperty::Property { value, .. } => {
                            self.validate_for_in_of_left(value)?;
                        }
                        ObjectProperty::SpreadElement(expr) => {
                            self.validate_for_in_of_left(expr)?;
                        }
                    }
                }
                Ok(())
            }
            // Invalid: call expressions
            Expression::CallExpression { .. } => {
                Err(syntax_error(
                    "Invalid left-hand side in for-in/for-of",
                    self.last_position.clone(),
                ))
            }
            // Invalid: parenthesized object/array (cannot be destructuring targets)
            Expression::ParenthesizedExpression { expression, .. } => {
                match &**expression {
                    Expression::Identifier { name, .. } => {
                        if self.strict_mode && (name == "arguments" || name == "eval") {
                            return Err(syntax_error(
                                &format!("'{}' cannot be assigned in strict mode", name),
                                self.last_position.clone(),
                            ));
                        }
                        Ok(())
                    }
                    Expression::MemberExpression { optional, .. } => {
                        if *optional {
                            Err(syntax_error(
                                "Invalid left-hand side in for-in/for-of: optional chaining not allowed",
                                self.last_position.clone(),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    Expression::ObjectExpression { .. } | Expression::ArrayExpression { .. } => {
                        Err(syntax_error(
                            "Invalid left-hand side in for-in/for-of",
                            self.last_position.clone(),
                        ))
                    }
                    _ => Err(syntax_error(
                        "Invalid left-hand side in for-in/for-of",
                        self.last_position.clone(),
                    )),
                }
            }
            // All other expressions are invalid
            _ => Err(syntax_error(
                "Invalid left-hand side in for-in/for-of",
                self.last_position.clone(),
            )),
        }
    }

    /// Validate that an identifier is not a reserved word
    /// Validate an identifier reference (not binding)
    /// This is called when an identifier is used as a reference (e.g., in expressions)
    fn validate_identifier(&self, name: &str) -> Result<(), JsError> {
        if self.is_reserved_word(name) {
            return Err(syntax_error(
                &format!("'{}' is a reserved word and cannot be used as an identifier", name),
                self.last_position.clone(),
            ));
        }

        if self.strict_mode && self.is_strict_reserved_word(name) {
            return Err(syntax_error(
                &format!("'{}' is a reserved word in strict mode", name),
                self.last_position.clone(),
            ));
        }

        // 'await' is a reserved word in async function contexts
        if self.in_async && name == "await" {
            return Err(syntax_error(
                "'await' is not allowed as an identifier in async functions",
                self.last_position.clone(),
            ));
        }

        // 'yield' is a reserved word in generator function contexts
        if self.in_generator && name == "yield" {
            return Err(syntax_error(
                "'yield' is not allowed as an identifier in generator functions",
                self.last_position.clone(),
            ));
        }

        // In class static blocks, 'await' and 'arguments' are reserved
        if self.in_static_block {
            if name == "await" {
                return Err(syntax_error(
                    "'await' is not allowed as an identifier in class static blocks",
                    self.last_position.clone(),
                ));
            }
            if name == "arguments" {
                return Err(syntax_error(
                    "'arguments' is not allowed as an identifier in class static blocks",
                    self.last_position.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Validate a binding identifier (e.g., in variable declarations, function names, parameters)
    /// This is stricter than validate_identifier because it also restricts eval/arguments in strict mode
    fn validate_binding_identifier(&self, name: &str) -> Result<(), JsError> {
        // First apply all the regular identifier checks
        self.validate_identifier(name)?;

        // 'arguments' and 'eval' cannot be used as binding identifiers in strict mode
        if self.strict_mode && (name == "arguments" || name == "eval") {
            return Err(syntax_error(
                &format!("'{}' cannot be used as a binding identifier in strict mode", name),
                self.last_position.clone(),
            ));
        }

        Ok(())
    }

    /// Check if an expression is a valid assignment target
    fn is_valid_assignment_target(&self, expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Identifier { .. }
                | Expression::MemberExpression { .. }
        )
    }

    /// Collect all bound names from a pattern into a vector
    fn collect_bound_names(pattern: &Pattern, names: &mut Vec<String>) {
        match pattern {
            Pattern::Identifier(name) => names.push(name.clone()),
            Pattern::ObjectPattern(properties) => {
                for prop in properties {
                    // ObjectPatternProperty has key and value fields
                    Self::collect_bound_names(&prop.value, names);
                }
            }
            Pattern::ArrayPattern(elements) => {
                for elem in elements {
                    if let Some(pattern) = elem {
                        Self::collect_bound_names(pattern, names);
                    }
                }
            }
            Pattern::AssignmentPattern { left, .. } => {
                Self::collect_bound_names(left, names);
            }
            Pattern::RestElement(inner) => {
                Self::collect_bound_names(inner, names);
            }
            // Member expressions don't bind new names - they assign to existing properties
            Pattern::MemberExpression(_) => {}
        }
    }

    /// Check if a parameter list is "simple" (no defaults, rest, or destructuring)
    fn is_simple_parameter_list(params: &[Pattern]) -> bool {
        params.iter().all(|p| matches!(p, Pattern::Identifier(_)))
    }

    /// Check if body contains a "use strict" directive as first statement
    fn body_contains_use_strict(body: &[Statement]) -> bool {
        if let Some(first) = body.first() {
            if let Statement::ExpressionStatement { expression, .. } = first {
                if let Expression::Literal { value: crate::ast::Literal::String(s), .. } = expression {
                    return s == "use strict";
                }
            }
        }
        false
    }

    /// Validate that non-simple parameters are not used with "use strict" directive in body
    fn validate_params_with_body(&self, params: &[Pattern], body: &[Statement]) -> Result<(), JsError> {
        if Self::body_contains_use_strict(body) && !Self::is_simple_parameter_list(params) {
            return Err(syntax_error(
                "Illegal 'use strict' directive in function with non-simple parameter list",
                self.last_position.clone(),
            ));
        }
        Ok(())
    }

    /// Validate arrow function params with arrow body
    fn validate_arrow_params_with_body(&self, params: &[Pattern], body: &ArrowFunctionBody) -> Result<(), JsError> {
        if let ArrowFunctionBody::Block(stmts) = body {
            if Self::body_contains_use_strict(stmts) && !Self::is_simple_parameter_list(params) {
                return Err(syntax_error(
                    "Illegal 'use strict' directive in function with non-simple parameter list",
                    self.last_position.clone(),
                ));
            }
            // Also check for parameter/lexical declaration conflicts
            self.validate_params_body_lexical(params, stmts)?;
        }
        Ok(())
    }

    /// Collect lexically declared names (let/const) from top-level statements
    fn collect_lexically_declared_names(body: &[Statement], names: &mut Vec<String>) {
        for stmt in body {
            match stmt {
                Statement::VariableDeclaration { kind, declarations, .. } => {
                    // Only let and const are lexical declarations
                    if matches!(kind, crate::ast::VariableKind::Let | crate::ast::VariableKind::Const) {
                        for decl in declarations {
                            Self::collect_bound_names(&decl.id, names);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Validate that parameter names don't conflict with lexically declared names in body
    fn validate_params_body_lexical(&self, params: &[Pattern], body: &[Statement]) -> Result<(), JsError> {
        // Collect parameter bound names
        let mut param_names = Vec::new();
        for param in params {
            Self::collect_bound_names(param, &mut param_names);
        }

        // Collect lexically declared names from body
        let mut lexical_names = Vec::new();
        Self::collect_lexically_declared_names(body, &mut lexical_names);

        // Check for conflicts
        for param_name in &param_names {
            if lexical_names.contains(param_name) {
                return Err(syntax_error(
                    &format!("Identifier '{}' has already been declared", param_name),
                    self.last_position.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Validate parameters for duplicates based on context
    fn validate_parameters(&self, params: &[Pattern]) -> Result<(), JsError> {
        let mut names = Vec::new();
        for param in params {
            Self::collect_bound_names(param, &mut names);
        }

        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            if !seen.insert(name.clone()) {
                // Duplicate found
                // In strict mode or async functions, always error
                // In non-strict mode with non-simple params, also error
                if self.strict_mode || self.in_async || !Self::is_simple_parameter_list(params) {
                    return Err(syntax_error(
                        &format!("Duplicate parameter name '{}'", name),
                        self.last_position.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate arrow function parameters - always rejects duplicates and yield expressions
    fn validate_arrow_parameters(&self, params: &[Pattern]) -> Result<(), JsError> {
        let mut names = Vec::new();
        for param in params {
            Self::collect_bound_names(param, &mut names);
        }

        // Arrow functions ALWAYS reject duplicate parameters (even in non-strict mode)
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            if !seen.insert(name.clone()) {
                return Err(syntax_error(
                    &format!("Duplicate parameter name '{}'", name),
                    self.last_position.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Check if an expression contains a YieldExpression
    fn expression_contains_yield(expr: &Expression) -> bool {
        match expr {
            Expression::YieldExpression { .. } => true,
            Expression::BinaryExpression { left, right, .. } => {
                Self::expression_contains_yield(left) || Self::expression_contains_yield(right)
            }
            Expression::AssignmentExpression { right, .. } => {
                Self::expression_contains_yield(right)
            }
            Expression::ConditionalExpression { test, consequent, alternate, .. } => {
                Self::expression_contains_yield(test)
                    || Self::expression_contains_yield(consequent)
                    || Self::expression_contains_yield(alternate)
            }
            Expression::UnaryExpression { argument, .. } => {
                Self::expression_contains_yield(argument)
            }
            Expression::CallExpression { callee, arguments, .. } => {
                Self::expression_contains_yield(callee)
                    || arguments.iter().any(|arg| Self::expression_contains_yield(arg))
            }
            Expression::MemberExpression { object, property, .. } => {
                Self::expression_contains_yield(object)
                    || Self::expression_contains_yield(property)
            }
            Expression::ArrayExpression { elements, .. } => {
                elements.iter().any(|elem| {
                    if let Some(arr_elem) = elem {
                        match arr_elem {
                            crate::ast::ArrayElement::Expression(expr) => Self::expression_contains_yield(expr),
                            crate::ast::ArrayElement::Spread(expr) => Self::expression_contains_yield(expr),
                        }
                    } else {
                        false
                    }
                })
            }
            Expression::ObjectExpression { properties, .. } => {
                properties.iter().any(|prop| {
                    match prop {
                        crate::ast::ObjectProperty::Property { value, key, .. } => {
                            Self::expression_contains_yield(value)
                                || match key {
                                    crate::ast::PropertyKey::Computed(expr) => Self::expression_contains_yield(expr),
                                    _ => false,
                                }
                        }
                        crate::ast::ObjectProperty::SpreadElement(expr) => Self::expression_contains_yield(expr),
                    }
                })
            }
            Expression::SequenceExpression { expressions, .. } => {
                expressions.iter().any(|e| Self::expression_contains_yield(e))
            }
            _ => false,
        }
    }

    /// Check if a pattern contains a yield expression (in default values)
    fn pattern_contains_yield(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Identifier(_) => false,
            Pattern::ObjectPattern(properties) => {
                properties.iter().any(|prop| Self::pattern_contains_yield(&prop.value))
            }
            Pattern::ArrayPattern(elements) => {
                elements.iter().any(|elem| {
                    if let Some(pat) = elem {
                        Self::pattern_contains_yield(pat)
                    } else {
                        false
                    }
                })
            }
            Pattern::AssignmentPattern { left, right } => {
                Self::pattern_contains_yield(left) || Self::expression_contains_yield(right)
            }
            Pattern::RestElement(inner) => Self::pattern_contains_yield(inner),
            Pattern::MemberExpression(_) => false,
        }
    }

    /// Validate that arrow parameters don't contain yield expressions (when in generator)
    fn validate_arrow_params_no_yield(&self, params: &[Pattern]) -> Result<(), JsError> {
        if self.in_generator {
            for param in params {
                if Self::pattern_contains_yield(param) {
                    return Err(syntax_error(
                        "Arrow parameters cannot contain yield expressions",
                        self.last_position.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Check if an expression contains an AwaitExpression
    fn expression_contains_await(expr: &Expression) -> bool {
        match expr {
            Expression::AwaitExpression { .. } => true,
            Expression::BinaryExpression { left, right, .. } => {
                Self::expression_contains_await(left) || Self::expression_contains_await(right)
            }
            Expression::AssignmentExpression { right, .. } => {
                Self::expression_contains_await(right)
            }
            Expression::ConditionalExpression { test, consequent, alternate, .. } => {
                Self::expression_contains_await(test)
                    || Self::expression_contains_await(consequent)
                    || Self::expression_contains_await(alternate)
            }
            Expression::UnaryExpression { argument, .. } => {
                Self::expression_contains_await(argument)
            }
            Expression::CallExpression { callee, arguments, .. } => {
                Self::expression_contains_await(callee)
                    || arguments.iter().any(|arg| Self::expression_contains_await(arg))
            }
            Expression::MemberExpression { object, property, .. } => {
                Self::expression_contains_await(object)
                    || Self::expression_contains_await(property)
            }
            Expression::ArrayExpression { elements, .. } => {
                elements.iter().any(|elem| {
                    if let Some(arr_elem) = elem {
                        match arr_elem {
                            crate::ast::ArrayElement::Expression(expr) => Self::expression_contains_await(expr),
                            crate::ast::ArrayElement::Spread(expr) => Self::expression_contains_await(expr),
                        }
                    } else {
                        false
                    }
                })
            }
            Expression::ObjectExpression { properties, .. } => {
                properties.iter().any(|prop| {
                    match prop {
                        crate::ast::ObjectProperty::Property { value, key, .. } => {
                            Self::expression_contains_await(value)
                                || match key {
                                    crate::ast::PropertyKey::Computed(expr) => Self::expression_contains_await(expr),
                                    _ => false,
                                }
                        }
                        crate::ast::ObjectProperty::SpreadElement(expr) => Self::expression_contains_await(expr),
                    }
                })
            }
            Expression::SequenceExpression { expressions, .. } => {
                expressions.iter().any(|e| Self::expression_contains_await(e))
            }
            _ => false,
        }
    }

    /// Check if a pattern contains an await expression (in default values)
    fn pattern_contains_await(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Identifier(_) => false,
            Pattern::ObjectPattern(properties) => {
                properties.iter().any(|prop| Self::pattern_contains_await(&prop.value))
            }
            Pattern::ArrayPattern(elements) => {
                elements.iter().any(|elem| {
                    if let Some(pat) = elem {
                        Self::pattern_contains_await(pat)
                    } else {
                        false
                    }
                })
            }
            Pattern::AssignmentPattern { left, right } => {
                Self::pattern_contains_await(left) || Self::expression_contains_await(right)
            }
            Pattern::RestElement(inner) => Self::pattern_contains_await(inner),
            Pattern::MemberExpression(_) => false,
        }
    }

    /// Validate that formal parameters don't contain await expressions
    /// This is an early error for async functions and async generators
    fn validate_params_no_await(&self, params: &[Pattern]) -> Result<(), JsError> {
        for param in params {
            if Self::pattern_contains_await(param) {
                return Err(syntax_error(
                    "Formal parameters cannot contain await expressions",
                    self.last_position.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Validate that formal parameters don't contain yield expressions
    /// This is an early error for generator functions and async generator functions
    fn validate_params_no_yield(&self, params: &[Pattern]) -> Result<(), JsError> {
        for param in params {
            if Self::pattern_contains_yield(param) {
                return Err(syntax_error(
                    "Formal parameters cannot contain yield expressions",
                    self.last_position.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Check if an expression contains an `arguments` identifier reference
    /// This implements the ContainsArguments static semantics
    /// - Returns true if `arguments` identifier is found
    /// - Recurses into arrow functions (they don't have their own `arguments` binding)
    /// - Does NOT recurse into regular function expressions (they have their own `arguments`)
    fn expression_contains_arguments(expr: &Expression) -> bool {
        match expr {
            // Check for `arguments` identifier
            Expression::Identifier { name, .. } if name == "arguments" => true,

            // Binary and assignment expressions
            Expression::BinaryExpression { left, right, .. } => {
                Self::expression_contains_arguments(left) || Self::expression_contains_arguments(right)
            }
            Expression::AssignmentExpression { left, right, .. } => {
                Self::assignment_target_contains_arguments(left) || Self::expression_contains_arguments(right)
            }
            Expression::ConditionalExpression { test, consequent, alternate, .. } => {
                Self::expression_contains_arguments(test)
                    || Self::expression_contains_arguments(consequent)
                    || Self::expression_contains_arguments(alternate)
            }
            Expression::UnaryExpression { argument, .. } => {
                Self::expression_contains_arguments(argument)
            }
            Expression::UpdateExpression { argument, .. } => {
                Self::expression_contains_arguments(argument)
            }
            Expression::CallExpression { callee, arguments, .. } => {
                Self::expression_contains_arguments(callee)
                    || arguments.iter().any(|arg| Self::expression_contains_arguments(arg))
            }
            Expression::NewExpression { callee, arguments, .. } => {
                Self::expression_contains_arguments(callee)
                    || arguments.iter().any(|arg| Self::expression_contains_arguments(arg))
            }
            Expression::MemberExpression { object, property, computed, .. } => {
                Self::expression_contains_arguments(object)
                    || (*computed && Self::expression_contains_arguments(property))
            }
            Expression::ArrayExpression { elements, .. } => {
                elements.iter().any(|elem| {
                    if let Some(arr_elem) = elem {
                        match arr_elem {
                            crate::ast::ArrayElement::Expression(expr) => Self::expression_contains_arguments(expr),
                            crate::ast::ArrayElement::Spread(expr) => Self::expression_contains_arguments(expr),
                        }
                    } else {
                        false
                    }
                })
            }
            Expression::ObjectExpression { properties, .. } => {
                properties.iter().any(|prop| {
                    match prop {
                        crate::ast::ObjectProperty::Property { value, key, .. } => {
                            Self::expression_contains_arguments(value)
                                || match key {
                                    crate::ast::PropertyKey::Computed(expr) => Self::expression_contains_arguments(expr),
                                    _ => false,
                                }
                        }
                        crate::ast::ObjectProperty::SpreadElement(expr) => Self::expression_contains_arguments(expr),
                    }
                })
            }
            Expression::SequenceExpression { expressions, .. } => {
                expressions.iter().any(|e| Self::expression_contains_arguments(e))
            }
            Expression::TemplateLiteral { expressions, .. } => {
                expressions.iter().any(|e| Self::expression_contains_arguments(e))
            }
            Expression::TaggedTemplateExpression { tag, quasi, .. } => {
                Self::expression_contains_arguments(tag)
                    || Self::expression_contains_arguments(quasi)
            }

            // Arrow functions: RECURSE - they don't have their own `arguments` binding
            Expression::ArrowFunctionExpression { params, body, .. } => {
                // Check parameters for arguments in default values
                params.iter().any(|p| Self::pattern_contains_arguments(p))
                    || match body {
                        crate::ast::ArrowFunctionBody::Expression(expr) => Self::expression_contains_arguments(expr),
                        crate::ast::ArrowFunctionBody::Block(stmts) => Self::statements_contain_arguments(stmts),
                    }
            }

            // Regular function expressions: DO NOT recurse - they have their own `arguments`
            Expression::FunctionExpression { .. } => false,

            // Class expressions: DO NOT recurse - field initializers are checked separately
            Expression::ClassExpression { .. } => false,

            // Other expressions that don't contain arguments
            _ => false,
        }
    }

    /// Check if an assignment target contains `arguments`
    fn assignment_target_contains_arguments(target: &crate::ast::AssignmentTarget) -> bool {
        match target {
            crate::ast::AssignmentTarget::Identifier(name) => name == "arguments",
            crate::ast::AssignmentTarget::Member(expr) => Self::expression_contains_arguments(expr),
            crate::ast::AssignmentTarget::Pattern(pattern) => Self::pattern_contains_arguments(pattern),
        }
    }

    /// Check if a pattern contains `arguments` identifier (in default values)
    fn pattern_contains_arguments(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Identifier(name) => name == "arguments",
            Pattern::ObjectPattern(properties) => {
                properties.iter().any(|prop| {
                    Self::pattern_contains_arguments(&prop.value)
                        || match &prop.key {
                            crate::ast::PatternKey::Computed(expr) => Self::expression_contains_arguments(expr),
                            _ => false,
                        }
                })
            }
            Pattern::ArrayPattern(elements) => {
                elements.iter().any(|elem| {
                    if let Some(pat) = elem {
                        Self::pattern_contains_arguments(pat)
                    } else {
                        false
                    }
                })
            }
            Pattern::AssignmentPattern { left, right } => {
                Self::pattern_contains_arguments(left) || Self::expression_contains_arguments(right)
            }
            Pattern::RestElement(inner) => Self::pattern_contains_arguments(inner),
            Pattern::MemberExpression(expr) => Self::expression_contains_arguments(expr),
        }
    }

    /// Check if a list of statements contains `arguments` (for arrow function bodies)
    fn statements_contain_arguments(stmts: &[Statement]) -> bool {
        stmts.iter().any(|stmt| Self::statement_contains_arguments(stmt))
    }

    /// Check if a single statement contains `arguments`
    fn statement_contains_arguments(stmt: &Statement) -> bool {
        match stmt {
            Statement::ExpressionStatement { expression, .. } => Self::expression_contains_arguments(expression),
            Statement::ReturnStatement { argument, .. } => {
                argument.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
            }
            Statement::ThrowStatement { argument, .. } => Self::expression_contains_arguments(argument),
            Statement::IfStatement { test, consequent, alternate, .. } => {
                Self::expression_contains_arguments(test)
                    || Self::statement_contains_arguments(consequent)
                    || alternate.as_ref().map_or(false, |s| Self::statement_contains_arguments(s))
            }
            Statement::WhileStatement { test, body, .. } => {
                Self::expression_contains_arguments(test) || Self::statement_contains_arguments(body)
            }
            Statement::DoWhileStatement { test, body, .. } => {
                Self::expression_contains_arguments(test) || Self::statement_contains_arguments(body)
            }
            Statement::ForStatement { init, test, update, body, .. } => {
                init.as_ref().map_or(false, |i| match i {
                    crate::ast::ForInit::Expression(e) => Self::expression_contains_arguments(e),
                    crate::ast::ForInit::VariableDeclaration { declarations, .. } => {
                        declarations.iter().any(|d| {
                            Self::pattern_contains_arguments(&d.id)
                                || d.init.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
                        })
                    }
                })
                || test.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
                || update.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
                || Self::statement_contains_arguments(body)
            }
            Statement::ForInStatement { left, right, body, .. } => {
                Self::forin_left_contains_arguments(left)
                || Self::expression_contains_arguments(right)
                || Self::statement_contains_arguments(body)
            }
            Statement::ForOfStatement { left, right, body, .. } => {
                Self::forin_left_contains_arguments(left)
                || Self::expression_contains_arguments(right)
                || Self::statement_contains_arguments(body)
            }
            Statement::SwitchStatement { discriminant, cases, .. } => {
                Self::expression_contains_arguments(discriminant)
                    || cases.iter().any(|case| {
                        case.test.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
                            || Self::statements_contain_arguments(&case.consequent)
                    })
            }
            Statement::TryStatement { block, handler, finalizer, .. } => {
                Self::statements_contain_arguments(block)
                    || handler.as_ref().map_or(false, |h| {
                        h.param.as_ref().map_or(false, |p| Self::pattern_contains_arguments(p))
                            || Self::statements_contain_arguments(&h.body)
                    })
                    || finalizer.as_ref().map_or(false, |f| Self::statements_contain_arguments(f))
            }
            Statement::BlockStatement { body, .. } => Self::statements_contain_arguments(body),
            Statement::VariableDeclaration { declarations, .. } => {
                declarations.iter().any(|d| {
                    Self::pattern_contains_arguments(&d.id)
                        || d.init.as_ref().map_or(false, |e| Self::expression_contains_arguments(e))
                })
            }
            Statement::LabeledStatement { body, .. } => Self::statement_contains_arguments(body),
            Statement::WithStatement { object, body, .. } => {
                Self::expression_contains_arguments(object) || Self::statement_contains_arguments(body)
            }
            // Function/class declarations don't propagate arguments
            Statement::FunctionDeclaration { .. } => false,
            Statement::ClassDeclaration { .. } => false,
            _ => false,
        }
    }

    /// Check if a for-in/for-of left side contains `arguments`
    fn forin_left_contains_arguments(left: &crate::ast::ForInOfLeft) -> bool {
        match left {
            crate::ast::ForInOfLeft::Pattern(p) => Self::pattern_contains_arguments(p),
            crate::ast::ForInOfLeft::VariableDeclaration { id, .. } => Self::pattern_contains_arguments(id),
            crate::ast::ForInOfLeft::Expression(e) => Self::expression_contains_arguments(e),
        }
    }

    /// Check if an expression is a private name member access (for delete validation)
    /// This handles:
    /// - MemberExpression.PrivateName (e.g., obj.#prop)
    /// - CallExpression.PrivateName (e.g., fn().#prop)
    /// - Covered/parenthesized forms (e.g., (obj.#prop))
    fn expression_has_private_name_access(expr: &Expression) -> bool {
        match expr {
            // Direct member expression with private name
            Expression::MemberExpression { property, computed: false, .. } => {
                // Check if property is a private identifier (starts with #)
                if let Expression::Identifier { name, .. } = property.as_ref() {
                    name.starts_with('#')
                } else {
                    false
                }
            }
            // Parenthesized expression - unwrap and recurse
            Expression::ParenthesizedExpression { expression, .. } => {
                Self::expression_has_private_name_access(expression)
            }
            // Sequence expression with single item (fallback for some cases)
            Expression::SequenceExpression { expressions, .. } if expressions.len() == 1 => {
                Self::expression_has_private_name_access(&expressions[0])
            }
            // Handle other cases that might wrap the private name access
            _ => false,
        }
    }

    /// Parse a statement in a context where lexical declarations are not allowed
    /// (e.g., after if, while, for without braces)
    fn parse_substatement(&mut self) -> Result<Statement, JsError> {
        let token = self.lexer.peek_token()?.clone();

        // Lexical declarations (let, const) are not allowed in statement positions
        // But in sloppy mode, `let` can be an identifier if not followed by identifier or [
        if matches!(token, Token::Keyword(Keyword::Const)) {
            return Err(syntax_error(
                "Lexical declaration cannot appear in a single-statement context",
                self.last_position.clone(),
            ));
        }
        if matches!(token, Token::Keyword(Keyword::Let)) {
            // In strict mode, `let` is always a keyword
            if self.strict_mode {
                return Err(syntax_error(
                    "Lexical declaration cannot appear in a single-statement context",
                    self.last_position.clone(),
                ));
            }
            // In sloppy mode, peek ahead to see if this looks like a lexical declaration
            // `let` followed by identifier, `[`, or `{` (without line terminator) is a declaration
            // `let` followed by a line terminator is an identifier expression
            self.lexer.next_token()?; // consume 'let'
            // First peek the next token so that line_terminator_before_token is updated
            let next_token = self.lexer.peek_token()?.clone();
            let is_declaration = !self.lexer.line_terminator_before_token
                && matches!(
                    next_token,
                    Token::Identifier(_, _) | Token::Punctuator(Punctuator::LBracket) | Token::Punctuator(Punctuator::LBrace)
                );

            if is_declaration {
                return Err(syntax_error(
                    "Lexical declaration cannot appear in a single-statement context",
                    self.last_position.clone(),
                ));
            }

            // It's an expression statement with `let` as identifier
            // Parse the rest of the expression (if any) and create an ExpressionStatement
            let let_expr = Expression::Identifier {
                name: "let".to_string(),
                position: None,
            };

            // Check if there's more to the expression (e.g., let.foo or let())
            let expr = if self.lexer.line_terminator_before_token
                || self.check_punctuator(Punctuator::Semicolon)?
                || self.is_at_end()?
            {
                let_expr
            } else {
                // There's more - continue parsing as member/call expression
                // For now, just return the identifier and let ASI handle the rest
                let_expr
            };

            // Handle ASI
            if !self.check_punctuator(Punctuator::Semicolon)?
                && !self.lexer.line_terminator_before_token
                && !self.is_at_end()?
                && !self.check_punctuator(Punctuator::RBrace)?
            {
                return Err(syntax_error(
                    "Missing semicolon after expression statement",
                    self.last_position.clone(),
                ));
            }
            if self.check_punctuator(Punctuator::Semicolon)? {
                self.lexer.next_token()?;
            }

            return Ok(Statement::ExpressionStatement {
                expression: expr,
                position: None,
            });
        }

        // Class declarations are not allowed in statement positions
        if matches!(token, Token::Keyword(Keyword::Class)) {
            return Err(syntax_error(
                "Class declaration cannot appear in a single-statement context",
                self.last_position.clone(),
            ));
        }

        // Function declarations are not allowed in statement positions (ES6+)
        // This includes both sync and async functions
        if matches!(token, Token::Keyword(Keyword::Function)) {
            return Err(syntax_error(
                "Function declaration cannot appear in a single-statement context",
                self.last_position.clone(),
            ));
        }

        // Async function declarations are also not allowed in statement positions
        if matches!(token, Token::Keyword(Keyword::Async)) {
            // Peek ahead to see if this is an async function declaration
            let saved_pos = self.lexer.position;
            let saved_line = self.lexer.line;
            let saved_column = self.lexer.column;
            let saved_line_term = self.lexer.line_terminator_before_token;

            self.lexer.next_token()?;
            let next = self.lexer.peek_token()?;
            let is_async_function = matches!(next, Token::Keyword(Keyword::Function))
                && !self.lexer.line_terminator_before_token;

            // Restore position
            self.lexer.position = saved_pos;
            self.lexer.line = saved_line;
            self.lexer.column = saved_column;
            self.lexer.line_terminator_before_token = saved_line_term;
            self.lexer.current_token = Some(token.clone());

            if is_async_function {
                return Err(syntax_error(
                    "Async function declaration cannot appear in a single-statement context",
                    self.last_position.clone(),
                ));
            }
        }

        // Parse the statement normally
        self.parse_statement()
    }
}

/// Convert a Keyword enum to its string representation
fn keyword_to_string(k: Keyword) -> String {
    match k {
        Keyword::Let => "let".to_string(),
        Keyword::Const => "const".to_string(),
        Keyword::Var => "var".to_string(),
        Keyword::Function => "function".to_string(),
        Keyword::Return => "return".to_string(),
        Keyword::If => "if".to_string(),
        Keyword::Else => "else".to_string(),
        Keyword::While => "while".to_string(),
        Keyword::For => "for".to_string(),
        Keyword::Break => "break".to_string(),
        Keyword::Continue => "continue".to_string(),
        Keyword::Class => "class".to_string(),
        Keyword::Extends => "extends".to_string(),
        Keyword::New => "new".to_string(),
        Keyword::This => "this".to_string(),
        Keyword::Super => "super".to_string(),
        Keyword::Async => "async".to_string(),
        Keyword::Await => "await".to_string(),
        Keyword::True => "true".to_string(),
        Keyword::False => "false".to_string(),
        Keyword::Null => "null".to_string(),
        Keyword::Typeof => "typeof".to_string(),
        Keyword::Void => "void".to_string(),
        Keyword::Instanceof => "instanceof".to_string(),
        Keyword::In => "in".to_string(),
        Keyword::Try => "try".to_string(),
        Keyword::Catch => "catch".to_string(),
        Keyword::Finally => "finally".to_string(),
        Keyword::Throw => "throw".to_string(),
        Keyword::Yield => "yield".to_string(),
        Keyword::Import => "import".to_string(),
        Keyword::Export => "export".to_string(),
        Keyword::Default => "default".to_string(),
        Keyword::Delete => "delete".to_string(),
        Keyword::With => "with".to_string(),
        Keyword::Switch => "switch".to_string(),
        Keyword::Case => "case".to_string(),
        Keyword::Do => "do".to_string(),
        Keyword::Debugger => "debugger".to_string(),
        Keyword::Static => "static".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_program() {
        let mut parser = Parser::new("");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(stmts) if stmts.is_empty()));
    }

    #[test]
    fn test_parse_variable_declaration() {
        let mut parser = Parser::new("let x = 42;");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_parse_function_declaration() {
        let mut parser = Parser::new("function foo() { return 1; }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_parse_binary_expression() {
        let mut parser = Parser::new("1 + 2;");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_parse_arrow_function() {
        let mut parser = Parser::new("const f = () => 1;");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    // Automatic Semicolon Insertion (ASI) tests
    #[test]
    fn test_asi_at_end_of_file() {
        // ASI Rule 2: Insert semicolon at EOF
        let mut parser = Parser::new("let x = 1");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_asi_before_closing_brace() {
        // ASI Rule 3: Insert semicolon before }
        let mut parser = Parser::new("function f() { return 1 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_asi_after_newline() {
        // ASI Rule 1: Insert semicolon when next token is on new line
        let mut parser = Parser::new("let x = 1\nlet y = 2");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_asi_return_with_newline() {
        // Restricted production: return followed by newline becomes return;
        let mut parser = Parser::new("function f() {\nreturn\n1\n}");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast, ASTNode::Program(_)));
    }

    #[test]
    fn test_asi_throw_newline_error() {
        // Restricted production: throw with newline is an error
        let mut parser = Parser::new("throw\nError()");
        assert!(parser.parse().is_err());
    }

    // Syntax strictness tests

    #[test]
    fn test_reject_let_in_if_consequent() {
        let mut parser = Parser::new("if (true) let x = 1;");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Lexical declaration"));
    }

    #[test]
    fn test_reject_const_in_if_alternate() {
        let mut parser = Parser::new("if (false) ; else const y = 2;");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Lexical declaration"));
    }

    #[test]
    fn test_reject_class_in_while_body() {
        let mut parser = Parser::new("while (true) class C {}");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Class declaration"));
    }

    #[test]
    fn test_reject_let_in_for_body() {
        let mut parser = Parser::new("for (let i = 0; i < 10; i++) let x = 1;");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Lexical declaration"));
    }

    #[test]
    fn test_reject_break_outside_loop() {
        let mut parser = Parser::new("break;");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Illegal break"));
    }

    #[test]
    fn test_reject_continue_outside_loop() {
        let mut parser = Parser::new("continue;");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Illegal continue"));
    }

    #[test]
    fn test_accept_break_in_while_loop() {
        let mut parser = Parser::new("while (true) { break; }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_accept_continue_in_for_loop() {
        let mut parser = Parser::new("for (let i = 0; i < 10; i++) { continue; }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_accept_block_statement_in_if() {
        // Block statements are allowed in single-statement contexts
        let mut parser = Parser::new("if (true) { let x = 1; }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rest_parameter_with_array_pattern() {
        let mut parser = Parser::new("function f(...[a, b]) {}");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_rest_parameter_with_object_pattern() {
        let mut parser = Parser::new("function f(...{x, y}) {}");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_async_generator_method() {
        let mut parser = Parser::new("let o = { async *f(p = 1, x) {} };");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_two_string_expressions_asi() {
        // Two adjacent string expressions should work with ASI
        let mut parser = Parser::new("'a'\n'b'");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_switch_statement() {
        let mut parser = Parser::new("switch(x) { case 1: break; default: break; }");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_do_while_statement() {
        let mut parser = Parser::new("do { x++; } while(x < 10);");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_prototype_assignment() {
        let mut parser = Parser::new("Test262Error.prototype = new Error();");
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_harness_prelude() {
        let code = r#"
function Test262Error(message) {
    this.message = message || '';
    this.name = 'Test262Error';
}
Test262Error.prototype = new Error();
Test262Error.prototype.constructor = Test262Error;

var $262 = {
    createRealm: function() { return {}; },
    detachArrayBuffer: function(ab) { },
    gc: function() { },
    global: this
};

function assert(condition, message) {
    if (!condition) {
        throw new Error("Assertion failed: " + (message || ""));
    }
}

assert.sameValue = function(actual, expected, message) {
    if (actual !== expected) {
        throw new Error("Expected " + expected + " but got " + actual);
    }
};
"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_compound_assignment_operators() {
        // Test all compound assignment operators
        let operators = vec![
            "x |= 1;",
            "x &= 1;",
            "x ^= 1;",
            "x <<= 1;",
            "x >>= 1;",
            "x >>>= 1;",
            "x **= 2;",
            "x ||= 1;",
            "x &&= 1;",
            "x ??= 1;",
        ];
        for code in operators {
            let mut parser = Parser::new(code);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse '{}': {:?}", code, result.err());
        }
    }

    #[test]
    fn test_computed_property_with_assignment() {
        // Test computed property in object literal with compound assignment
        let code = r#"let x = 0; let o = { [x |= 1]: 5 };"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Object computed error: {:?}", result.err());

        // Test computed property getter in class
        let code2 = r#"let x = 0; class C { get [x |= 1]() { return 2; } }"#;
        let mut parser = Parser::new(code2);
        let result = parser.parse();
        assert!(result.is_ok(), "Class getter computed error: {:?}", result.err());
    }

    #[test]
    fn test_computed_destructuring_assignment() {
        // First test: numbers with trailing decimal
        let code0 = r#"var a = 1.;"#;
        let mut parser = Parser::new(code0);
        let result = parser.parse();
        assert!(result.is_ok(), "Trailing decimal number error: {:?}", result.err());

        let code = r#"
var a = 1;
var b, rest;
var vals = {[a]: 1, bar: 2 };
result = {[a]:b, ...rest} = vals;
"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Computed destructuring error: {:?}", result.err());
    }

    #[test]
    fn test_rest_array_destructuring() {
        // Test rest with array destructuring
        let code = r#"((...[x]) => {})();"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Rest array pattern error: {:?}", result.err());

        // Test rest with array destructuring and default
        let code2 = r#"((...[x = 1]) => {})();"#;
        let mut parser = Parser::new(code2);
        let result = parser.parse();
        assert!(result.is_ok(), "Rest array pattern with default error: {:?}", result.err());
    }

    #[test]
    fn test_async_identifier_in_for_of() {
        // async.x is a member expression where async is an identifier
        let code = "var async = { x: 0 }; for (async.x of [1]) ;";
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "async.x for-of error: {:?}", result.err());

        // for await (async of [7]) is valid inside async function
        let code2 = "async function fn() { for await (async of [7]); }";
        let mut parser2 = Parser::new(code2);
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "for await async error: {:?}", result2.err());

        // Full test262 test case: let async declared at top level, then used in for-await
        // First check that 'let async;' by itself works
        let code3a = "let async;";
        let mut parser3a = Parser::new(code3a);
        let result3a = parser3a.parse();
        assert!(result3a.is_ok(), "let async; error: {:?}", result3a.err());

        // Now the full case
        let code3 = "let async; async function fn() { for await (async of [7]); }";
        let mut parser3 = Parser::new(code3);
        let result3 = parser3.parse();
        assert!(result3.is_ok(), "full test262 case error: {:?}", result3.err());
    }
}
