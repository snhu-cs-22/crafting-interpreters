use super::value::{Value, ValueArray};
use crate::impl_convert_enum_u8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    Equal,
    SetGlobal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Closure,
    Return,
}

impl_convert_enum_u8!(OpCode, Return);

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
struct LineNumber {
    pub number: u32,
    pub count: u32,
}

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Chunk {
    pub code: Vec<u8>,
    lines: Vec<LineNumber>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Default::default(),
            lines: Default::default(),
            constants: Default::default(),
        }
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        if let Some(last_line) = self.lines.last_mut() {
            if last_line.number == line {
                last_line.count += 1;
            } else {
                self.lines.push(LineNumber { number: line, count: 1 });
            }
        } else {
            self.lines.push(LineNumber { number: line, count: 1 });
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        return self.constants.len() - 1;
    }
}

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:04} ", self.get_line(offset));
        }

        let instruction = self.code[offset];
        return match instruction.try_into() {
            Ok(OpCode::Constant) => self.constant_instruction("OpConstant", offset),
            Ok(OpCode::Nil) => self.simple_instruction("OpNil", offset),
            Ok(OpCode::True) => self.simple_instruction("OpTrue", offset),
            Ok(OpCode::False) => self.simple_instruction("OpFalse", offset),
            Ok(OpCode::Pop) => self.simple_instruction("OpPop", offset),
            Ok(OpCode::SetGlobal) => self.constant_instruction("OpSetGlobal", offset),
            Ok(OpCode::Equal) => self.simple_instruction("OpEqual", offset),
            Ok(OpCode::GetLocal) => self.byte_instruction("OpGetLocal", offset),
            Ok(OpCode::SetLocal) => self.byte_instruction("OpSetLocal", offset),
            Ok(OpCode::GetGlobal) => self.constant_instruction("OpGetGlobal", offset),
            Ok(OpCode::DefineGlobal) => self.constant_instruction("OpDefineGlobal", offset),
            Ok(OpCode::Greater) => self.simple_instruction("OpGreater", offset),
            Ok(OpCode::Less) => self.simple_instruction("OpLess", offset),
            Ok(OpCode::Add) => self.simple_instruction("OpAdd", offset),
            Ok(OpCode::Subtract) => self.simple_instruction("OpSubtract", offset),
            Ok(OpCode::Multiply) => self.simple_instruction("OpMultiply", offset),
            Ok(OpCode::Divide) => self.simple_instruction("OpDivide", offset),
            Ok(OpCode::Not) => self.simple_instruction("OpNot", offset),
            Ok(OpCode::Negate) => self.simple_instruction("OpNegate", offset),
            Ok(OpCode::Jump) => self.jump_instruction("OpJump", 1, offset),
            Ok(OpCode::JumpIfFalse) => self.jump_instruction("OpJumpIfFalse", 1, offset),
            Ok(OpCode::Print) => self.simple_instruction("OpPrint", offset),
            Ok(OpCode::Loop) => self.jump_instruction("OpLoop", -1, offset),
            Ok(OpCode::Call) => self.byte_instruction("OpCall", offset),
            Ok(OpCode::Closure) => {
                let constant = self.code[offset + 1];
                print!("{:-16} {:04}", "OpClosure", constant);
                println!();
                offset + 2
            }
            Ok(OpCode::Return) => self.simple_instruction("OpReturn", offset),
            Err(_) => {
                println!("Unknown opcode {:?}", &instruction);
                offset + 1
            }
        };
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:-16} {:04}", name, slot);
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i8, offset: usize) -> usize {
        let mut jump = (self.code[offset + 1] as u16) << 8;
        jump |= self.code[offset + 2] as u16;
        println!("{:-16} {:04} -> {:04}", name, offset, offset as isize + 3 + sign as isize * jump as isize);
        offset + 3
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        println!("{:-16} {:04} '{}'", name, constant, self.constants[constant as usize]);
        return offset + 2;
    }

    pub fn get_line(&self, index: usize) -> u32 {
        let mut number = 0;
        let mut current_position = 0;
        for line in &self.lines {
            if current_position > index {
                break;
            } else {
                current_position += line.count as usize;
                number = line.number;
            }
        }
        return number;
    }
}
