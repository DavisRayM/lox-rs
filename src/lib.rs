mod environment;
pub mod errors;
mod expression;
pub mod interpreter;
pub mod parser;
mod runner;
mod scanner;
mod statement;
pub mod token;
mod token_type;
pub use expression::Expression;
pub use interpreter::Interpreter;
pub use parser::Parser;
pub use runner::Runner;
pub use scanner::Scanner;
pub use statement::Statement;
pub use token::{Literal, Token};
pub use token_type::TokenType;

#[derive(Debug, Clone, Copy)]
pub struct LocationInfo {
    column: usize,
    line: usize,
    len: usize,
}

impl PartialEq for LocationInfo {
    fn eq(&self, other: &Self) -> bool {
        self.column == other.column && self.line == other.line && self.len == other.len
    }
}
