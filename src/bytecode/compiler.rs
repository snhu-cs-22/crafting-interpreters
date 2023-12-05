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

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Parser<'a> {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: Scanner<'a>,
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

    fn string(&mut self, _can_assign: bool) {
        self.emit_constant(Value::Obj(Obj::new_string(self.previous.lexeme[1..self.previous.lexeme.len() - 1].to_string())));
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.identifier_constant(&name);
        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetGlobal.into(), arg);
        } else {
            self.emit_bytes(OpCode::GetGlobal.into(), arg);
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

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
        self.identifier_constant(&self.previous.clone())
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }


    fn get_rule(&mut self, r#type: TokenType) -> Option<ParseRule> {
        use Precedence as Prec;

        macro_rules! rule_fn {
            ($fn:ident) => {
                Some(|parser, can_assign| parser.$fn(can_assign))
            }
        }

        match r#type {
            TokenType::LeftParen => Some(ParseRule::new(rule_fn!(grouping), None, Prec::None)),
            TokenType::RightParen => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::LeftBrace => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::RightBrace => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Comma => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Dot => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Minus => Some(ParseRule::new(rule_fn!(unary), rule_fn!(binary), Prec::Term)),
            TokenType::Plus => Some(ParseRule::new(None, rule_fn!(binary), Prec::Term)),
            TokenType::Semicolon => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Slash => Some(ParseRule::new(None, rule_fn!(binary), Prec::Factor)),
            TokenType::Star => Some(ParseRule::new(None, rule_fn!(binary), Prec::Factor)),
            TokenType::Bang => Some(ParseRule::new(rule_fn!(unary), None, Prec::None)),
            TokenType::BangEqual => Some(ParseRule::new(None, rule_fn!(binary), Prec::Equality)),
            TokenType::Equal => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::EqualEqual => Some(ParseRule::new(None, rule_fn!(binary), Prec::Equality)),
            TokenType::Greater => Some(ParseRule::new(None, rule_fn!(binary), Prec::Comparison)),
            TokenType::GreaterEqual => Some(ParseRule::new(None, rule_fn!(binary), Prec::Comparison)),
            TokenType::Less => Some(ParseRule::new(None, rule_fn!(binary), Prec::Comparison)),
            TokenType::LessEqual => Some(ParseRule::new(None, rule_fn!(binary), Prec::Comparison)),
            TokenType::Identifier => Some(ParseRule::new(rule_fn!(variable), None, Prec::None)),
            TokenType::String => Some(ParseRule::new(rule_fn!(string), None, Prec::None)),
            TokenType::Number => Some(ParseRule::new(rule_fn!(number), None, Prec::None)),
            TokenType::And => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Class => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Else => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::False => Some(ParseRule::new(rule_fn!(literal), None, Prec::None)),
            TokenType::For => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Fun => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::If => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Nil => Some(ParseRule::new(rule_fn!(literal), None, Prec::None)),
            TokenType::Or => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Print => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Return => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Super => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::This => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::True => Some(ParseRule::new(rule_fn!(literal), None, Prec::None)),
            TokenType::Var => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::While => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Error => Some(ParseRule::new(None, None, Prec::None)),
            TokenType::Eof => Some(ParseRule::new(None, None, Prec::None)),
            _ => None,
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
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

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
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
        } else {
            self.expression_statement();
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

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
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
        eprint!("[line {}] Error", &token.line);

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
