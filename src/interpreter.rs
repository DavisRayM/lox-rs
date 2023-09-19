use crate::analyzers::{Parser, Scanner};
use crate::{Environment, EvaluationError, InterpreterError, Literal, Statement};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

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
