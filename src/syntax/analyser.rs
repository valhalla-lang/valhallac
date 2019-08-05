use super::ast;

/// Constant folding.
/// A static optimisation that relieves the runtime of having to perform
/// pre-computable trivial calculations, by doing them at compile time
/// instead.  This function takes a node and recurses down, looking
/// for arithmetic operations containing exactly two numeric type nodes
/// as operands, and performs the stated operation.
fn constant_fold(node : &ast::Nodes) -> Option<ast::Nodes> {
    if node.num().is_some() { return Some(node.clone()); }
    if node.call().is_some() && node.call().unwrap().is_binary() {
        let operation = node.call().unwrap().callee.call().unwrap().callee.ident();
        if let Some(op) = operation {
            match op.value.as_str() {
                "+" | "-" | "*" | "/" => (),
                _ => {
                    let mut new_call = *node.call().unwrap().callee.clone();
                    let mut new_op   = node.call().unwrap().operands[0].clone();

                    let maybe_call = constant_fold(&new_call);
                    let maybe_op   = constant_fold(&new_op);

                    if let Some(call) = maybe_call {
                        new_call = call;
                    }
                    if maybe_op.is_some() {
                        new_op = maybe_op.unwrap();
                    }
                    return Some(ast::CallNode::new(new_call, vec![new_op]));
                }
            }
            let right = node.call().unwrap().operands.get(0);
            let left = node.call().unwrap().callee.call().unwrap().operands.get(0);

            if left.is_none()
            || right.is_none()
            { return None; }

            let l_value;
            let r_value;

            if left.unwrap().num().is_some()
            && right.unwrap().num().is_some() {
                l_value = left.unwrap().num().unwrap().value;
                r_value = right.unwrap().num().unwrap().value;
            } else {
                let mut l = constant_fold(left.unwrap());
                let mut r = constant_fold(right.unwrap());
                if l.is_none() && r.is_none() { return None; }
                if l.is_some() {
                    r = Some(right.unwrap().clone());
                } else {
                    l = Some(left.unwrap().clone());
                }

                let foldl = constant_fold(&l.unwrap());
                let foldr = constant_fold(&r.unwrap());
                if foldl.is_none() || foldr.is_none() { return None; }

                l_value = foldl.unwrap().num().unwrap().value;
                r_value = foldr.unwrap().num().unwrap().value;
            }
            let value = match op.value.as_str() {
                "+" => l_value + r_value,
                "-" => l_value - r_value,
                "*" => l_value * r_value,
                "/" => {
                    if r_value == ast::Numerics::Natural(0) {
                        return Some(ast::CallNode::new(
                                ast::CallNode::new(ast::IdentNode::new("/"),
                                    vec![ast::Nodes::Num(ast::NumNode { value : l_value })]),
                                vec![ast::NumNode::new(0)]));
                    }
                    l_value / r_value
                },
                _ => return None
            };
            return Some(ast::Nodes::Num(ast::NumNode { value }));
        }
    }
    None
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

fn cast_strenght(st : &ast::StaticTypes) -> i32 {
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
                    if cast_strenght(&left_yield) != cast_strenght(&right_yield) {

                        let casting_right = cast_strenght(&left_yield) >  cast_strenght(&right_yield);
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
                    if cast_strenght(&left_yield) > cast_strenght(&right_yield) {
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
    }
    return node.to_owned();
}

pub fn replace(root : &mut ast::Root) {
    let length = root.branches.len();
    let mut i = 0;
    while i < length {
        { // START TOP-LEVEL CONSTANT FOLD
            let new = constant_fold(&root.branches[i]);
            if let Some(branch) = new {
                root.branches[i] = branch;
            }
        } // END TOP-LEVEL CONSTANT FOLD
        { // START TOP-LEVEL TYPE BALANCING
            let new = balance_types(&root.branches[i]);
            root.branches[i] = new;
        } // END TOP-LEVEL TYPE BALANCING
        i += 1;
    }
}