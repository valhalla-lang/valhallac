use std::fmt;
use std::collections::{HashMap, VecDeque};

use super::super::err;

use super::super::syntax;
use syntax::ast;
use syntax::ast::Nodes;

use super::element;
use super::instructions;

use element::{Element, Symbol};
use instructions::{Instr, Operators};
use num_traits::cast::FromPrimitive;

use super::internal_functions;

fn append_unique<T : Clone + PartialEq>(v : &mut Vec<T>, e : T) -> usize {
    let index = v.iter().position(|c| c == &e);
    if index.is_none() { v.push(e); }
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
struct IdentTypePair<'a>(String, &'a Nodes);

#[derive(Clone)]
pub struct LocalBlock<'a> {
    pub name : String,
    pub filename : String,
    pub constants : Vec<Element<'a>>,
    pub instructions : Vec<Instr>,
    pub globals : Vec<String>,
    pub operand_type : ast::StaticTypes,
    pub return_type  : ast::StaticTypes,

    // Used only for compilation:
    pub locals_map : HashMap<String, u16>,
    types_to_check : VecDeque<IdentTypePair<'a>>,
    current_line  : usize,
    current_depth : usize,
    pub stack_depth   : usize,
    last_instruction : Instr,
    last_const_push_index : u16,
    last_depth_delta : isize,
}

impl<'a> PartialEq for LocalBlock<'a> {
    fn eq(&self, other : &Self) -> bool {
        self.constants == other.constants
        && self.instructions == other.instructions
    }
}

impl<'a> LocalBlock<'a> {
    pub fn new(n : &str, f : &str) -> Self {
        LocalBlock {
            name: n.to_string(),
            filename: f.to_string(),
            constants: vec![],
            instructions: vec![],
            globals: vec![],
            operand_type: ast::StaticTypes::TUnknown,
            return_type:  ast::StaticTypes::TUnknown,

            locals_map: HashMap::new(),
            types_to_check: VecDeque::new(),
            current_line:  0,
            stack_depth:   0,
            current_depth: 0,
            last_instruction: Instr::Operator(0),
            last_const_push_index: 0xffff,
            last_depth_delta: 0,
        }
    }

    fn push_const_instr(&mut self, e : Element<'a>) {
        let index = append_unique(&mut self.constants, e) as u16;

        // Don't push constant if:
        //    (already on stack) and (stack depth has stayed the same)
        if !(index == self.last_const_push_index && self.last_depth_delta == 0) {
            self.push_operator(Operators::PUSH_CONST);
            self.push_operand(index);
            self.last_const_push_index = index;
        }
    }

    fn change_stack_depth(&mut self, i : isize) {
        self.last_depth_delta = i;
        self.current_depth = (
            (self.current_depth as isize) + i
        ) as usize;
        if self.current_depth > self.stack_depth {
            self.stack_depth = self.current_depth;
        }
    }

    fn push_operator(&mut self, o : Operators) {
        let instr = Instr::Operator(o as u8);
        if !o.takes_operand() {
            self.change_stack_depth(instr.depth_delta(None));
        }
        self.last_instruction = instr;
        self.instructions.push(instr);
    }

    fn push_operand(&mut self, i : u16) {
        let operand = Instr::Operand(i);
        self.instructions.push(operand);
        self.change_stack_depth(
            self.last_instruction.depth_delta(
                Some(operand)));
    }

    fn insert_local(&mut self, s : String) -> u16 {
        let index = self.locals_map.len() as u16;
        self.locals_map.insert(s, index);
        index
    }

    fn ident_assignment(&mut self, left : &'a ast::IdentNode, right : &'a Nodes) {
        if self.types_to_check.is_empty() {
            issue!(TypeError, &self.filename, err::NO_TOKEN, self.current_line,
                "You must state what set `{}' is a member of. No type-annotation found.", left.value);
        }
        if self.locals_map.contains_key(&left.value) {
            issue!(CompError, &self.filename, err::NO_TOKEN, self.current_line,
                "Cannot mutate value of `{}', as it is already bound.", left.value);
        }
        let index = self.insert_local(left.value.to_owned());

        self.emit(right);
        if left.static_type == ast::StaticTypes::TUnknown
        || left.static_type != right.yield_type() {
            self.push_operator(Operators::DUP);
            let type_node = self.types_to_check.pop_front().unwrap().1;
            self.emit(type_node);
            self.push_operator(Operators::CHECK_TYPE);
        } else {  // Otherwise just pop, type was already checked statically so
                  //  its of no use to include in the compiled program,
                  //  as no dynamic checking is needed.
            self.types_to_check.pop_front();
        }
        self.push_operator(Operators::STORE_LOCAL);
        self.push_operand(index);
    }

    fn function_assign(&mut self, left : &ast::CallNode, right : &'a Nodes) {
        let mut arguments = left.collect();
        let base_node = arguments.remove(0);

        if let Nodes::Ident(ident) = base_node {
            let name = format!("__{}_final", ident.value.to_owned());

            let mut last_block = LocalBlock::new(&name, &self.filename);
            // TODO: Be more careful here, not always an ident.
            //  NEED TO DEAL WITH PATTERN MATCHING.
            last_block.insert_local(arguments.last().unwrap().ident().unwrap().value.to_owned());
            last_block.emit(right);
            last_block.yield_last();

            for i in (0..(arguments.len() - 1)).rev() {
                let name = format!("__{}_{}", ident.value, i);
                let mut super_block = LocalBlock::new(
                    &name,
                    &self.filename);
                // Also TODO: Pattern matching, be careful in the future.
                super_block.insert_local(arguments[i].ident().unwrap().value.to_owned());

                let block_name = last_block.name.clone();
                super_block.push_const_instr(Element::ECode(Box::new(last_block)));
                super_block.push_const_instr(Element::ESymbol(Symbol::new(&block_name)));
                super_block.push_operator(Operators::MAKE_FUNC);
                super_block.yield_last();
                last_block = super_block;
            }

            let index = self.insert_local(ident.value.to_owned());

            self.push_const_instr(Element::ECode(Box::new(last_block)));
            self.push_const_instr(Element::ESymbol(Symbol::new(&ident.value)));
            self.push_operator(Operators::MAKE_FUNC);
            self.push_operator(Operators::STORE_LOCAL);
            self.push_operand(index);
            return;
        }

        // A function of multiple arguments (say 3 f.eks),
        //  must generate a function, which when called returns
        //  a function, and when that function is called, it returns
        //  the final value.
    }

    fn annotation(&mut self, left : &ast::IdentNode, right : &'a Nodes) {
        self.types_to_check.push_back(IdentTypePair(left.value.to_owned(), right));
    }

    fn emit(&mut self, node : &'a Nodes) {
        let current_line = node.location().line as usize;
        if self.current_line != current_line {
            let len = self.instructions.len();
            if len > 1
            && self.instructions[len - 2]
               == Instr::Operator(Operators::SET_LINE as u8) {
                self.instructions.pop();
                self.instructions.pop();
            }
            self.current_line = current_line;
            self.push_operator(Operators::SET_LINE);
            self.push_operand(self.current_line as u16);
        }

        match node {
            Nodes::Ident(ident_node) => {
                let s = &ident_node.value;
                if !self.locals_map.contains_key(s) {
                    self.push_operator(Operators::PUSH_SUPER);
                    let index = append_unique(&mut self.globals, s.to_owned()) as u16;
                    self.push_operand(index);
                    return;
                }

                self.push_operator(Operators::PUSH_LOCAL);
                self.push_operand(self.locals_map[s]);
            },
            Nodes::Num(num_node) => {
                self.push_const_instr(numerics_to_element(&num_node.value));
            },
            Nodes::Str(str_node) => {
                self.push_const_instr(Element::EString(&str_node.value));
            },
            Nodes::Sym(sym_node) => {
                self.push_const_instr(Element::ESymbol(Symbol::new(&sym_node.value)));
            },
            Nodes::Call(call_node) => {
                if let Nodes::Ident(ident_node) = &*call_node.callee {
                    let mut do_return = true;
                    match ident_node.value.as_str() {
                        "__raw_print" => {
                            self.emit(&call_node.operands[0]);
                            self.push_operator(Operators::RAW_PRINT);
                        }
                        _ => do_return = false
                    };
                    if do_return { return; }
                }
                if call_node.is_binary() {
                    let ident = call_node.callee.call().unwrap().callee.ident().unwrap();
                    let args = vec![
                        &call_node.callee.call().unwrap().operands[0], // left
                        &call_node.operands[0],                        // right
                    ];

                    // Check for cast.
                    if ident.value == "cast" {
                        self.emit(args[0]);
                        self.push_operator(Operators::CAST);

                        if let Some(cast_name) = args[1].get_name() {
                            let cast_to : u16 = match cast_name {
                                "Real" => 0b0000_0011,
                                "Int"  => 0b0000_0010,
                                "Nat"  => 0b0000_0001,
                                _ => issue!(TypeError, &self.filename, err::NO_TOKEN, self.current_line,
                                    "Compiler does not know how to cast to `{}'.", cast_name)
                            };
                            let cast_from = match args[0].yield_type() {
                                ast::StaticTypes::TReal    => 0b0000_0011,
                                ast::StaticTypes::TInteger => 0b0000_0010,
                                ast::StaticTypes::TNatural => 0b0000_0001,
                                _ => issue!(TypeError, &self.filename, err::NO_TOKEN, self.current_line,
                                    "Compiler does not know how to cast from `{}'.", args[0].yield_type())
                            };
                            self.push_operand(cast_from << 8 | cast_to);
                        } else {
                            issue!(CompError, &self.filename, err::NO_TOKEN, self.current_line,
                                "Cast-type provided to `cast' has to be a type-name.")
                        }
                        return;
                    }

                    // Check for assignment.
                    if ident.value == "=" {
                        // Direct variable assignment:
                        if let Nodes::Ident(left) = args[0] {
                            self.ident_assignment(left, args[1]);
                        } else if let Nodes::Call(left) = args[0] {
                            self.function_assign(left, args[1]);
                        }
                        return;
                    }

                    // Check for type annotation.
                    if ident.value == ":" {
                        // If the LHS is not an ident, it is not a
                        //   valid annotation.
                        if args[0].ident().is_none() {
                            issue!(CompError, &self.filename, err::NO_TOKEN, self.current_line,
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
                    if let Instr::Operator(operator) = op {
                        self.emit(args[1]);
                        self.emit(args[0]);
                        self.push_operator(Operators::from_u8(operator).unwrap());
                        return;
                    }}
                }
                // TODO: Optimise to implicitly ignore currying and use CALL_N instead.
                //  Also, check that we are indeed calling a function, and not anything else
                //  by checking the static yield type.
                self.emit(&call_node.operands[0]);
                self.emit(&*call_node.callee);
                self.push_operator(Operators::CALL_1);
            },
            _ => ()
        };
    }

    fn yield_last(&mut self) {
        self.push_operator(Operators::YIELD);
    }

    pub fn generate(&mut self, nodes : &'a [Nodes]) {
        for node in nodes {
            self.emit(node);
        }
        self.yield_last();
    }
}

impl<'a> fmt::Display for LocalBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in &self.constants {
            if let Element::ECode(local_block_box) = c {
                write!(f, "{}", *local_block_box)?;
            }
        }
        write!(f, "\n{}:", self.name)?;
        writeln!(f,"
  |[meta]:
  |  stack-depth: {}
  |    file-name: {}",
            self.stack_depth,
            self.filename)?;

        writeln!(f, "  |====Constants===============")?;
        for (i, c) in self.constants.iter().enumerate() {
            writeln!(f, "  | {: >3} |  {}", i, c)?;
        }
        writeln!(f, "  |====Locals==================")?;
        for key in self.locals_map.keys() {
            writeln!(f, "  | {: >3} |  {}", self.locals_map[key], key)?;
        }
        writeln!(f, "  |====Globals=================")?;
        for (i, c) in self.globals.iter().enumerate() {
            writeln!(f, "  | {: >3} |  {}", i, c)?;
        }
        writeln!(f, "  |====Bytecodes===============")?;
        for inst in &self.instructions {
            if let Instr::Operand(_) = inst {
                write!(f, "{}", inst)?;
            } else {
                write!(f, "  | {}", inst)?;
            }
        }
        write!(f, "")
    }
}
