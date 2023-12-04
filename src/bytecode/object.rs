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
