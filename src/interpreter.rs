use std::io;

use crate::{errors::RuntimeError, statement::Statement, token::Literal};

#[derive(Debug, PartialEq, Clone)]
pub enum ActionType {
    Store,
    Display,
    None,
}

#[derive(Debug, Clone)]
pub(crate) struct Action {
    pub _type: ActionType,
    pub value: Literal,
}

pub struct ActionBuilder {
    action: Action,
}

pub struct Interpreter<T: io::Write> {
    out: T,
}

impl<T: io::Write> Interpreter<T> {
    pub fn new(out: T) -> Self {
        Self { out }
    }

    pub fn evaluate(&mut self, stmt: &Statement) -> Result<(), RuntimeError> {
        let action = stmt.eval()?;

        match action._type {
            ActionType::None => (),
            ActionType::Display => {
                writeln!(&mut self.out, "{}", action.value);
            }
            ActionType::Store => (),
        }

        Ok(())
    }
}

impl ActionBuilder {
    pub fn default() -> Self {
        Self {
            action: Action {
                _type: ActionType::None,
                value: Literal::None,
            },
        }
    }

    pub fn action_type(mut self, action_type: ActionType) -> Self {
        self.action._type = action_type;
        self
    }

    pub fn value(mut self, val: Literal) -> Self {
        self.action.value = val;
        self
    }

    pub fn build(self) -> Action {
        self.action
    }
}
