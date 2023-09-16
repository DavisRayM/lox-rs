use std::fmt;

#[derive(Clone, Debug)]
pub struct InterpreterError {
    line: u16,
    message: String,
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line [{}]: {}", self.line, self.message)
    }
}
