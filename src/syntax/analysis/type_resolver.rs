#![allow(unused_variables)]
#![allow(dead_code)]

use super::ast;
use ast::{Nodes, StaticTypes};

use super::type_balancer;

use lazy_static::lazy_static;
use std::collections::HashSet;

use crate::issue;

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
        false
    }

    pub fn was_defined(&mut self) -> bool {
        let def = self.defined;
        self.defined = true;
        def
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

// Rest is the implementation of the resolution context.
impl ResolutionContext {

pub fn new() -> Self {
     Self {
         table_chain: vec![SymbolTable::new("GLOBAL")],
         filename: String::from("unspecified")
    }
}

fn current_table(&mut self) -> &mut SymbolTable {
    self.table_chain.last_mut()
        .expect("Somehow there is no current scope. This is a bug.")
}

fn search_chain(&mut self, ident : &str) -> Option<&mut SymbolTable> {
    for table in self.table_chain.iter_mut().rev() {
        if table.contains(ident) {
            return Some(table);
        }
    }
    return None;
}

fn unwrap_set(&self, set : &StaticTypes) -> StaticTypes {
    if let StaticTypes::TSet(internal) = set {
        *internal.clone()
    } else {
        // We should never get here, we should always have
        // checked earlier if a function signature tries to map
        // between non-sets.
        use crate::site::Site;
        issue!(TypeError, Site::new().with_filename(&self.filename),
            "Cannot create mapping (function) between non-sets.")
            .crash_and_burn()
    }
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
                issue!(ParseError,
                    ident.site.with_filename(&self.filename),
                    "Variable has multiple type signatures. Overloading \
                    types is only possible with functions.")
                        .print();
            }
            // We can unwrap this because we know it contains exactly
            // one (1) element.
            let signature = signatures.iter().next().unwrap();
            // Give the identifier it's signature.
            ident.static_type = signature.clone();
        } else { // Variable has not been declared.
            issue!(ParseError,
                ident.site.with_filename(&self.filename),
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
                //"->" => panic!("We should have prevented this."),
                ":" => {
                    self.resolve_annotation(appl_0_clone, appl_1.clone());
                    // FIXME: Should we really replace the annotation with a nil?
                    // I know it isn't useful any more, but maybe we should keep it
                    // and just ignore it when compiling.  Returning nil might
                    // add complexity to the rest of the type checking.

                    // Return nil?
                    //return ast::NilNode::new(ident_1.location);

                    // A type signature should evaluate to the type it has assigned.
                    // i.e. `T = (_ : Nat)` is the same as `T = Nat`.
                    // Pattern matching on signatures is also allowed:
                    // `f (n : Nat) = n + 2`, which matches on n that is natural.

                    return node;
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
                    if let Some(operand) = appl_0_clone.operand() {
                        appl_0.operands[0] = self.resolve_branch(operand);
                    }
                    if let Some(operand) = appl_1.operand() {
                        appl_1.operands[0] = self.resolve_branch(operand);
                    }
                    let cloned_node = node.clone();
                    // This HAS to be rewritten.
                    return type_balancer::default(&cloned_node);
                }
                _ => ()
            }
        }}
        // Any call should resolve its callee type, and check if it is legal
        // to apply an operand of such a (resolved) type.
        // This entire call expression must thus also be typed, unrolling
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
                fatal!(TypeError,
                    (*appl_0.callee).site().with_filename(&self.filename),
                    "Function should map from a set, it does not.")
                    .print();
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
                // We should also emit a warning (always?) when
                // an implicit cast has taken place.
                issue!(TypeError,
                    appl_0.operands[0].site().with_filename(&self.filename),
                    "Mismatching type in function call.
                     Expected argument of element \
                     of `{}', instead got a `{}'.",
                    op_inner_type, op_0_st)
                        .print();
            }
            // If so, we can continue to unroll the type and
            // assign it to this expression.

            // When applied, we end up with a value with
            // a type of element of box_ret_t.
            let return_type = (*box_ret_t).set_inner();
            if return_type.is_none() {
                // Fatal, see similar comment above.
                issue!(TypeError,
                    (*appl_0.callee).site().with_filename(&self.filename),
                    "Function should map to a set, it does not.")
                        .print();
            }
            appl_0.return_type = return_type.unwrap().clone();
        } else {
            issue!(TypeError,
                appl_0.callee.site().with_filename(&self.filename),
                "Function-application / juxtaposition is not \
                 defined on type of `{}'.", appl_0_st)
                    .print();
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

    // Assignment with no type annotation needs to be
    // more flexible when it comes to types, i.e.
    // ```
    //    a : Int
    //    a = 3
    //    a = "Somethin"  -- is illegal, doesn't match type.
    // ```
    // Compared to:
    // ```
    //    a = 3
    //    a = "Something"  -- legal, `a' has type `Nat | String` now.
    // ```

    // TODO: '=' with a type annotation should
    // cast the value on the right if possible, to
    // match the annotation.  If not possible, throw
    // an issue, saying types must match!

    // TODO: Handle if the assignment is defining
    // a function (e.g. `f x = x + 1`).

    let filename = &self.filename.to_owned();
    let rhs = appl_0.operands[0].clone();
    let lhs = &appl_1.operands[0];
    // Handle variable (identifier) assignment:
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
                // of assignment.

                // TODO: Check if types can be coerced.
                let rhs_type = appl_0.operands[0].yield_type();
                if rhs_type != entry.signature {
                    // TODO: Can cast? if so, do
                    // and don't throw an error.
                    issue!(TypeError,
                        appl_0.operands[0].site().with_filename(filename),
                        "Signature does not match \
                         right-hand-side of assignment.
                         Expected `{}', got `{}'.",
                        entry.signature, rhs_type)
                            .print();
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
    } else if let Nodes::Call(call_op_1) = lhs {
        let base_call = call_op_1.base_call();
        if !base_call.is_ident() {
            // Fatal, we must define the call on some sort of ident.
            fatal!(ParseError,
                base_call.site().with_filename(&self.filename),
                "You have to assign to a call on an identifier,
                 this identifier is the function you are defining.")
                .note(&format!("Expected an `identifier', found `{}'!",
                    base_call.node_type()))
                .print();
        }
        // We've checked, and we may unwrap it.
        let base_call = base_call.ident().unwrap();

        let func_type;
        if let Some(table) = self.search_chain(&base_call.value) {
            let signatures = table.collect_signatures(&base_call.value);
            let mut sig_iter = signatures.iter();
            // TODO: Select which signature to use (establish some order).
            // Specifically need to select the correct overload.
            // For now pick a random one.
            func_type = sig_iter.next().unwrap().to_owned(); // We know this exists.
        } else {
            // TODO: Determine implicit type for this function.
            // This has to be done in a way that considers the structure
            // of the LHS, considering all pattern matches in each of the
            // cases, and what their types are.  The inductive case
            // should also be analysed for what kind of functions it calls,
            // in order to narrow down the type as much as possible.
            func_type = StaticTypes::TUnknown;  // FIXME.
        }

        let mut left_type  = StaticTypes::TUnknown;
        let mut right_type = StaticTypes::TUnknown;
        // Check if we do actually have a function type.
        if let StaticTypes::TFunction(l, r) = func_type {
            left_type  = self.unwrap_set(&*l);  // This should have already been
            right_type = self.unwrap_set(&*r); // checked for.
        } else { // Fatal, needs to be a function.
            fatal!(TypeError, call_op_1.site.with_filename(&self.filename),
                "Trying to define a function on a variable that does \
                 not have type of `function'.")
                .note(&format!("`{}' has type of `{}', which is not a function.",
                    base_call.value, func_type))
                .print();
        }

        let lhs_operands = call_op_1.collect_operands();
        let operand_count = lhs_operands.len();
        let mut function_scope = SymbolTable::new(&base_call.value);
        for (i, lhs_operand) in lhs_operands.iter().enumerate() {
            if let Nodes::Ident(lhs_op_ident) = lhs_operand {
                function_scope.push(&lhs_op_ident.value, left_type.clone(), true);
                if i == operand_count - 1 {
                    break;  // No need to disect any further.
                }

                if let StaticTypes::TFunction(l, r) = right_type {
                    left_type  = self.unwrap_set(&*l);
                    right_type = self.unwrap_set(&*r);
                } else {
                    fatal!(TypeError,
                        lhs_operands
                            .last().unwrap()
                            .site().with_filename(&self.filename),
                        "Function definition provided with too many arguments.
                         The type signature disagrees with the number
                         of arguments you have provided.")
                        .note("Consider removing this, or altering \
                               the type signature.")
                        .print();
                }
            } else {
                // TODO: Not an ident, that means we're
                // pattern matching.  This will need a general
                // implementation in the future.
            }
        }
        // Now the function scope is populated with the arguments.
        self.table_chain.push(function_scope); // Add the scope to the stack.
        // Type the right side of the equality:
        let typed_rhs = self.resolve_branch(&rhs);
        // Check if the RHS has the correct type.
        if typed_rhs.yield_type() == right_type {
            appl_0.operands[0] = typed_rhs;
        } else {
            // TODO: If the the types disagree, but the type is
            // a subset, just cast the type.  For now, it's only an error:
            issue!(TypeError, rhs.site().with_filename(&self.filename),
                "Right hand side of function definition does not agree \
                 with type signature.
                 Expected type of `{}', got `{}'.",
                &right_type, &typed_rhs.yield_type())
                .note("Either convert the value, or alter the type signature.")
                .print();
        }
        // The function scope is no longer in use.
        self.table_chain.pop();
    } else {
        // TODO: Pattern matching etc.

        issue!(ParseError,
            appl_1.operands[0].site().with_filename(&self.filename),
            "Cannot assign to `{}' structure.",
            appl_1.operands[0].node_type())
                .print();
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
                issue!(TypeError,
                    op_0.site().with_filename(&self.filename),
                    "Right of type annotation must be a set; \
                     instead got type of `{}'.", set_signature)
                        .print();
            }
        } else {
            issue!(ParseError,
                op_1.site().with_filename(&self.filename),
                "Left of `:` type annotator must be \
                 an identifier; found `{}'.", op_1.node_type())
                    .note("Has to be a variable.")
                    .print();
        }
    } else {
        issue!(ParseError,
            appl_1.site.with_filename(&self.filename),
            "No expression found left of `:`.")
                .print();
    }
}
}
