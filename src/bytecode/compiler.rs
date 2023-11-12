use super::scanner::{Scanner, TokenType};

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);
    let mut line = 0;
    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:04} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:02} '{}'", token.r#type as u8, token.slice);

        if token.r#type == TokenType::Eof {
            break;
        }
    }
}
