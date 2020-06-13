use super::ast;
use ast::Nodes;

fn create_cast(node : &Nodes, cast : &ast::StaticTypes) -> Nodes {
    let to_type = match cast {
        ast::StaticTypes::TReal => ":Real",
        ast::StaticTypes::TInteger => ":Int",
        ast::StaticTypes::TNatural => ":Nat",
        _ => panic!(".is_number() must be broken.")
    };

    let mut cast_node = ast::CallNode::new(
        ast::CallNode::new(
            ast::IdentNode::new("cast", node.site()),
            vec![node.clone()],
            node.site()),
        vec![ast::SymNode::new(to_type, node.site())],
        node.site());
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

                        let casting_right = cast_strength(&left_yield) > cast_strength(&right_yield);
                        let mut cast_to = (if casting_right { &left } else { &right }).yield_type();
                        if cast_to == ast::StaticTypes::TNatural
                        && bin_op.value == "-" {
                            cast_to = ast::StaticTypes::TInteger;
                        }

                        let mut new_call = if casting_right {
                            ast::CallNode::new(
                                *call.callee.clone(),
                                vec![create_cast(&right, &cast_to)],
                                call.callee.site())
                        } else {
                            ast::CallNode::new(
                                ast::CallNode::new(
                                    *call.callee.call().unwrap().callee.clone(),
                                    vec![create_cast(&left, &cast_to)],
                                    call.callee.site()),
                                vec![right],
                                call.site.clone())
                        };
                        if let Nodes::Call(ref mut c) = new_call {
                            c.set_return_type(cast_to);
                        }
                        return new_call;
                    } else {
                        let mut cloned_node = node.clone();
                        let mut cast_to = right_yield;

                        if cast_to == ast::StaticTypes::TNatural
                        && bin_op.value == "-" {
                            cast_to = ast::StaticTypes::TInteger;
                        }

                        if let Nodes::Call(ref mut c) = cloned_node {
                            c.set_return_type(cast_to);
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
                    call.callee.site());
                if let Nodes::Call(ref mut c) = new_call {
                    c.set_return_type(left_yield);
                }
                return new_call;
            }
        }
        let mut non_bi = ast::CallNode::new(
            balance_types(&*call.callee),
            vec![balance_types(&call.operands[0])],
            call.callee.site());
        if let Nodes::Call(ref mut c) = non_bi {
            c.set_return_type(call.return_type.clone());
        }
        return non_bi;
    }
    return node.to_owned();
}

#[allow(non_upper_case_globals)]
pub static default : fn(&Nodes) -> Nodes = balance_types;
