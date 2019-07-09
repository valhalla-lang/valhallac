mod location;
mod token;

pub mod lexer;
pub mod parser;

use std::fs;
use token::ShowStream;

pub fn parse_file(filename : &str) {
    let code = fs::read_to_string(filename)
        .expect("Could not open file for reading.");
    println!("Code:\n{}\n", code);

    let stream = lexer::lex(code);
    println!("Stream:\n{}\n", stream.to_string());
}