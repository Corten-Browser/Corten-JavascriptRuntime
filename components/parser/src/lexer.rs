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
    /// delete keyword
    Delete,
    /// with keyword
    With,
    /// switch keyword
    Switch,
    /// case keyword
    Case,
    /// do keyword
    Do,
    /// debugger keyword
    Debugger,
    /// static keyword
    Static,
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
    /// Exponentiation equals
    StarStarEq,
    /// Bitwise AND equals
    AndEq,
    /// Bitwise OR equals
    OrEq,
    /// Bitwise XOR equals
    XorEq,
    /// Left shift equals
    LtLtEq,
    /// Right shift equals
    GtGtEq,
    /// Unsigned right shift equals
    GtGtGtEq,
    /// Logical AND equals
    AndAndEq,
    /// Logical OR equals
    OrOrEq,
    /// Nullish coalescing equals
    NullishCoalesceEq,
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
    /// Identifier (variable name, etc.). Second field is true if the identifier
    /// contained Unicode escape sequences (important for escaped keywords)
    Identifier(String, bool),
    /// Private identifier (#name) for class private fields/methods
    PrivateIdentifier(String),
    /// Number literal
    Number(f64),
    /// BigInt literal (integer with 'n' suffix)
    BigIntLiteral(String),
    /// String literal
    String(String),
    /// Template literal with no substitutions (no `${}`)
    TemplateLiteral(String),
    /// Template head: from ` to first ${
    TemplateHead(String),
    /// Template middle: from } to next ${
    TemplateMiddle(String),
    /// Template tail: from } to closing `
    TemplateTail(String),
    /// Regular expression literal (pattern, flags)
    RegExp(String, String),
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
    pub position: usize,
    pub line: u32,
    pub column: u32,
    pub current_token: Option<Token>,
    /// Tracks if a line terminator was encountered before the current token
    /// Used for Automatic Semicolon Insertion (ASI)
    pub line_terminator_before_token: bool,
    /// Previous line number (used to detect line changes)
    pub previous_line: u32,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let mut lexer = Self {
            source,
            chars,
            position: 0,
            line: 1,
            column: 1,
            current_token: None,
            line_terminator_before_token: false,
            previous_line: 1,
        };

        // Handle hashbang comment at the start of the file
        lexer.skip_hashbang();
        lexer
    }

    /// Skip hashbang comment (#!) at the beginning of the source
    fn skip_hashbang(&mut self) {
        // Hashbang is only valid at position 0
        if self.position == 0 && !self.is_at_end() {
            if self.peek() == '#' && self.peek_next() == Some('!') {
                // Skip until end of line (any LineTerminator)
                while !self.is_at_end() && !self.is_line_terminator(self.peek()) {
                    self.advance();
                }
                // Advance past the line terminator if present
                if !self.is_at_end() {
                    let c = self.peek();
                    if c == '\r' {
                        self.advance();
                        if !self.is_at_end() && self.peek() == '\n' {
                            self.advance();
                        }
                    } else if self.is_line_terminator(c) {
                        self.advance();
                    }
                    self.line = 2;
                    self.column = 1;
                }
            }
        }
    }

    /// Check if character is a line terminator (per ECMAScript spec)
    fn is_line_terminator(&self, c: char) -> bool {
        matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
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

    /// Check if the token after an identifier is the arrow (=>)
    /// Used for detecting single-parameter arrow functions like: x => expr
    pub fn check_arrow_after_identifier(&mut self) -> Result<bool, JsError> {
        // Save lexer state
        let saved_position = self.position;
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_previous_line = self.previous_line;
        let saved_line_term = self.line_terminator_before_token;

        // Clear current token cache and scan fresh
        self.current_token = None;

        // Scan the identifier token (skip it)
        let _ = self.scan_token()?;

        // Scan the next token and check if it's =>
        let next = self.scan_token()?;
        let is_arrow = matches!(next, Token::Punctuator(Punctuator::Arrow));

        // Restore lexer state
        self.position = saved_position;
        self.line = saved_line;
        self.column = saved_column;
        self.previous_line = saved_previous_line;
        self.line_terminator_before_token = saved_line_term;
        self.current_token = None; // Clear so next peek re-scans

        Ok(is_arrow)
    }

    fn scan_token(&mut self) -> Result<Token, JsError> {
        // Record the line before skipping whitespace
        let line_before = self.line;

        self.skip_whitespace_and_comments()?;

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
                } else if !self.is_at_end() && self.peek().is_ascii_digit() {
                    // Number with leading decimal: .5, .123, etc.
                    self.scan_leading_decimal_number()
                } else {
                    Ok(Token::Punctuator(Punctuator::Dot))
                }
            }

            '?' => {
                if self.match_char('?') {
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::NullishCoalesceEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::NullishCoalesce))
                    }
                } else if !self.is_at_end() && self.chars[self.position] == '.' {
                    // Check if followed by decimal digit - if so, this is NOT optional chaining
                    // Per spec: OptionalChainingPunctuator :: ?.[lookahead ∉ DecimalDigit]
                    let after_dot = self.position + 1;
                    if after_dot < self.chars.len() && self.chars[after_dot].is_ascii_digit() {
                        // This is `? .N` (ternary with decimal), not `?.` (optional chaining)
                        Ok(Token::Punctuator(Punctuator::Question))
                    } else {
                        // Consume the dot and return optional chaining
                        self.position += 1;
                        self.column += 1;
                        Ok(Token::Punctuator(Punctuator::OptionalChain))
                    }
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
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::StarStarEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::StarStar))
                    }
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
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::LtLtEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::LtLt))
                    }
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::LtEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Lt))
                }
            }

            '>' => {
                if self.match_char('>') {
                    if self.match_char('>') {
                        if self.match_char('=') {
                            Ok(Token::Punctuator(Punctuator::GtGtGtEq))
                        } else {
                            Ok(Token::Punctuator(Punctuator::GtGtGt))
                        }
                    } else if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::GtGtEq))
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
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::AndAndEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::AndAnd))
                    }
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::AndEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::And))
                }
            }

            '|' => {
                if self.match_char('|') {
                    if self.match_char('=') {
                        Ok(Token::Punctuator(Punctuator::OrOrEq))
                    } else {
                        Ok(Token::Punctuator(Punctuator::OrOr))
                    }
                } else if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::OrEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Or))
                }
            }

            '^' => {
                if self.match_char('=') {
                    Ok(Token::Punctuator(Punctuator::XorEq))
                } else {
                    Ok(Token::Punctuator(Punctuator::Xor))
                }
            }

            '`' => self.scan_template_literal(),

            '"' | '\'' => self.scan_string(ch),

            _ if ch.is_ascii_digit() => self.scan_number(ch),

            _ if is_id_start(ch) => self.scan_identifier(ch),

            // Unicode escape sequence starting an identifier: \u0041 or \u{41}
            '\\' if !self.is_at_end() && self.peek() == 'u' => {
                self.scan_identifier_with_unicode_start()
            }

            // Private identifier (#name) for class private fields/methods
            '#' => self.scan_private_identifier(),

            _ => Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: format!("Unexpected character: '{}'", ch),
                stack: vec![],
                source_position: Some(start_pos.clone()),
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
                        source_position: Some(start_pos.clone()),
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
                    // Line continuation: backslash followed by actual line terminator
                    '\n' => {
                        // Line continuation - skip the newline, don't add anything
                        self.line += 1;
                        self.column = 1;
                    }
                    '\r' => {
                        // Line continuation with carriage return
                        if self.peek() == '\n' {
                            self.advance(); // consume the \n after \r
                        }
                        self.line += 1;
                        self.column = 1;
                    }
                    // Line separator U+2028 and Paragraph separator U+2029 are also line terminators
                    '\u{2028}' | '\u{2029}' => {
                        self.line += 1;
                        self.column = 1;
                    }
                    _ => value.push(escaped),
                }
            } else if self.peek() == '\n' {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Unterminated string literal".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
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
                source_position: Some(start_pos.clone()),
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
                // Template expression - return TemplateHead and stop
                self.advance(); // $
                self.advance(); // {
                return Ok(Token::TemplateHead(value));
            } else if self.peek() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.advance();
                    self.scan_template_escape(escaped, &mut value, &start_pos)?;
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
                source_position: Some(start_pos.clone()),
            });
        }

        self.advance(); // Closing backtick
        Ok(Token::TemplateLiteral(value))
    }

    /// Handle escape sequences in template literals with proper validation
    fn scan_template_escape(&mut self, escaped: char, value: &mut String, start_pos: &SourcePosition) -> Result<(), JsError> {
        match escaped {
            'n' => value.push('\n'),
            't' => value.push('\t'),
            'r' => value.push('\r'),
            'b' => value.push('\u{0008}'),
            'f' => value.push('\u{000C}'),
            'v' => value.push('\u{000B}'),
            '0' if self.is_at_end() || !self.peek().is_ascii_digit() => {
                value.push('\0');
            }
            '0'..='7' => {
                // Legacy octal escapes are NOT allowed in template literals
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Octal escape sequences are not allowed in template literals".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }
            '8' | '9' => {
                // \8 and \9 are NOT allowed in template literals
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: format!("\\{} is not allowed in template literals", escaped),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }
            'x' => {
                // Hex escape: \xHH (exactly 2 hex digits required)
                if self.is_at_end() || !self.peek().is_ascii_hexdigit() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Invalid hexadecimal escape sequence".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                let h1 = self.advance();
                if self.is_at_end() || !self.peek().is_ascii_hexdigit() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Invalid hexadecimal escape sequence".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                let h2 = self.advance();
                let code = u8::from_str_radix(&format!("{}{}", h1, h2), 16).unwrap();
                value.push(code as char);
            }
            'u' => {
                // Unicode escape: \uHHHH or \u{H...}
                if self.is_at_end() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Invalid Unicode escape sequence".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                if self.peek() == '{' {
                    // \u{H...} form
                    self.advance(); // consume {
                    let mut hex = String::new();
                    while !self.is_at_end() && self.peek() != '}' {
                        if !self.peek().is_ascii_hexdigit() {
                            return Err(JsError {
                                kind: ErrorKind::SyntaxError,
                                message: "Invalid Unicode escape sequence".to_string(),
                                stack: vec![],
                                source_position: Some(start_pos.clone()),
                            });
                        }
                        hex.push(self.advance());
                    }
                    if self.is_at_end() || hex.is_empty() {
                        return Err(JsError {
                            kind: ErrorKind::SyntaxError,
                            message: "Invalid Unicode escape sequence".to_string(),
                            stack: vec![],
                            source_position: Some(start_pos.clone()),
                        });
                    }
                    self.advance(); // consume }
                    let code = u32::from_str_radix(&hex, 16).unwrap_or(0x110000);
                    if code > 0x10FFFF {
                        return Err(JsError {
                            kind: ErrorKind::SyntaxError,
                            message: "Invalid Unicode escape sequence".to_string(),
                            stack: vec![],
                            source_position: Some(start_pos.clone()),
                        });
                    }
                    if let Some(ch) = char::from_u32(code) {
                        value.push(ch);
                    }
                } else {
                    // \uHHHH form (exactly 4 hex digits)
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if self.is_at_end() || !self.peek().is_ascii_hexdigit() {
                            return Err(JsError {
                                kind: ErrorKind::SyntaxError,
                                message: "Invalid Unicode escape sequence".to_string(),
                                stack: vec![],
                                source_position: Some(start_pos.clone()),
                            });
                        }
                        hex.push(self.advance());
                    }
                    let code = u16::from_str_radix(&hex, 16).unwrap();
                    if let Some(ch) = char::from_u32(code as u32) {
                        value.push(ch);
                    }
                }
            }
            '\\' => value.push('\\'),
            '`' => value.push('`'),
            '$' => value.push('$'),
            '\n' => {
                // Line continuation
                self.line += 1;
                self.column = 1;
            }
            '\r' => {
                // Line continuation (CRLF or CR)
                self.line += 1;
                self.column = 1;
                if !self.is_at_end() && self.peek() == '\n' {
                    self.advance();
                }
            }
            _ => value.push(escaped),
        }
        Ok(())
    }

    /// Scan the continuation of a template literal after an expression.
    /// Called by the parser after it has consumed the closing `}` token.
    /// The lexer position should now be right after the `}`.
    pub fn scan_template_middle(&mut self) -> Result<Token, JsError> {
        // Clear any buffered token since we're switching to template mode
        self.current_token = None;

        let start_pos = self.current_position();
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '`' {
            if self.peek() == '$' && self.peek_next() == Some('{') {
                // Another template expression - return TemplateMiddle and stop
                self.advance(); // $
                self.advance(); // {
                return Ok(Token::TemplateMiddle(value));
            } else if self.peek() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.advance();
                    self.scan_template_escape(escaped, &mut value, &start_pos)?;
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
                source_position: Some(start_pos.clone()),
            });
        }

        self.advance(); // Closing backtick
        Ok(Token::TemplateTail(value))
    }

    /// Scan a regular expression literal.
    /// This should be called by the parser when it sees a '/' in a context where
    /// a regex literal is expected (not division).
    pub fn scan_regexp(&mut self) -> Result<Token, JsError> {
        let start_pos = self.current_position();

        // We expect to be positioned at the opening '/'
        if self.is_at_end() || self.peek() != '/' {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Expected '/' at start of regexp".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }
        self.advance(); // Skip opening '/'

        let mut pattern = String::new();
        let mut in_class = false; // Inside character class [...]

        // Parse the pattern
        loop {
            if self.is_at_end() {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Unterminated regular expression".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }

            let ch = self.peek();

            // Line terminators are not allowed in regex
            if self.is_line_terminator(ch) {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Unterminated regular expression".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }

            if ch == '\\' {
                // Escape sequence - include both backslash and next char
                pattern.push(ch);
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.peek();
                    pattern.push(escaped);
                    self.advance();
                }
            } else if ch == '[' {
                in_class = true;
                pattern.push(ch);
                self.advance();
            } else if ch == ']' && in_class {
                in_class = false;
                pattern.push(ch);
                self.advance();
            } else if ch == '/' && !in_class {
                // End of pattern
                self.advance(); // Skip closing '/'
                break;
            } else {
                pattern.push(ch);
                self.advance();
            }
        }

        // Parse flags
        let mut flags = String::new();
        while !self.is_at_end() {
            let ch = self.peek();
            // Valid regex flags: g, i, m, s, u, y, d
            if ch.is_ascii_alphabetic() || ch == '$' || ch == '_' {
                flags.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = Token::RegExp(pattern, flags);
        // Don't set current_token - the caller will handle the token directly
        // and the next peek_token will scan fresh from the new position
        Ok(token)
    }

    fn scan_number(&mut self, first: char) -> Result<Token, JsError> {
        let start_pos = self.current_position();
        let mut num_str = first.to_string();
        let mut is_float = false;
        let mut radix: Option<u32> = None; // None = decimal, Some(16) = hex, etc.
        let mut is_legacy_octal = false; // Track legacy octal for BigInt rejection

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
                            source_position: Some(start_pos.clone()),
                        });
                    }
                    while !self.is_at_end() {
                        let ch = self.peek();
                        if ch.is_ascii_hexdigit() {
                            num_str.push(self.advance());
                        } else if ch == '_' {
                            self.advance(); // consume but don't add
                        } else {
                            break;
                        }
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
                            source_position: Some(start_pos.clone()),
                        });
                    }
                    while !self.is_at_end() {
                        let ch = self.peek();
                        if ch == '0' || ch == '1' {
                            num_str.push(self.advance());
                        } else if ch == '_' {
                            self.advance(); // consume but don't add
                        } else {
                            break;
                        }
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
                            source_position: Some(start_pos.clone()),
                        });
                    }
                    while !self.is_at_end() {
                        let ch = self.peek();
                        if ('0'..='7').contains(&ch) {
                            num_str.push(self.advance());
                        } else if ch == '_' {
                            self.advance(); // consume but don't add
                        } else {
                            break;
                        }
                    }
                    radix = Some(8);
                }
_ => {
                    // Regular decimal number starting with 0 (could be legacy octal)
                    // Check if it's a legacy octal: 0 followed by 0-7 digits only
                    // Scan digits and check for legacy octal pattern
                    is_legacy_octal = true; // Assume legacy octal until we see a digit > 7
                    while !self.is_at_end() {
                        let ch = self.peek();
                        if ch.is_ascii_digit() {
                            if ch > '7' {
                                is_legacy_octal = false;
                            }
                            num_str.push(self.advance());
                        } else if ch == '_' {
                            self.advance(); // consume but don't add
                        } else {
                            break;
                        }
                    }
                    // If only the initial '0' and no more digits, it's just 0 (not legacy octal)
                    if num_str.len() == 1 {
                        is_legacy_octal = false;
                    }
                    // Handle decimal point and exponent if present
                    if !is_float && !self.is_at_end() && self.peek() == '.' {
                        // Look ahead to see what follows the dot
                        if let Some(next_after_dot) = self.peek_next() {
                            if next_after_dot.is_ascii_digit() {
                                is_float = true;
                                is_legacy_octal = false; // No longer legacy octal
                                num_str.push(self.advance()); // consume '.'
                                while !self.is_at_end() && self.peek().is_ascii_digit() {
                                    num_str.push(self.advance());
                                }
                            }
                        }
                    }
                    // Handle exponent
                    if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
                        is_float = true;
                        is_legacy_octal = false;
                        num_str.push(self.advance());
                        if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                            num_str.push(self.advance());
                        }
                        while !self.is_at_end() && self.peek().is_ascii_digit() {
                            num_str.push(self.advance());
                        }
                    }
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
                    source_position: Some(start_pos.clone()),
                });
            }
            // Legacy octal literals cannot have BigInt suffix
            if is_legacy_octal {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Invalid BigInt literal: legacy octal literals cannot have BigInt suffix".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
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
                    source_position: Some(start_pos.clone()),
                })?
            }
            None => {
                // Parse decimal
                num_str.parse::<f64>().map_err(|_| JsError {
                    kind: ErrorKind::SyntaxError,
                    message: format!("Invalid number: {}", num_str),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                })?
            }
        };

        Ok(Token::Number(value))
    }

    fn scan_decimal_digits(&mut self, num_str: &mut String, is_float: &mut bool) {
        // Scan integer part (with optional numeric separators)
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_digit() {
                num_str.push(self.advance());
            } else if ch == '_' {
                // Numeric separator - must be between digits
                self.advance(); // consume underscore but don't add to num_str
            } else {
                break;
            }
        }

        // Handle decimal point
        // JavaScript allows both "1.5" and "1." (trailing decimal)
        // We need to distinguish from member access like "123.toString()"
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to see what follows the dot
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() || next == '_' {
                    // Definitely a decimal: 1.5 or 1.5_0
                    *is_float = true;
                    num_str.push(self.advance()); // consume '.'
                    while !self.is_at_end() {
                        let ch = self.peek();
                        if ch.is_ascii_digit() {
                            num_str.push(self.advance());
                        } else if ch == '_' {
                            self.advance(); // consume but don't add
                        } else {
                            break;
                        }
                    }
                } else if !is_id_start(next) {
                    // Trailing decimal: 1. followed by non-identifier (like ; or whitespace)
                    // This is valid: "1." equals 1.0
                    *is_float = true;
                    num_str.push(self.advance()); // consume '.'
                }
                // If next is an identifier start (like 'toString'), leave the dot for member access
            } else {
                // End of file after dot: 1.EOF is valid
                *is_float = true;
                num_str.push(self.advance()); // consume '.'
            }
        }

        // Handle exponent
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            *is_float = true;
            num_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                num_str.push(self.advance());
            }
            while !self.is_at_end() {
                let ch = self.peek();
                if ch.is_ascii_digit() {
                    num_str.push(self.advance());
                } else if ch == '_' {
                    self.advance(); // consume but don't add
                } else {
                    break;
                }
            }
        }
    }

    /// Scan a number that starts with a decimal point: .5, .123, .1e5, etc.
    fn scan_leading_decimal_number(&mut self) -> Result<Token, JsError> {
        let mut num_str = String::from("0."); // Add leading 0 for parsing

        // Scan digits after the decimal point (with optional numeric separators)
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_digit() {
                num_str.push(self.advance());
            } else if ch == '_' {
                // Numeric separator - consume but don't add to string
                self.advance();
            } else {
                break;
            }
        }

        // Handle exponent
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            num_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                num_str.push(self.advance());
            }
            // Scan exponent digits (with optional numeric separators)
            while !self.is_at_end() {
                let ch = self.peek();
                if ch.is_ascii_digit() {
                    num_str.push(self.advance());
                } else if ch == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse as float
        let value = num_str.parse::<f64>().map_err(|_| JsError {
            kind: ErrorKind::SyntaxError,
            message: format!("Invalid number: {}", num_str),
            stack: vec![],
            source_position: Some(self.current_position()),
        })?;

        Ok(Token::Number(value))
    }

    fn scan_identifier(&mut self, first: char) -> Result<Token, JsError> {
        let mut ident = first.to_string();
        let mut has_escape = false;

        while !self.is_at_end() {
            if self.peek() == '\\' && self.peek_next() == Some('u') {
                // Unicode escape in identifier
                has_escape = true;
                self.advance(); // consume '\'
                let ch = self.parse_unicode_escape()?;
                if !is_id_continue(ch) {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: format!("Invalid Unicode escape sequence in identifier"),
                        stack: vec![],
                        source_position: Some(self.current_position()),
                    });
                }
                ident.push(ch);
            } else if is_id_continue(self.peek()) {
                ident.push(self.advance());
            } else {
                break;
            }
        }

        // If identifier had escape sequences, it cannot be a keyword
        // Per spec, escaped keywords are identifiers
        if has_escape {
            return Ok(Token::Identifier(ident, true));
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
            "delete" => Token::Keyword(Keyword::Delete),
            "with" => Token::Keyword(Keyword::With),
            "switch" => Token::Keyword(Keyword::Switch),
            "case" => Token::Keyword(Keyword::Case),
            "do" => Token::Keyword(Keyword::Do),
            "debugger" => Token::Keyword(Keyword::Debugger),
            "static" => Token::Keyword(Keyword::Static),
            // Note: 'constructor' is NOT a keyword - it's just a special method name
            _ => Token::Identifier(ident, false),
        };

        Ok(token)
    }

    /// Scan a private identifier (#name)
    fn scan_private_identifier(&mut self) -> Result<Token, JsError> {
        let start_pos = self.current_position();

        // The '#' has already been consumed by advance()
        // Next character must be a valid identifier start OR a Unicode escape

        let mut name = String::new();

        // Check for first character: either Unicode escape or regular id start
        if self.is_at_end() {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Private identifier must have a name".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }

        // Handle first character
        if self.peek() == '\\' && self.peek_next() == Some('u') {
            // Unicode escape at start
            self.advance(); // consume '\'
            let ch = self.parse_unicode_escape()?;
            if !is_id_start(ch) {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Invalid Unicode escape sequence in private identifier start".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }
            name.push(ch);
        } else if is_id_start(self.peek()) {
            name.push(self.advance());
        } else {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Private identifier must have a name".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }

        // Scan the rest of the identifier
        while !self.is_at_end() {
            if self.peek() == '\\' && self.peek_next() == Some('u') {
                // Unicode escape in identifier
                self.advance(); // consume '\'
                let ch = self.parse_unicode_escape()?;
                if !is_id_continue(ch) {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Invalid Unicode escape sequence in private identifier".to_string(),
                        stack: vec![],
                        source_position: Some(self.current_position()),
                    });
                }
                name.push(ch);
            } else if is_id_continue(self.peek()) {
                name.push(self.advance());
            } else {
                break;
            }
        }

        Ok(Token::PrivateIdentifier(name))
    }

    /// Scan an identifier that starts with a Unicode escape sequence
    fn scan_identifier_with_unicode_start(&mut self) -> Result<Token, JsError> {
        let start_pos = self.current_position();

        // Parse the initial Unicode escape (we already consumed '\')
        let first_char = self.parse_unicode_escape()?;

        // Validate it's a valid identifier start character
        if !is_id_start(first_char) {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: format!("Invalid Unicode escape sequence in identifier start"),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }

        let mut ident = first_char.to_string();

        // Continue scanning the rest of the identifier
        while !self.is_at_end() {
            if self.peek() == '\\' && self.peek_next() == Some('u') {
                // Unicode escape in identifier
                self.advance(); // consume '\'
                let ch = self.parse_unicode_escape()?;
                if !is_id_continue(ch) {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: format!("Invalid Unicode escape sequence in identifier"),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                ident.push(ch);
            } else if is_id_continue(self.peek()) {
                ident.push(self.advance());
            } else {
                break;
            }
        }

        // Check for keywords (identifiers with escapes cannot be keywords)
        // Per spec, escaped keywords are still identifiers, not keywords
        Ok(Token::Identifier(ident, true)) // true = has Unicode escapes
    }

    /// Parse a Unicode escape sequence: \uXXXX or \u{XXXX}
    fn parse_unicode_escape(&mut self) -> Result<char, JsError> {
        let start_pos = self.current_position();

        // Consume 'u'
        if self.is_at_end() || self.advance() != 'u' {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Expected 'u' in Unicode escape".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }

        if self.is_at_end() {
            return Err(JsError {
                kind: ErrorKind::SyntaxError,
                message: "Unexpected end of input in Unicode escape".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            });
        }

        let code_point = if self.peek() == '{' {
            // \u{XXXX} format (ES6 Unicode code point escape)
            self.advance(); // consume '{'
            let mut hex = String::new();
            while !self.is_at_end() && self.peek() != '}' {
                if !self.peek().is_ascii_hexdigit() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Invalid hex digit in Unicode escape".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                hex.push(self.advance());
            }
            if self.is_at_end() || self.peek() != '}' {
                return Err(JsError {
                    kind: ErrorKind::SyntaxError,
                    message: "Expected '}' in Unicode escape".to_string(),
                    stack: vec![],
                    source_position: Some(start_pos.clone()),
                });
            }
            self.advance(); // consume '}'

            u32::from_str_radix(&hex, 16).map_err(|_| JsError {
                kind: ErrorKind::SyntaxError,
                message: "Invalid Unicode code point".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            })?
        } else {
            // \uXXXX format (4 hex digits)
            let mut hex = String::new();
            for _ in 0..4 {
                if self.is_at_end() || !self.peek().is_ascii_hexdigit() {
                    return Err(JsError {
                        kind: ErrorKind::SyntaxError,
                        message: "Expected 4 hex digits in Unicode escape".to_string(),
                        stack: vec![],
                        source_position: Some(start_pos.clone()),
                    });
                }
                hex.push(self.advance());
            }
            u32::from_str_radix(&hex, 16).map_err(|_| JsError {
                kind: ErrorKind::SyntaxError,
                message: "Invalid Unicode code point".to_string(),
                stack: vec![],
                source_position: Some(start_pos.clone()),
            })?
        };

        // Convert to char
        char::from_u32(code_point).ok_or_else(|| JsError {
            kind: ErrorKind::SyntaxError,
            message: format!("Invalid Unicode code point: {}", code_point),
            stack: vec![],
            source_position: Some(start_pos.clone()),
        })
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), JsError> {
        while !self.is_at_end() {
            match self.peek() {
                // ECMAScript WhiteSpace: TAB, VT, FF, SP, NBSP, ZWNBSP (BOM), and other Zs category
                ' ' | '\t' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' |
                // Unicode Space_Separator category (Zs) characters
                '\u{1680}' | // OGHAM SPACE MARK
                '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' |
                '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | // EN QUAD through HAIR SPACE
                '\u{202F}' | // NARROW NO-BREAK SPACE
                '\u{205F}' | // MEDIUM MATHEMATICAL SPACE
                '\u{3000}'   // IDEOGRAPHIC SPACE
                => {
                    self.advance();
                }
                '\n' | '\u{2028}' | '\u{2029}' => {
                    // Line Feed (LF), Line Separator (LS), Paragraph Separator (PS)
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '\r' => {
                    // Carriage Return (CR) - handle CRLF as single line terminator
                    self.advance();
                    if !self.is_at_end() && self.peek() == '\n' {
                        self.advance();
                    }
                    self.line += 1;
                    self.column = 1;
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment - also terminate on LS and PS
                        while !self.is_at_end() {
                            let ch = self.peek();
                            if ch == '\n' || ch == '\r' || ch == '\u{2028}' || ch == '\u{2029}' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        let comment_start_line = self.line;
                        let comment_start_col = self.column;
                        self.advance(); // /
                        self.advance(); // *
                        let mut found_end = false;
                        while !self.is_at_end() {
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); // *
                                self.advance(); // /
                                found_end = true;
                                break;
                            }
                            // Track all line terminators: LF, CR, LS (U+2028), PS (U+2029)
                            let ch = self.peek();
                            if ch == '\n' || ch == '\u{2028}' || ch == '\u{2029}' {
                                self.line += 1;
                                self.column = 1;
                            } else if ch == '\r' {
                                self.line += 1;
                                self.column = 1;
                                // Handle CRLF as single line terminator
                                self.advance();
                                if !self.is_at_end() && self.peek() == '\n' {
                                    self.advance();
                                }
                                continue;
                            }
                            self.advance();
                        }
                        if !found_end {
                            return Err(JsError {
                                kind: ErrorKind::SyntaxError,
                                message: "Unterminated multi-line comment".to_string(),
                                stack: vec![],
                                source_position: Some(SourcePosition {
                                    line: comment_start_line,
                                    column: comment_start_col,
                                    offset: 0,
                                }),
                            });
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(())
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

/// Check if a character is a valid identifier start character.
/// Per ECMAScript spec, this includes:
/// - Unicode ID_Start (Unicode categories: Lu, Ll, Lt, Lm, Lo, Nl)
/// - Other_ID_Start (special exceptions)
/// - $ (dollar sign)
/// - _ (underscore)
/// Excludes: Characters with Pattern_Syntax property
fn is_id_start(ch: char) -> bool {
    // Pattern_Syntax characters are never valid, even if alphabetic
    if is_pattern_syntax(ch) {
        return false;
    }
    ch == '_' || ch == '$' || ch.is_alphabetic() || is_unicode_id_start(ch)
        || is_other_id_start(ch)
}

/// Characters with Other_ID_Start property (special exceptions)
/// These are symbols that can legally start identifiers despite not being letters
fn is_other_id_start(ch: char) -> bool {
    matches!(ch,
        '\u{2118}' |  // ℘ SCRIPT CAPITAL P (Weierstrass p)
        '\u{212E}' |  // ℮ ESTIMATED SYMBOL
        '\u{309B}' |  // ゛ KATAKANA-HIRAGANA VOICED SOUND MARK
        '\u{309C}'    // ゜ KATAKANA-HIRAGANA SEMI-VOICED SOUND MARK
    )
}

/// Check if a character is a valid identifier continue character.
/// Per ECMAScript spec, this includes:
/// - Unicode ID_Continue (ID_Start + Mn, Mc, Nd, Pc)
/// - Other_ID_Start (also valid for continue)
/// - Other_ID_Continue (special exceptions)
/// - $ (dollar sign)
/// - U+200C (Zero Width Non-Joiner)
/// - U+200D (Zero Width Joiner)
/// Excludes: Characters with Pattern_Syntax property
fn is_id_continue(ch: char) -> bool {
    // Pattern_Syntax characters are never valid, even if alphabetic
    if is_pattern_syntax(ch) {
        return false;
    }
    ch == '_' || ch == '$' || ch.is_alphanumeric() || is_unicode_id_continue(ch)
        || is_other_id_start(ch) || is_other_id_continue(ch)
        || ch == '\u{200C}' || ch == '\u{200D}'
}

/// Characters with Other_ID_Continue property (special exceptions)
fn is_other_id_continue(ch: char) -> bool {
    matches!(ch,
        '\u{00B7}' |  // · MIDDLE DOT
        '\u{0387}' |  // · GREEK ANO TELEIA
        '\u{1369}'..='\u{1371}' | // Ethiopic digits 1-9
        '\u{19DA}' |  // ᧚ NEW TAI LUE THAM DIGIT ONE
        '\u{30FB}' |  // ・ KATAKANA MIDDLE DOT (Unicode 15.1)
        '\u{FF65}'    // ・ HALFWIDTH KATAKANA MIDDLE DOT (Unicode 15.1)
    )
}

/// Check if character is in Unicode ID_Start
fn is_unicode_id_start(ch: char) -> bool {
    // Characters with Pattern_Syntax or Pattern_White_Space are NOT valid identifier chars
    if is_pattern_syntax(ch) {
        return false;
    }
    // Check common Unicode letter categories
    matches!(unicode_category(ch),
        UnicodeCategory::Lu | // Uppercase_Letter
        UnicodeCategory::Ll | // Lowercase_Letter
        UnicodeCategory::Lt | // Titlecase_Letter
        UnicodeCategory::Lm | // Modifier_Letter
        UnicodeCategory::Lo | // Other_Letter
        UnicodeCategory::Nl   // Letter_Number
    )
}

/// Check if character is in Unicode ID_Continue
fn is_unicode_id_continue(ch: char) -> bool {
    // Characters with Pattern_Syntax or Pattern_White_Space are NOT valid identifier chars
    if is_pattern_syntax(ch) {
        return false;
    }
    is_unicode_id_start(ch) || matches!(unicode_category(ch),
        UnicodeCategory::Mn | // Nonspacing_Mark
        UnicodeCategory::Mc | // Spacing_Combining_Mark
        UnicodeCategory::Nd | // Decimal_Number
        UnicodeCategory::Pc   // Connector_Punctuation
    )
}

/// Check if character has Pattern_Syntax property
/// Characters with this property cannot be used in identifiers
/// Note: We exclude $ (0x0024) and _ (0x005F) as they are valid in identifiers
fn is_pattern_syntax(ch: char) -> bool {
    let code = ch as u32;
    // Pattern_Syntax ranges from Unicode
    // https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
    // Note: We split ranges to exclude $ (0x0024) and _ (0x005F) which ARE valid identifier chars
    matches!(code,
        0x0021..=0x0023 | // !"#
        0x0025..=0x002F | // %&'()*+,-./  (skip $ = 0x0024)
        0x003A..=0x0040 | // :;<=>?@
        0x005B..=0x005E | // [\]^  (0x005F = _ is valid, 0x0060 = ` handled separately)
        0x0060 |          // `
        0x007B..=0x007E | // {|}~
        0x00A1..=0x00A7 | // ¡¢£¤¥¦§
        0x00A9 |          // ©
        0x00AB..=0x00AC | // «¬
        0x00AE |          // ®
        0x00B0..=0x00B1 | // °±
        0x00B6 |          // ¶
        0x00BB |          // »
        0x00BF |          // ¿
        0x00D7 |          // ×
        0x00F7 |          // ÷
        0x2010..=0x2027 | // Various dashes, quotation marks, bullets
        0x2030..=0x203E | // Per mille, prime, etc.
        0x2041..=0x2053 | // Various punctuation
        0x2055..=0x205E | // Various punctuation
        0x2190..=0x245F | // Arrows and math symbols
        0x2500..=0x2775 | // Box drawing, blocks, geometric shapes
        0x2794..=0x2BFF | // Arrows, math symbols
        0x2E00..=0x2E7F | // Supplemental Punctuation (includes VERTICAL TILDE U+2E2F)
        0x3001..=0x3003 | // CJK punctuation
        0x3008..=0x3020 | // CJK brackets and symbols
        0x3030 |          // WAVY DASH
        0xFD3E..=0xFD3F | // Arabic ornate parentheses
        0xFE45..=0xFE46   // SESAME DOT and WHITE SESAME DOT
    )
}

/// Unicode General Category (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnicodeCategory {
    Lu, // Uppercase_Letter
    Ll, // Lowercase_Letter
    Lt, // Titlecase_Letter
    Lm, // Modifier_Letter
    Lo, // Other_Letter
    Nl, // Letter_Number
    Mn, // Nonspacing_Mark
    Mc, // Spacing_Combining_Mark
    Nd, // Decimal_Number
    Pc, // Connector_Punctuation
    Other,
}

/// Get the Unicode general category of a character
fn unicode_category(ch: char) -> UnicodeCategory {
    // Use Rust's built-in Unicode properties where possible
    if ch.is_uppercase() {
        UnicodeCategory::Lu
    } else if ch.is_lowercase() {
        UnicodeCategory::Ll
    } else if ch.is_alphabetic() {
        // Other alphabetic characters (Lo, Lm, Lt categories)
        UnicodeCategory::Lo
    } else if ch.is_numeric() && !ch.is_ascii_digit() {
        // Non-ASCII numeric (could be Nl or Nd)
        UnicodeCategory::Nd
    } else if ch.is_ascii_digit() {
        UnicodeCategory::Nd
    } else if is_connector_punctuation(ch) {
        UnicodeCategory::Pc
    } else if is_combining_mark(ch) {
        UnicodeCategory::Mn
    } else {
        UnicodeCategory::Other
    }
}

/// Check if character is connector punctuation (Pc category)
fn is_connector_punctuation(ch: char) -> bool {
    matches!(ch,
        '_' | // LOW LINE
        '\u{203F}' | // UNDERTIE
        '\u{2040}' | // CHARACTER TIE
        '\u{2054}' | // INVERTED UNDERTIE
        '\u{FE33}' | // PRESENTATION FORM FOR VERTICAL LOW LINE
        '\u{FE34}' | // PRESENTATION FORM FOR VERTICAL WAVY LOW LINE
        '\u{FE4D}' | // DASHED LOW LINE
        '\u{FE4E}' | // CENTRELINE LOW LINE
        '\u{FE4F}' | // WAVY LOW LINE
        '\u{FF3F}'   // FULLWIDTH LOW LINE
    )
}

/// Check if character is a combining mark (Mn or Mc categories)
fn is_combining_mark(ch: char) -> bool {
    let code = ch as u32;
    // Common combining mark ranges - BMP
    (0x0300..=0x036F).contains(&code) || // Combining Diacritical Marks
    (0x0483..=0x0489).contains(&code) || // Cyrillic combining marks
    (0x0591..=0x05BD).contains(&code) || // Hebrew combining marks
    (0x05BF..=0x05BF).contains(&code) ||
    (0x05C1..=0x05C2).contains(&code) ||
    (0x05C4..=0x05C5).contains(&code) ||
    (0x05C7..=0x05C7).contains(&code) ||
    (0x0610..=0x061A).contains(&code) || // Arabic combining marks
    (0x064B..=0x065F).contains(&code) ||
    (0x0670..=0x0670).contains(&code) ||
    (0x06D6..=0x06DC).contains(&code) ||
    (0x06DF..=0x06E4).contains(&code) ||
    (0x06E7..=0x06E8).contains(&code) ||
    (0x06EA..=0x06ED).contains(&code) ||
    (0x0711..=0x0711).contains(&code) || // Syriac
    (0x0730..=0x074A).contains(&code) ||
    (0x07A6..=0x07B0).contains(&code) || // Thaana
    (0x07EB..=0x07F3).contains(&code) || // NKo
    (0x07FD..=0x07FD).contains(&code) || // NKo Dantayama (Unicode 11.0)
    (0x0816..=0x0819).contains(&code) || // Samaritan
    (0x081B..=0x0823).contains(&code) ||
    (0x0825..=0x0827).contains(&code) ||
    (0x0829..=0x082D).contains(&code) ||
    (0x0859..=0x085B).contains(&code) || // Mandaic
    (0x0897..=0x08D2).contains(&code) || // Arabic Extended-B (Unicode 14.0-16.0)
    (0x08D3..=0x08E1).contains(&code) || // Arabic Extended-A
    (0x08E3..=0x0903).contains(&code) ||
    (0x093A..=0x093C).contains(&code) || // Devanagari
    (0x093E..=0x094F).contains(&code) ||
    (0x0951..=0x0957).contains(&code) ||
    (0x0962..=0x0963).contains(&code) ||
    (0x0981..=0x0983).contains(&code) || // Bengali
    (0x09BC..=0x09BC).contains(&code) ||
    (0x09BE..=0x09C4).contains(&code) ||
    (0x09C7..=0x09C8).contains(&code) ||
    (0x09CB..=0x09CD).contains(&code) ||
    (0x09D7..=0x09D7).contains(&code) ||
    (0x09E2..=0x09E3).contains(&code) ||
    (0x09FE..=0x09FE).contains(&code) || // Bengali Sandhi Mark (Unicode 11.0)
    (0x0A01..=0x0A03).contains(&code) || // Gurmukhi
    (0x0A3C..=0x0A3C).contains(&code) ||
    (0x0A3E..=0x0A42).contains(&code) ||
    (0x0A47..=0x0A48).contains(&code) ||
    (0x0A4B..=0x0A4D).contains(&code) ||
    (0x0A51..=0x0A51).contains(&code) ||
    (0x0A70..=0x0A71).contains(&code) ||
    (0x0A75..=0x0A75).contains(&code) ||
    (0x0A81..=0x0A83).contains(&code) || // Gujarati
    (0x0ABC..=0x0ABC).contains(&code) ||
    (0x0ABE..=0x0AC5).contains(&code) ||
    (0x0AC7..=0x0AC9).contains(&code) ||
    (0x0ACB..=0x0ACD).contains(&code) ||
    (0x0AE2..=0x0AE3).contains(&code) ||
    (0x0AFA..=0x0AFF).contains(&code) || // Gujarati Extended (Unicode 10.0)
    (0x0B01..=0x0B03).contains(&code) || // Oriya
    (0x0B3C..=0x0B3C).contains(&code) ||
    (0x0B55..=0x0B55).contains(&code) || // Oriya Sign Overline (Unicode 13.0)
    (0x0B3E..=0x0B44).contains(&code) ||
    (0x0B47..=0x0B48).contains(&code) ||
    (0x0B4B..=0x0B4D).contains(&code) ||
    (0x0B56..=0x0B57).contains(&code) ||
    (0x0B62..=0x0B63).contains(&code) ||
    (0x0B82..=0x0B82).contains(&code) || // Tamil
    (0x0BBE..=0x0BC2).contains(&code) ||
    (0x0BC6..=0x0BC8).contains(&code) ||
    (0x0BCA..=0x0BCD).contains(&code) ||
    (0x0BD7..=0x0BD7).contains(&code) ||
    (0x0C00..=0x0C04).contains(&code) || // Telugu
    (0x0C3C..=0x0C3C).contains(&code) || // Telugu Sign Nukta (Unicode 14.0)
    (0x0C3E..=0x0C44).contains(&code) ||
    (0x0C46..=0x0C48).contains(&code) ||
    (0x0C4A..=0x0C4D).contains(&code) ||
    (0x0C55..=0x0C56).contains(&code) ||
    (0x0C62..=0x0C63).contains(&code) ||
    (0x0C81..=0x0C83).contains(&code) || // Kannada
    (0x0CBC..=0x0CBC).contains(&code) ||
    (0x0CBE..=0x0CC4).contains(&code) ||
    (0x0CC6..=0x0CC8).contains(&code) ||
    (0x0CCA..=0x0CCD).contains(&code) ||
    (0x0CD5..=0x0CD6).contains(&code) ||
    (0x0CE2..=0x0CE3).contains(&code) ||
    (0x0CF3..=0x0CF3).contains(&code) || // Kannada Sign Combining Anusvara (Unicode 15.0)
    (0x0D00..=0x0D03).contains(&code) || // Malayalam (including Unicode 10.0)
    (0x0D3B..=0x0D3C).contains(&code) || // Malayalam (Unicode 10.0)
    (0x0D3E..=0x0D44).contains(&code) ||
    (0x0D46..=0x0D48).contains(&code) ||
    (0x0D4A..=0x0D4D).contains(&code) ||
    (0x0D57..=0x0D57).contains(&code) ||
    (0x0D62..=0x0D63).contains(&code) ||
    (0x0D81..=0x0D83).contains(&code) || // Sinhala (including Unicode 13.0)
    (0x0DCA..=0x0DCA).contains(&code) ||
    (0x0DCF..=0x0DD4).contains(&code) ||
    (0x0DD6..=0x0DD6).contains(&code) ||
    (0x0DD8..=0x0DDF).contains(&code) ||
    (0x0DF2..=0x0DF3).contains(&code) ||
    (0x0E31..=0x0E31).contains(&code) || // Thai
    (0x0E34..=0x0E3A).contains(&code) ||
    (0x0E47..=0x0E4E).contains(&code) ||
    (0x0EB1..=0x0EB1).contains(&code) || // Lao
    (0x0EB4..=0x0EBC).contains(&code) || // includes U+0EBA (Unicode 12.0)
    (0x0EC8..=0x0ECE).contains(&code) || // includes U+0ECE (Unicode 15.0)
    (0x0F18..=0x0F19).contains(&code) || // Tibetan
    (0x0F35..=0x0F35).contains(&code) ||
    (0x0F37..=0x0F37).contains(&code) ||
    (0x0F39..=0x0F39).contains(&code) ||
    (0x0F3E..=0x0F3F).contains(&code) ||
    (0x0F71..=0x0F84).contains(&code) ||
    (0x0F86..=0x0F87).contains(&code) ||
    (0x0F8D..=0x0F97).contains(&code) ||
    (0x0F99..=0x0FBC).contains(&code) ||
    (0x0FC6..=0x0FC6).contains(&code) ||
    (0x102B..=0x103E).contains(&code) || // Myanmar
    (0x1056..=0x1059).contains(&code) ||
    (0x105E..=0x1060).contains(&code) ||
    (0x1062..=0x1064).contains(&code) ||
    (0x1067..=0x106D).contains(&code) ||
    (0x1071..=0x1074).contains(&code) ||
    (0x1082..=0x108D).contains(&code) ||
    (0x108F..=0x108F).contains(&code) ||
    (0x109A..=0x109D).contains(&code) ||
    (0x135D..=0x135F).contains(&code) || // Ethiopic
    (0x1712..=0x1715).contains(&code) || // Tagalog (includes U+1715, Unicode 14.0)
    (0x1732..=0x1734).contains(&code) || // Hanunoo
    (0x1752..=0x1753).contains(&code) || // Buhid
    (0x1772..=0x1773).contains(&code) || // Tagbanwa
    (0x17B4..=0x17D3).contains(&code) || // Khmer
    (0x17DD..=0x17DD).contains(&code) ||
    (0x180B..=0x180F).contains(&code) || // Mongolian (includes U+180F, Unicode 14.0)
    (0x1885..=0x1886).contains(&code) || // Mongolian Extended
    (0x18A9..=0x18A9).contains(&code) ||
    (0x1920..=0x192B).contains(&code) || // Limbu
    (0x1930..=0x193B).contains(&code) ||
    (0x1A17..=0x1A1B).contains(&code) || // Buginese
    (0x1A55..=0x1A5E).contains(&code) || // Tai Tham
    (0x1A60..=0x1A7C).contains(&code) ||
    (0x1A7F..=0x1A7F).contains(&code) ||
    (0x1AB0..=0x1AEB).contains(&code) || // Combining Diacritical Marks Extended (including Unicode 13.0-17.0)
    (0x1B00..=0x1B04).contains(&code) || // Balinese
    (0x1B34..=0x1B44).contains(&code) ||
    (0x1B6B..=0x1B73).contains(&code) ||
    (0x1B80..=0x1B82).contains(&code) || // Sundanese
    (0x1BA1..=0x1BAD).contains(&code) ||
    (0x1BE6..=0x1BF3).contains(&code) || // Batak
    (0x1C24..=0x1C37).contains(&code) || // Lepcha
    (0x1CD0..=0x1CD2).contains(&code) || // Vedic Extensions
    (0x1CD4..=0x1CE8).contains(&code) ||
    (0x1CED..=0x1CED).contains(&code) ||
    (0x1CF2..=0x1CF4).contains(&code) ||
    (0x1CF7..=0x1CF9).contains(&code) || // Vedic (Unicode 10.0)
    (0x1DC0..=0x1DFF).contains(&code) || // Combining Diacritical Marks Supplement (includes U+1DFA, Unicode 14.0)
    (0x20D0..=0x20F0).contains(&code) || // Combining Diacritical Marks for Symbols
    (0x2CEF..=0x2CF1).contains(&code) || // Coptic
    (0x2D7F..=0x2D7F).contains(&code) || // Tifinagh
    (0x2DE0..=0x2DFF).contains(&code) || // Cyrillic Extended-A
    (0x302A..=0x302F).contains(&code) || // CJK Symbols
    (0x3099..=0x309A).contains(&code) || // Hiragana/Katakana
    (0xA66F..=0xA672).contains(&code) || // Combining Cyrillic
    (0xA674..=0xA67D).contains(&code) ||
    (0xA69E..=0xA69F).contains(&code) ||
    (0xA6F0..=0xA6F1).contains(&code) || // Bamum
    (0xA802..=0xA802).contains(&code) || // Syloti Nagri
    (0xA806..=0xA806).contains(&code) ||
    (0xA80B..=0xA80B).contains(&code) ||
    (0xA823..=0xA827).contains(&code) ||
    (0xA82C..=0xA82C).contains(&code) || // Syloti Nagri Sign Alternate Hasanta (Unicode 13.0)
    (0xA880..=0xA881).contains(&code) || // Saurashtra
    (0xA8B4..=0xA8C5).contains(&code) ||
    (0xA8E0..=0xA8F1).contains(&code) || // Devanagari Extended
    (0xA8FF..=0xA8FF).contains(&code) || // Devanagari Vowel Sign AY (Unicode 11.0)
    (0xA926..=0xA92D).contains(&code) || // Kayah Li
    (0xA947..=0xA953).contains(&code) || // Rejang
    (0xA980..=0xA983).contains(&code) || // Javanese
    (0xA9B3..=0xA9C0).contains(&code) ||
    (0xA9E5..=0xA9E5).contains(&code) || // Myanmar Extended-B
    (0xAA29..=0xAA36).contains(&code) || // Cham
    (0xAA43..=0xAA43).contains(&code) ||
    (0xAA4C..=0xAA4D).contains(&code) ||
    (0xAA7B..=0xAA7D).contains(&code) || // Myanmar Extended-A
    (0xAAB0..=0xAAB0).contains(&code) || // Tai Viet
    (0xAAB2..=0xAAB4).contains(&code) ||
    (0xAAB7..=0xAAB8).contains(&code) ||
    (0xAABE..=0xAABF).contains(&code) ||
    (0xAAC1..=0xAAC1).contains(&code) ||
    (0xAAEB..=0xAAEF).contains(&code) || // Meetei Mayek Extensions
    (0xAAF5..=0xAAF6).contains(&code) ||
    (0xABE3..=0xABEA).contains(&code) || // Meetei Mayek
    (0xABEC..=0xABED).contains(&code) ||
    (0xFB1E..=0xFB1E).contains(&code) || // Hebrew Point Judeo-Spanish Varika
    (0xFE00..=0xFE0F).contains(&code) || // Variation Selectors
    (0xFE20..=0xFE2F).contains(&code) || // Combining Half Marks
    // Astral plane combining marks (for Unicode 10.0+)
    (0x101FD..=0x101FD).contains(&code) || // Phaistos Disc
    (0x102E0..=0x102E0).contains(&code) || // Coptic Epact
    (0x10376..=0x1037A).contains(&code) || // Old Permic
    (0x10A01..=0x10A03).contains(&code) || // Kharoshthi
    (0x10A05..=0x10A06).contains(&code) ||
    (0x10A0C..=0x10A0F).contains(&code) ||
    (0x10A38..=0x10A3A).contains(&code) ||
    (0x10A3F..=0x10A3F).contains(&code) ||
    (0x10AE5..=0x10AE6).contains(&code) || // Manichaean
    (0x10D24..=0x10D27).contains(&code) || // Hanifi Rohingya (Unicode 11.0)
    (0x10D69..=0x10D6D).contains(&code) || // Garay (Unicode 16.0)
    (0x10EAB..=0x10EAC).contains(&code) || // Yezidi (Unicode 13.0)
    (0x10EFA..=0x10EFC).contains(&code) || // Arabic Extended-C (Unicode 15.0-16.0)
    (0x10EFD..=0x10EFF).contains(&code) || // Arabic Extended-C (Unicode 15.0)
    (0x10F46..=0x10F50).contains(&code) || // Sogdian (Unicode 11.0)
    (0x10F82..=0x10F85).contains(&code) || // Old Sogdian (Unicode 14.0)
    (0x11000..=0x11002).contains(&code) || // Brahmi
    (0x11038..=0x11046).contains(&code) ||
    (0x11070..=0x11070).contains(&code) || // Brahmi number joiner (Unicode 14.0)
    (0x11073..=0x11074).contains(&code) || // Brahmi vowel signs (Unicode 14.0)
    (0x1107F..=0x11082).contains(&code) || // Kaithi
    (0x110B0..=0x110BA).contains(&code) ||
    (0x110C2..=0x110C2).contains(&code) || // Kaithi vowel sign (Unicode 14.0)
    (0x11100..=0x11102).contains(&code) || // Chakma
    (0x11127..=0x11134).contains(&code) ||
    (0x11145..=0x11146).contains(&code) || // Chakma (Unicode 11.0)
    (0x11173..=0x11173).contains(&code) || // Mahajani
    (0x11180..=0x11182).contains(&code) || // Sharada
    (0x111B3..=0x111C0).contains(&code) ||
    (0x111C9..=0x111CC).contains(&code) || // Sharada (Unicode 11.0)
    (0x111CE..=0x111CF).contains(&code) || // Sharada vowel signs (Unicode 13.0)
    (0x1122C..=0x11237).contains(&code) || // Khojki
    (0x11241..=0x11241).contains(&code) || // Chakma vowel sign (Unicode 15.0)
    (0x1123E..=0x1123E).contains(&code) ||
    (0x112DF..=0x112EA).contains(&code) || // Khudawadi
    (0x11300..=0x11303).contains(&code) || // Grantha
    (0x1133B..=0x1133C).contains(&code) || // Grantha (Unicode 11.0)
    (0x1133E..=0x11344).contains(&code) ||
    (0x11347..=0x11348).contains(&code) ||
    (0x1134B..=0x1134D).contains(&code) ||
    (0x11357..=0x11357).contains(&code) ||
    (0x11362..=0x11363).contains(&code) ||
    (0x11366..=0x1136C).contains(&code) ||
    (0x11370..=0x11374).contains(&code) ||
    (0x113B8..=0x113E2).contains(&code) || // Tulu-Tigalari (Unicode 16.0)
    (0x11435..=0x11446).contains(&code) || // Newa
    (0x1145E..=0x1145E).contains(&code) || // Newa (Unicode 11.0)
    (0x114B0..=0x114C3).contains(&code) || // Tirhuta
    (0x115AF..=0x115B5).contains(&code) || // Siddham
    (0x115B8..=0x115C0).contains(&code) ||
    (0x115DC..=0x115DD).contains(&code) ||
    (0x11630..=0x11640).contains(&code) || // Modi
    (0x116AB..=0x116B7).contains(&code) || // Takri
    (0x116D0..=0x116E3).contains(&code) || // Ol Onal (Unicode 16.0)
    (0x1171D..=0x1172B).contains(&code) || // Ahom
    (0x1182C..=0x1183A).contains(&code) || // Dogra (Unicode 11.0)
    (0x119D1..=0x119D7).contains(&code) || // Nandinagari
    (0x119DA..=0x119E0).contains(&code) ||
    (0x119E4..=0x119E4).contains(&code) ||
    (0x11930..=0x11935).contains(&code) || // Dives Akuru (Unicode 13.0)
    (0x11937..=0x11938).contains(&code) ||
    (0x1193B..=0x1193E).contains(&code) ||
    (0x11940..=0x11940).contains(&code) ||
    (0x11942..=0x11943).contains(&code) ||
    (0x11A01..=0x11A0A).contains(&code) || // Zanabazar Square (Unicode 10.0)
    (0x11A33..=0x11A39).contains(&code) ||
    (0x11A3B..=0x11A3E).contains(&code) ||
    (0x11A47..=0x11A47).contains(&code) ||
    (0x11A51..=0x11A5B).contains(&code) || // Soyombo (Unicode 10.0)
    (0x11A8A..=0x11A99).contains(&code) ||
    (0x11C2F..=0x11C36).contains(&code) || // Bhaiksuki
    (0x11C38..=0x11C3F).contains(&code) ||
    (0x11B60..=0x11B67).contains(&code) || // Ahom vowel signs (Unicode 17.0)
    (0x11BF0..=0x11BF9).contains(&code) || // Sunuwar digits (Unicode 16.0)
    (0x11C92..=0x11CA7).contains(&code) || // Marchen
    (0x11CA9..=0x11CB6).contains(&code) ||
    (0x11D31..=0x11D36).contains(&code) || // Masaram Gondi (Unicode 10.0)
    (0x11D3A..=0x11D3A).contains(&code) ||
    (0x11D3C..=0x11D3D).contains(&code) ||
    (0x11D3F..=0x11D45).contains(&code) ||
    (0x11D47..=0x11D47).contains(&code) ||
    (0x11D8A..=0x11D8E).contains(&code) || // Gunjala Gondi (Unicode 11.0)
    (0x11D90..=0x11D91).contains(&code) ||
    (0x11D93..=0x11D97).contains(&code) ||
    (0x11DE0..=0x11DE9).contains(&code) || // Garay digits (Unicode 17.0)
    (0x11EF3..=0x11EF6).contains(&code) || // Makasar (Unicode 11.0)
    (0x11F00..=0x11F01).contains(&code) || // Kawi signs (Unicode 15.0)
    (0x11F03..=0x11F03).contains(&code) ||
    (0x11F34..=0x11F3A).contains(&code) ||
    (0x11F3E..=0x11F42).contains(&code) ||
    (0x11F5A..=0x11F5A).contains(&code) || // Kawi sign repha (Unicode 16.0)
    (0x13440..=0x13455).contains(&code) || // Egyptian Hieroglyph Format Controls (Unicode 15.0)
    (0x16AF0..=0x16AF4).contains(&code) || // Bassa Vah
    (0x16B30..=0x16B36).contains(&code) || // Pahawh Hmong
    (0x16F4F..=0x16F4F).contains(&code) || // Miao Consonant Modifier (Unicode 12.0)
    (0x16F51..=0x16F87).contains(&code) || // Miao (extended for Unicode 12.0)
    (0x16F8F..=0x16F92).contains(&code) ||
    (0x16FE4..=0x16FE4).contains(&code) || // Khitan Small Script Filler (Unicode 13.0)
    (0x1611E..=0x16139).contains(&code) || // Gurung Khema (Unicode 16.0)
    (0x16D70..=0x16D79).contains(&code) || // Kirat Rai digits (Unicode 16.0)
    (0x16FF0..=0x16FF1).contains(&code) || // Vietnamese Alternate Reading (Unicode 13.0)
    (0x1BC9D..=0x1BC9E).contains(&code) || // Duployan
    (0x1CCF0..=0x1CCF9).contains(&code) || // Outlined digits (Unicode 16.0)
    (0x1CF00..=0x1CF46).contains(&code) || // Znamenny Musical marks (Unicode 14.0)
    (0x1D165..=0x1D169).contains(&code) || // Musical Symbols
    (0x1D16D..=0x1D172).contains(&code) ||
    (0x1D17B..=0x1D182).contains(&code) ||
    (0x1D185..=0x1D18B).contains(&code) ||
    (0x1D1AA..=0x1D1AD).contains(&code) ||
    (0x1D242..=0x1D244).contains(&code) || // Byzantine Musical Symbols
    (0x1DA00..=0x1DA36).contains(&code) || // Sutton SignWriting
    (0x1DA3B..=0x1DA6C).contains(&code) ||
    (0x1DA75..=0x1DA75).contains(&code) ||
    (0x1DA84..=0x1DA84).contains(&code) ||
    (0x1DA9B..=0x1DA9F).contains(&code) ||
    (0x1DAA1..=0x1DAAF).contains(&code) ||
    (0x1E000..=0x1E006).contains(&code) || // Glagolitic Supplement
    (0x1E008..=0x1E018).contains(&code) ||
    (0x1E01B..=0x1E021).contains(&code) ||
    (0x1E023..=0x1E024).contains(&code) ||
    (0x1E026..=0x1E02A).contains(&code) ||
    (0x1E8D0..=0x1E8D6).contains(&code) || // Mende Kikakui
    (0x1E130..=0x1E136).contains(&code) || // Nyiakeng Puachue Hmong Tone (Unicode 12.0)
    (0x1E08F..=0x1E08F).contains(&code) || // Cyrillic vzmet (Unicode 15.0)
    (0x1E2AE..=0x1E2AE).contains(&code) || // Toto sign rising tone (Unicode 14.0)
    (0x1E2EC..=0x1E2EF).contains(&code) || // Wancho Tone (Unicode 12.0)
    (0x1E4EC..=0x1E4F9).contains(&code) || // Nag Mundari (Unicode 15.0)
    (0x1E5EE..=0x1E5FA).contains(&code) || // Todhri (Unicode 16.0)
    (0x1E6E3..=0x1E6F5).contains(&code) || // Sidetic (Unicode 17.0)
    (0x1E944..=0x1E94A).contains(&code) || // Adlam
    (0x1FBF0..=0x1FBF9).contains(&code) || // Segmented digits (Unicode 13.0)
    (0xE0100..=0xE01EF).contains(&code)    // Variation Selectors Supplement
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
        assert!(matches!(token, Token::Identifier(s, false) if s == "foo"));
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
            Token::Identifier(s, _) if s == "foo"
        ));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Identifier(s, _) if s == "bar"
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
