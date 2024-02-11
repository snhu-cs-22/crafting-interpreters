use std::fmt;
use std::hash;

use super::chunk::Chunk;
use super::table::hash_string;
use super::value::Value;

pub trait Object: Clone + fmt::Debug + fmt::Display + hash::Hash + PartialEq + Eq {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Obj {
    String(Box<StringObj>),
    Closure(Box<Closure>),
    Function(Box<Function>),
    NativeFunction(Box<NativeFunction>),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Obj::String(string) => write!(f, "{}", string),
            Obj::Function(function) => write!(f, "{}", function),
            Obj::Closure(closure) => write!(f, "{}", closure),
            Obj::NativeFunction(native_function) => write!(f, "{}", native_function),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StringObj {
    pub string: String,
    pub hash: u32,
}

impl StringObj {
    pub fn new(string: String) -> Self {
        StringObj {
            hash: hash_string(&string),
            string,
        }
    }
}

impl Object for StringObj {}

impl fmt::Display for StringObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Closure {
    pub function: Box<Function>,
}

impl Closure {
    pub fn new(function: Box<Function>) -> Self {
        Closure {
            function,
        }
    }
}

impl Object for Closure {}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: Option<Box<StringObj>>,
}

impl Function {
    pub fn new() -> Self {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: None,
        }
    }
}

impl Object for Function {}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "<fn {}>", name)
        } else {
            write!(f, "<script>")
        }
    }
}

pub type NativeFn = fn(arg_count: u8, args: &[Value]) -> Value;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NativeFunction {
    pub function: NativeFn,
}

impl NativeFunction {
    pub fn new(function: NativeFn) -> Self {
        NativeFunction {
            function,
        }
    }
}

impl Object for NativeFunction {}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn>")
    }
}
