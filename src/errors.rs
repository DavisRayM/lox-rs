//! Error types for Lox

use std::fmt;

use crate::LocationInfo;

/// An exception/unrecoverable state was reached by the Runner
#[derive(Debug, Clone)]
pub struct RunnerError {
    pub msg: String,
}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runner exception: {}", self.msg)
    }
}

/// Scanner encountered an unexpected token definition
#[derive(Debug, Clone)]
pub(crate) struct ScannerError {
    pub cause: String,
    pub location: LocationInfo,
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.cause, self.location.line, self.location.column
        )
    }
}

/// Parser encountered an error while parsing expressions
#[derive(Debug, Clone)]
pub(crate) struct ParserError {
    pub cause: String,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

/// Runtime error
///
/// Errors encountered during runtime; These usually happen when exceptions
/// are evaluated
#[derive(Debug, Clone)]
pub(crate) struct RuntimeError {
    pub cause: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}
