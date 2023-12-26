use crate::impl_convert_enum_u8;

// TODO: Implement C-style comma operator
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TokenType {
    #[default]
    None,
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

impl_convert_enum_u8!(TokenType, Eof);

#[derive(Default, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: Box<str>,
    pub line: u32,
    pub col: u32,
}

#[derive(Clone)]
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
    col: u32,
}

impl Scanner<'_> {
    pub fn new<'a>(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        match c {
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            '0'..='9' => self.number(),
            '"' => self.string(),
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                let r#type = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(r#type)
            }
            '=' => {
                let r#type = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(r#type)
            }
            '<' => {
                let r#type = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(r#type)
            }
            '>' => {
                let r#type = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(r#type)
            }
            _ => self.error_token(&format!("Unexpected character. '{}'", c)),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> char {
        let c = self.peek();
        self.current += 1;

        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        c
    }

    fn start(&self, index: usize) -> char {
        self.source.chars().nth(self.start + index).unwrap()
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn make_token(&self, r#type: TokenType) -> Token {
        Token {
            r#type,
            lexeme: (&self.source[self.start..self.current]).into(),
            line: self.line,
            col: self.col,
        }
    }

    fn error_token<'a>(&'a self, message: &'a str) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: message.into(),
            line: self.line,
            col: self.col,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' | '\n' => {
                    self.advance();
                }
                '/' => {
                    if self.matches('/') {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else if self.matches('*') {
                        self.multi_line_comment();
                    } else {
                        break;
                    }
                }
                _ => return,
            }
        }
    }
    
    fn multi_line_comment(&mut self) {
        let mut nest_depth = 1;

        while nest_depth > 0 && !self.is_at_end() {
            if self.peek() == '/' && self.peek_next() == '*' {
                nest_depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                nest_depth -= 1;
            }

            self.advance();
        }

        // The closing "*/"
        self.advance();
        self.advance();
    }

    fn identifier(&mut self) -> Token {
        while self.peek() == '_' || self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let r#type = match &self.source[self.start..self.current] {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        self.make_token(r#type)
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // The closing ".
        self.advance();
        self.make_token(TokenType::String)
    }
}
