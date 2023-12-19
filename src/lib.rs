pub mod errors;
mod runner;
mod scanner;
mod token;
mod token_type;
pub use runner::Runner;

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn runner_creation() {
        let runner = Runner::new(None);
    }
}
