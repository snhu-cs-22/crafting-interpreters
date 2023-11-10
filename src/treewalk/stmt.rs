use crate::treewalk::expr::Expr;
use crate::treewalk::token::Token;

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
