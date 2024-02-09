use std::fmt;

use super::chunk::Chunk;
use super::table::hash_string;
use super::value::Value;

pub type NativeFn = fn(arg_count: u8, args: &[Value]) -> Value;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Obj {
    String {
        string: String,
        hash: u32,
    },
    Closure {
        function: Box<Obj>,
    },
    Function {
        arity: u8,
        chunk: Chunk,
        name: Option<Box<Obj>>,
    },
    NativeFunction {
        obj: Option<Box<Obj>>,
        function: NativeFn,
    },
}

impl Obj {
    pub fn new_string(string: String) -> Self {
        Obj::String {
            hash: hash_string(&string),
            string,
        }
    }

    pub fn new_function() -> Self {
        Obj::Function {
            arity: 0,
            chunk: Chunk::new(),
            name: None,
        }
    }

    pub fn new_closure(function: Box<Obj>) -> Self {
        Obj::Closure {
            function,
        }
    }
    
    pub fn new_native(function: NativeFn) -> Self {
        Obj::NativeFunction {
            obj: None,
            function,
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Obj::String{ string, .. } => write!(f, "{}", string),
            Obj::Function{ name, .. } => {
                if let Some(name) = name {
                    write!(f, "<fn {}>", name)
                } else {
                    write!(f, "<script>")
                }
            }
            Obj::Closure { function, .. } => write!(f, "{}", function),
            Obj::NativeFunction{ .. } => write!(f, "<native fn>"),
        }
    }
}
