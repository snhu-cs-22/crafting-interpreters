use super::report;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};

#[derive(Debug)]
pub struct ParseError;

type ParseResult<T> = Result<T, ParseError>;

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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration().unwrap());
        }
        statements
    }

    fn expression(&mut self) -> Option<Expr> {
        // TODO: Decide whether the `Option` return is necessary
        Some(self.assignment())
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        // TODO: catch errors and synchronize
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression().unwrap();
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value.into()))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        // TODO: Remove clone
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer = None;
        if self.matches(&[TokenType::Equal]) {
            initializer = self.expression().map(|expr| expr.into());
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var(name, initializer))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression().unwrap();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr.into()))
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.equality();

        if self.matches(&[TokenType::Equal]) {
            // TODO: Remove clone
            let equals = self.previous().clone();
            let value = self.assignment();

            match expr {
                Expr::Variable(name) => return Expr::Assign(name, value.into()),
                // We report an error if the left-hand side isn’t a valid assignment target, but we
                // don’t throw it because the parser isn’t in a confused state where we need to go
                // into panic mode and synchronize.
                _ => self.error(&equals, "Invalid assignment target."),
            };
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        expr
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

    fn primary(&mut self) -> ParseResult<Expr> {
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

        if self.matches(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
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

    fn consume(&mut self, r#type: TokenType, message: &str) -> ParseResult<&Token> {
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
