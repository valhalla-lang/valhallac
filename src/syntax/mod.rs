//! Syntax, parsing and analysis.

/// location manages line and column location of
/// lexical tokens as well as their span.
pub mod location;

/// Provides token classes and methods
pub mod token;

/// Abstract Syntax Tree nodes and methods.
pub mod ast;

/// Dealing with associativity and precedence.
pub mod operators;

/// Lexer splits code up into a token-stream
/// of relevant lexical tokens, making the
/// parsing step a lot easier.
pub mod lexer;

/// Converts a token-stream into a nested AST.
pub mod parser;

/// Tree static analysis.
#[macro_use]
pub mod analysis;

use std::fs;
use std::collections::HashSet;

#[cfg(feature="debug")]
use token::ShowStream;

pub fn parse_source(code : &str, filename : &str) -> ast::Root {
    // First lex:
    #[cfg(feature="debug")]
    println!("Code:\n{}\n", code);

    let stream = lexer::lex(&code, filename);

    #[cfg(feature="debug")]
    println!("Stream:\n{}\n", stream.to_string());

    let mut tree = parser::parse(stream, filename);

    #[allow(unused_variables)]
    let transformations = transformations![
        TYPE_RESOLUTION,
        CONSTANT_FOLDING
    ];

    // No optimisations in debug.
    #[cfg(feature="debug")]
    let transformations = transformations![
        TYPE_RESOLUTION
    ];

    analysis::replace(&mut tree, transformations);

    #[cfg(feature="debug")]
    println!("AST:\n{}\n", tree);

    tree

}

/// Parses a given file, calling various methods from
/// the `syntax` sub-module.
pub fn parse_file(filename : &str) -> ast::Root {
    let code = fs::read_to_string(filename)
        .expect("Could not open file for reading.");
    parse_source(&code, filename)
}


