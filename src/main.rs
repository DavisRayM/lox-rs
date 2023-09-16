use lox::{run_file, run_prompt};
use std::{error::Error, process::exit};

#[cfg(target_os = "windows")]
const USAGE: &str = "
USAGE:
    lox.exe <script.lx>
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
USAGE:
    lox <script.lx>
";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        println!("{}", USAGE);
        exit(1);
    } else if args.len() == 1 {
        run_prompt().unwrap();
    } else {
        run_file(&args[1]).unwrap();
    }

    Ok(())
}
