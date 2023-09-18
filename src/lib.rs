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

#[cfg(test)]
pub fn get_statement_string(statement: Statement) -> String {
    match statement {
        Statement::Assign(token, expr) => {
            let str_rep: String = expr.evaluate().unwrap().into();
            format!("let {} = {};", token.lexeme, str_rep)
        }
        Statement::Variable(expr) => expr.evaluate().unwrap().into(),
        Statement::Expression(expr) => expr.evaluate().unwrap().into(),
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

#[derive(Debug, Clone, Default)]
pub struct Environment {
    values: HashMap<String, Literal>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    fn new(parent: Environment) -> Self {
        Self {
            values: HashMap::default(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<Literal> {
        match self.values.get(&name).cloned() {
            None => {
                if let Some(parent) = self.parent.as_ref() {
                    parent.get(name)
                } else {
                    None
                }
            }
            Some(val) => Some(val),
        }
    }
}

pub struct Interpreter {
    content: String,
    global_env: Environment,
    current_env: Option<Environment>,
}

impl Interpreter {
    pub fn new(content: String) -> Self {
        Self {
            content,
            global_env: Environment::default(),
            current_env: None,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            content: fs::read_to_string(path)?,
            global_env: Environment::default(),
            current_env: None,
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
            self.evaluate_statement(statement)
                .map_err(|e| InterpreterError { msg: e.to_string() })?;
        }

        println!("{:#?}", self.global_env);
        Ok(())
    }

    fn get_from_env(&self, key: String) -> Option<Literal> {
        if let Some(environment) = self.current_env.as_ref() {
            environment.get(key)
        } else {
            self.global_env.get(key)
        }
    }

    fn evaluate_statements(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<(), expression::EvaluationError> {
        for statement in statements {
            self.evaluate_statement(statement)?;
        }
        Ok(())
    }

    fn evaluate_statement(
        &mut self,
        statement: Statement,
    ) -> Result<Option<Literal>, expression::EvaluationError> {
        match statement {
            Statement::Expression(expr) => Ok(Some(expr.evaluate()?)),
            Statement::Block(statements) => {
                let parent_env = self.current_env.clone();
                match parent_env {
                    Some(parent_env) => {
                        self.current_env = Some(Environment::new(parent_env.clone()));
                        self.evaluate_statements(statements)?;
                        self.current_env = Some(parent_env);
                        Ok(None)
                    }
                    None => {
                        self.evaluate_statements(statements)?;
                        Ok(None)
                    }
                }
            }
            Statement::Variable(expr) => {
                if let Literal::Variable(name) = expr.evaluate()? {
                    println!("Got {}", name);
                    Ok(self.get_from_env(name))
                } else {
                    Ok(None)
                }
            }
            Statement::Assign(token, literal) => {
                let name = token.lexeme;
                self.global_env.define(name, literal.evaluate()?);
                Ok(None)
            }
        }
    }
}
