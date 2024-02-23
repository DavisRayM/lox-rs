mod environment;
pub mod errors;
mod expression;
mod interpreter;
mod parser;
mod runner;
mod scanner;
mod statement;
mod token;
mod token_type;
pub use runner::Runner;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocationInfo {
    column: usize,
    line: usize,
    len: usize,
}

impl PartialEq for LocationInfo {
    fn eq(&self, other: &Self) -> bool {
        self.column == other.column && self.line == other.line && self.len == other.len
    }
}
