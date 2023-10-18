use std::process::ExitCode;

use crafting_interpreters::{run_file, run_prompt};

fn main() -> ExitCode {
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: jlox [script]");
            return ExitCode::from(64);
        }
    }

    ExitCode::SUCCESS
}
