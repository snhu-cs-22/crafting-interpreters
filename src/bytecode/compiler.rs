use std::mem;

use super::scanner::{Scanner, Token, TokenType};
use super::chunk::{Chunk, OpCode};
use super::object::Obj;
use super::value::Value;
use crate::impl_convert_enum_u8;

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment, // =
    Or, // or
    And, // and
    Equality, // == !=
    Comparison, // < > <= >=
    Term, // + -
    Factor, // * /
    Unary, // ! -
    Call, // . ()
    Primary,
}

impl Precedence {
    pub fn next_highest(self) -> Option<Precedence> {
        (Into::<u8>::into(self) + 1).try_into().ok()
    }
}

impl_convert_enum_u8!(Precedence, Primary);

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut parser = Parser {
        current: Default::default(),
        previous: Default::default(),
        had_error: false,
        panic_mode: false,
        scanner: Scanner::new(source),
        compiler: Compiler::new(),
        compiling_chunk: chunk,
    };

    parser.advance();

    while !parser.matches(TokenType::Eof) {
        parser.declaration()
    }

    parser.consume(TokenType::Eof, "Expect end of expression.");
    parser.end_compiler();
    !parser.had_error
}

type ParseFn = fn(&mut Parser, bool);

#[derive(Debug)]
pub struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence,
}

#[derive(Default, Clone)]
struct Local {
    pub name: Token,
    pub depth: Option<usize>,
}

struct Compiler {
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            locals: Vec::with_capacity(u8::MAX as usize + 1),
            scope_depth: 0,
        }
    }
}

pub struct Parser<'a> {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: Scanner<'a>,
    compiler: Compiler,
    compiling_chunk: &'a mut Chunk,
}

impl Parser<'_> {
    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    fn advance(&mut self) {
        self.previous = mem::take(&mut self.current);

        loop {
            self.current = self.scanner.scan_token();
            if self.current.r#type != TokenType::Error {
                break;
            }
            let lexeme = &self.current.lexeme.clone();
            self.error_at_current(lexeme);
        }
    }

    fn consume(&mut self, r#type: TokenType, message: &str) {
        if self.current.r#type == r#type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check(&self, r#type: TokenType) -> bool {
        self.current.r#type == r#type
    }

    pub fn matches(&mut self, r#type: TokenType) -> bool {
        if !self.check(r#type) {
            return false;
        }
        self.advance();
        true
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line;
        self.current_chunk().write(byte, line);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if cfg!(debug_assertions) {
            if !self.had_error {
                self.current_chunk().disassemble("code");
            }
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.r#type;
        let rule = self.get_rule(operator_type).unwrap();
        self.parse_precedence(rule.precedence.next_highest().unwrap());

        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal.into(), OpCode::Not.into()),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal.into()),
            TokenType::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less.into(), OpCode::Not.into()),
            TokenType::Less => self.emit_byte(OpCode::Less.into()),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater.into(), OpCode::Not.into()),
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.r#type {
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        let end_jump = self.emit_jump(OpCode::Jump.into());

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop.into());

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn string(&mut self, _can_assign: bool) {
        self.emit_constant(Value::Obj(Obj::new_string(self.previous.lexeme[1..self.previous.lexeme.len() - 1].to_string())));
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.resolve_local(&name);
        let (arg, get_op, set_op) = if let Some(arg) = arg {
            (
                arg,
                OpCode::GetLocal,
                OpCode::SetLocal,
            )
        } else {
            (
                self.identifier_constant(&name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            )
        };

        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op.into(), arg);
        } else {
            self.emit_bytes(get_op.into(), arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.r#type;

        // Compile the operand
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match operator_type {
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let can_assign = precedence <= Precedence::Assignment;

        if let Some(rule) = self.get_rule(self.previous.r#type) {
            if let Some(prefix_rule) = rule.prefix {
                prefix_rule(self, can_assign);
            }
        } else {
            self.error("Expect expression");
            return;
        };

        while precedence <= self.get_rule(self.current.r#type).unwrap().precedence {
            self.advance();
            if let Some(infix_rule) = self.get_rule(self.previous.r#type).unwrap().infix {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        self.make_constant(Value::Obj(Obj::new_string(name.lexeme.to_string())))
    }

    #[inline]
    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        a.lexeme.len() == b.lexeme.len() && a.lexeme == b.lexeme
    }

    fn resolve_local(&mut self, name: &Token) -> Option<u8> {
        for (i, local) in self.compiler.locals.iter().enumerate().rev() {
            if self.identifiers_equal(name, &local.name) {
                if local.depth.is_none() {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(i.try_into().unwrap());
            }
        }

        None
    }

    fn add_local(&mut self, name: &Token) {
        if self.compiler.locals.len() > u8::MAX.into() {
            self.error("Too many local variables in function.");
            return;
        }

        let local = Local {
            name: name.clone(),
            depth: None,
        };
        self.compiler.locals.push(local);
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = &self.previous.clone();
        for local in self.compiler.locals.clone().iter().rev() {
            if let Some(depth) = local.depth {
                if depth < self.compiler.scope_depth {
                    break;
                }
            }

            if self.identifiers_equal(name, &local.name) {
                self.error("Already a variable with this name in this scope.");
            }
        }

        self.add_local(name);
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(&self.previous.clone())
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals.last_mut().unwrap().depth = self.compiler.scope_depth.try_into().unwrap();
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse.into());

        self.emit_byte(OpCode::Pop.into());
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn get_rule(&mut self, r#type: TokenType) -> Option<ParseRule> {
        macro_rules! rule_fn {
            ($fn:ident) => {
                Some(|parser, can_assign| parser.$fn(can_assign))
            }
        }

        macro_rules! parse_rule {
            ($prefix:expr, $infix:expr, $precedence:ident) => {
                Some(ParseRule {
                    prefix: $prefix,
                    infix: $infix,
                    precedence: Precedence::$precedence,
                })
            }
        }

        match r#type {
            TokenType::LeftParen => parse_rule!(rule_fn!(grouping), None, None),
            TokenType::RightParen => parse_rule!(None, None, None),
            TokenType::LeftBrace => parse_rule!(None, None, None),
            TokenType::RightBrace => parse_rule!(None, None, None),
            TokenType::Comma => parse_rule!(None, None, None),
            TokenType::Dot => parse_rule!(None, None, None),
            TokenType::Minus => parse_rule!(rule_fn!(unary), rule_fn!(binary), Term),
            TokenType::Plus => parse_rule!(None, rule_fn!(binary), Term),
            TokenType::Semicolon => parse_rule!(None, None, None),
            TokenType::Slash => parse_rule!(None, rule_fn!(binary), Factor),
            TokenType::Star => parse_rule!(None, rule_fn!(binary), Factor),
            TokenType::Bang => parse_rule!(rule_fn!(unary), None, None),
            TokenType::BangEqual => parse_rule!(None, rule_fn!(binary), Equality),
            TokenType::Equal => parse_rule!(None, None, None),
            TokenType::EqualEqual => parse_rule!(None, rule_fn!(binary), Equality),
            TokenType::Greater => parse_rule!(None, rule_fn!(binary), Comparison),
            TokenType::GreaterEqual => parse_rule!(None, rule_fn!(binary), Comparison),
            TokenType::Less => parse_rule!(None, rule_fn!(binary), Comparison),
            TokenType::LessEqual => parse_rule!(None, rule_fn!(binary), Comparison),
            TokenType::Identifier => parse_rule!(rule_fn!(variable), None, None),
            TokenType::String => parse_rule!(rule_fn!(string), None, None),
            TokenType::Number => parse_rule!(rule_fn!(number), None, None),
            TokenType::And => parse_rule!(None, rule_fn!(and), And),
            TokenType::Class => parse_rule!(None, None, None),
            TokenType::Else => parse_rule!(None, None, None),
            TokenType::False => parse_rule!(rule_fn!(literal), None, None),
            TokenType::For => parse_rule!(None, None, None),
            TokenType::Fun => parse_rule!(None, None, None),
            TokenType::If => parse_rule!(None, None, None),
            TokenType::Nil => parse_rule!(rule_fn!(literal), None, None),
            TokenType::Or => parse_rule!(None, rule_fn!(or), Or),
            TokenType::Print => parse_rule!(None, None, None),
            TokenType::Return => parse_rule!(None, None, None),
            TokenType::Super => parse_rule!(None, None, None),
            TokenType::This => parse_rule!(None, None, None),
            TokenType::True => parse_rule!(rule_fn!(literal), None, None),
            TokenType::Var => parse_rule!(None, None, None),
            TokenType::While => parse_rule!(None, None, None),
            TokenType::Error => parse_rule!(None, None, None),
            TokenType::Eof => parse_rule!(None, None, None),
            _ => None,
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.matches(TokenType::Semicolon) {
            // No initializer.
        } else if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = None;
        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false.
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse.into()));
            self.emit_byte(OpCode::Pop.into());
        }

        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump.into());
            let increment_start = self.current_chunk().code.len();
            self.expression();
            self.emit_byte(OpCode::Pop.into());
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::Pop.into()); // Condition.
        }

        self.end_scope();
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump.into());

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop.into());

        if self.matches(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop.into());
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.r#type != TokenType::Eof {
            if self.previous.r#type == TokenType::Semicolon {
                return;
            }
            match self.current.r#type {
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
        }

        self.advance();
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::For) {
            self.for_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while self.compiler.locals.len() > 0 && self.compiler.locals.last().unwrap().depth > self.compiler.scope_depth.try_into().unwrap() {
            self.emit_byte(OpCode::Pop.into());
            self.compiler.locals.pop();
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant > u8::MAX.into() {
            self.error("Too many constant in one chunk.");
            return 0;
        }

        return constant as u8;
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant.into(), constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.current_chunk().code.len() - offset - 2;

        if jump > u16::MAX.into() {
            self.error("Too much code to jump over.");
        }

        self.current_chunk().code[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop.into());

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX.into() {
            self.error("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.current_chunk().code.len() - 2
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.previous.clone(), message);
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.clone(), message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[{}:{}] Error", &token.line, token.col as usize - &token.lexeme.len());

        if token.r#type == TokenType::Eof {
            eprint!(" at end");
        } else if token.r#type == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }
}
