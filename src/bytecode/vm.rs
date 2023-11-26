use super::chunk::{Chunk, OpCode};
use super::compiler::compile;
use super::value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize, // TODO: make this an actual pointer
    stack: Vec<Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

macro_rules! binary_op {
    ($vm:ident, $op:tt) => {{
        let b = $vm.stack.pop().unwrap();
        let a = $vm.stack.pop().unwrap();
        $vm.stack.push(a $op b);
    }}
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Default::default(),
            ip: Default::default(),
            stack: Default::default(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();

        if !compile(source, &mut chunk) {
            return InterpretResult::CompileError;
        }

        self.chunk = chunk;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if cfg!(debug_assertions) {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {} ]", self.chunk.print_value(*slot));
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte().try_into();
            match instruction {
                Ok(OpCode::OpConstant) => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                Ok(OpCode::OpAdd) => binary_op!(self, +),
                Ok(OpCode::OpSubtract) => binary_op!(self, -),
                Ok(OpCode::OpMultiply) => binary_op!(self, *),
                Ok(OpCode::OpDivide) => binary_op!(self, /),
                Ok(OpCode::OpNegate) => {
                    *self.stack.last_mut().unwrap() *= -1.0;
                }
                Ok(OpCode::OpReturn) => {
                    let value = self.stack.pop().unwrap();
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
