//! Crate responsible for parsing and compiling
//! the generated AST to Brokkr-bytecode for the
//! Valhalla set theoretic programming language.

const VERSION : [u8; 3] = [0, 0, 1];

/// Error messages.
#[macro_use]
pub mod err;

/// Syntax submodule, responsible for lexical analysis,
/// parsing and static analysis.
pub mod syntax;

/// Compiler, transforms AST into stack-based bytecode
/// instructions for the Brokkr VM, and marshals the instructions.
pub mod compiler;

pub fn parse(filename : &str) -> syntax::ast::Root {
    syntax::parse_file(filename)
}

pub fn compile<'a>(root : &'a syntax::ast::Root) -> compiler::block::LocalBlock<'a> {
    let mut code_block = compiler::block::LocalBlock::new("<main>", &root.filename);

    code_block.generate(&root.branches);
    println!("Code Blocks:\n{}", code_block);
    code_block
}

pub fn binary_blob(block : &compiler::block::LocalBlock) -> Vec<u8> {
    compiler::marshal::generate_binary(block)
}
