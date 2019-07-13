//! Crate responsible for parsing and compiling
//! the generated AST to Brokkr-bytecode for the
//! Valhalla set theoretic programming language.

/// Syntax submodule, responsible for lexical analysis,
/// parsing and static analysis.
mod syntax;


fn main() {
    println!("\nTill Valhalla!\n");
    
    syntax::parse_file("./test.vh");
}

