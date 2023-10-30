use std::collections::HashMap;

use super::report;
use crate::interpreter::{RuntimeError, RuntimeResult};
use crate::token::{Literal, Token};

fn error(token: &Token, message: &str) {
    report(token.line, &format!(" at \"{}\"", token.lexeme), message);
}

#[derive(Default, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<Literal>>,
}

impl Environment {
    pub fn new(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(enclosing.into()),
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        if let Some(value) = self.values.get(&name.lexeme.to_string()) {
            if let Some(value) = value {
                // TODO: Remove clone
                Ok(value.clone())
            } else {
                Err(self.error(name, "Variable must be assigned to a value."))
            }
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.get(name)
            } else {
                Err(self.error(name, "Undefined variable."))
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme.to_string()) {
            self.values.insert(name.lexeme.to_string(), Some(value));
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
        RuntimeError
    }
}
