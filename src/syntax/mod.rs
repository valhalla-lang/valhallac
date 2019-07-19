//! Syntax, parsing and analysis.

/// location manages line and column location of
/// lexical tokens as well as their span.
mod location;

/// Provides token classes and methods
mod token;

/// Abstract Syntax Tree nodes and methods.
mod ast;

/// Dealing with associativity and precedence.
mod operators;

/// Error messages.
#[macro_use]
mod err;

/// Lexer splits code up into a token-stream
/// of relevant lexical tokens, making the
/// parsing step a lot easier.
pub mod lexer;

/// Converts a token-stream into a nested AST.
pub mod parser;

use std::fs;
use token::ShowStream;

/// Parses a given file, calling various methods from
/// the `syntax` sub-module.
pub fn parse_file(filename : &'static str) {
    let code = fs::read_to_string(filename)
        .expect("Could not open file for reading.");
    println!("Code:\n{}\n", code);

    let stream = lexer::lex(&code);
    println!("Stream:\n{}\n", stream.to_string());

    let tree = parser::parse(stream, filename);
    println!("AST:\n{}\n", tree);
}