//! JavaScript Lexer - tokenizes source code into tokens

use core_types::{ErrorKind, JsError, SourcePosition};

/// JavaScript keyword types
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    /// let keyword
    Let,
    /// const keyword
    Const,
    /// var keyword
    Var,
    /// function keyword
    Function,
    /// return keyword
    Return,
    /// if keyword
    If,
    /// else keyword
    Else,
    /// while keyword
    While,
    /// for keyword
    For,
    /// break keyword
    Break,
    /// continue keyword
    Continue,
    /// class keyword
    Class,
    /// extends keyword
    Extends,
    /// new keyword
    New,
    /// this keyword
    This,
    /// super keyword
    Super,
    /// async keyword
    Async,
    /// await keyword
    Await,
    /// true keyword
    True,
    /// false keyword
    False,
    /// null keyword
    Null,
    // Note: 'undefined' is NOT a keyword - it's a global property
    /// typeof keyword
    Typeof,
    /// void keyword
    Void,
    /// instanceof keyword
    Instanceof,
    /// in keyword
    In,
    /// try keyword
    Try,
    /// catch keyword
    Catch,
    /// finally keyword
    Finally,
    /// throw keyword
    Throw,
    /// yield keyword
    Yield,
    /// import keyword
    Import,
    /// export keyword
    Export,
    /// default keyword
    Default,
    // Note: 'constructor' is NOT a keyword - it's a special method name in classes
}

/// JavaScript punctuators (operators and delimiters)
#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    /// Opening parenthesis
    LParen,
    /// Closing parenthesis
    RParen,
    /// Opening brace
    LBrace,
    /// Closing brace
    RBrace,
    /// Opening bracket
    LBracket,
    /// Closing bracket
    RBracket,
    /// Semicolon
    Semicolon,
    /// Comma
    Comma,
    /// Dot
    Dot,
    /// Spread operator
    Spread,
    /// Optional chaining
    OptionalChain,
    /// Colon
    Colon,
    /// Question mark
    Question,
    /// Assignment
    Assign,
    /// Arrow function
    Arrow,
    /// Plus
    Plus,
    /// Minus
    Minus,
    /// Multiply
    Star,
    /// Divide
    Slash,
    /// Modulo
    Percent,
    /// Exponentiation
    StarStar,
    /// Equality
    EqEq,
    /// Strict equality
    EqEqEq,
    /// Inequality
    NotEq,
    /// Strict inequality
    NotEqEq,
    /// Less than
    Lt,
    /// Less than or equal
    LtEq,
    /// Greater than
    Gt,
    /// Greater than or equal
    GtEq,
    /// Logical AND
    AndAnd,
    /// Logical OR
    OrOr,
    /// Nullish coalescing
    NullishCoalesce,
    /// Logical NOT
    Not,
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Bitwise NOT
    Tilde,
    /// Left shift
    LtLt,
    /// Right shift
    GtGt,
    /// Unsigned right shift
    GtGtGt,
    /// Plus equals
    PlusEq,
    /// Minus equals
    MinusEq,
    /// Multiply equals
    StarEq,
    /// Divide equals
    SlashEq,
    /// Modulo equals
    PercentEq,
    /// Increment
    PlusPlus,
    /// Decrement
    MinusMinus,
    /// Backtick (template literal)
    Backtick,
}

/// Token produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Identifier (variable name, etc.)
    Identifier(String),
    /// Number literal
    Number(f64),
    /// BigInt literal (integer with 'n' suffix)
    BigIntLiteral(String),
    /// String literal
    String(String),
    /// Template literal part
    TemplateLiteral(String),
    /// Keyword
    Keyword(Keyword),
    /// Punctuator/operator
    Punctuator(Punctuator),
    /// End of file
    EOF,
}

/// Lexer for JavaScript source code
pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    position: usize,
    line: u32,
    column: u32,
    current_token: Option<Token>,
    /// Tracks if a line terminator was encountered before the current token
    /// Used for Automatic Semicolon Insertion (ASI)
    pub line_terminator_before_token: bool,
    /// Previous line number (used to detect line changes)
    previous_line: u32,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            current_token: None,
            line_terminator_before_token: false,
            previous_line: 1,
        }
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> Result<Token, JsError> {
        if let Some(token) = self.current_token.take() {
            return Ok(token);
        }
        self.scan_token()
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Result<&Token, JsError> {
        if self.current_token.is_none() {
            self.current_token = Some(self.scan_token()?);
        }
        Ok(self.current_token.as_ref().unwrap())
    }

    fn scan_token(&mut self) -> Result<Token, JsError> {
        // Record the line before skipping whitespace
        let line_before = self.line;

        self.skip_whitespace_and_comments();

        // Check if we crossed a line boundary (for ASI)
        self.line_terminator_before_token = self.line > line_before;
        self.previous_line = self.line;

        if self.is_at_end() {
            return Ok(Token::EOF);
        }

        let start_pos = self.current_position();
        let ch = self.advance();

        match ch {
            '(' => Ok(Token::Punctuator(Punctuator::LParen)),
            ')' => Ok(Token::Punctuator(Punctuator::RParen)),
            '{' => Ok(Token::Punctuator(Punctuator::LBrace)),
            '}' => Ok(Token::Punctuator(Punctuator::RBrace)),
            '[' => Ok(Token::Punctuator(Punctuator::LBracket)),
            ']' => Ok(Token::Punctuator(Punctuator::RBracket)),
            ';' => Ok(Token::Punctuator(Punctuator::Semicolon)),
            ',' => Ok(Token::Punctuator(Punctuator::Comma)),
            ':' => Ok(Token::Punctuator(Punctuator::Colon)),
            '~' => Ok(Token::Punctuator(Punctuator::Tilde)),

            '.' => {
                if self.match_char('.') && self.match_char('.') {
                    Ok(Token::Punctuator(Punctuator::Spread))
                } else {
                    Ok(Token::Punctuator(Punctuator::Dot))
                }
            }

            '?' => {
                if self.match_char('?') {
                    Ok(Token::Punctuator(Punctuator::NullishCoalesce))
                } else if self.match_char('.') {
                    Ok(Token::Punctuator(Punctuator::OptionalChain))
                } else {
                    Ok(Token::Punctuator(Punctuator::Question))
                }
            }

            '=' => {
                if self.match_char('>') {
                    Ok(Token::Punctuator(Punctuator::Arrow))
                } else if self.match_char('=') {
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::EqEqEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::EqEq))
                    }
                } else {
                    Ok(Token::Punctuator(Punctuator::Assign))
                }
            }

            '+' => {
                if self.match_char('+') {
                    Ok(Token::Punctuator(Punctuator::PlusPlus))
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::PlusEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Plus))
                }
            }

            '-' => {
                if self.match_char('-') {
                    Ok(Token::Punctuator(Punctuator::MinusMinus))
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::MinusEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Minus))
                }
            }

            '*' => {
                if self.match_char('*') {
                    Ok(Token::Punctuator(Punctuator::StarStar))
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::StarEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Star))
                }
            }

            '/' => {
                if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::SlashEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Slash))
                }
            }

            '%' => {
                if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::PercentEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Percent))
                }
            }

            '!' => {
                if self.match_char('=') {
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::NotEqEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::NotEq))
                    }
                } else {
                    Ok(Token::Punctuator(Punctuator::Not))
                }
            }

            '<' => {
                if self.match_char('<') {
                    Ok(Token::Punctuator(Punctuator::LtLt))
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::LtEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Lt))
                }
            }

            '>' => {
                if self.match_char('>') {
                    if self.match_char('>') {
                        Ok(Token::Punctuator(Punctuator::GtGtGt))
                    } else {
                        Ok(Token::Punctuator(Punctuator::GtGt))
                    }
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::GtEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Gt))
                }
            }

            '&' => {
                if self.match_char('&') {
                    Ok(Token::Punctuator(Punctuator::AndAnd))
                } else {
                    Ok(Token::Punctuator(Punctuator::And))
                }
            }

            '|' => {
                if self.match_char('|') {
                    Ok(Token::Punctuator(Punctuator::OrOr))
                } else {
                    Ok(Token::Punctuator(Punctuator::Or))
                }
            }

            '^' => Ok(Token::Punctuator(Punctuator::Xor)),

            '`' => self.scan_template_literal(),

            '"' | '\'' => self.scan_string(ch),

            _ if ch.is_ascii_digit() => self.scan_number(ch),

            _ if is_id_start(ch) => self.scan_identifier(ch),

            _ => Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: format!("Unexpected character: '{}'", ch),
                stack: vec![],
                source_position: Some(start_pos),
            }),
        }
    }

    fn scan_string(&mut self, quote: char) -> Result<Token, JsError> {
        let start_pos = self.current_position();
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Unterminated string".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos),
                    });
                }
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '\'' => value.push('\''),
                    '"' => value.push('"'),
                    '0' => value.push('\0'),
                    _ => value.push(escaped),
                }
            } else if self.peek() == '\n' {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Unterminated string literal".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos),
                });
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Unterminated string".to_string(),
                stack: vec![],
                source_position: Some(start_pos),
            });
        }

        self.advance(); // Closing quote
        Ok(Token::String(value))
    }

    fn scan_template_literal(&mut self) -> Result<Token, JsError> {
        let start_pos = self.current_position();
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '`' {
            if self.peek() == '$' && self.peek_next() == Some('{') {
                // Template expression - for now, just include as part of string
                value.push(self.advance());
            } else if self.peek() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '`' => value.push('`'),
                        '$' => value.push('$'),
                        _ => value.push(escaped),
                    }
                }
            } else {
                let ch = self.advance();
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Unterminated template literal".to_string(),
                stack: vec![],
                source_position: Some(start_pos),
            });
        }

        self.advance(); // Closing backtick
        Ok(Token::TemplateLiteral(value))
    }

    fn scan_number(&mut self, first: char) -> Result<Token, JsError> {
        let start_pos = self.current_position();
        let mut num_str = first.to_string();
        let mut is_float = false;
        let mut radix: Option<u32> = None; // None = decimal, Some(16) = hex, etc.

        // Check for hex (0x), binary (0b), or octal (0o) literals
        if first == '0' && !self.is_at_end() {
            let next = self.peek();
            match next {
                'x' | 'X' => {
                    // Hexadecimal
                    self.advance(); // skip the 'x' (don't add to num_str for parsing)
                    num_str.clear(); // We'll build the digits-only string
                    if self.is_at_end() || !self.peek().is_ascii_hexdigit() {
                        return Err(JsError {
                            kind: ErrorKind::SyntaxError,
                            message: "Invalid hexadecimal literal".to_string(),
                            stack: vec![],
                            source_position: Some(start_pos),
                        });
                    }
                    while !self.is_at_end() && self.peek().is_ascii_hexdigit() {
                        num_str.push(self.advance());
                    }
                    radix = Some(16);
                }
                'b' | 'B' => {
                    // Binary
                    self.advance(); // skip the 'b'
                    num_str.clear();
                    if self.is_at_end() || (self.peek() != '0' && self.peek() != '1') {
                        return Err(JsError {
                            kind: ErrorKind::SyntaxError,
                            message: "Invalid binary literal".to_string(),
                            stack: vec![],
                            source_position: Some(start_pos),
                        });
                    }
                    while !self.is_at_end() && (self.peek() == '0' || self.peek() == '1') {
                        num_str.push(self.advance());
                    }
                    radix = Some(2);
                }
                'o' | 'O' => {
                    // Octal
                    self.advance(); // skip the 'o'
                    num_str.clear();
                    if self.is_at_end() || !('0'..='7').contains(&self.peek()) {
                        return Err(JsError {
                            kind: ErrorKind::SyntaxError,
                            message: "Invalid octal literal".to_string(),
                            stack: vec![],
                            source_position: Some(start_pos),
                        });
                    }
                    while !self.is_at_end() && ('0'..='7').contains(&self.peek()) {
                        num_str.push(self.advance());
                    }
                    radix = Some(8);
                }
                _ => {
                    // Regular decimal number starting with 0
                    self.scan_decimal_digits(&mut num_str, &mut is_float);
                }
            }
        } else {
            // Regular decimal number
            self.scan_decimal_digits(&mut num_str, &mut is_float);
        }

        // Check for BigInt suffix 'n'
        if !self.is_at_end() && self.peek() == 'n' {
            if is_float {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "BigInt literals cannot have decimal points".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos),
                });
            }
            self.advance(); // consume 'n'
            // For BigInt, reconstruct the full literal
            let bigint_str = match radix {
                Some(16) => format!("0x{}", num_str),
                Some(8) => format!("0o{}", num_str),
                Some(2) => format!("0b{}", num_str),
                _ => num_str,
            };
            return Ok(Token::BigIntLiteral(bigint_str));
        }

        // Parse as regular number
        let value = match radix {
            Some(base) => {
                // Parse hex, binary, or octal
                u64::from_str_radix(&num_str, base).map(|n| n as f64).map_err(|_| JsError {
                    kind: ErrorKind::SyntaxError,
                    message: format!("Invalid number literal"),
                    stack: vec![],
                    source_position: Some(start_pos),
                })?
            }
            None => {
                // Parse decimal
                num_str.parse::<f64>().map_err(|_| JsError {
                    kind: ErrorKind::SyntaxError,
                    message: format!("Invalid number: {}", num_str),
                    stack: vec![],
                    source_position: Some(start_pos),
                })?
            }
        };

        Ok(Token::Number(value))
    }

    fn scan_decimal_digits(&mut self, num_str: &mut String, is_float: &mut bool) {
        // Scan integer part
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        // Handle decimal point
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to ensure it's not a method call (e.g., 123.toString())
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    *is_float = true;
                    num_str.push(self.advance()); // consume '.'
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        num_str.push(self.advance());
                    }
                }
            }
        }

        // Handle exponent
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            *is_float = true;
            num_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                num_str.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
        }
    }

    fn scan_identifier(&mut self, first: char) -> Result<Token, JsError> {
        let mut ident = first.to_string();

        while !self.is_at_end() && is_id_continue(self.peek()) {
            ident.push(self.advance());
        }

        // Check for keywords
        let token = match ident.as_str() {
            "let" => Token::Keyword(Keyword::Let),
            "const" => Token::Keyword(Keyword::Const),
            "var" => Token::Keyword(Keyword::Var),
            "function" => Token::Keyword(Keyword::Function),
            "return" => Token::Keyword(Keyword::Return),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),
            "while" => Token::Keyword(Keyword::While),
            "for" => Token::Keyword(Keyword::For),
            "break" => Token::Keyword(Keyword::Break),
            "continue" => Token::Keyword(Keyword::Continue),
            "class" => Token::Keyword(Keyword::Class),
            "extends" => Token::Keyword(Keyword::Extends),
            "new" => Token::Keyword(Keyword::New),
            "this" => Token::Keyword(Keyword::This),
            "super" => Token::Keyword(Keyword::Super),
            "async" => Token::Keyword(Keyword::Async),
            "await" => Token::Keyword(Keyword::Await),
            "true" => Token::Keyword(Keyword::True),
            "false" => Token::Keyword(Keyword::False),
            "null" => Token::Keyword(Keyword::Null),
            // Note: "undefined" is NOT a keyword - it's a global property that can be shadowed
            // It's handled as an identifier and resolved at runtime
            "typeof" => Token::Keyword(Keyword::Typeof),
            "void" => Token::Keyword(Keyword::Void),
            "instanceof" => Token::Keyword(Keyword::Instanceof),
            "in" => Token::Keyword(Keyword::In),
            "try" => Token::Keyword(Keyword::Try),
            "catch" => Token::Keyword(Keyword::Catch),
            "finally" => Token::Keyword(Keyword::Finally),
            "throw" => Token::Keyword(Keyword::Throw),
            "yield" => Token::Keyword(Keyword::Yield),
            "import" => Token::Keyword(Keyword::Import),
            "export" => Token::Keyword(Keyword::Export),
            "default" => Token::Keyword(Keyword::Default),
            // Note: 'constructor' is NOT a keyword - it's just a special method name
            _ => Token::Identifier(ident),
        };

        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        self.advance(); // /
                        self.advance(); // *
                        while !self.is_at_end() {
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); // *
                                self.advance(); // /
                                break;
                            }
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 1;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.chars.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.position]
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.chars.len() {
            Some(self.chars[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.position];
        self.position += 1;
        self.column += 1;
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.position] != expected {
            false
        } else {
            self.position += 1;
            self.column += 1;
            true
        }
    }

    fn current_position(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            column: self.column,
            offset: self.position,
        }
    }
}

fn is_id_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_id_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_empty_source() {
        let mut lexer = Lexer::new("");
        assert!(matches!(lexer.next_token().unwrap(), Token::EOF));
    }

    #[test]
    fn test_lexer_identifier() {
        let mut lexer = Lexer::new("foo");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::Identifier(s) if s == "foo"));
    }

    #[test]
    fn test_lexer_number() {
        let mut lexer = Lexer::new("123.45");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::Number(n) if n == 123.45));
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(r#""hello""#);
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::String(s) if s == "hello"));
    }

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("let const var");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Keyword(Keyword::Let)
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Keyword(Keyword::Const)
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Keyword(Keyword::Var)
        ));
    }

    #[test]
    fn test_lexer_punctuators() {
        let mut lexer = Lexer::new("= === ==");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::Assign)
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::EqEqEq)
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::EqEq)
        ));
    }

    #[test]
    fn test_lexer_arrow_function() {
        let mut lexer = Lexer::new("=>");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::Arrow)
        ));
    }

    #[test]
    fn test_lexer_spread() {
        let mut lexer = Lexer::new("...");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::Spread)
        ));
    }

    #[test]
    fn test_lexer_nullish_coalescing() {
        let mut lexer = Lexer::new("??");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::NullishCoalesce)
        ));
    }

    #[test]
    fn test_lexer_optional_chaining() {
        let mut lexer = Lexer::new("?.");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Punctuator(Punctuator::OptionalChain)
        ));
    }

    #[test]
    fn test_lexer_comments() {
        let mut lexer = Lexer::new("// comment\nfoo /* block */ bar");
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Identifier(s) if s == "foo"
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Identifier(s) if s == "bar"
        ));
    }

    #[test]
    fn test_lexer_bigint_decimal() {
        let mut lexer = Lexer::new("123n");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::BigIntLiteral(s) if s == "123"));
    }

    #[test]
    fn test_lexer_bigint_hex() {
        let mut lexer = Lexer::new("0x1fn");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::BigIntLiteral(s) if s == "0x1f"));
    }

    #[test]
    fn test_lexer_bigint_binary() {
        let mut lexer = Lexer::new("0b101n");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::BigIntLiteral(s) if s == "0b101"));
    }

    #[test]
    fn test_lexer_bigint_octal() {
        let mut lexer = Lexer::new("0o77n");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::BigIntLiteral(s) if s == "0o77"));
    }

    #[test]
    fn test_lexer_bigint_error_float() {
        let mut lexer = Lexer::new("123.45n");
        let result = lexer.next_token();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("BigInt"));
    }

    #[test]
    fn test_lexer_hex_number() {
        let mut lexer = Lexer::new("0x1f");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::Number(_)));
    }

    #[test]
    fn test_lexer_binary_number() {
        let mut lexer = Lexer::new("0b101");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::Number(_)));
    }

    #[test]
    fn test_lexer_octal_number() {
        let mut lexer = Lexer::new("0o77");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Token::Number(_)));
    }
}
