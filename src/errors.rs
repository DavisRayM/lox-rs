use std::fmt;

#[derive(Clone, Debug)]
pub struct InterpreterError {
    pub line: usize,
    pub column: usize,
    pub msg: String,
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Line [{}] Column [{}]: {}",
            self.line, self.column, self.msg
        )
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
