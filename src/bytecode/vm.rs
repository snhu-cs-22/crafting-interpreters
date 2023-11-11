use super::chunk::{Chunk, OpCode};
use super::value::Value;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize, // TODO: make this an actual pointer
    stack: [Value; STACK_MAX],
    stack_top: usize, // TODO: make this an actual pointer
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

macro_rules! binary_op {
    ($vm:ident, $op:tt) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        $vm.push(a $op b);
    }}
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Default::default(),
            ip: Default::default(),
            stack: [Default::default(); STACK_MAX],
            stack_top: 0,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        return self.run();
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if cfg!(debug_assertions) {
                print!("          ");
                for slot in &mut self.stack[0..self.stack_top] {
                    print!("[ {} ]", self.chunk.print_value(*slot));
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte().try_into();
            match instruction {
                Ok(OpCode::OpConstant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Ok(OpCode::OpAdd) => binary_op!(self, +),
                Ok(OpCode::OpSubtract) => binary_op!(self, -),
                Ok(OpCode::OpMultiply) => binary_op!(self, *),
                Ok(OpCode::OpDivide) => binary_op!(self, /),
                Ok(OpCode::OpNegate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Ok(OpCode::OpReturn) => {
                    let value = self.pop();
                    println!("{}", self.chunk.print_value(value));
                    return InterpretResult::Ok;
                }
                _ => (),
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.chunk.code[self.ip];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte() as usize;
        self.chunk.constants[byte]
    }
}
