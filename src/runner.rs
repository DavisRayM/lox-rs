use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use crate::errors::RunnerError;

/// Lox interpreter runner
pub struct Runner {
    source: Option<PathBuf>,
}

impl Runner {
    /// Returns a new runner
    ///
    /// # Arguments
    ///
    /// * `source` - An optional string that dictates whether the runner will
    ///              process a file or start a REPL session
    pub fn new(source: Option<String>) -> Result<Self, RunnerError> {
        let mut path: Option<PathBuf> = None;

        if let Some(s) = source {
            let p = PathBuf::from(s);

            if !p.exists() {
                return Err(RunnerError {
                    msg: format!("no source file found at {}", p.to_str().unwrap()),
                });
            };

            path = Some(p);
        }

        Ok(Runner { source: path })
    }

    /// Starts the runner process loop
    pub fn run(&mut self) -> Result<(), RunnerError> {
        let source = self.source.take();

        match source {
            Some(source) => {
                let content = fs::read_to_string(source).map_err(|_| RunnerError {
                    msg: "failed to read file content".to_string(),
                })?;
                self._run(&content)
            }
            None => self._run_repl(),
        }
    }

    fn _run(&mut self, content: &str) -> Result<(), RunnerError> {
        todo!()
    }

    fn _run_repl(&mut self) -> Result<(), RunnerError> {
        let mut expr: String = String::new();
        loop {
            print!("> ");
            io::stdout().flush().map_err(|_| RunnerError {
                msg: String::from("failed to write to output interface"),
            })?;

            io::stdin().read_line(&mut expr).map_err(|_| RunnerError {
                msg: String::from("failed to parse inputted prompt"),
            })?;

            if expr == "\n" {
                break Ok(());
            }

            self._run(&expr)?;

            expr.clear();
        }
    }
}
