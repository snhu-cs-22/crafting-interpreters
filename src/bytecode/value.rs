#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

pub type ValueArray = Vec<Value>;
