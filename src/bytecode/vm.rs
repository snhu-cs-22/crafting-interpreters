use super::chunk::{Chunk, OpCode};
use super::compiler::compile;
// use super::table::Table;
type Table = std::collections::HashMap<Obj, Value>;
use super::object::Obj;
use super::value::Value;


pub struct VM {
    chunk: Chunk,
    ip: usize, // TODO: make this an actual pointer
    stack: Vec<Value>,
    strings: Table,
    globals: Table,
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
            strings: Table::new(),
            globals: Table::new(),
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
                Ok(OpCode::Pop) => {
                    self.stack.pop();
                }
                Ok(OpCode::GetLocal) => {
                    let slot = self.read_byte() as usize;
                    self.stack.push(self.stack[slot].clone());
                }
                Ok(OpCode::SetLocal) => {
                    let slot = self.read_byte() as usize;
                    self.stack[slot] = self.peek(0);
                }
                Ok(OpCode::GetGlobal) => {
                    if let Value::Obj(name) = self.read_constant() {
                        if let Some(value) = self.globals.get(&name) {
                            self.stack.push(value.clone());
                        } else {
                            runtime_error!(self, "Undefined variable '{}'.", &name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                Ok(OpCode::DefineGlobal) => {
                    if let Value::Obj(name) = self.read_constant() {
                        self.globals.insert(name, self.peek(0));
                        self.stack.pop();
                    }
                }
                Ok(OpCode::SetGlobal) => {
                    if let Value::Obj(name) = self.read_constant() {
                        if self.globals.insert(name.clone(), self.peek(0)).is_none() {
                            self.globals.remove(&name);
                            runtime_error!(self, "Undefined variable '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
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
                        (Value::Obj(Obj::String{ string: b, .. }), Value::Obj(Obj::String{ string: a, .. })) => {
                            let value = Value::Obj(self.allocate_string(a + &b));
                            self.stack.push(value);
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
                Ok(OpCode::Print) => {
                    println!("{}", self.stack.pop().unwrap());
                }
                Ok(OpCode::Jump) => {
                    let offset = self.read_short() as usize;
                    self.ip += offset;
                }
                Ok(OpCode::JumpIfFalse) => {
                    let offset = self.read_short() as usize;
                    if self.is_falsey(self.peek(0)) {
                        self.ip += offset;
                    }
                }
                Ok(OpCode::Loop) => {
                    let offset = self.read_short() as usize;
                    self.ip -= offset;
                }
                Ok(OpCode::Return) => {
                    // Exit interpreter.
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

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        ((self.chunk.code[self.ip - 2] as u16) << 8) | self.chunk.code[self.ip - 1] as u16
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte() as usize;
        self.chunk.constants[byte].clone()
    }

    pub fn reset_stack(&mut self) {
        self.stack = Default::default();
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - distance - 1].clone()
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(value) => !value,
            _ => false,
        }
    }

    fn allocate_string(&mut self, string: String) -> Obj {
        let string = Obj::new_string(string);
        self.strings.insert(string.clone(), Value::Nil);
        string
    }
}
