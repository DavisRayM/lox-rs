use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use crate::{errors::RunnerError, Interpreter, Parser, Scanner};

/// Lox interpreter runner
pub struct Runner {
    source: Option<PathBuf>,
    interpreter: Interpreter<io::Stdout>,
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

        Ok(Runner {
            source: path,
            interpreter: Interpreter::new(io::stdout()),
        })
    }

    /// Starts the runner process loop
    pub fn run(&mut self) -> Result<(), RunnerError> {
        let source = self.source.take();

        match source {
            Some(source) => {
                let content = fs::read_to_string(source).map_err(|_| RunnerError {
                    msg: "failed to read file content".to_string(),
                })?;
                self._run(&content, true)
            }
            None => self._run_repl(),
        }
    }

    fn _run(&mut self, content: &str, strict: bool) -> Result<(), RunnerError> {
        let s = Scanner::new(content.to_string());
        match s.run() {
            Ok(symbols) => {
                let mut p = Parser::new(symbols, io::stdout(), strict);

                self.interpreter
                    .interpret(p.parse())
                    .map_err(|e| RunnerError {
                        msg: format!("parse error: {}", e),
                    })
            }
            Err(e) => Err(RunnerError {
                msg: format!("scan error: {}", e),
            }),
        }
    }

    fn _run_repl(&mut self) -> Result<(), RunnerError> {
        let mut expr: String = String::new();
        self.interpreter.debug(true);
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

            if let Err(err) = self._run(&expr, false) {
                println!("{}", err);
            }

            expr.clear();
        }
    }
}
