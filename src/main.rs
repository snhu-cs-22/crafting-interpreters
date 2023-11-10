use std::process::ExitCode;

use crafting_interpreters::{bytecode, treewalk};

fn main() -> ExitCode {
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        1 => treewalk::run_prompt(),
        2 => treewalk::run_file(&args[1]),
        _ => {
            println!("Usage: jlox [script]");
            return ExitCode::from(64);
        }
    }

    ExitCode::SUCCESS
}
