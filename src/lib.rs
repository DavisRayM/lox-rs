mod analyzers;
pub mod errors;
pub mod repl;
mod types;

use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::PathBuf,
};

use analyzers::*;
use errors::{EvaluationError, InterpreterError};
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

pub struct Interpreter {
    content: String,
    enclosing: Environment,
}

impl Interpreter {
    pub fn new(content: String) -> Self {
        Self {
            content,
            enclosing: Environment::default(),
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            content: fs::read_to_string(path)?,
            enclosing: Environment::default(),
        })
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn interpret(&mut self, strict: bool) -> Result<(), InterpreterError> {
        let scanner =
            Scanner::new(&self.content).map_err(|e| InterpreterError { msg: e.to_string() })?;
        let mut parser = Parser::new(scanner.tokens, strict);
        let statements = parser
            .parse()
            .map_err(|e| InterpreterError { msg: e.to_string() })?;
        for statement in statements {
            let literal = self
                .evaluate_statement(statement)
                .map_err(|e| InterpreterError { msg: e.to_string() })?;
            if let Some(literal) = literal {
                let literal: String = literal.into();
                println!("{}", literal);
            }
        }

        Ok(())
    }
    fn evaluate_statements(&mut self, statements: Vec<Statement>) -> Result<(), EvaluationError> {
        for statement in statements {
            self.evaluate_statement(statement)?;
        }
        Ok(())
    }

    fn evaluate_statement(
        &mut self,
        statement: Statement,
    ) -> Result<Option<Literal>, EvaluationError> {
        match statement {
            Statement::Expression(expr) => Ok(Some(expr.evaluate(&self.enclosing)?)),
            Statement::Block(statements) => {
                self.enclosing.enter_block();
                self.evaluate_statements(statements)?;
                self.enclosing.leave_block();
                Ok(None)
            }
            Statement::Variable(expr) => Ok(Some(expr.evaluate(&self.enclosing)?)),
            Statement::Assign(token, expr) => {
                let name = token.lexeme.to_owned();
                let literal = expr.evaluate(&self.enclosing)?;
                self.enclosing.define(name, literal);
                Ok(None)
            }
        }
    }
}
