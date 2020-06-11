/*!
 * Analyse the syntax tree, assign types, cast types, and
 * perform a battery of optimisations.
 */

use std::collections::HashSet;
use super::ast;

mod type_resolver;
mod type_balancer;
mod type_checker;
mod constant_fold;


#[macro_export]
macro_rules! transformations {
    ( $( $trans:ident ),* ) => {
        {
            let mut set = HashSet::new();
            $(
                set.insert(crate::syntax::analysis::Transform::$trans);
            )*
            set
        }
    };
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash)]
pub enum Transform {
    // Tree Typing
    TYPE_RESOLUTION, TYPE_PROPAGATION,
    TYPE_INFERENCE, TYPE_BALANCING,
    TYPE_CHECKING,
    // Simplification and Optimisation
    BETA_REDUCTION, CONSTANT_FOLDING,
    DEAD_CODE_ELIMINATION, TAIL_CALL,
    LOOP_INVARIANT_CODE_MOTION,
    COMMON_SUBEXPRESSION_ELIMINATION,
    LOOP_FISSION, LOOP_FUSION,
    LOOP_UNROLLING
}

pub fn replace(root : &mut ast::Root, transforms : HashSet<Transform>) {
    let mut checker_context = type_checker::TypeChecker::new();
    let mut resolution_context = type_resolver::ResolutionContext::new();

    let length = root.branches.len();
    let mut i = 0;


    while i < length {
        if transforms.contains(&Transform::TYPE_RESOLUTION) {
            let new = resolution_context.resolve_branch(&root.branches[i]);
            root.branches[i] = new;
        }
        if transforms.contains(&Transform::TYPE_CHECKING) {
            let new = checker_context.type_branch(&root.branches[i]);
            root.branches[i] = new;
        }
        if transforms.contains(&Transform::CONSTANT_FOLDING) {
            let new = constant_fold::default(&root.branches[i]);
            root.branches[i] = new;
        }
        i += 1;
    }
}
