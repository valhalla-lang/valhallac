use std::collections::HashMap;

use crate::issue;

use super::ast;
use ast::Nodes;

#[derive(Clone)]
pub struct TypeChecker {
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
        self.source_line = clone.location().line.unwrap();
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
                                        callee.operands[0].site().with_filename(&self.source_file),
                                        "The left side of the member-of operator (`:`), must be an identifier.
                                         You supplied a type of `{}'.
                                         Only variable names can be declared as being members of sets.",
                                        callee.operands[0].node_type())
                                            .print();
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

                                        let mut operands = assignee.collect_operands();
                                        let mut func_checker = self.clone();

                                        let base_node = operands.remove(0);
                                        if base_node.ident().is_none() {
                                            issue!(ParseError,
                                                base_node.site().with_filename(&self.source_file),
                                                "Function definitions must have the defining function's base caller
                                                be an identifier! You're trying to define a function that has
                                                `{}' as base caller...", base_node.node_type())
                                                    .print();
                                        }

                                        let maybe_type = self.ident_map.get(&base_node.ident().unwrap().value);
                                        if maybe_type.is_none() {
                                            #[cfg(feature="debug")] {
                                                println!("{}", base_node);
                                                println!("{:?}", self.ident_map);
                                            }
                                            issue!(TypeError,
                                                base_node.site().with_filename(&self.source_file),
                                                "Cannot find type annotation for the
                                                 function definition of `{}'.",
                                                 base_node.ident().unwrap().value)
                                                    .print();
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
