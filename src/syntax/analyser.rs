use super::ast;


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

pub fn replace(root : &mut ast::Root) {
    let length = root.branches.len();
    let mut i = 0;
    while i < length {
        let node = &root.branches[i];
        { // START TOP-LEVEL CONSTANT FOLD
            let new = constant_fold(node);
            if let Some(branch) = new {
                root.branches[i] = branch;
            }
        } // END TOP-LEVEL CONSTANT FOLD
        i += 1;
    }
    println!("\n\n{}", root);
}