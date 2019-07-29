use std::fmt;
use std::collections::HashMap;

use super::super::err;

use super::super::syntax;
use syntax::ast;

use super::element;
use super::instructions;

use element::{Element, Symbol};
use instructions::{Instr, Operators};

use super::internal_functions;

fn append_unique<'a>(v : &mut Vec<Element<'a>>, e : Element<'a>) -> usize {
    let index = v.iter().position(|c| c == &e);
    if index.is_none() { v.push(e.clone()); }
    index.unwrap_or(v.len() - 1)
}

pub fn numerics_to_element<'a>(num : &ast::Numerics) -> Element<'a> {
    match num {
        ast::Numerics::Natural(n) => Element::ENatural(*n),
        ast::Numerics::Integer(n) => Element::EInteger(*n),
        ast::Numerics::Real(n)    => Element::EReal(*n)
    }
}

#[derive(Clone, PartialEq)]
pub struct LocalBlock<'a> {
    pub name : &'a str,
    filename : &'a str,
    constants : Vec<Element<'a>>,
    locals : Vec<Element<'a>>,
    instructions : Vec<Instr>,

    // Used only for compilation:
    locals_map : HashMap<String, usize>,
    current_line : usize,
}

impl<'a> LocalBlock<'a> {
    pub fn new(name : &'a str, filename : &'a str) -> Self {
        LocalBlock {
            name,
            filename,
            constants: vec![],
            locals: vec![],
            instructions: vec![],

            locals_map: HashMap::new(),
            current_line: 0,
        }
    }

    fn push_const_instr(&mut self, e : Element<'a>) {
        let index = append_unique(&mut self.constants, e);
        self.instructions.push(Instr::Operator(Operators::PUSH_CONST as u8));
        self.instructions.push(Instr::Operand(index as u16));
    }

    fn emit(&mut self, node : &'a ast::Nodes) {
        match node {
            ast::Nodes::Line(line_node) => {
                self.current_line = line_node.line;
                self.instructions.push(Instr::Operator(Operators::SET_LINE as u8));
                self.instructions.push(Instr::Operand(self.current_line as u16));
            }
            ast::Nodes::Ident(ident_node) => {
                let s = &ident_node.value;
                if !self.locals_map.contains_key(s) {
                    issue!(err::Types::CompError, self.filename, err::NO_TOKEN, self.current_line,
                        "Trying to use unbound local variable `{}`.", s);
                }
            },
            ast::Nodes::Num(num_node) => {
                self.push_const_instr(numerics_to_element(&num_node.value));
            },
            ast::Nodes::Str(str_node) => {
                self.push_const_instr(Element::EString(&str_node.value));
            },
            ast::Nodes::Sym(sym_node) => {
                self.push_const_instr(Element::ESymbol(Symbol::new(&sym_node.value)));
            },
            ast::Nodes::Call(call_node) => {
                if call_node.is_binary() {
                    let ident = call_node.callee.call().unwrap().callee.ident().unwrap();

                    let args = vec![
                        &call_node.operands[0],
                        &call_node.callee.call().unwrap().operands[0],
                    ];

                    let inop = internal_functions::get_internal_op(&ident.value, Some(&args));
                    if let Some(op) = inop {
                        self.emit(args[0]);
                        self.emit(args[1]);
                        self.instructions.push(op)
                    }
                }
            },
            _ => ()
        };
    }

    pub fn generate(&mut self, nodes : &'a Vec<ast::Nodes>) {
        for node in nodes {
            self.emit(node);
        }
    }
}

impl<'a> fmt::Display for LocalBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "===Constants===============\n")?;
        for (i, c) in self.constants.iter().enumerate() {
            write!(f, "{: >3} |  {} |\n", i, c)?;
        }
        write!(f, "===Bytecodes===============\n")?;
        for inst in &self.instructions {
            write!(f, "{}", inst)?;
        }
        write!(f, "")
    }
}