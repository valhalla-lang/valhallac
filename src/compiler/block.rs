use std::fmt;
use std::collections::{HashMap, VecDeque};

use super::super::err;

use super::super::syntax;
use syntax::ast;

use super::element;
use super::instructions;

use element::{Element, Symbol};
use instructions::{Instr, Operators};

use super::internal_functions;

fn append_unique<'a, T : Clone + PartialEq>(v : &mut Vec<T>, e : T) -> usize {
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

#[derive(Clone)]
struct IdentTypePair<'a>(String, &'a ast::Nodes);

#[derive(Clone)]
pub struct LocalBlock<'a> {
    pub name : &'a str,
    filename : &'a str,
    constants : Vec<Element<'a>>,
    instructions : Vec<Instr>,
    globals : Vec<String>,
    pub operand_type : ast::StaticTypes,
    pub return_type  : ast::StaticTypes,

    // Used only for compilation:
    locals_map : HashMap<String, u16>,
    types_to_check : VecDeque<IdentTypePair<'a>>,
    current_line : usize,
}

impl<'a> PartialEq for LocalBlock<'a> {
    fn eq(&self, other : &Self) -> bool {
        self.constants == other.constants
        && self.instructions == other.instructions
    }
}

impl<'a> LocalBlock<'a> {
    pub fn new(name : &'a str, filename : &'a str) -> Self {
        LocalBlock {
            name,
            filename,
            constants: vec![],
            instructions: vec![],
            globals: vec![],
            operand_type: ast::StaticTypes::TUnknown,
            return_type:  ast::StaticTypes::TUnknown,

            locals_map: HashMap::new(),
            types_to_check: VecDeque::new(),
            current_line: 0,
        }
    }

    fn push_const_instr(&mut self, e : Element<'a>) {
        let index = append_unique(&mut self.constants, e);
        self.instructions.push(Instr::Operator(Operators::PUSH_CONST as u8));
        self.instructions.push(Instr::Operand(index as u16));
    }

    fn ident_assignment(&mut self, left : &ast::IdentNode, right : &'a ast::Nodes) {
        if self.types_to_check.is_empty() {
            issue!(err::Types::TypeError, self.filename, err::NO_TOKEN, self.current_line,
                "You must state what set `{}' is a member of. No type-annotation found.", left.value);
        }
        if self.locals_map.contains_key(&left.value) {
            issue!(err::Types::CompError, self.filename, err::NO_TOKEN, self.current_line,
                "Cannot mutate value of `{}', as is already bound.", left.value);
        }
        let index = self.locals_map.len() as u16;
        self.locals_map.insert(left.value.to_owned(), index);

        self.emit(right);
        if left.static_type == ast::StaticTypes::TUnknown
        || left.static_type != right.yield_type() {
            self.instructions.push(Instr::Operator(Operators::DUP as u8));
            let type_node = self.types_to_check.pop_front().unwrap().1;
            self.emit(type_node);
            self.instructions.push(Instr::Operator(Operators::CHECK_TYPE as u8));
        } else {  // Otherwise just pop, type was already checked statically so
                  //  its of no use to include in the compiled program,
                  //  as no dynamic checking is needed.
            self.types_to_check.pop_front();
        }
        self.instructions.push(Instr::Operator(Operators::STORE_LOCAL as u8));
        self.instructions.push(Instr::Operand(index));
    }

    fn annotation(&mut self, left : &ast::IdentNode, right : &'a ast::Nodes) {
        self.types_to_check.push_back(IdentTypePair(left.value.to_owned(), right));
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
                    self.instructions.push(Instr::Operator(Operators::PUSH_SUPER as u8));
                    let index = append_unique(&mut self.globals, s.to_owned());
                    self.instructions.push(Instr::Operand(index as u16));
                    return;
                }

                self.instructions.push(Instr::Operator(Operators::PUSH_LOCAL as u8));
                self.instructions.push(Instr::Operand(self.locals_map[s]));
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
                        &call_node.callee.call().unwrap().operands[0], // left
                        &call_node.operands[0],                        // right
                    ];

                    // Check for cast.
                    if ident.value == "cast" {
                        self.emit(args[0]);
                        self.instructions.push(Instr::Operator(Operators::CAST as u8));

                        if let Some(cast_name) = args[1].get_name() {
                            let cast_to : u16 = match cast_name {
                                "Real" => 0b00000011,
                                "Int"  => 0b00000010,
                                "Nat"  => 0b00000001,
                                _ => issue!(err::Types::TypeError, self.filename, err::NO_TOKEN, self.current_line,
                                    "Compiler does not know how to cast to `{}'.", cast_name)
                            };
                            let cast_from = match args[0].yield_type() {
                                ast::StaticTypes::TReal    => 0b00000011,
                                ast::StaticTypes::TInteger => 0b00000010,
                                ast::StaticTypes::TNatural => 0b00000001,
                                _ => issue!(err::Types::TypeError, self.filename, err::NO_TOKEN, self.current_line,
                                    "Compiler does not know how to cast from `{}'.", args[0].yield_type())
                            };
                            self.instructions.push(Instr::Operand(cast_from << 8 | cast_to));
                        } else {
                            issue!(err::Types::CompError, self.filename, err::NO_TOKEN, self.current_line,
                                "Cast-type provided to `cast' has to be a type-name.")
                        }
                        return;
                    }

                    // Check for assignment.
                    if ident.value == "=" {
                        // Direct variable assignment:
                        if let ast::Nodes::Ident(left) = args[0] {
                            self.ident_assignment(left, args[1]);
                        }
                        return;
                    }

                    // Check for type annotation.
                    if ident.value == ":" {
                        // If the LHS is not an ident, it is not a
                        //   valid annotation.
                        if args[0].ident().is_none() {
                            issue!(err::Types::CompError, self.filename, err::NO_TOKEN, self.current_line,
                                "Left of `:` type annotator must be an identifier.");
                        }
                        let left = args[0].ident().unwrap();

                        // Annotation of variable or function.
                        self.annotation(left, args[1]);
                        return;
                    }

                    // Check for fast internal binary operations such as +, -, *, /, etc.
                    let maybe_op = internal_functions::get_internal_op(&ident.value, Some(&args));
                    if let Some(op) = maybe_op {
                        self.emit(args[1]);
                        self.emit(args[0]);
                        self.instructions.push(op);
                        return;
                    }
                }
                self.emit(&call_node.operands[0]);
                self.emit(&*call_node.callee);
                self.instructions.push(Instr::Operator(Operators::CALL_1 as u8));
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
        write!(f, "===Locals==================\n")?;
        for key in self.locals_map.keys() {
            write!(f, "{: >3} |  {}\n", self.locals_map[key], key)?;
        }
        write!(f, "===Globals=================\n")?;
        for (i, c) in self.globals.iter().enumerate() {
            write!(f, "{: >3} |  {}\n", i, c)?;
        }
        write!(f, "===Bytecodes===============\n")?;
        for inst in &self.instructions {
            write!(f, "{}", inst)?;
        }
        write!(f, "")
    }
}