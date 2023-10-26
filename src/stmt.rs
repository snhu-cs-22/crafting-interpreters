use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Var(Token, Option<Box<Expr>>),
}
