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
        }
    }

    /// Parse the source into an AST
    pub fn parse(&mut self) -> Result<ASTNode, JsError> {
        let mut statements = Vec::new();

        while !self.is_at_end()? {
            statements.push(self.parse_statement()?);
        }

        Ok(ASTNode::Program(statements))
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

        match token {
            Token::Keyword(Keyword::Let)
            | Token::Keyword(Keyword::Const)
            | Token::Keyword(Keyword::Var) => self.parse_variable_declaration(),
            Token::Keyword(Keyword::Function) => self.parse_function_declaration(),
            Token::Keyword(Keyword::Async) => self.parse_async_function_or_expression(),
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

    fn parse_pattern(&mut self) -> Result<Pattern, JsError> {
        self.update_position()?;
        let token = self.lexer.peek_token()?.clone();

        match token {
            Token::Identifier(name, _has_escapes) => {
                self.lexer.next_token()?;
                // Validate the identifier is not a reserved word
                // Per ES spec, even escaped reserved words are invalid as identifiers
                self.validate_identifier(&name)?;
                Ok(Pattern::Identifier(name))
            }
            Token::Punctuator(Punctuator::LBrace) => self.parse_object_pattern(),
            Token::Punctuator(Punctuator::LBracket) => self.parse_array_pattern(),
            _ => Err(syntax_error("Expected pattern", self.last_position.clone())),
        }
    }

    fn parse_object_pattern(&mut self) -> Result<Pattern, JsError> {
        self.expect_punctuator(Punctuator::LBrace)?;
        let mut properties = Vec::new();

        while !self.check_punctuator(Punctuator::RBrace)? {
            if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                properties.push(ObjectPatternProperty {
                    key: String::new(),
                    value: Pattern::RestElement(Box::new(pattern)),
                    shorthand: false,
                });
            } else {
                let key = self.expect_identifier()?;
                let (value, shorthand) = if self.check_punctuator(Punctuator::Colon)? {
                    self.lexer.next_token()?;
                    (self.parse_pattern()?, false)
                } else {
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
                    key,
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

        while !self.check_punctuator(Punctuator::RBracket)? {
            if self.check_punctuator(Punctuator::Comma)? {
                elements.push(None);
            } else if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                elements.push(Some(Pattern::RestElement(Box::new(pattern))));
            } else {
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
        let params = self.parse_parameters()?;
        let body = self.parse_function_body()?;

        Ok(Statement::FunctionDeclaration {
            name,
            params,
            body,
            is_async: false,
            is_generator,
            position: None,
        })
    }

    fn parse_async_function_or_expression(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Async)?;

        if self.check_keyword(Keyword::Function)? {
            self.lexer.next_token()?;

            // Check for async generator: async function *name()
            let is_generator = self.check_punctuator(Punctuator::Star)?;
            if is_generator {
                self.lexer.next_token()?;
            }

            let name = self.expect_identifier()?;
            let params = self.parse_parameters()?;
            let body = self.parse_function_body()?;

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
        let name = self.expect_identifier()?;

        let super_class = if self.check_keyword(Keyword::Extends)? {
            self.lexer.next_token()?;
            Some(Box::new(self.parse_left_hand_side_expression()?))
        } else {
            None
        };

        let body = self.parse_class_body()?;

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

        while !self.check_punctuator(Punctuator::RBrace)? {
            // Check for static
            let is_static = if self.check_keyword(Keyword::Static)? {
                self.lexer.next_token()?;
                true
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
                    || matches!(next, Token::Punctuator(Punctuator::LBracket));

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
            let is_private = self.check_private_identifier()?;

            // Check for get/set
            let mut kind = MethodKind::Method;
            let mut key_name = String::new();

            if is_private {
                // Private field/method
                key_name = self.expect_private_identifier()?;
            } else if !is_generator && self.check_identifier("get")? {
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_line_term = self.lexer.line_terminator_before_token;

                self.lexer.next_token()?;
                let next = self.lexer.peek_token()?;
                if matches!(next, Token::Identifier(_, _)) || matches!(next, Token::Keyword(_))
                    || matches!(next, Token::Punctuator(Punctuator::LBracket)) {
                    kind = MethodKind::Get;
                    key_name = self.expect_identifier_or_keyword()?;
                } else {
                    // It's a method named "get"
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = None;
                    key_name = self.expect_identifier_or_keyword()?;
                }
            } else if !is_generator && self.check_identifier("set")? {
                let saved_pos = self.lexer.position;
                let saved_line = self.lexer.line;
                let saved_column = self.lexer.column;
                let saved_line_term = self.lexer.line_terminator_before_token;

                self.lexer.next_token()?;
                let next = self.lexer.peek_token()?;
                if matches!(next, Token::Identifier(_, _)) || matches!(next, Token::Keyword(_))
                    || matches!(next, Token::Punctuator(Punctuator::LBracket)) {
                    kind = MethodKind::Set;
                    key_name = self.expect_identifier_or_keyword()?;
                } else {
                    // It's a method named "set"
                    self.lexer.position = saved_pos;
                    self.lexer.line = saved_line;
                    self.lexer.column = saved_column;
                    self.lexer.line_terminator_before_token = saved_line_term;
                    self.lexer.current_token = None;
                    key_name = self.expect_identifier_or_keyword()?;
                }
            } else {
                key_name = self.expect_identifier_or_keyword()?;
            }

            if key_name == "constructor" && !is_static {
                kind = MethodKind::Constructor;
            }

            if self.check_punctuator(Punctuator::LParen)? {
                // Method
                let params = self.parse_parameters()?;
                let body = self.parse_function_body()?;

                elements.push(ClassElement::MethodDefinition {
                    key: key_name,
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
                });
            } else {
                // Property
                let value = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    Some(self.parse_assignment_expression()?)
                } else {
                    None
                };
                elements.push(ClassElement::PropertyDefinition {
                    key: key_name,
                    value,
                    is_static,
                    is_private,
                });
                self.consume_semicolon()?;
            }
        }

        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(elements)
    }

    fn parse_parameters(&mut self) -> Result<Vec<Pattern>, JsError> {
        self.expect_punctuator(Punctuator::LParen)?;
        let mut params = Vec::new();

        while !self.check_punctuator(Punctuator::RParen)? {
            if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let pattern = self.parse_pattern()?;
                // Check for default value on rest element
                let final_pattern = if self.check_punctuator(Punctuator::Assign)? {
                    self.lexer.next_token()?;
                    let default_value = self.parse_assignment_expression()?;
                    Pattern::AssignmentPattern {
                        left: Box::new(Pattern::RestElement(Box::new(pattern))),
                        right: Box::new(default_value),
                    }
                } else {
                    Pattern::RestElement(Box::new(pattern))
                };
                params.push(final_pattern);
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

        while !self.check_punctuator(Punctuator::RBrace)? {
            statements.push(self.parse_statement()?);
        }

        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(statements)
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
        self.consume_semicolon()?;
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
        self.expect_punctuator(Punctuator::LParen)?;

        // Check for for-in/for-of first
        if self.check_punctuator(Punctuator::Semicolon)? {
            // Empty init - regular for loop
            return self.parse_regular_for(None);
        }

        // Parse left side
        if self.check_keyword(Keyword::Let)?
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
                    r#await: false,
                    position: None,
                });
            }

            // Regular for loop with variable declaration
            let init_expr = if self.check_punctuator(Punctuator::Assign)? {
                self.lexer.next_token()?;
                Some(self.parse_assignment_expression()?)
            } else {
                None
            };
            let init = Some(ForInit::VariableDeclaration {
                kind,
                declarations: vec![VariableDeclarator {
                    id,
                    init: init_expr,
                }],
            });
            return self.parse_regular_for(init);
        }

        // Expression as left side - could be for-in/for-of or regular for
        let left_expr = self.parse_left_hand_side_expression()?;

        // Check for in/of
        if self.check_keyword(Keyword::In)? {
            self.lexer.next_token()?; // consume 'in'
            let right = self.parse_expression()?;
            self.expect_punctuator(Punctuator::RParen)?;
            self.loop_depth += 1;
            let body = Box::new(self.parse_substatement()?);
            self.loop_depth -= 1;

            // Convert expression to pattern
            let left = self.expression_to_pattern(left_expr)?;
            return Ok(Statement::ForInStatement {
                left: ForInOfLeft::Pattern(left),
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

            // Convert expression to pattern
            let left = self.expression_to_pattern(left_expr)?;
            return Ok(Statement::ForOfStatement {
                left: ForInOfLeft::Pattern(left),
                right,
                body,
                r#await: false,
                position: None,
            });
        }

        // Regular for loop with expression init
        // Need to finish parsing the full init expression (may have comma operator)
        let init_expr = if self.check_punctuator(Punctuator::Comma)? {
            self.lexer.next_token()?;
            let rest = self.parse_expression()?;
            Expression::SequenceExpression {
                expressions: vec![left_expr, rest],
                position: None,
            }
        } else {
            left_expr
        };
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

        // Break must be inside a loop or switch
        if self.loop_depth == 0 {
            return Err(syntax_error(
                "Illegal break statement",
                self.last_position.clone(),
            ));
        }

        self.consume_semicolon()?;
        Ok(Statement::BreakStatement {
            label: None,
            position: None,
        })
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, JsError> {
        self.expect_keyword(Keyword::Continue)?;

        // Continue must be inside a loop
        if self.loop_depth == 0 {
            return Err(syntax_error(
                "Illegal continue statement",
                self.last_position.clone(),
            ));
        }

        self.consume_semicolon()?;
        Ok(Statement::ContinueStatement {
            label: None,
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

        while !self.check_punctuator(Punctuator::RBrace)? {
            statements.push(self.parse_statement()?);
        }

        self.expect_punctuator(Punctuator::RBrace)?;
        Ok(statements)
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
                self.lexer.next_token()?; // consume ':'
                let body = Box::new(self.parse_statement()?);
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
        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> Result<Expression, JsError> {
        let expr = self.parse_conditional_expression()?;

        // Check for single-parameter arrow function: identifier => expr
        // After parsing an identifier, if next token is =>, this is an arrow function
        if let Expression::Identifier { ref name, .. } = expr {
            if self.check_punctuator(Punctuator::Arrow)? {
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

    fn expression_to_assignment_target(
        &self,
        expr: Expression,
    ) -> Result<AssignmentTarget, JsError> {
        match expr {
            Expression::Identifier { name, .. } => Ok(AssignmentTarget::Identifier(name)),
            Expression::MemberExpression { .. } => Ok(AssignmentTarget::Member(Box::new(expr))),
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
            _ => None,
        };
        Ok(op)
    }

    fn parse_conditional_expression(&mut self) -> Result<Expression, JsError> {
        let test = self.parse_nullish_coalescing_expression()?;

        if self.check_punctuator(Punctuator::Question)? {
            self.lexer.next_token()?;
            let consequent = Box::new(self.parse_assignment_expression()?);
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
        self.parse_equality_expression()
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
        let mut left = self.parse_additive_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Lt) => BinaryOperator::Lt,
                Token::Punctuator(Punctuator::LtEq) => BinaryOperator::LtEq,
                Token::Punctuator(Punctuator::Gt) => BinaryOperator::Gt,
                Token::Punctuator(Punctuator::GtEq) => BinaryOperator::GtEq,
                Token::Keyword(Keyword::Instanceof) => BinaryOperator::Instanceof,
                Token::Keyword(Keyword::In) => BinaryOperator::In,
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
        let mut left = self.parse_unary_expression()?;

        loop {
            let op = match self.lexer.peek_token()? {
                Token::Punctuator(Punctuator::Star) => BinaryOperator::Mul,
                Token::Punctuator(Punctuator::Slash) => BinaryOperator::Div,
                Token::Punctuator(Punctuator::Percent) => BinaryOperator::Mod,
                _ => break,
            };
            self.lexer.next_token()?;
            let right = self.parse_unary_expression()?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                position: None,
            };
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
            return Ok(Expression::UnaryExpression {
                operator,
                argument,
                prefix: true,
                position: None,
            });
        }

        if self.check_keyword(Keyword::Await)? {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_unary_expression()?);
            return Ok(Expression::AwaitExpression {
                argument,
                position: None,
            });
        }

        self.parse_update_expression()
    }

    fn parse_update_expression(&mut self) -> Result<Expression, JsError> {
        // Prefix ++/--
        if self.check_punctuator(Punctuator::PlusPlus)? {
            self.lexer.next_token()?;
            let argument = Box::new(self.parse_left_hand_side_expression()?);
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
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument,
                prefix: true,
                position: None,
            });
        }

        let expr = self.parse_left_hand_side_expression()?;

        // Postfix ++/--
        if self.check_punctuator(Punctuator::PlusPlus)? {
            self.lexer.next_token()?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Box::new(expr),
                prefix: false,
                position: None,
            });
        }

        if self.check_punctuator(Punctuator::MinusMinus)? {
            self.lexer.next_token()?;
            return Ok(Expression::UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Box::new(expr),
                prefix: false,
                position: None,
            });
        }

        Ok(expr)
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
                self.lexer.next_token()?;
                let property = Box::new(self.parse_expression()?);
                self.expect_punctuator(Punctuator::RBracket)?;
                expr = Expression::MemberExpression {
                    object: Box::new(expr),
                    property,
                    computed: true,
                    optional: false,
                    position: None,
                };
            } else if self.check_punctuator(Punctuator::LParen)? {
                let arguments = self.parse_arguments()?;
                expr = Expression::CallExpression {
                    callee: Box::new(expr),
                    arguments,
                    optional: false,
                    position: None,
                };
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
                self.lexer.next_token()?;
                let property = Box::new(self.parse_expression()?);
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
                Ok(Expression::SuperExpression { position: None })
            }
            Token::Keyword(Keyword::Function) => self.parse_function_expression(),
            Token::Keyword(Keyword::Async) => self.parse_async_function_expression(),
            Token::Punctuator(Punctuator::LParen) => self.parse_parenthesized_or_arrow(),
            Token::Punctuator(Punctuator::LBracket) => self.parse_array_literal(),
            Token::Punctuator(Punctuator::LBrace) => self.parse_object_literal(),
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
        } else {
            None
        };
        let params = self.parse_parameters()?;
        let body = self.parse_function_body()?;

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

        if self.check_keyword(Keyword::Function)? {
            self.lexer.next_token()?;

            // Check for async generator: async function *name() or async function *()
            let is_generator = self.check_punctuator(Punctuator::Star)?;
            if is_generator {
                self.lexer.next_token()?;
            }

            let name = if let Token::Identifier(_, _) = self.lexer.peek_token()? {
                Some(self.expect_identifier()?)
            } else {
                None
            };
            let params = self.parse_parameters()?;
            let body = self.parse_function_body()?;

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
            let params = self.parse_parameters()?;
            self.expect_punctuator(Punctuator::Arrow)?;
            let body = self.parse_arrow_body()?;

            Ok(Expression::ArrowFunctionExpression {
                params,
                body,
                is_async: true,
                position: None,
            })
        } else if let Token::Identifier(name, _) = self.lexer.peek_token()?.clone() {
            // Async arrow function without parens: async x => body
            self.lexer.next_token()?;
            self.expect_punctuator(Punctuator::Arrow)?;
            let body = self.parse_arrow_body()?;

            Ok(Expression::ArrowFunctionExpression {
                params: vec![Pattern::Identifier(name)],
                body,
                is_async: true,
                position: None,
            })
        } else {
            Err(syntax_error("Expected function or arrow function after async", None))
        }
    }

    fn parse_parenthesized_or_arrow(&mut self) -> Result<Expression, JsError> {
        self.lexer.next_token()?; // consume (

        // Check for empty params ()
        if self.check_punctuator(Punctuator::RParen)? {
            self.lexer.next_token()?;
            if self.check_punctuator(Punctuator::Arrow)? {
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

        // Check for rest parameter as first param: (...args)
        if self.check_punctuator(Punctuator::Spread)? {
            self.lexer.next_token()?;
            let rest_name = self.expect_identifier()?;
            self.expect_punctuator(Punctuator::RParen)?;
            if self.check_punctuator(Punctuator::Arrow)? {
                self.lexer.next_token()?;
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::RestElement(Box::new(Pattern::Identifier(rest_name)))],
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
                self.lexer.next_token()?;
                let param = self.expression_to_pattern(first)?;
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params: vec![param],
                    body,
                    is_async: false,
                    position: None,
                });
            }
            return Ok(first);
        }

        // Multiple params or expressions
        if self.check_punctuator(Punctuator::Comma)? {
            let mut exprs = vec![first];
            let mut has_rest = false;
            let mut rest_param: Option<Pattern> = None;

            while self.check_punctuator(Punctuator::Comma)? {
                self.lexer.next_token()?;
                // Check for rest parameter: (a, b, ...c)
                if self.check_punctuator(Punctuator::Spread)? {
                    self.lexer.next_token()?;
                    let rest_name = self.expect_identifier()?;
                    has_rest = true;
                    rest_param = Some(Pattern::RestElement(Box::new(Pattern::Identifier(rest_name))));
                    break; // Rest must be last
                }
                exprs.push(self.parse_assignment_expression()?);
            }
            self.expect_punctuator(Punctuator::RParen)?;

            if self.check_punctuator(Punctuator::Arrow)? {
                self.lexer.next_token()?;
                let mut params: Vec<Pattern> = exprs
                    .into_iter()
                    .map(|e| self.expression_to_pattern(e))
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(rest) = rest_param {
                    params.push(rest);
                }
                let body = self.parse_arrow_body()?;
                return Ok(Expression::ArrowFunctionExpression {
                    params,
                    body,
                    is_async: false,
                    position: None,
                });
            }

            if has_rest {
                return Err(syntax_error("Rest parameter must be in arrow function", None));
            }

            // Sequence expression
            return Ok(Expression::SequenceExpression {
                expressions: exprs,
                position: None,
            });
        }

        self.expect_punctuator(Punctuator::RParen)?;
        Ok(first)
    }

    fn expression_to_pattern(&self, expr: Expression) -> Result<Pattern, JsError> {
        match expr {
            Expression::Identifier { name, .. } => Ok(Pattern::Identifier(name)),
            Expression::SpreadElement { argument, .. } => {
                let inner = self.expression_to_pattern(*argument)?;
                Ok(Pattern::RestElement(Box::new(inner)))
            }
            _ => Err(syntax_error("Invalid parameter", None)),
        }
    }

    fn parse_arrow_body(&mut self) -> Result<ArrowFunctionBody, JsError> {
        if self.check_punctuator(Punctuator::LBrace)? {
            let body = self.parse_function_body()?;
            Ok(ArrowFunctionBody::Block(body))
        } else {
            let expr = self.parse_assignment_expression()?;
            Ok(ArrowFunctionBody::Expression(Box::new(expr)))
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expression, JsError> {
        self.expect_punctuator(Punctuator::LBracket)?;
        let mut elements = Vec::new();

        while !self.check_punctuator(Punctuator::RBracket)? {
            if self.check_punctuator(Punctuator::Comma)? {
                elements.push(None);
            } else if self.check_punctuator(Punctuator::Spread)? {
                self.lexer.next_token()?;
                let expr = self.parse_assignment_expression()?;
                elements.push(Some(ArrayElement::Spread(expr)));
            } else {
                let expr = self.parse_assignment_expression()?;
                elements.push(Some(ArrayElement::Expression(expr)));
            }

            if !self.check_punctuator(Punctuator::Comma)? {
                break;
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
            } else if self.check_punctuator(Punctuator::LBracket)? {
                // Computed property: [expr]: value or [expr]() {}
                self.lexer.next_token()?;
                let key_expr = self.parse_assignment_expression()?;
                self.expect_punctuator(Punctuator::RBracket)?;

                if self.check_punctuator(Punctuator::LParen)? {
                    // Computed method: [expr]() {}
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
                    // Getter accessor: get prop() {}
                    // Allow keywords as accessor property names
                    let key = self.expect_property_name()?;
                    self.expect_punctuator(Punctuator::LParen)?;
                    self.expect_punctuator(Punctuator::RParen)?;
                    let body = self.parse_function_body()?;

                    let func = Expression::FunctionExpression {
                        name: Some(key.clone()),
                        params: vec![],
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
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
                    // Setter accessor: set prop(v) {}
                    // Allow keywords as accessor property names
                    let key = self.expect_property_name()?;
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
                let params = self.parse_parameters()?;
                let body = self.parse_function_body()?;

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
                    let params = self.parse_parameters()?;
                    let body = self.parse_function_body()?;

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
        if let Token::Identifier(name, _has_escapes) = token {
            // Validate the identifier is not a reserved word
            // Per ES spec, even escaped reserved words are invalid as identifiers
            self.validate_identifier(&name)?;
            Ok(name)
        } else {
            Err(unexpected_token(
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

    /// Validate that an identifier is not a reserved word
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

    /// Parse a statement in a context where lexical declarations are not allowed
    /// (e.g., after if, while, for without braces)
    fn parse_substatement(&mut self) -> Result<Statement, JsError> {
        let token = self.lexer.peek_token()?.clone();

        // Lexical declarations (let, const) are not allowed in statement positions
        if matches!(token, Token::Keyword(Keyword::Let) | Token::Keyword(Keyword::Const)) {
            return Err(syntax_error(
                "Lexical declaration cannot appear in a single-statement context",
                self.last_position.clone(),
            ));
        }

        // Class declarations are not allowed in statement positions
        if matches!(token, Token::Keyword(Keyword::Class)) {
            return Err(syntax_error(
                "Class declaration cannot appear in a single-statement context",
                self.last_position.clone(),
            ));
        }

        // Function declarations are only allowed in non-strict mode
        if matches!(token, Token::Keyword(Keyword::Function)) && self.strict_mode {
            return Err(syntax_error(
                "Function declaration cannot appear in a single-statement context in strict mode",
                self.last_position.clone(),
            ));
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
}
