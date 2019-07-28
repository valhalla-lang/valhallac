//! Syntax, parsing and analysis.

/// location manages line and column location of
/// lexical tokens as well as their span.
pub mod location;

/// Provides token classes and methods
pub mod token;

/// Abstract Syntax Tree nodes and methods.
pub mod ast;

/// Dealing with associativity and precedence.
mod operators;

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
pub fn parse_file(filename : &str) -> ast::Root {
    let code = fs::read_to_string(filename)
        .expect("Could not open file for reading.");
    println!("Code:\n{}\n", code);

    let stream = lexer::lex(&code);
    println!("Stream:\n{}\n", stream.to_string());

    let tree = parser::parse(stream, filename);
    println!("AST:\n{}\n", tree);
    tree
}