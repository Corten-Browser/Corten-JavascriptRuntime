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
        // JavaScript allows both "1.5" and "1." (trailing decimal)
        // We need to distinguish from member access like "123.toString()"
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to see what follows the dot
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    // Definitely a decimal: 1.5
                    *is_float = true;
                    num_str.push(self.advance()); // consume '.'
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        num_str.push(self.advance());
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
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
        }
    }

    /// Scan a number that starts with a decimal point: .5, .123, .1e5, etc.
    fn scan_leading_decimal_number(&mut self) -> Result<Token, JsError> {
        let mut num_str = String::from("0."); // Add leading 0 for parsing

        // Scan digits after the decimal point
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        // Handle exponent
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            num_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                num_str.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
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
                source_position: Some(start_pos),
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
                    source_position: Some(start_pos),
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
                source_position: Some(start_pos),
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
                source_position: Some(start_pos),
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
                        source_position: Some(start_pos),
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
                source_position: Some(start_pos),
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
            source_position: Some(start_pos),
        })
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), JsError> {
        while !self.is_at_end() {
            match self.peek() {
                // ECMAScript WhiteSpace: TAB, VT, FF, SP, NBSP, ZWNBSP (BOM), and other Zs category
                ' ' | '\t' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' => {
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
fn is_id_start(ch: char) -> bool {
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
fn is_id_continue(ch: char) -> bool {
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
        '\u{19DA}'    // ᧚ NEW TAI LUE THAM DIGIT ONE
    )
}

/// Check if character is in Unicode ID_Start
fn is_unicode_id_start(ch: char) -> bool {
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
    is_unicode_id_start(ch) || matches!(unicode_category(ch),
        UnicodeCategory::Mn | // Nonspacing_Mark
        UnicodeCategory::Mc | // Spacing_Combining_Mark
        UnicodeCategory::Nd | // Decimal_Number
        UnicodeCategory::Pc   // Connector_Punctuation
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
    // Common combining mark ranges
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
    (0x0816..=0x0819).contains(&code) || // Samaritan
    (0x081B..=0x0823).contains(&code) ||
    (0x0825..=0x0827).contains(&code) ||
    (0x0829..=0x082D).contains(&code) ||
    (0x0859..=0x085B).contains(&code) || // Mandaic
    (0x08D4..=0x08E1).contains(&code) || // Arabic Extended-A
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
    (0x0B01..=0x0B03).contains(&code) || // Oriya
    (0x0B3C..=0x0B3C).contains(&code) ||
    (0x0B3E..=0x0B44).contains(&code) ||
    (0x0B47..=0x0B48).contains(&code) ||
    (0x0B4B..=0x0B4D).contains(&code) ||
    (0x0B56..=0x0B57).contains(&code) ||
    (0x0B62..=0x0B63).contains(&code) ||
    (0x0B82..=0x0B82).contains(&code) || // Tamil
    (0x0BBE..=0x0BC2).contains(&code) ||
    (0x0BC6..=0x0BC8).contains(&code) ||
    (0x0BCA..=0x0BCD).contains(&code) ||
    (0x0BD7..=0x0BD7).contains(&code)
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
