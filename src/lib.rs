mod expr;
mod parser;
mod scanner;
mod token;

use std::fs;
use std::io::prelude::*;
use std::io::{self, BufReader};

use parser::Parser;
use scanner::Scanner;

pub fn run_file(path: &str) {
    let bytes = fs::read_to_string(path).unwrap();
    run(&bytes);
}

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
    let expression = parser.parse();

    println!("{:#?}", expression);
}

fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error{location}: {message}");
}
