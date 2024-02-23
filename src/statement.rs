use crate::{expression::Expression, token::Token};

pub enum Statement {
    Expr(Expression),
    Var(Token, Option<Expression>),
}
