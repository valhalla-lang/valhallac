use super::super::syntax;
use syntax::ast;

use super::element;
use super::instructions;

use element::Element;
use instructions::{Instr, Operators};

#[derive(Debug)]
pub struct LocalBlock<'a> {
    constants : Vec<Element<'a>>,
    locals : Vec<Element<'a>>,
    instructions : Vec<Instr>
}

impl<'a> LocalBlock<'a> {
    pub fn new() -> Self {
        LocalBlock {
            constants: vec![],
            locals: vec![],
            instructions: vec![]
        }
    }

    pub fn generate(&mut self, node : ast::Nodes) {
        match node {
            ast::Nodes::Num(num_node) => {
                let elem = match num_node.value {
                    ast::Numerics::Natural(n) => Element::ENatural(n),
                    ast::Numerics::Integer(n) => Element::EInteger(n),
                    ast::Numerics::Real(n)    => Element::EReal(n)
                };
                let index = self.append_const(elem);
                self.instructions.push(Instr::Operator(Operators::PUSH_CONST as u8));
                self.instructions.push(Instr::Operand(index as u16))
            },
            _ => ()
        };
    }

    fn append_const(&mut self, e : Element<'a>) -> usize {
        let index = self.constants.iter().position(|c| c == &e);
        if index.is_none() { self.constants.push(e); }
        index.unwrap_or(self.constants.len() - 1)
    }
}
