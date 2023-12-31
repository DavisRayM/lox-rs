use std::io;
use std::io::Write;

use crate::errors::InterpreterError;
use crate::Interpreter;

pub type InterpreterResult<T> = Result<T, InterpreterError>;

pub fn run_prompt() -> InterpreterResult<()> {
    let mut interpreter = Interpreter::new("".into());
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut statement = String::new();
        io::stdin()
            .read_line(&mut statement)
            .expect("failed to read in statement");

        if statement.len() <= 1 {
            break;
        }
        interpreter.set_content(statement);
        interpreter.interpret(false)?
    }

    Ok(())
}

pub fn run_file(path: &str) -> InterpreterResult<()> {
    let mut interpreter =
        Interpreter::from_file(path.into()).map_err(|e| InterpreterError { msg: e.to_string() })?;
    interpreter.interpret(true)?;
    Ok(())
}
