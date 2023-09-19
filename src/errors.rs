use std::fmt;

use crate::Token;

#[derive(Clone, Debug)]
pub struct EvaluationError {
    msg: String,
    line: usize,
    column: usize,
}

impl EvaluationError {
    pub fn new(msg: &str, line: usize, column: usize) -> Self {
        Self {
            msg: msg.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "evaluation error: {} at line {} column {}",
            self.msg, self.line, self.column
        )
    }
}

#[derive(Clone, Debug)]
pub struct InterpreterError {
    pub msg: String,
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Clone, Debug)]
pub struct ScanError {
    pub line: usize,
    pub column: usize,
    pub msg: String,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scan error at {}:{}; {}",
            self.line, self.column, self.msg
        )
    }
}

#[derive(Debug, Clone)]
pub enum ExceptionType {
    RuntimeException,
}

impl ToString for ExceptionType {
    fn to_string(&self) -> String {
        match self {
            ExceptionType::RuntimeException => "runtime exception".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParserError {
    exc_type: ExceptionType,
    line: usize,
    column: usize,
    msg: String,
}

impl ParserError {
    pub fn new(msg: &str, token: &Token, exc: ExceptionType) -> Self {
        Self {
            msg: msg.into(),
            line: token.line,
            column: token.column,
            exc_type: exc,
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} at line {} column {}",
            self.exc_type.to_string(),
            self.msg,
            self.line,
            self.column
        )
    }
}
