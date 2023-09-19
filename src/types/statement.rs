use crate::{Expression, Token};

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Variable(Expression),
    Assign(Token, Expression),
    Block(Vec<Statement>),
}
