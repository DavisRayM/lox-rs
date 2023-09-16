use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::errors::InterpreterError;
use crate::scanner::Scanner;

pub type InterpreterResult<T> = Result<T, InterpreterError>;

pub fn run_prompt() -> InterpreterResult<()> {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut statement = String::new();
        io::stdin()
            .read_line(&mut statement)
            .expect("failed to read in statement");

        let scanner = match Scanner::new(statement) {
            Ok(scanner) => scanner,
            Err(e) => {
                return Err(InterpreterError {
                    line: e.line,
                    column: e.column,
                    msg: e.msg,
                });
            }
        };

        println!("{:#?}", scanner.tokens);
        if scanner.tokens.is_empty() {
            break;
        }
    }

    Ok(())
}

pub fn run_file(path: &str) -> InterpreterResult<()> {
    let path = Path::new(&path);
    let contents = fs::read_to_string(path.to_str().expect("failed to parse script location"))
        .expect("failed to read script file");
    let scanner = match Scanner::new(contents) {
        Ok(scanner) => scanner,
        Err(e) => {
            return Err(InterpreterError {
                line: e.line,
                column: e.column,
                msg: e.msg,
            });
        }
    };

    println!("{:#?}", scanner.tokens);
    Ok(())
}
