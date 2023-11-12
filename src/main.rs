use std::process::ExitCode;

use crafting_interpreters::{bytecode, treewalk};
use bytecode::{repl, run_file};
use bytecode::vm::{VM, InterpretResult};

fn main() -> ExitCode {
    let mut vm = VM::new();

    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        1 => repl(&mut vm),
        2 => match run_file(&mut vm, &args[1]) {
            Ok(InterpretResult::CompileError) => return ExitCode::from(65),
            Ok(InterpretResult::RuntimeError) => return ExitCode::from(70),
            Err(_) => {
                println!("Could not open file \"{}\".", &args[1]);
                return ExitCode::from(74);
            }
            _ => (),
        },
        _ => {
            println!("Usage: jlox [script]");
            return ExitCode::from(64);
        }
    }

    ExitCode::SUCCESS
}
