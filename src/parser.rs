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
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }
        statements
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.matches(&[TokenType::Fun]) {
            self.function("function")
        } else if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        // Catch errors and synchronize
        result.ok().or_else(|| {
            self.synchronize();
            None
        })
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after \"for\".")?;

        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment.into())]);
        }

        body = Stmt::While(
            condition
                .unwrap_or(Expr::Literal(Literal::Bool(true)))
                .into(),
            body.into(),
        );

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after \"if\".")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(self.statement()?.into())
        } else {
            None
        };

        Ok(Stmt::If(condition.into(), then_branch.into(), else_branch))
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value.into()))
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self.previous().clone();
        let value = if !self.check(&TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal::Nil)
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword, value.into()))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        // TODO: Remove clone
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();

        let initializer = if self.matches(&[TokenType::Equal]) {
            Some(self.expression()?.into())
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var(name, initializer))
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after \"while\".")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While(condition.into(), body.into()))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr.into()))
    }

    fn function(&mut self, kind: &str) -> ParseResult<Stmt> {
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {} name.", kind))?
            .clone();

        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.")?
                        .clone(),
                );

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '}}' before {} parameters.", kind),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(name, parameters, body))
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.ternary()?;

        if self.matches(&[TokenType::Equal]) {
            // TODO: Remove clone
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => return Ok(Expr::Assign(name, value.into())),
                // We report an error if the left-hand side isn’t a valid assignment target, but we
                // don’t throw it because the parser isn’t in a confused state where we need to go
                // into panic mode and synchronize.
                _ => self.error(&equals, "Invalid assignment target."),
            };
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> ParseResult<Expr> {
        let mut expr = self.or()?;

        while self.matches(&[TokenType::QuestionMark]) {
            let operator_1 = self.previous().clone();
            let middle = self.ternary()?;

            if self.matches(&[TokenType::Colon]) {
                let operator_2 = self.previous().clone();
                let right = self.ternary()?;
                expr = Expr::Ternary(
                    expr.into(),
                    operator_1,
                    middle.into(),
                    operator_2,
                    right.into(),
                );
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            // TODO: Remove clone
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            // TODO: Remove clone
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(expr.into(), operator.into(), right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator.into(), right.into()));
        }

        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call(callee.into(), paren.clone(), arguments))
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
            let expr = self.expression()?;
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
