use std::collections::{HashMap, VecDeque};

use crate::err;

use super::ast;
use ast::Nodes;

/// Constant folding.
/// A static optimisation that relieves the runtime of having to perform
/// pre-computable trivial calculations, by doing them at compile time
/// instead.  This function takes a node and recurses down, looking
/// for arithmetic operations containing exactly two numeric type nodes
/// as operands, and performs the stated operation.
fn const_fold(node : &Nodes) -> Nodes {
    if let Nodes::Call(call) = node {
        if call.is_binary() {
            let bin_op = call.callee.call().unwrap().callee.ident().unwrap();
            let left  = const_fold(&call.callee.call().unwrap().operands[0]);
            let right = const_fold(&call.operands[0]);

            let default = ast::CallNode::new(
                ast::CallNode::new(
                    const_fold(&*call.callee.call().unwrap().callee),
                    vec![left.clone()]),
                vec![right.clone()]);

            let is_num_left  =  left.num().is_some();
            let is_num_right = right.num().is_some();

            if is_num_left && is_num_right {
                let l_value =  left.num().unwrap().value;
                let r_value = right.num().unwrap().value;
                let value = match bin_op.value.as_str() {
                    "+" => l_value + r_value,
                    "-" => l_value - r_value,
                    "*" => l_value * r_value,
                    "/" => {
                        if r_value == ast::Numerics::Natural(0) {
                            return default;
                        }
                        l_value / r_value
                    },
                    _ => {
                        return default;
                    }
                };
                return Nodes::Num(ast::NumNode { value });
            } else {
                return default;
            }
        }
        return ast::CallNode::new(
            const_fold(&*call.callee),
            vec![const_fold(&call.operands[0])]);
    }
    return node.to_owned();
}


fn create_cast(node : &Nodes, cast : &ast::StaticTypes) -> Nodes {
    let to_type = match cast {
        ast::StaticTypes::TReal => ":Real",
        ast::StaticTypes::TInteger => ":Int",
        ast::StaticTypes::TNatural => ":Nat",
        _ => panic!(".is_number() must be broken.")
    };

    let mut cast_node = ast::CallNode::new(
        ast::CallNode::new(
            ast::IdentNode::new("cast"),
            vec![node.clone()]),
        vec![ast::SymNode::new(to_type)]);
    if let Nodes::Call(ref mut call) = cast_node {
        call.set_return_type(cast.clone())
    }
    cast_node
}

fn cast_strength(st : &ast::StaticTypes) -> i32 {
    match st {
        ast::StaticTypes::TReal    => 4,
        ast::StaticTypes::TInteger => 2,
        ast::StaticTypes::TNatural => 0,
        _ => -1,
    }
}

/// The type balancer is a static utility that checks if something
/// like an arithmetic operator has unequal types (e.g. 4.3 + 6 (Real + Natural)).
/// If it does, it balances the two sides of the expressions by injecting a type
/// cast call to one of the arguments.
/// We always cast up (without loss of information), so, 4.3 + 6 will cast the 6
/// to be 6.0.    i.e. 4.3 + 6 ==> 4.3 + (cast 6 :Real) <=> 4.3 + 6.0.
fn balance_types(node : &Nodes) -> Nodes {
    if let Nodes::Call(call) = node {
        if call.is_binary() {
            let bin_op = call.callee.call().unwrap().callee.ident().unwrap();
            let left  = balance_types(&call.callee.call().unwrap().operands[0]);
            let right = balance_types(&call.operands[0].clone());

            let left_yield  =  left.yield_type();
            let right_yield = right.yield_type();
            if ["+", "-", "*", "/"].contains(&bin_op.value.as_str()) {
                if left_yield.is_number() && right_yield.is_number() {
                    if cast_strength(&left_yield) != cast_strength(&right_yield) {

                        let casting_right = cast_strength(&left_yield) >  cast_strength(&right_yield);
                        let cast_to = (if casting_right { &left } else { &right }).yield_type();

                        let mut new_call;
                        if casting_right {
                            new_call = ast::CallNode::new(
                                *call.callee.clone(),
                                vec![create_cast(&right, &cast_to)]);
                        } else {
                            new_call = ast::CallNode::new(
                                ast::CallNode::new(
                                    *call.callee.call().unwrap().callee.clone(),
                                    vec![create_cast(&left, &cast_to)]),
                                vec![right]);
                        }
                        if let Nodes::Call(ref mut c) = new_call {
                            c.set_return_type(cast_to);
                        }
                        return new_call;
                    } else {
                        let mut cloned_node = node.clone();
                        if let Nodes::Call(ref mut c) = cloned_node {
                            c.set_return_type(right_yield);
                        }
                        return cloned_node;
                    }
                }
            } else if bin_op.value == "=" {
                if left_yield.is_number() {
                    if cast_strength(&left_yield) > cast_strength(&right_yield) {
                        let mut new_call = ast::CallNode::new(
                            *call.callee.clone(),
                            vec![create_cast(&right, &left_yield)]);
                        if let Nodes::Call(ref mut c) = new_call {
                            c.set_return_type(left_yield);
                        }
                        return new_call;
                    }
                }
            }
        }
        let mut non_bi = ast::CallNode::new(
            balance_types(&*call.callee),
            vec![balance_types(&call.operands[0])]);
        if let Nodes::Call(ref mut c) = non_bi {
            c.set_return_type(node.yield_type());
        }
        return non_bi;
    }
    return node.to_owned();
}

type VarType = (String, ast::StaticTypes);

struct TypeChecker {
    source_line : usize,
    source_file : String,
    annotations : VecDeque<VarType>,
    last_annotated : Option<VarType>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            source_line: 0,
            source_file: String::from("UNANNOUNCED_FILE"),
            annotations: VecDeque::new(),
            last_annotated: None,
        }
    }

    pub fn type_branch(&mut self, node : &Nodes) -> Nodes {
        let mut clone = node.to_owned();
        match clone {
            Nodes::Line(l) => self.source_line = l.line,
            Nodes::File(f) => self.source_file = f.filename.to_owned(),
            Nodes::Ident(ref mut i) => {
                for pairs in &self.annotations {
                    if pairs.0 == i.value {
                        if let ast::StaticTypes::TSet(class) = pairs.1.to_owned() {
                            i.static_type = *class;
                        }
                    }
                }
                return Nodes::Ident(i.to_owned());
            }
            Nodes::Call(ref mut call) => {
                if let Nodes::Call(ref mut callee) = *call.callee {
                    if let Nodes::Ident(ref binary_ident) = *callee.callee {
                        match binary_ident.value.as_str() {
                            ":" => {
                                if let Nodes::Ident(ref mut annotatee) = callee.operands[0] {
                                    let annotation = (
                                        annotatee.value.to_owned(),
                                        self.type_branch(&call.operands[0]).yield_type()
                                    );
                                    self.last_annotated = Some(annotation.clone());
                                    self.annotations.push_back(annotation.clone());

                                    if let ast::StaticTypes::TSet(class) = annotation.1 {
                                        annotatee.static_type = *class;
                                    }
                                    return clone;
                                } else {
                                    // Error: We need the left to be an ident.
                                    issue!(err::Types::TypeError,
                                        self.source_file.as_str(),
                                        err::NO_TOKEN, self.source_line,
                                        "The left side of the member-of operator (`:`), must be an identifier.
                                         Only variable names can be declared as being members of sets.");
                                }
                            },
                            _ => ()
                        }
                    }
                }
                call.callee = Box::new(self.type_branch(&*call.callee));
                call.operands = vec![self.type_branch(&call.operands[0])];
                return Nodes::Call(call.to_owned());
            },
            _ => ()
        };
        node.to_owned()
    }
}

pub fn replace(root : &mut ast::Root) {
    let mut type_checker = TypeChecker::new();

    let length = root.branches.len();
    let mut i = 0;
    while i < length {
        { // START TOP-LEVEL TYPE-CHECKING
            let new = type_checker.type_branch(&root.branches[i]);
            root.branches[i] = new;
        } // END TOP-LEVEL TYPE-CHECKING
        { // START TOP-LEVEL CONSTANT FOLD
            let new = const_fold(&root.branches[i]);
            root.branches[i] = new;
        } // END TOP-LEVEL CONSTANT FOLD
        { // START TOP-LEVEL TYPE BALANCING
            let new = balance_types(&root.branches[i]);
            root.branches[i] = new;
        } // END TOP-LEVEL TYPE BALANCING
        i += 1;
    }
}