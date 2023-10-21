use super::report;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};

#[derive(Debug)]
pub struct ParseError;

fn error(token: &Token, message: &str) {
    if token.r#type == TokenType::Eof {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!(" at \"{}\"", token.lexeme), message);
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// TODO: In C, a block is a statement form that allows you to pack a series of statements where a
// single one is expected. The comma operator is an analogous syntax for expressions. A
// comma-separated series of expressions can be given where a single expression is expected (except
// inside a function call’s argument list). At runtime, the comma operator evaluates the left
// operand and discards the result. Then it evaluates and returns the right operand.
// Add support for comma expressions. Give them the same precedence and associativity as in C.

// TODO: Add error productions to handle each binary operator appearing without a left-hand
// operand. In other words, detect a binary operator appearing at the beginning of an expression.
// Report that as an error, but also parse and discard a right-hand operand with the appropriate
// precedence.
// TODO: Move tokens into expression tree, don't clone them.
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison();

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        Some(expr)
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::Unary(operator.into(), right.into());
        }

        self.primary().unwrap()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.matches(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.matches(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }

        if self.matches(&[TokenType::String, TokenType::Number]) {
            return Ok(Expr::Literal(self.previous().clone().literal));
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression().unwrap();
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(expr.into()));
        }

        Err(self.error(&self.peek(), "Expect expression"))
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for r#type in types.iter() {
            if self.check(r#type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, r#type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(&r#type) {
            return Ok(self.advance());
        }

        Err(self.error(&self.peek(), message))
    }

    fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().r#type == *r#type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        error(token, message);
        ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().r#type == TokenType::Semicolon {
                return;
            }

            match self.peek().r#type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}
