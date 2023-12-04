use std::fmt;

use super::table::hash_string;

#[derive(Clone, Debug, PartialEq)]
pub enum Obj {
    String{
        string: String,
        hash: u32,
    },
}

impl Obj {
    pub fn new_string(string: String) -> Self {
        Obj::String {
            hash: hash_string(&string),
            string,
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Obj::String{ string, .. } => write!(f, "{}", string),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Obj),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Number(value) => write!(f, "{}", value),
            Value::Obj(value) => write!(f, "{}", value),
        }
    }
}

pub type ValueArray = Vec<Value>;
