use super::ast;
use ast::{Nodes, StaticTypes};

use super::type_balancer;

use lazy_static::lazy_static;
use std::collections::HashSet;

use crate::err;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SymbolEntry {
    pub identifier : String,
    pub signature : StaticTypes,
    pub defined : bool,
}

impl SymbolEntry {
    pub fn is_function(&self) -> bool {
        if let StaticTypes::TFunction(_, _) = self.signature {
            return true;
        }
        return false;
    }

    pub fn was_defined(&mut self) {
        self.defined = true;
    }
}

#[derive(Debug, Clone)]
struct SymbolTable {
    table : Vec<SymbolEntry>,
    pub scope : String
}

impl SymbolTable {
    pub fn new(scope : &str) -> Self {
        Self {
            table: vec![],
            scope: String::from(scope)
        }
    }

    pub fn collect(self) -> Vec<SymbolEntry> {
        self.table
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item=&SymbolEntry> {
        self.table.iter()
    }

    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item=&mut SymbolEntry> {
        self.table.iter_mut()
    }

    pub fn get(&self, i : usize) -> Option<&SymbolEntry> {
        self.table.get(i)
    }

    pub fn get_mut(&mut self, i : usize) -> Option<&mut SymbolEntry> {
        self.table.get_mut(i)
    }

    pub fn push(&mut self, ident : &str, sig : StaticTypes, def : bool) {
        #[cfg(feature="debug")] {
            println!("Type added to table `{}':", self.scope);
            println!("\t`{}', is a: {}", &ident, &sig);
        }
        self.table.push(SymbolEntry {
            identifier: String::from(ident),
            signature: sig,
            defined: def,
        });
    }

    pub fn contains(&self, ident : &str) -> bool {
        for elem in &self.table {
            if elem.identifier == ident {
                return true;
            }
        }
        return false;
    }

    pub fn collect_signatures(&self, ident : &str) -> HashSet<StaticTypes> {
        self.table.iter()
            .filter(|e| e.identifier == ident)
            .map(|e| e.signature.to_owned())
            .collect()
    }
}


pub struct ResolutionContext {
    table_chain : Vec<SymbolTable>,
    filename : String
}

// TODO: Arithmetic operators will be properly defined
// in some sort of prelude lib.
lazy_static! {
    static ref INTERNAL_IDENTS : HashSet<String> = vec![
        "=", ":", "->", "__raw_print", "+", "-", "*", "/", "^"
    ].into_iter().map(String::from).collect();
}

impl ResolutionContext {
    pub fn new() -> Self {
         Self {
             table_chain: vec![SymbolTable::new("GLOBAL")],
             filename: String::from("unspecified")
        }
    }

    fn current_table(&mut self) -> &mut SymbolTable {
        self.table_chain.last_mut()
            .expect("Somehow there is no current scope.")
    }

    fn search_chain(&mut self, ident : &str) -> Option<&mut SymbolTable> {
        for table in self.table_chain.iter_mut().rev() {
            if table.contains(ident) {
                return Some(table);
            }
        }
        return None;
    }

    /// # Terminology
    /// `appl_0` - refers to the the 0th (base) application (call).
    /// `appl_n` - refers to any nested application n-levels deep.
    /// # Function
    /// Entry point for type resolution of AST branches.
    /// Returns a clone of the branch but with added type-information.
    pub fn resolve_branch(&mut self, branch : &Nodes) -> Nodes {
        if let Nodes::File(file_node) = branch {
            self.filename = file_node.filename.to_owned();
            return branch.to_owned();
        }

        let mut node = branch.to_owned();
        // Assign type to signatures, add to table.

        // If we have an ident (variable) not being used as a function
        // call (i.e. it's type cannot be overloaded).
        if let Nodes::Ident(ref mut ident) = node {
            // Ignore certain variables (internals, not user declared).
            if INTERNAL_IDENTS.contains(&ident.value) {
                return node;
            }

            // Search for variable in tables, to give it a type.
            let maybe_table = self.search_chain(&ident.value);
            if let Some(table) = maybe_table { // It is in the table.
                // Get signatures. Variables cannot have multiple signatures.
                let signatures = table.collect_signatures(&ident.value);
                if signatures.len() > 1 {
                    // TODO: Partial application not considered.
                    issue!(ParseError, &self.filename,
                        err::LOC, &ident.location,
                        "Variable has multiple type signatures. Overloading \
                        types is only possible with functions.");
                }
                // We can unwrap this because we know it contains exactly
                // one (1) element.
                let signature = signatures.iter().next().unwrap();
                // Give the identifier it's signature.
                ident.static_type = signature.clone();
            } else { // Variable has not been declared.
                issue!(ParseError, &self.filename, err::LOC, &ident.location,
                    "Variable `{}' is used, but has not been declared.",
                    &ident.value);
            }
        // What to do, if we have a call to resolve.
        } else if let Nodes::Call(ref mut appl_0) = node {
            let appl_0_clone = appl_0.clone();
            let mut skip_type_check = false;

            // Some day we'll have `let-chains'.
            if let Nodes::Call(ref mut appl_1) = *appl_0.callee {
            if let Nodes::Ident(ref ident_1) = *appl_1.callee {
                match ident_1.value.as_ref() {
                    "->" => panic!("We should have prevented this."),
                    ":" => {
                        self.resolve_annotation(appl_0_clone, appl_1.clone());
                        // FIXME: Should we really replace the annotation with a nil?
                        // I know it isn't useful anymore, but maybe we should keep it
                        // and just ignore it when compiling.  Returning nil might
                        // add complexity to the rest of the type checking.
                        return ast::NilNode::new(ident_1.location);
                    },
                    "=" => {
                        *appl_0 = self.resolve_assignment(appl_0_clone, appl_1.clone());
                        skip_type_check = true;
                    },
                    // Internal functions, with internal
                    // types/definitions, etc.
                      "+" | "-"
                    | "*" | "/"
                    | "^" => {  // Arithmetic operations typing.
                        // Resolve on both sides as much as possible.
                        println!("balancing on line {}.", appl_0.location.line);
                        if let Some(operand) = appl_0_clone.operand() {
                            appl_0.operands[0] = self.resolve_branch(operand);
                        }
                        if let Some(operand) = appl_1.operand() {
                            appl_1.operands[0] = self.resolve_branch(operand);
                        }
                        let cloned_node = node.clone();
                        return type_balancer::default(&cloned_node);
                    }
                    _ => ()
                }
            }}
            // Any call should resolve its callee type, and check if it is legal
            // to apply an operand of such a (resolved) type.
            // This entier call expression must thus also be typed, unrolling
            // the type from the callee.
            if skip_type_check {
                return node;
            }
            // Recursively resolve both sides of the expression.
            appl_0.callee = Box::new(self.resolve_branch(&*appl_0.callee));
            if let Some(operand) = appl_0.operand() {
                appl_0.operands[0] = self.resolve_branch(operand);
            }
            // Check application is legal.
            let appl_0_st = (*appl_0.callee).yield_type().to_owned(); 
            if let StaticTypes::TFunction(box_op_t, box_ret_t) = appl_0_st {
                // Check if operand type checks out.
                let op_0_st = appl_0.operands[0].yield_type();
                let maybe_op_inner_type = (*box_op_t).set_inner();

                if maybe_op_inner_type.is_none() {
                    // Fatal, we should really never get here,
                    // because we _should_ check for this earlier.
                    issue!(TypeError, &self.filename,
                           err::LOC, &(*appl_0.callee).location(),
                           "Function should map from a set, it does not.");
                }

                // Safe to unwrap, we've checked for none.
                let op_inner_type = maybe_op_inner_type.unwrap();

                if op_0_st != op_inner_type {
                    // TODO: If the types don't match, BUT,
                    // the type is a strict subset of the other,
                    // we may cast up the internal type (if possible).
                    // This should be done on 'Int' cast up to 'Real'
                    // (if a Real was expected, and got an Int), for
                    // example.
                    // We should alos emit a warning (always?) when
                    // an implicit cast has taken place.
                    issue!(TypeError, &self.filename,
                           err::LOC, &appl_0.operands[0].location(),
                           "Mismatching type in function call.
                             Expected argument of element \
                             of `{}', instead got a `{}'.",
                             op_inner_type, op_0_st);
                }
                // If so, we can continue to unroll the type and
                // assign it to this expression.

                // When applied, we end up with a value with
                // a type of element of box_ret_t.
                let return_type = (*box_ret_t).set_inner();
                if return_type.is_none() {
                    // Fatal, see simlar comment above.
                    issue!(TypeError, &self.filename,
                           err::LOC, &(*appl_0.callee).location(),
                           "Function should map to a set, it does not.");
                }
                appl_0.return_type = return_type.unwrap().clone();
            } else {
                issue!(TypeError, &self.filename,
                       err::LOC, &appl_0.callee.location(),
                       "Function-application / juxtaposition is not \
                       defined on type of `{}'.",
                       appl_0_st);
            }
        }

        node
    }

    fn resolve_assignment(&mut self,
                          mut appl_0 : ast::CallNode,
                          appl_1 : ast::CallNode) -> ast::CallNode {
        // TODO: Assignment means implicit type
        // is given, if no type signature found,
        // OR, it means we are defining a declared
        // variable given the latest signature for it.
        // Either way, we must say it is now 'defined'
        // (as well as 'declared') in the table.

        // TODO: '=' with a type annotation should
        // cast the value on the right if possible, to
        // match the annotation.  If not possible, throw
        // an issue, saying types must match!

        // TODO: Handle if the assignment is defining
        // a function (e.g. `f x = x + 1`).

        let filename = &self.filename.to_owned();
        let lhs = &appl_1.operands[0]; 
        // Handle variable (identifier) assignemnt:
        if let Nodes::Ident(ident_op_1) = lhs {
            // Recursively resolve RHS of assignment.
            appl_0.operands[0] = self.resolve_branch(&appl_0.operands[0]);
            // Check if an signature exists.
            let maybe_table = self.search_chain(&ident_op_1.value);
            if let Some(table) = maybe_table { 
                // TODO: Could be a function overload!
                let mut entries : Vec<&mut SymbolEntry> = table
                    .iter_mut()
                    .filter(|entry|
                            entry.identifier == ident_op_1.value)
                    .collect();

                // Search did not give `None`, so entries
                // should never be empty!
                assert!(entries.len() > 0);
                #[cfg(feature="debug")] {
                    println!("Assignment of `{}':", ident_op_1.value);
                    println!("- RHS-type: {}", appl_0.operands[0].yield_type());
                    println!("- Entries: {:#?}", entries);
                }

                if entries.len() == 1 { // Not overloaded.
                    let ref mut entry = entries[0];
                    // Check entry matches type of RHS
                    // of assigment.

                    // TODO: Check if types can be coerced.
                    let rhs_type = appl_0.operands[0].yield_type(); 
                    if rhs_type != entry.signature {
                        // TODO: Can cast? if so, do
                        // and don't throw an error.
                        issue!(TypeError, filename,
                               err::LOC, &appl_0.operands[0].location(),
                               "Signature does not match \
                                right-hand-side of assignemnt.
                                Expected `{}', got `{}'.",
                               entry.signature, rhs_type);
                    }
                    // Otherwise, all is fine,
                    // and we can update whether it has
                    // been defined.
                    entry.was_defined();
                } else { // Overloaded.
                    // TODO: Check if it is valid to overload
                    // here. Non-functions cannot be overloaded
                }
            } else {
                // Variable has implicit type, and
                // adding the type to the symbol table
                // is handled here.
            }
        } else if let Nodes::Call(op_1) = lhs {
            ()
        } else {
            // TODO: Assigment to functions.
            // TODO: Pattern matching etc.
            issue!(ParseError,
                   &self.filename, err::LOC, &appl_1.operands[0].location(),
                   "Cannot assign to `{}' structure.",
                   appl_1.operands[0].node_type());
        }

        return appl_0;
    }

    fn resolve_annotation(&mut self, appl_0 : ast::CallNode, appl_1 : ast::CallNode) {
        let maybe_op_1 = appl_1.operand();
        if let Some(op_1) = maybe_op_1 {
            if let Nodes::Ident(op_id_1) = op_1 {
                let op_0 = appl_0.operands[0].clone();
                let set_signature = op_0.yield_type();
                if let StaticTypes::TSet(signature) = set_signature {
                    self.current_table().push(
                        &op_id_1.value, *signature, false);
                } else {
                    issue!(TypeError, &self.filename, err::LOC, &op_0.location(),
                           "Right of type annotation must be a set. \
                            Instead got `{}`.", set_signature);
                }
            } else {
                issue!(ParseError, &self.filename,
                    err::LOC, &op_1.location(),
                    "Left of `:` type annotator must be \
                        an identifier; found `{}'.", op_1.node_type());
            }
        } else {
            issue!(ParseError,
                &self.filename, err::LOC,
                &appl_1.location,
                "No expression found left of `:`.");
        }
    }
}
