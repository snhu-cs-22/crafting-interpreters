use super::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        // SAFETY: Because `OpCode` is marked `repr(u8)`, all conversions to u8 are valid.
        unsafe { std::mem::transmute(self) }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // SAFETY: This isn't safe as not all `u8`s translate to a valid `OpCode`. Too bad!
        Ok(unsafe { std::mem::transmute(value) })
        // Err(())
    }
}

struct LineNumber {
    pub number: u32,
    pub count: u32,
}

#[derive(Default)]
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
            Ok(OpCode::Add) => self.simple_instruction("OpAdd", offset),
            Ok(OpCode::Subtract) => self.simple_instruction("OpSubtract", offset),
            Ok(OpCode::Multiply) => self.simple_instruction("OpMultiply", offset),
            Ok(OpCode::Divide) => self.simple_instruction("OpDivide", offset),
            Ok(OpCode::Negate) => self.simple_instruction("OpNegate", offset),
            Ok(OpCode::Return) => self.simple_instruction("OpReturn", offset),
            Err(_) => {
                println!("Unknown opcode {:?}", &instruction);
                offset + 1
            }
        };
    }

    pub fn print_value(&self, value: Value) -> String {
        format!("{}", value)
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        println!("{:-16} {:04} '{}'", name, constant, self.print_value(self.constants[constant as usize]));
        return offset + 2;
    }

    fn get_line(&self, index: usize) -> u32 {
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
