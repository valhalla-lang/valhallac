//! Crate responsible for parsing and compiling
//! the generated AST to Brokkr-bytecode for the
//! Valhalla set theoretic programming language.

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

/// Error messages.
#[macro_use]
mod err;

/// Syntax submodule, responsible for lexical analysis,
/// parsing and static analysis.
mod syntax;

/// Compiler, transforms AST into stack-based bytecode
/// instructions for the Brokkr VM, and marshals the instructions.
mod compiler;

pub fn parse(filename : &str) {
    let root = syntax::parse_file(filename);
    let mut code_block = compiler::block::LocalBlock::new();


    code_block.generate(&root.branches);
    println!("Code Block:\n{}", code_block)
}

