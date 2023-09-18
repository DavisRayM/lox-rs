pub mod errors;
pub mod expression;
pub mod literal;
pub mod parser;
pub mod repl;
pub mod scanner;
pub mod token;

use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::PathBuf,
};

use errors::InterpreterError;
use literal::Literal;
use parser::{Parser, Statement};
pub use repl::{run_file, run_prompt};
use scanner::Scanner;

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    fn new() -> Self {
        Self {
            values: HashMap::default(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<Literal> {
        self.values.get(&name).cloned()
    }
}

pub struct Interpreter {
    content: String,
    env: Environment,
}

impl Interpreter {
    pub fn new(content: String) -> Self {
        Self {
            content,
            env: Environment::new(),
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            content: fs::read_to_string(path)?,
            env: Environment::new(),
        })
    }

    pub fn interpret(&mut self) -> Result<(), InterpreterError> {
        let scanner =
            Scanner::new(&self.content).map_err(|e| InterpreterError { msg: e.to_string() })?;
        let mut parser = Parser::new(scanner.tokens);
        let statements = parser
            .parse()
            .map_err(|e| InterpreterError { msg: e.to_string() })?;
        for statement in statements {
            self.evaluate_statement(statement)
                .map_err(|e| InterpreterError { msg: e.to_string() })?;
        }
        Ok(())
    }

    fn evaluate_statement(
        &mut self,
        statement: Statement,
    ) -> Result<Option<Literal>, expression::EvaluationError> {
        match statement {
            Statement::Expression(expr) => Ok(Some(expr.evaluate()?)),
            Statement::Variable(expr) => {
                if let Literal::Variable(name) = expr.evaluate()? {
                    Ok(self.env.get(name))
                } else {
                    Ok(None)
                }
            }
            Statement::Assign(token, literal) => {
                let name = token.lexeme;
                self.env.define(name, literal.evaluate()?);
                Ok(None)
            }
        }
    }
}
