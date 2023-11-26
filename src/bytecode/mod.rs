pub mod chunk;
pub mod compiler;
pub mod scanner;
pub mod value;
pub mod vm;

use std::fs;
use std::io::prelude::*;
use std::io::{self, BufReader};

use vm::VM;

pub fn repl(vm: &mut VM) {
    let input = io::stdin();
    let mut reader = BufReader::new(input);

    println!("Lox Interactive REPL\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line);

        if line.clone().trim().is_empty() {
            println!();
            println!("Quitting REPL...");
            println!();
            break;
        }
        vm.interpret(&line);
    }
}

pub fn run_file(vm: &mut VM, path: &str) -> io::Result<vm::InterpretResult> {
    let source = fs::read_to_string(path)?;
    Ok(vm.interpret(&source))
}
