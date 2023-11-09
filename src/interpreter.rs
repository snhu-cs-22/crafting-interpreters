use std::time::{Duration, SystemTime, UNIX_EPOCH};

use substring::Substring;

use super::report;
use crate::environment::Environment;
use crate::expr::Expr;
use crate::function::{Callable, Function, NativeFunction};
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};

fn error(token: &Token, message: &str) {
    report(token.line, &format!(" at \"{}\"", token.lexeme), message);
}

pub enum RuntimeError {
    Err,
    Return(Literal),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals: Environment = Default::default();
        globals.define(
            "clock",
            Some(Literal::NativeFunction(
                NativeFunction {
                    arity: 0,
                    callable: |_, _| {
                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(Duration::new(0, 0))
                            .as_millis() as f64;
                        Ok(Literal::Number(time))
                    },
                }
                .into(),
            )),
        );

        Interpreter {
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for statement in statements {
            match statement {
                Stmt::Print(value) => {
                    let value = self.evaluate(value)?;
                    println!("{}", self.stringify(value))
                }
                Stmt::Return(_, value) => {
                    let value = match **value {
                        Expr::Literal(Literal::Nil) => Literal::Nil,
                        _ => self.evaluate(value)?,
                    };
                    return Err(RuntimeError::Return(value));
                }
                Stmt::Expression(expr) => {
                    self.evaluate(expr)?;
                }
                Stmt::Function(name, _, _) => {
                    let function = Literal::Function(
                        Function {
                            declaration: statement.clone(),
                            closure: self.environment.clone(),
                        }
                        .into(),
                    );
                    self.environment.define(&name.lexeme, Some(function));
                }
                Stmt::If(condition, then_branch, else_branch) => {
                    let condition = &self.evaluate(condition)?;
                    // TODO: Remove clone
                    if self.is_truthy(condition) {
                        self.interpret(&[*then_branch.clone()])?;
                    } else if let Some(else_branch) = else_branch {
                        self.interpret(&[*else_branch.clone()])?;
                    }
                }
                Stmt::Var(name, initializer) => {
                    let value = match initializer {
                        Some(init_expression) => Some(self.evaluate(init_expression)?),
                        None => None,
                    };

                    self.environment.define(&name.lexeme, value);
                }
                Stmt::While(condition, body) => {
                    // TODO: Implement `break` statements:
                    //
                    // The syntax is a break keyword followed by a semicolon. It should
                    // be a syntax error to have a break statement appear outside of any
                    // enclosing loop. At runtime, a break statement causes execution to
                    // jump to the end of the nearest enclosing loop and proceeds from
                    // there. Note that the break may be nested inside other blocks and
                    // if statements that also need to be exited.
                    let mut condition_value = self.evaluate(condition)?;
                    while self.is_truthy(&condition_value) {
                        self.interpret(&[*body.clone()])?;

                        condition_value = self.evaluate(condition)?;
                    }
                }
                Stmt::Block(statements) => {
                    self.environment.push_new();
                    self.interpret(statements)?;
                    self.environment.pop();
                }
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult<Literal> {
        match expr {
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.r#type {
                    // Equality
                    TokenType::BangEqual => Ok(self.is_equal(left, right)),
                    TokenType::EqualEqual => Ok(self.is_equal(left, right)),

                    // Comparison
                    TokenType::Greater => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Bool(left > right))
                    }
                    TokenType::GreaterEqual => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Bool(left >= right))
                    }
                    TokenType::Less => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Bool(left < right))
                    }
                    TokenType::LessEqual => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Bool(left <= right))
                    }

                    // Arithmetic
                    TokenType::Minus => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Number(left - right))
                    }
                    TokenType::Slash => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        if right == 0.0 {
                            return Err(self.error(operator, "Cannot divide by zero"));
                        }

                        Ok(Literal::Number(left / right))
                    }
                    TokenType::Star => {
                        let (left, right) = self.check_number_operands(operator, left, right)?;
                        Ok(Literal::Number(left * right))
                    }
                    TokenType::Plus => {
                        match (&left, &right) {
                            // Non-casting operations
                            (Literal::Number(_), Literal::Number(_)) => {
                                let (left, right) =
                                    self.check_number_operands(operator, left, right)?;
                                Ok(Literal::Number(left + right))
                            }
                            (Literal::String(left), Literal::String(right)) => {
                                Ok(Literal::String((left.to_string() + right).into()))
                            }

                            // Casting operations
                            (Literal::Number(left), Literal::String(right)) => {
                                Ok(Literal::String((left.to_string() + right).into()))
                            }
                            (Literal::String(left), Literal::Number(right)) => Ok(Literal::String(
                                (left.to_string() + &right.to_string()).into(),
                            )),

                            (_, _) => Err(self
                                .error(operator, "Operands must be two numbers or two strings.")),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Call(callee, paren, arguments) => {
                let arguments = arguments
                    .iter()
                    .map(|expr| self.evaluate(expr))
                    .collect::<Result<Vec<_>, _>>()?;
                match self.evaluate(callee)? {
                    Literal::Function(mut function) => {
                        if arguments.len() != function.arity() {
                            return Err(self.error(
                                paren,
                                &format!(
                                    "Expected {} argument(s) but got {}.",
                                    function.arity(),
                                    arguments.len()
                                ),
                            ));
                        }

                        Ok(function.call(self, arguments)?)
                    }
                    Literal::NativeFunction(mut function) => {
                        if arguments.len() != function.arity() {
                            return Err(self.error(
                                paren,
                                &format!(
                                    "Expected {} argument(s) but got {}.",
                                    function.arity(),
                                    arguments.len()
                                ),
                            ));
                        }

                        Ok(function.call(self, arguments)?)
                    }
                    _ => Err(self.error(paren, "Only functions and classes are callable.")),
                }
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            // TODO: Remove clone
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(left)?;

                if operator.r#type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left.clone());
                    }
                } else if !self.is_truthy(&left) {
                    return Ok(left.clone());
                }


                self.evaluate(right)
            }
            Expr::Ternary(left, _, middle, _, right) => {
                let left = self.evaluate(left)?;
                if self.is_truthy(&left) {
                    self.evaluate(middle)
                } else {
                    self.evaluate(right)
                }
            }
            Expr::Unary(operator, right) => {
                let right = self.evaluate(right)?;

                match operator.r#type {
                    TokenType::Minus => {
                        let right = self.check_number_operand(operator, right)?;
                        Ok(Literal::Number(-right))
                    }
                    TokenType::Bang => Ok(Literal::Bool(!self.is_truthy(&right))),
                    _ => todo!(),
                }
            }
            Expr::Variable(name) => self.environment.get(name),
            Expr::Assign(name, value) => {
                let value = self.evaluate(value)?;
                // TODO: Remove clone
                self.environment.assign(name, value.clone())?;
                Ok(value)
            }
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: Literal,
        right: Literal,
    ) -> RuntimeResult<(f64, f64)> {
        match (left, right) {
            (Literal::Number(left), Literal::Number(right)) => Ok((left, right)),
            (_, _) => Err(self.error(operator, "Operands must be numbers.")),
        }
    }

    fn check_number_operand(&self, operator: &Token, operand: Literal) -> RuntimeResult<f64> {
        match operand {
            Literal::Number(value) => Ok(value),
            _ => Err(self.error(operator, "Operand must be a number.")),
        }
    }

    fn is_truthy(&self, literal: &Literal) -> bool {
        match literal {
            Literal::Bool(value) => *value,
            Literal::Nil => false,
            _ => true,
        }
    }

    fn is_equal(&self, a: Literal, b: Literal) -> Literal {
        if a == Literal::Nil && b == Literal::Nil {
            return Literal::Bool(true);
        }
        if a == Literal::Nil {
            return Literal::Bool(false);
        }

        Literal::Bool(a == b)
    }

    fn stringify(&self, literal: Literal) -> Box<str> {
        match literal {
            Literal::Nil => "nil".into(),
            Literal::String(value) => format!("\"{}\"", value).into(),
            Literal::Number(value) => {
                let mut text = value.to_string();
                if text.contains(".0") {
                    text = text.substring(0, text.len() - 2).to_string();
                }
                text.into()
            }
            Literal::Bool(value) => value.to_string().into(),
            Literal::Function(_) => "<fn>".into(), // TODO: Print functions as <fn function_name>
            Literal::NativeFunction(_) => "<native fn>".into(),
        }
    }

    fn error(&self, token: &Token, message: &str) -> RuntimeError {
        error(token, message);
        RuntimeError::Err
    }
}
