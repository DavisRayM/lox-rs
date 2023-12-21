use lox_rs::{errors::RunnerError, Runner};
use std::env;

fn main() -> Result<(), RunnerError> {
    let mut args = env::args().collect::<Vec<String>>();
    let mut source: Option<String> = None;

    if args.len() == 2 {
        source = args.pop();
    }

    Runner::new(source)?.run()?;

    Ok(())
}
