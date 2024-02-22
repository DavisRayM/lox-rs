use crate::{
    errors::RuntimeError,
    expression::Expression,
    interpreter::{Action, ActionBuilder},
};

pub enum Statement {
    Expr(Expression),
}

impl Statement {
    pub fn eval(&self) -> Result<Action, RuntimeError> {
        let mut action = ActionBuilder::default();
        match self {
            Self::Expr(expr) => {
                let val = expr.eval()?;
                action = action.value(val);
            }
        }

        Ok(action.build())
    }
}
