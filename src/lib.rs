//! Crate responsible for parsing and compiling
//! the generated AST to Brokkr-bytecode for the
//! Valhalla set theoretic programming language.
#![warn(clippy::all)]
#![allow(clippy::needless_return)]
#![allow(clippy::single_match)]
#![allow(clippy::new_ret_no_self)]

#![feature(stmt_expr_attributes)]

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub const VERSION : (u8, u8, u8) = read_version();

/// Source code sites (location, line, filename, etc.).
pub mod site;

/// Issue messages (warnings, errors, info, etc.).
#[macro_use]
pub mod issue;

/// Syntax submodule, responsible for lexical analysis,
/// parsing and static analysis.
pub mod syntax;

/// Compiler, transforms AST into stack-based bytecode
/// instructions for the Brokkr VM, and marshals the instructions.
pub mod compiler;

pub use syntax::parse_source;

/// Parses the contents of a file with path `filename : &str`.
pub fn parse(filename : &str) -> syntax::ast::Root {
    syntax::parse_file(filename)
}

/// Compile the parse tree.
pub fn compile(root : &syntax::ast::Root) -> compiler::block::LocalBlock {
    let mut code_block = compiler::block::LocalBlock::new("<main>", &root.filename);

    code_block.generate(&root.branches);

    #[cfg(feature="debug")]
    println!("Code Blocks:\n{}", code_block);
    code_block
}

pub fn binary_blob(block : &compiler::block::LocalBlock) -> Vec<u8> {
    compiler::marshal::generate_binary(block)
}

// Set panic message for compiler bugs and issue messages.
use std::panic;
use colored::*;

pub static mut PANIC_MESSAGE : &str = "";

pub fn set_panic() {
    unsafe {
        panic::set_hook(Box::new(|msg| {
            if PANIC_MESSAGE.is_empty() {
                eprintln!("\n{}", "The compiler panicked! This is a bug."
                    .white().bold());
                eprintln!("{} {}\n",
                    ">>>".blue(),
                    msg.to_string().white());
            } else {
                eprintln!(" {} {} {}",
                    "::".white().bold(),
                    "Halt".blue().bold(),
                    PANIC_MESSAGE.white());
                #[cfg(any(feature="loud-panic", feature="debug"))] {
                    eprintln!("{} {}\n",
                        ">>>".blue(),
                        msg.to_string().white());
                }
            }
        }));
    }
}
