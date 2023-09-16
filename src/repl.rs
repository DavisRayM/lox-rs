use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::errors::InterpreterError;

pub type InterpreterResult<T> = Result<T, InterpreterError>;

pub fn run_prompt() -> InterpreterResult<()> {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut statement = String::new();
        io::stdin()
            .read_line(&mut statement)
            .expect("failed to read in statement");

        if statement.trim() == "exit" {
            break;
        }
    }

    Ok(())
}

pub fn run_file(path: &str) -> InterpreterResult<()> {
    let path = Path::new(&path);
    let contents = fs::read_to_string(path.to_str().expect("failed to parse script location"))
        .expect("failed to read script file");
    println!("{:?}", contents);
    Ok(())
}
