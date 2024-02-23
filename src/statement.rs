use crate::{expression::Expression, token::Token};

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(Expression),
    Var(Token, Option<Expression>),
    Block(Vec<Statement>),
}
