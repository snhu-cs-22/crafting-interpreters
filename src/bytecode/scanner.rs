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
}

#[derive(Clone)]
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner<'_> {
    pub fn new<'a>(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        if self.is_alpha(c) {
            return self.identifier();
        }
        if c.is_ascii_digit() {
            return self.number();
        }

        match c {
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
            '"' => self.string(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn start(&self, index: usize) -> char {
        self.source.chars().nth(self.start + index).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn make_token(&self, r#type: TokenType) -> Token {
        Token {
            r#type,
            lexeme: (&self.source[self.start..self.current]).into(),
            line: self.line,
        }
    }

    fn error_token<'a>(&'a self, message: &'a str) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: message.into(),
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
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
        let mut comment_nest_depth = 1;

        while comment_nest_depth > 0 && !self.is_at_end() {
            if self.peek() == '/' && self.peek_next() == '*' {
                comment_nest_depth += 1;
            }

            if self.peek() == '*' && self.peek_next() == '/' {
                comment_nest_depth -= 1;
            }

            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // The closing "*/"
        self.advance();
        self.advance();
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, r#type: TokenType) -> TokenType {
        if self.current - self.start == start + length && self.source[self.start+start..self.start+start+length] == *rest {
            return r#type;
        }

        TokenType::Identifier
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.start(0) {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.start(1) {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.start(1) {
                        'h' => self.check_keyword(2, 2, "is", TokenType::This),
                        'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
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
            if self.peek() == '\n' {
                self.line += 1;
            }
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
