use crate::interpreter::{Interpreter, RuntimeError, RuntimeResult};
use crate::stmt::Stmt;
use crate::token::Literal;

pub trait Callable: std::fmt::Debug + Clone {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> RuntimeResult<Literal>;
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub callable: fn(&mut Interpreter, &[Literal]) -> RuntimeResult<Literal>,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity.into()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> RuntimeResult<Literal> {
        (self.callable)(interpreter, &arguments)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub declaration: Stmt,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        if let Stmt::Function(_, params, _) = &self.declaration {
            return params.len();
        }
        unreachable!();
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> RuntimeResult<Literal> {
        interpreter.environment.push_new();
        if let Stmt::Function(_, params, body) = &self.declaration {
            for (param, argument) in std::iter::zip(params, arguments) {
                interpreter.environment.define(&param.lexeme, Some(argument));
            }

            match interpreter.interpret(body) {
                Ok(_) => {
                    interpreter.environment.pop();
                    return Ok(Literal::Nil);
                }
                Err(error) => {
                    interpreter.environment.pop();
                    return match error {
                        RuntimeError::Err => Err(error),
                        RuntimeError::Return(value) => Ok(value),
                    };
                },
            }
        }
        unreachable!();
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
