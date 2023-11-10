mod environment;
mod expr;
mod function;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;

use std::fs;
use std::io::prelude::*;
use std::io::{self, BufReader};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

pub fn run_file(path: &str) {
    let bytes = fs::read_to_string(path).unwrap();
    run(&bytes);
}

// TODO: Fix this
pub fn run_prompt() {
    let input = io::stdin();
    let mut reader = BufReader::new(input);

    println!("Lox Interactive REPL\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line);
        run(&line);
    }
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    let mut interpreter = Interpreter::new();

    interpreter.interpret(&statements);
}

fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error{location}: {message}");
}
