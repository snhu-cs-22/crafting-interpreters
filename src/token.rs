// TODO: Implement C-style comma operator
// TODO: Implement C-style ternary operator ("?:"). What precedence level is allowed between the ?
// and :? Is the whole operator left-associative or right-associative?
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
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

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: Box<str>,
    pub literal: Literal,
    pub line: u32,
}

impl Token {
    pub fn new(r#type: TokenType, lexeme: &str, literal: Literal, line: u32) -> Token {
        Token {
            r#type,
            lexeme: lexeme.into(),
            literal,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {:?}", self.r#type, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    None,
    String(Box<str>),
    Number(f64),
    Bool(bool),
    Nil,
}
