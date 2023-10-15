mod scanner;
mod token;

use std::fs;
use std::io::prelude::*;
use std::io::{self, BufReader};

use scanner::Scanner;

pub fn run_file(path: &str) {
    // TODO: figure out why I can't read text files
    let bytes = fs::read_to_string(path).unwrap();
    run(&bytes);
}

pub fn run_prompt() {
    let input = io::stdin();
    let mut reader = BufReader::new(input);

    println!("Lox Interactive REPL\n");

    loop {
        print!("> ");
        let mut line = String::new();
        reader.read_line(&mut line);
        run(&line);
    }
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    // For now, just print the tokens.
    for token in tokens {
        println!("{}", token);
    }
}

fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error{location}: {message}");
}
