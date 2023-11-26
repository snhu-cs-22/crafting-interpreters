use super::scanner::{Scanner, Token, TokenType};
use super::chunk::{Chunk, OpCode};
use super::value::Value;

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

impl Into<u8> for Precedence {
    fn into(self) -> u8 {
        // SAFETY: Because `Precedence` is marked `repr(u8)`, all conversions to u8 are valid.
        unsafe { std::mem::transmute(self) }
    }
}

impl TryFrom<u8> for Precedence {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // SAFETY: This isn't safe as not all `u8`s translate to a valid `Precedence`. Too bad!
        Ok(unsafe { std::mem::transmute(value) })
        // Err(())
    }
}

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
    parser.expression();
    parser.consume(TokenType::Eof, "Expect end of expression.");
    parser.end_compiler();
    !parser.had_error
}

type ParseFn = fn(&mut Parser);

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
        self.previous = std::mem::take(&mut self.current);

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

    fn binary(&mut self) {
        let operator_type = self.previous.r#type;
        let rule = self.get_rule(operator_type).unwrap();
        self.parse_precedence((Into::<u8>::into(rule.precedence) + 1).try_into().unwrap());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        let value = self.previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn unary(&mut self) {
        let operator_type = self.previous.r#type;

        // Compile the operand
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(rule) = self.get_rule(self.previous.r#type) {
            if let Some(prefix_rule) = rule.prefix {
                prefix_rule(self);
            }
        } else {
            self.error("Expect expression");
            return;
        };

        while precedence <= self.get_rule(self.current.r#type).unwrap().precedence {
            self.advance();
            if let Some(infix_rule) = self.get_rule(self.previous.r#type).unwrap().infix {
                infix_rule(self);
            }
        }
    }

    fn get_rule(&mut self, r#type: TokenType) -> Option<ParseRule> {
        match r#type {
            TokenType::LeftParen => Some(ParseRule::new(Some(|p| p.grouping()), None, Precedence::None)),
            TokenType::RightParen => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::LeftBrace => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::RightBrace => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Comma => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Dot => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Minus => Some(ParseRule::new(Some(|p| p.unary()), Some(|p| p.binary()), Precedence::Term)),
            TokenType::Plus => Some(ParseRule::new(None, Some(|p| p.binary()), Precedence::Term)),
            TokenType::Semicolon => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Slash => Some(ParseRule::new(None, Some(|p| p.binary()), Precedence::Factor)),
            TokenType::Star => Some(ParseRule::new(None, Some(|p| p.binary()), Precedence::Factor)),
            TokenType::Bang => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::BangEqual => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Equal => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::EqualEqual => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Greater => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::GreaterEqual => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Less => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::LessEqual => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Identifier => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::String => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Number => Some(ParseRule::new(Some(|p| p.number()), None, Precedence::None)),
            TokenType::And => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Class => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Else => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::False => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::For => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Fun => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::If => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Nil => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Or => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Print => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Return => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Super => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::This => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::True => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Var => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::While => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Error => Some(ParseRule::new(None, None, Precedence::None)),
            TokenType::Eof => Some(ParseRule::new(None, None, Precedence::None)),
            _ => None,
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
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
