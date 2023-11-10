use std::collections::hash_map::{Entry, HashMap};
use std::mem;

use super::report;
use crate::treewalk::interpreter::{RuntimeError, RuntimeResult};
use crate::treewalk::token::{Literal, Token};

fn error(token: &Token, message: &str) {
    report(token.line, &format!(" at \"{}\"", token.lexeme), message);
}

#[derive(Default, Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    pub values: HashMap<String, Option<Literal>>,
}

impl Environment {
    pub fn push_new(&mut self) {
        let mut new = Environment {
            enclosing: Some(mem::take(self).into()),
            values: HashMap::new(),
        };
        mem::swap(self, &mut new);
    }

    pub fn pop(&mut self) {
        let mut old = mem::take(self.enclosing.as_mut().unwrap());
        mem::swap(self, &mut old);
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        // TODO: Make ../test/function/mutual_recursion.lox work
        if let Some(value) = self.values.get(&name.lexeme.to_string()) {
            if let Some(value) = value {
                // TODO: Remove clone
                Ok(value.clone())
            } else {
                Err(self.error(name, "Variable must be assigned to a value."))
            }
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(self.error(name, "Undefined variable."))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> RuntimeResult<()> {
        if let Entry::Occupied(mut e) = self.values.entry(name.lexeme.to_string()) {
            e.insert(Some(value));
            return Ok(());
        }

        if let Some(ref mut enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }

        Err(self.error(name, &format!("Undefined variable \"{}\".", name.lexeme)))
    }

    pub fn define(&mut self, name: &str, value: Option<Literal>) {
        self.values.insert(name.to_string(), value);
    }

    fn error(&self, token: &Token, message: &str) -> RuntimeError {
        error(token, message);
        RuntimeError::Err
    }
}
