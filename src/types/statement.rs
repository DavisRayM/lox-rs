use crate::{Expression, Token};

#[derive(Debug, Clone)]
/// A statement is an action/set of instructions for the interpreted to
/// evaluate and execute
pub enum Statement {
    Expression(Expression),
    Variable(Expression),
    Assign(Token, Expression),
    Block(Vec<Statement>),
}
