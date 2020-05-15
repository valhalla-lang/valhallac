/*!
 * Constant folding.
 * A static optimisation that relieves the runtime of having to perform
 * pre-computable trivial calculations, by doing them at compile time
 * instead.  This function takes a node and recurses down, looking
 * for arithmetic operations containing exactly two numeric type nodes
 * as operands, and performs the stated operation.
 */


use super::ast;
use ast::Nodes;

fn const_fold(node : &Nodes) -> Nodes {
    if let Nodes::Call(call) = node {
        if call.is_binary() {
            let bin_op = call.callee.call().unwrap().callee.ident().unwrap();
            let left  = const_fold(&call.callee.call().unwrap().operands[0]);
            let right = const_fold(&call.operands[0]);

            let def = Nodes::Call(ast::CallNode {
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
                            return def;
                        }
                        l_value / r_value
                    },
                    _ => {
                        return def;
                    }
                };
                return Nodes::Num(ast::NumNode { value, location: call.location });
            } else {
                return def;
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

#[allow(non_upper_case_globals)]
pub static default : fn(&Nodes) -> Nodes = const_fold;
