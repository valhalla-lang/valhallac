use super::ast;

fn constant_fold(node : &ast::Nodes) -> Option<ast::Nodes> {
    if node.num().is_some() { return Some(node.clone()); }
    if node.call().is_some() && node.call().unwrap().is_binary() {
        let operation = node.call().unwrap().callee.call().unwrap().callee.ident();
        if let Some(op) = operation {
            let right = node.call().unwrap().operands.get(0);
            let left = node.call().unwrap().callee.call().unwrap().operands.get(0);

            if left.is_none()
            || right.is_none()
            { return None; }

            let mut l_value = ast::Numerics::Natural(0);
            let mut r_value = ast::Numerics::Natural(0);

            if left.unwrap().num().is_some()
            && right.unwrap().num().is_some() {
                l_value = left.unwrap().num().unwrap().value;
                r_value = right.unwrap().num().unwrap().value;
            } else {
                let l = constant_fold(left.unwrap());
                let r = constant_fold(right.unwrap());
                if l.is_none() || r.is_none() { return None; }

                let foldl = constant_fold(&l.unwrap());
                let foldr = constant_fold(&r.unwrap());
                if foldl.is_none() || foldr.is_none() { return None; }

                l_value = foldl.unwrap().num().unwrap().value;
                r_value = foldr.unwrap().num().unwrap().value;
            }
            return Some(ast::Nodes::Num(ast::NumNode {
                value: match op.value.as_str() {
                    "+" => l_value + r_value,
                    "-" => l_value - r_value,
                    "*" => l_value * r_value,
                    "/" => l_value / r_value,
                    _ => ast::Numerics::Natural(0)
                }
            }));
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
            if let Some(nbranch) = new {
                root.branches[i] = nbranch;
            }
        } // END TOP-LEVEL CONSTANT FOLD
        i += 1;
    }
}