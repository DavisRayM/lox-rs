use crate::{expression::Expression, token::Token};

#[derive(Debug, Clone)]
pub enum Statement {
    // expr
    Expr(Expression),
    // print (expr)
    Print(Expression),
    // If (condition) (then statement) (else statement)?
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    // Var (name) (expr)?
    Var(Token, Option<Expression>),
    // (statement *)
    Block(Vec<Statement>),
}
