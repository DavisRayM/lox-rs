mod analyzers;
pub mod errors;
pub mod interpreter;
pub mod repl;
mod types;

use std::collections::HashMap;

use errors::{EvaluationError, InterpreterError};
pub use interpreter::Interpreter;
pub use repl::{run_file, run_prompt};
use types::*;

#[cfg(test)]
pub fn get_statement_string(statement: Statement) -> String {
    let environment = Environment::default();
    match statement {
        Statement::Assign(token, expr) => {
            let str_rep: String = expr.evaluate(&environment).unwrap().into();
            format!("let {} = {};", token.lexeme, str_rep)
        }
        Statement::Variable(expr) => expr.evaluate(&environment).unwrap().into(),
        Statement::Expression(expr) => expr.evaluate(&environment).unwrap().into(),
        Statement::Block(statements) => {
            let mut actual = String::new();
            for statement in statements {
                actual.push_str(&get_statement_string(statement));
                actual.push_str("\n");
            }
            actual
        }
    }
}

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Literal>>,
    depth: usize,
}

impl Default for Environment {
    fn default() -> Self {
        let scopes = vec![HashMap::new()];
        Self { scopes, depth: 0 }
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: Literal) {
        self.scopes[self.depth].insert(name, value);
    }

    pub fn enter_block(&mut self) {
        self.depth += 1;
        self.scopes[self.depth] = HashMap::new();
    }

    pub fn leave_block(&mut self) {
        self.scopes.remove(self.depth);
        self.depth -= 1;
    }

    pub fn get(&self, name: String) -> Option<Literal> {
        for i in 0..=self.depth {
            let option = self.scopes[self.depth - i].get(&name);
            if let Some(option) = option {
                return Some(option.clone());
            }
        }
        None
    }
}
