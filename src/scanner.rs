use std::collections::HashMap;

use lazy_static::lazy_static;
use substring::Substring;

use super::error;
use crate::token::{Token, TokenType, Literal};

pub struct Scanner {
    source: Box<str>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and", TokenType::And);
        m.insert("class", TokenType::Class);
        m.insert("else", TokenType::Else);
        m.insert("false", TokenType::False);
        m.insert("for", TokenType::For);
        m.insert("fun", TokenType::Fun);
        m.insert("if", TokenType::If);
        m.insert("nil", TokenType::Nil);
        m.insert("or", TokenType::Or);
        m.insert("print", TokenType::Print);
        m.insert("return", TokenType::Return);
        m.insert("super", TokenType::Super);
        m.insert("this", TokenType::This);
        m.insert("true", TokenType::True);
        m.insert("var", TokenType::Var);
        m.insert("while", TokenType::While);
        m
    };
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.into(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", Literal::None, self.line));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let r#type = if self.matches('=') { TokenType::BangEqual } else { TokenType::Bang };
                self.add_token(r#type);
            },
            '=' => {
                let r#type = if self.matches('=') { TokenType::EqualEqual } else { TokenType::Equal };
                self.add_token(r#type);
            },
            '<' => {
                let r#type = if self.matches('=') { TokenType::LessEqual } else { TokenType::Less };
                self.add_token(r#type);
            },
            '>' => {
                let r#type = if self.matches('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                self.add_token(r#type);
            },
            // TODO: Implement C-style multi-line comments (/* ... */)
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphabetic() {
                    self.identifier();
                } else {
                    error(self.line, "Unexpected character");
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let text = self.source.substring(self.start, self.current);
        let r#type = KEYWORDS.get(text).unwrap_or(&TokenType::Identifier);
        self.add_token(*r#type);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token_with_literal(
            TokenType::Number,
            Literal::Number(self.source.substring(self.start, self.current).parse().unwrap())
        );
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string.");
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value = self.source.substring(self.start + 1, self.current - 1);
        self.add_token_with_literal(TokenType::String, Literal::String(value.into()));
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;

        true
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

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.add_token_with_literal(r#type, Literal::None);
    }

    fn add_token_with_literal(&mut self, r#type: TokenType, literal: Literal) {
        let text = self.source.substring(self.start, self.current);
        self.tokens
            .push(Token::new(r#type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
