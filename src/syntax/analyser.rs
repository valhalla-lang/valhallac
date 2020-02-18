/*!
 * Analyse the syntax tree, assign types, cast types, and
 * perform a battery of optimisations.
 */
use std::collections::HashMap;

use crate::err;

use super::ast;
use ast::Nodes;

// This entire file, should be rewritten.
// Or, just deleted, and replaced with a folder containing
// a file for each function (also rewritten) (since they get very big).

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

            let default = Nodes::Call(ast::CallNode {
                callee: Box::new(Nodes::Call(ast::CallNode {
                    callee: Box::new(const_fold(&*call.callee.call().unwrap().callee)),
                    operands: vec![left.clone()],
                    return_type: call.callee.yield_type(),
                    location: call.callee.call().unwrap().location
                })),
                operands: vec![right.clone()],
                return_type: call.return_type.clone(),
                location: call.location
            });

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
                return Nodes::Num(ast::NumNode { value, location: call.location });
            } else {
                return default;
            }
        }
        return Nodes::Call(ast::CallNode {
            callee: Box::new(const_fold(&*call.callee)),
            operands: vec![const_fold(&call.operands[0])],
            return_type: call.return_type.clone(),
            location: call.location
        });
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
            ast::IdentNode::new("cast", node.location()),
            vec![node.clone()],
            node.location()),
        vec![ast::SymNode::new(to_type, node.location())],
        node.location());
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

                        let mut new_call = if casting_right {
                            ast::CallNode::new(
                                *call.callee.clone(),
                                vec![create_cast(&right, &cast_to)],
                                call.callee.location())
                        } else {
                            ast::CallNode::new(
                                ast::CallNode::new(
                                    *call.callee.call().unwrap().callee.clone(),
                                    vec![create_cast(&left, &cast_to)],
                                    call.callee.location()),
                                vec![right],
                                call.location)
                        };
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
            } else if bin_op.value == "="
            && left_yield.is_number()
            && cast_strength(&left_yield) > cast_strength(&right_yield) {
                let mut new_call = ast::CallNode::new(
                    *call.callee.clone(),
                    vec![create_cast(&right, &left_yield)],
                    call.callee.location());
                if let Nodes::Call(ref mut c) = new_call {
                    c.set_return_type(left_yield);
                }
                return new_call;
            }
        }
        let mut non_bi = ast::CallNode::new(
            balance_types(&*call.callee),
            vec![balance_types(&call.operands[0])],
            call.callee.location());
        if let Nodes::Call(ref mut c) = non_bi {
            c.set_return_type(call.return_type.clone());
        }
        return non_bi;
    }
    return node.to_owned();
}

#[derive(Clone)]
struct TypeChecker {
    pub source_line : usize,
    pub source_file : String,
    ident_map : HashMap<String, ast::StaticTypes>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            source_line: 0,
            source_file: String::from("UNANNOUNCED_FILE"),
            ident_map: HashMap::new(),
        }
    }

    pub fn type_branch(&mut self, node : &Nodes) -> Nodes {
        let mut clone = node.to_owned();
        self.source_line = clone.location().line as usize;
        match clone {
            Nodes::File(f) => self.source_file = f.filename,
            Nodes::Ident(ref mut i) => {
                if let Some(annotation) = self.ident_map.get(&i.value) {
                    if let ast::StaticTypes::TSet(class) = annotation.clone() {
                        i.static_type = *class;
                    } else {
                        i.static_type = annotation.clone();
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

                                    self.ident_map.insert(annotation.0.clone(), annotation.1.clone());

                                    if let ast::StaticTypes::TSet(class) = annotation.1 {
                                        annotatee.static_type = *class;
                                    } else {
                                        // Error, can only be element of set.
                                    }

                                    return clone;
                                } else {
                                    // Error: We need the left to be an ident.
                                    issue!(ParseError,
                                        self.source_file.as_str(),
                                        err::NO_TOKEN, self.source_line,
                                        "The left side of the member-of operator (`:`), must be an identifier.
                                         You supplied a type of `{}'.
                                         Only variable names can be declared as being members of sets.",
                                        callee.operands[0].node_type());
                                }
                            },
                            "=" => {
                                // This is useful for checking variables in functions.
                                match &callee.operands[0] {
                                    Nodes::Call(ref assignee) => {
                                        // Check all the types in the annotation (A -> B -> C)
                                        //  and match them to the arguments found on the left side
                                        //  of the assignment (=). Compile these matches into a list
                                        //  and pass that list into a new TypeChecker object which checks
                                        //  the right hand side of the assignment, matching up the sub-scoped
                                        //  variables.

                                        // A -> B -> C -> D
                                        // f a b c = d
                                        // <=>
                                        //              (A -> (B -> (C  -> D)))
                                        // ( ((=) ( (((f a)    b)    c) )) d)

                                        let mut operands = assignee.collect();
                                        let mut func_checker = self.clone();

                                        let base_node = operands.remove(0);
                                        if base_node.ident().is_none() {
                                            issue!(ParseError,
                                                &self.source_file, err::NO_TOKEN, self.source_line,
                                                "Function definitions must have the defining function's base caller
                                                be an identifier! You're trying to define a function that has
                                                `{}' as base caller...", base_node.node_type());
                                        }

                                        let maybe_type = self.ident_map.get(&base_node.ident().unwrap().value);
                                        if maybe_type.is_none() {
                                            println!("{}", base_node);
                                            println!("{:?}", self.ident_map);
                                            issue!(TypeError,
                                                self.source_file.as_str(),
                                                err::NO_TOKEN, self.source_line,
                                                "Cannot find type annotation for the
                                                 function definition of `{}'.",
                                                 base_node.ident().unwrap().value);
                                        }
                                        let mut t = maybe_type.unwrap().clone();

                                        for operand in operands {
                                            if let Nodes::Ident(ident) = operand {
                                                if let ast::StaticTypes::TSet(f) = &t {
                                                    if let ast::StaticTypes::TFunction(i, o) = *f.clone() {
                                                        func_checker.ident_map.insert(ident.value, *i.clone());
                                                        t = *o.clone();
                                                    }
                                                }
                                            }
                                        }

                                        call.operands[0] = func_checker.type_branch(&call.operands[0]);
                                        return clone;
                                    }
                                    Nodes::Ident(_assignee) => {
                                        // TODO:
                                        // Here, if the ident exists in the ident_map, that means
                                        //  we need to check if both sides of the `=`'s types match up.
                                        //  If it does not exist, we need to infer its type by looking at
                                        //  the RHS and statically determine the RHS's type, and adding that
                                        //  type to the ident_map for the assignee.
                                    }
                                    _ => ()
                                }
                            }
                            _ => ()
                        }
                    }
                }
                // TODO HERE:
                //  We need to check to see if the function being called
                //  has a statically determined type, and if so, check that
                //  the operand to that function call has the exact same
                //  static type.
                //  If there is a type-mismatch, just throw an `issue!`.
                //  (If the function is statically typed, so
                //    must all the arguments be as well).
                //  The call must have a yield of type `function` and the
                //  input part of the function (input |-> output), must match
                //  the type of the operand.  :^)
                call.callee = Box::new(self.type_branch(&*call.callee));
                call.operands = vec![self.type_branch(&call.operands[0])];

                if let ast::StaticTypes::TFunction(_, o) = call.callee.yield_type() {
                    if let ast::StaticTypes::TSet(t) = *o {
                        call.return_type = *t;
                    } else {
                        call.return_type = *o;
                    }
                }

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
