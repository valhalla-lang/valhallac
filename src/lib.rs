//! Crate responsible for parsing and compiling
//! the generated AST to Brokkr-bytecode for the
//! Valhalla set theoretic programming language.

/// Syntax submodule, responsible for lexical analysis,
/// parsing and static analysis.
mod syntax;
mod compiler;

pub fn parse(filename : &str) {    
    syntax::parse_file(filename);
    let mut code_block = compiler::block::LocalBlock::new();
    code_block.generate(syntax::ast::NumNode::new(3.14));
    code_block.generate(syntax::ast::NumNode::new(34));
    code_block.generate(syntax::ast::NumNode::new(3.14));
    println!("Code Block:\n{:#?}", code_block)
}

