use super::chunk::{Chunk, OpCode};
use super::compiler::compile;
use super::value::{Value, Obj};

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
    ($vm:ident, $value_type:expr, $op:tt) => {{
        match ($vm.stack.pop().unwrap(), $vm.stack.pop().unwrap()) {
            (Value::Number(b), Value::Number(a)) => {
                $vm.stack.push($value_type(a $op b));
            }
            (_, _) => return InterpretResult::RuntimeError,
        }
    }}
}

macro_rules! runtime_error {
    ($vm:ident, $format:literal$(, )?$($args:expr),*) => {{
        eprintln!($format, $($args, )*);

        let instruction = $vm.ip - 1;
        let line = $vm.chunk.get_line(instruction);
        eprintln!("[line {}] in script", line);
        $vm.reset_stack();
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
                    print!("[ {} ]", *slot);
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte().try_into();
            match instruction {
                Ok(OpCode::Constant) => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                Ok(OpCode::Nil) => self.stack.push(Value::Nil),
                Ok(OpCode::True) => self.stack.push(Value::Bool(true)),
                Ok(OpCode::False) => self.stack.push(Value::Bool(false)),
                Ok(OpCode::Equal) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                Ok(OpCode::Greater) => binary_op!(self, Value::Bool, >),
                Ok(OpCode::Less) => binary_op!(self, Value::Bool, <),
                Ok(OpCode::Add) => {
                    match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
                        (Value::Number(b), Value::Number(a)) => {
                            self.stack.push(Value::Number(a + b));
                        }
                        (Value::Obj(Obj::String(b)), Value::Obj(Obj::String(a))) => {
                            self.stack.push(Value::Obj(Obj::String(a + &b)));
                        }
                        (_, _) => {
                            runtime_error!(self, "Operands must be two numbers or two strings.");
                            return InterpretResult::RuntimeError
                        }
                    }
                }
                Ok(OpCode::Subtract) => binary_op!(self, Value::Number, -),
                Ok(OpCode::Multiply) => binary_op!(self, Value::Number, *),
                Ok(OpCode::Divide) => binary_op!(self, Value::Number, /),
                Ok(OpCode::Not) => {
                    let value = self.stack.pop().unwrap();
                    let value = Value::Bool(self.is_falsey(value));
                    self.stack.push(value);
                }
                Ok(OpCode::Negate) => {
                    if let Value::Number(ref mut value) = *self.stack.last_mut().unwrap() {
                        *value *= -1.0;
                    } else {
                        runtime_error!(self, "Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                }
                Ok(OpCode::Return) => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value);
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
        self.chunk.constants[byte].clone()
    }

    pub fn reset_stack(&mut self) {
        self.stack = Default::default();
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - distance].clone()
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(value) => !value,
            _ => false,
        }
    }
}
