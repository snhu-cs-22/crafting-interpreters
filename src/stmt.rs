use crate::expr::Expr;
use crate::token::Token;

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Var(Token, Option<Box<Expr>>),
    While(Box<Expr>, Box<Stmt>),
}
