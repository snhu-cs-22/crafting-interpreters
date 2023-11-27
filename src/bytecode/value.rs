use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Obj {
    String(String),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Obj::String(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
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
