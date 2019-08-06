use super::ast;

/// Constant folding.
/// A static optimisation that relieves the runtime of having to perform
/// pre-computable trivial calculations, by doing them at compile time
/// instead.  This function takes a node and recurses down, looking
/// for arithmetic operations containing exactly two numeric type nodes
/// as operands, and performs the stated operation.
fn const_fold(node : &ast::Nodes) -> ast::Nodes {
    if let ast::Nodes::Call(call) = node {
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
                return ast::Nodes::Num(ast::NumNode { value });
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


fn create_cast(node : &ast::Nodes, cast : &ast::StaticTypes) -> ast::Nodes {
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
    if let ast::Nodes::Call(ref mut call) = cast_node {
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
fn balance_types(node : &ast::Nodes) -> ast::Nodes {
    if let ast::Nodes::Call(call) = node {
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
                        if let ast::Nodes::Call(ref mut c) = new_call {
                            c.set_return_type(cast_to);
                        }
                        return new_call;
                    } else {
                        let mut cloned_node = node.clone();
                        if let ast::Nodes::Call(ref mut c) = cloned_node {
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
                        if let ast::Nodes::Call(ref mut c) = new_call {
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
        if let ast::Nodes::Call(ref mut c) = non_bi {
            c.set_return_type(node.yield_type());
        }
        return non_bi;
    }
    return node.to_owned();
}

pub fn replace(root : &mut ast::Root) {
    let length = root.branches.len();
    let mut i = 0;
    while i < length {
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