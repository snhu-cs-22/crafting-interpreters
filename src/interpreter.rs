use substring::Substring;

use super::report;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};

fn error(token: &Token, message: &str) {
    report(token.line, &format!(" at \"{}\"", token.lexeme), message);
}

pub struct RuntimeError;

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&self, expression: &Expr) -> Result<(), RuntimeError> {
        println!("{}", self.stringify(self.evaluate(expression)?));
        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<Literal, RuntimeError> {
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
                    _ => Err(self.error(operator, "Unreachable. This is a bug.")),
                }
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            // TODO: Remove clone
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Unary(operator, right) => {
                let right = self.evaluate(right)?;

                match operator.r#type {
                    TokenType::Minus => {
                        let right = self.check_number_operand(operator, right)?;
                        Ok(Literal::Number(-right))
                    }
                    TokenType::Bang => Ok(Literal::Bool(!self.is_truthy(right))),
                    _ => todo!(),
                }
            }
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: Literal,
        right: Literal,
    ) -> Result<(f64, f64), RuntimeError> {
        match (left, right) {
            (Literal::Number(left), Literal::Number(right)) => Ok((left, right)),
            (_, _) => Err(self.error(operator, "Operands must be numbers.")),
        }
    }

    fn check_number_operand(
        &self,
        operator: &Token,
        operand: Literal,
    ) -> Result<f64, RuntimeError> {
        match operand {
            Literal::Number(value) => Ok(value),
            _ => Err(self.error(operator, "Operand must be a number.")),
        }
    }

    fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Literal::Bool(value) => value,
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
            Literal::Nil | Literal::None => "nil".into(),
            Literal::String(value) => format!("\"{}\"", value).into(),
            Literal::Number(value) => {
                let mut text = value.to_string();
                if text.contains(".0") {
                    text = text.substring(0, text.len() - 2).to_string();
                }
                text.into()
            }
            Literal::Bool(value) => value.to_string().into(),
        }
    }

    fn error(&self, token: &Token, message: &str) -> RuntimeError {
        error(token, message);
        RuntimeError
    }
}
