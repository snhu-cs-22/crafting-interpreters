use std::fmt;
use std::hash;

use super::object::Obj;
use crate::{impl_wrapper_type, impl_binary_ops_for_wrapper_type};

pub type ValueArray = Vec<Value>;

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(HashableF64),
    Obj(Obj),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Bool(value) => !value,
            _ => false,
        }
    }
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

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct HashableF64(pub f64);

impl_wrapper_type!(HashableF64, f64);

impl_binary_ops_for_wrapper_type!(HashableF64, f64, std::ops::Add, add, +);
impl_binary_ops_for_wrapper_type!(HashableF64, f64, std::ops::Div, div, /);
impl_binary_ops_for_wrapper_type!(HashableF64, f64, std::ops::Mul, mul, *);
impl_binary_ops_for_wrapper_type!(HashableF64, f64, std::ops::Sub, sub, -);

impl fmt::Display for HashableF64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl hash::Hash for HashableF64 {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        hasher.write(&self.0.to_be_bytes());
    }
}

impl Eq for HashableF64 {}
