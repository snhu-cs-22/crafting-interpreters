use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::chunk::{Chunk, OpCode};
use super::compiler::compile;
// use super::table::Table;
type Table = std::collections::HashMap<Obj, Value>;
use super::object::{Obj, NativeFn};
use super::value::{HashableF64, Value};

struct CallFrame {
    function: Box<Obj>,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    pub fn new(function: Box<Obj>, slot: usize) -> Self {
        CallFrame {
            function,
            slot,
            ip: 0,
        }
    }

    pub fn chunk(&mut self) -> &mut Chunk {
        match *self.function.as_mut() {
            Obj::Function { ref mut chunk, .. } => chunk,
            _ => unreachable!(),
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        let ip = self.ip - 1;
        self.chunk().code[ip]
    }

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        let upper = self.ip - 2;
        let lower = self.ip - 1;
        let chunk = self.chunk();
        ((chunk.code[upper] as u16) << 8) | chunk.code[lower] as u16
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte() as usize;
        self.chunk().constants[byte].clone()
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
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
        match ($vm.pop(), $vm.pop()) {
            (Value::Number(b), Value::Number(a)) => {
                $vm.push($value_type(a $op b));
            }
            (_, _) => return InterpretResult::RuntimeError,
        }
    }}
}

macro_rules! runtime_error {
    ($vm:ident, $format:literal$(, )?$($args:expr),*) => {{
        eprintln!($format, $($args, )*);

        for frame in $vm.frames.iter().rev() {
            if let Obj::Function { chunk, name, .. } = frame.function.as_ref() {
                let instruction = frame.ip - 1;
                eprint!("[line {}] in ", chunk.get_line(instruction));
                if let Some(name) = name {
                    eprintln!("{}()", name);
                } else {
                    eprintln!("script");
                }
            }
        }

        $vm.reset_stack();
    }}
}

fn clock_native(_arg_count: u8, _args: &[Value]) -> Value {
    Value::Number(
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::new(0, 0))
            .as_millis() as f64).into()
    )
}

impl VM {
    pub fn new() -> Self {
        let mut result = VM {
            frames: Default::default(),
            stack: Default::default(),
            strings: Table::new(),
            globals: Table::new(),
        };

        result.define_native("clock", clock_native);

        result
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let function = compile(source);
        if let Some(function) = function {
            if let Obj::Function { .. } = function {
                let frame = CallFrame::new(function.clone().into(), 0);
                self.frames.push(frame);
            } else {
                unreachable!();
            }

            self.push(Value::Obj(function.clone()));
            self.call(function.clone(), 0);
        } else {
            return InterpretResult::CompileError;
        }

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
                let ip = self.current_frame().ip;
                self.current_frame().chunk().disassemble_instruction(ip);
            }

            let instruction = self.current_frame().read_byte().try_into();
            match instruction {
                Ok(OpCode::Constant) => {
                    let constant = self.current_frame().read_constant();
                    self.push(constant);
                }
                Ok(OpCode::Nil) => self.push(Value::Nil),
                Ok(OpCode::True) => self.push(Value::Bool(true)),
                Ok(OpCode::False) => self.push(Value::Bool(false)),
                Ok(OpCode::Pop) => {
                    self.pop();
                }
                Ok(OpCode::GetLocal) => {
                    let slot = self.current_frame().read_byte() as usize;
                    let slot_index = self.current_frame().slot + slot;
                    let slot_value = self.stack[slot_index].clone();
                    self.push(slot_value);
                }
                Ok(OpCode::SetLocal) => {
                    let slot = self.current_frame().read_byte() as usize;
                    let value = self.peek(0);
                    self.stack[slot] = value;
                }
                Ok(OpCode::GetGlobal) => {
                    if let Value::Obj(name) = self.current_frame().read_constant() {
                        if let Some(value) = self.globals.get(&name) {
                            self.push(value.clone());
                        } else {
                            runtime_error!(self, "Undefined variable '{}'.", &name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                Ok(OpCode::DefineGlobal) => {
                    if let Value::Obj(name) = self.current_frame().read_constant() {
                        let value = self.peek(0);
                        self.globals.insert(name, value);
                        self.pop();
                    }
                }
                Ok(OpCode::SetGlobal) => {
                    if let Value::Obj(name) = self.current_frame().read_constant() {
                        let value = self.peek(0);
                        if self.globals.insert(name.clone(), value).is_none() {
                            self.globals.remove(&name);
                            runtime_error!(self, "Undefined variable '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                Ok(OpCode::Equal) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Ok(OpCode::Greater) => binary_op!(self, Value::Bool, >),
                Ok(OpCode::Less) => binary_op!(self, Value::Bool, <),
                Ok(OpCode::Add) => {
                    match (self.pop(), self.pop()) {
                        (Value::Number(b), Value::Number(a)) => {
                            self.push(Value::Number(a + b));
                        }
                        (Value::Obj(Obj::String{ string: b, .. }), Value::Obj(Obj::String{ string: a, .. })) => {
                            let value = Value::Obj(self.allocate_string(a + &b));
                            self.push(value);
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
                    let value = Value::Bool(self.pop().is_falsey());
                    self.push(value);
                }
                Ok(OpCode::Negate) => {
                    if let Value::Number(ref mut value) = *self.stack.last_mut().unwrap() {
                        *value = *value * HashableF64(-1.0);
                    } else {
                        runtime_error!(self, "Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                }
                Ok(OpCode::Print) => {
                    println!("{}", self.pop());
                }
                Ok(OpCode::Jump) => {
                    let offset = self.current_frame().read_short() as usize;
                    self.current_frame().ip += offset;
                }
                Ok(OpCode::JumpIfFalse) => {
                    let offset = self.current_frame().read_short() as usize;
                    if self.peek(0).is_falsey() {
                        self.current_frame().ip += offset;
                    }
                }
                Ok(OpCode::Loop) => {
                    let offset = self.current_frame().read_short() as usize;
                    self.current_frame().ip -= offset;
                }
                Ok(OpCode::Call) => {
                    let arg_count = self.current_frame().read_byte();
                    let value = self.peek(arg_count.into());
                    if !self.call_value(value, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                Ok(OpCode::Return) => {
                    let result = self.pop();
                    let prev_frame = self.frames.pop().unwrap();
                    if self.frames.len() <= 1 {
                        self.pop();
                        return InterpretResult::Ok;
                    }

                    self.stack.truncate(prev_frame.slot);
                    self.push(result);
                }
                _ => (),
            }
        }
    }

    pub fn reset_stack(&mut self) {
        self.stack = Default::default();
        self.frames = Default::default();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - distance - 1].clone()
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn call(&mut self, function: Obj, arg_count: u8) -> bool {
        if let Obj::Function { arity, .. } = function {
            if arg_count != arity {
                runtime_error!(self, "Expected {} argument(s) but got {}.", arity, arg_count);
                return false;
            }

            if self.frames.len() > 256 {
                runtime_error!(self, "Stack overflow");
                return false;
            }

            let frame = CallFrame::new(
                function.into(),
                self.stack.len() - arg_count as usize - 1
            );
            self.frames.push(frame);
            true
        } else {
            false
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        if let Value::Obj(callee) = callee {
            match callee {
                Obj::Function { .. } => self.call(callee, arg_count),
                Obj::NativeFunction { function, .. } => {
                    let result = function(arg_count, &[self.peek(arg_count as usize)]);
                    let new_stack_size = self.stack.len() - arg_count as usize + 1;
                    self.stack.truncate(new_stack_size);
                    self.push(result);
                    true
                }
                _ => {
                    runtime_error!(self, "Can only call functions and classes.");
                    false
                }
            }
        } else {
            runtime_error!(self, "Can only call functions and classes.");
            false
        }
    }

    fn allocate_string(&mut self, string: String) -> Obj {
        let string = Obj::new_string(string);
        self.strings.insert(string.clone(), Value::Nil);
        string
    }

    fn define_native(&mut self, name: &str, function: NativeFn) {
        let name = Obj::new_string(name.to_string());
        let function = Value::Obj(Obj::new_native(function));
        self.push(Value::Obj(name.clone()));
        self.push(function.clone());
        self.globals.insert(name, function);
        self.pop();
        self.pop();
    }
}
