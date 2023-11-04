use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Function(Token, Vec<Token>, Vec<Stmt>),
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Return(Token, Box<Expr>),
    Var(Token, Option<Box<Expr>>),
    While(Box<Expr>, Box<Stmt>),
}
