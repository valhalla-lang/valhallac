use std::collections::{HashMap, VecDeque};

use super::token;
use super::ast;
use super::operators;

use super::super::err;

use token::{Token, TokenType};
use ast::Nodes;
use ast::StaticTypes as ST;

pub fn parse(stream : Vec<Token>, file : &str) -> ast::Root {
    let mut environment = ParseEnvironment::new(stream, file);
    environment.optable.new_fun("max", 4);

    environment.start();

    environment.root
}

struct ParseEnvironment<'a> {
    pub root : ast::Root,
    pub stream : Vec<Token>,
    pub optable : operators::PrecedenceTable<'a>,
    pub file : &'a str,
    pub annotations : VecDeque<ast::CallNode>,
    pub ident_types : HashMap<String, ast::StaticTypes>,

    ignore_newline : bool,
    line_number : usize,
}

impl<'a> ParseEnvironment<'a> {
    pub fn new(stream : Vec<Token>, file : &'a str) -> Self {
        ParseEnvironment {
            root: ast::Root::new(),
            stream: stream,
            optable: operators::PrecedenceTable::new(),
            annotations: VecDeque::new(),
            ident_types: HashMap::new(),
            file,

            ignore_newline: false,
            line_number: 0,
        }
    }

    pub fn start(&mut self) {
        self.root.branches.push(ast::FileNode::new(self.file.to_owned()));

        let mut current = self.stream.first();
        while current.is_some() && current.unwrap().class != TokenType::EOF {
            if current.unwrap().class == TokenType::Term {
                self.shift();
                current = self.stream.get(0);
                continue;
            }
            let e = self.expr(0);
            self.root.branches.push(e);
            current = self.stream.get(0);
        }
        self.assign_types();
    }

    fn get_type(&self, node : &ast::Nodes) -> ast::StaticTypes {
        if let Some(ident) = node.ident() {
            return match ident.value.as_str() {
                "Nat"  => ST::TSet(Box::new(ST::TNatural)),
                "Int"  => ST::TSet(Box::new(ST::TInteger)),
                "Real" => ST::TSet(Box::new(ST::TReal)),
                "Universal" => ST::TSet(Box::new(ST::TUnknown)),
                _ => ident.static_type.clone()
            };
        }
        node.yield_type()
    }

    fn remember_type(&mut self, k : String, v : ast::StaticTypes) {
        if self.ident_types.contains_key(&k) {
            self.ident_types.insert(k, ast::StaticTypes::TUnknown);
            return;
        }
        self.ident_types.insert(k, v);
    }

    fn shift(&mut self) -> Token {
        let shifted = self.stream.remove(0);
        if shifted.location.line as usize != self.line_number {
            if self.root.branches.last().is_some()
            && self.root.branches.last().unwrap().line().is_some() {
                self.root.branches.pop();
            }
            self.line_number = shifted.location.line as usize;
            self.root.branches.push(ast::LineNode::new(self.line_number));
        }
        shifted
    }

    fn skip_newlines(&mut self) {
        while !self.stream.is_empty() && self.stream[0].string == "\n" {
            self.shift();
        }
    }

    fn null_den(&mut self, token : &Token) -> Nodes {
        match token.class {
            TokenType::Op | TokenType::Ident => {
                let is_op = self.optable.exists(&token.string);
                if is_op {
                    let prefix = self.optable.lookup(&token.string, 1);
                    return match self.stream[0].class {
                        TokenType::RParen => {
                            ast::IdentNode::new(&token.string)
                        },
                        _ => {
                            if prefix.is_none() {
                                ast::CallNode::new(
                                    ast::CallNode::new(
                                        ast::IdentNode::new(&token.string),
                                        vec![ast::EmptyNode::new()]),
                                    vec![self.expr(500)])
                            } else {
                                ast::CallNode::new(
                                    ast::IdentNode::new(&token.string),
                                    vec![self.expr(500)])
                            }
                        }
                    };
                }
                ast::IdentNode::new(&token.string)
            },
            TokenType::Num => ast::NumNode::new(&*token.string),
            TokenType::Str => ast::StrNode::new(&token.string),
            TokenType::Sym => ast::SymNode::new(&token.string),
            TokenType::LParen => {
                let current = self.stream.get(0);
                if current.is_none() || current.unwrap().class == TokenType::EOF {
                    self.expect(TokenType::RParen, current)
                } else if current.unwrap().class == TokenType::RParen {
                    self.shift();
                    return ast::EmptyNode::new();
                }


                self.ignore_newline = true;
                self.skip_newlines();
                let expr = self.expr(0);
                self.skip_newlines();
                self.ignore_newline = false;
                self.expect(TokenType::RParen, self.stream.get(0));
                self.shift();
                expr
            }
            _ => issue!(err::Types::ParseError, self.file, token,
                    "`{}` has no null-denotation.", token.class)
        }
    }

    fn expr(&mut self, right_prec : i32) -> Nodes {
        let mut popped = self.shift();
        while !self.stream.is_empty() && self.ignore_newline && popped.string == "\n" {
            popped = self.shift();
        }
        let mut left = self.null_den(&popped);

        if self.ignore_newline { self.skip_newlines(); }
        if self.stream.is_empty()
            || self.stream[0].class == TokenType::EOF
            || self.stream[0].class == TokenType::Term
            { return left; }


        while self.optable.precedence(&self.stream[0].string).unwrap_or(190) > right_prec {
            let next = &(&self.stream[0].string).clone();

            if self.ignore_newline && next == "\n" {
                self.shift();
                continue;
            }
            if next == "\0" || next == "\n" || next == ")" { break; }

            let maybe_op = self.optable.lookup(next, 2);
            if let Some(op) = maybe_op {
                let cloned = operators::Operator::new(next, op.precedence, op.associativity, 2);
                self.shift();
                left = self.left_den(left, cloned);
            } else {  // Function call.
                left = self.func_apply(left);
            }
        }
        if left.call().is_none() || !left.call().unwrap().is_binary() { return left; }
        if let Some(call_ident) = left.call().unwrap().callee.call().unwrap().callee.ident() {
            if call_ident.value == ":" {
                self.annotations.push_back(left.call().unwrap().clone());
            }
            if call_ident.value == "=" {
                let maybe_annotation = self.annotations.pop_front();
                if let Some(annotation) = maybe_annotation {
                    let maybe_set = self.get_type(&annotation.operands[0]);
                    if let ast::StaticTypes::TSet(set) = maybe_set {
                        self.remember_type(
                            left.call().unwrap().callee.call().unwrap().operands[0].ident().unwrap().value.to_owned(),
                            *set);
                    } else {
                        // Error, annotation must be set.
                    }
                } else {
                    // Error, missing annotation for assignment.
                }
            }
        }
        return left;
    }

    fn func_apply(&mut self, mut left : Nodes) -> Nodes {
        let mut pushed = false;
        match left {
            Nodes::Call(ref mut call) => {
                if call.operands.is_empty() {
                    call.operands.push(self.expr(190));
                    pushed = true;
                }
            },
            _ => ()
        };
        if pushed { return left; }
        ast::CallNode::new(left, vec![self.expr(190)])
    }

    fn left_den(&mut self, left : Nodes, op : operators::Operator) -> Nodes {
        let first_apply = ast::CallNode::new(ast::IdentNode::new(op.name), vec![left]);
        if self.stream[0].class == TokenType::RParen {
            return first_apply;
        }
        let right = self.expr(op.precedence - (if op.is_right() { 1 } else { 0 }));
        ast::CallNode::new(first_apply, vec![right])
    }

    fn expect(&self, tt : TokenType, maybe_t : Option<&Token>) {
        if maybe_t.is_none() {
            issue!(err::Types::ParseError, self.file, self.stream.last().unwrap(),
                "Unexpected end of stream.");
        }
        let t = maybe_t.unwrap();
        if t.class != tt {
            issue!(err::Types::ParseError, self.file, t,
                "Unexpected token type: `{}`, expected: `{}`.", t.class, tt);
        }
    }



    fn assign_types(&mut self) {

        fn recurse_type_assign(subtree : &Nodes, map : &HashMap<String, ST>) -> Nodes {
            match subtree {
                Nodes::Ident(ident_node) => {
                    if map.contains_key(&ident_node.value) {
                        let mut cloned_ident = ident_node.clone();
                        cloned_ident.static_type = map[&ident_node.value].clone();
                        return Nodes::Ident(cloned_ident);
                    }
                },
                Nodes::Call(call_node) => {
                    let mut cloned_call = call_node.clone();
                    cloned_call.callee = Box::new(recurse_type_assign(&*call_node.callee, map));
                    cloned_call.operands = vec![recurse_type_assign(&call_node.operands[0], map)];
                    return Nodes::Call(cloned_call);
                },
                _ => ()
            };
            return subtree.to_owned();
        };

        let mut i = 0;
        let tree_size = self.root.branches.len();

        while i < tree_size {
            self.root.branches[i] = recurse_type_assign(&self.root.branches[i], &self.ident_types);
            i += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::Numerics;

    #[test]
    fn numeric_parsing() {
        assert_eq!(ast::NumNode::new(2).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2usize).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2u32).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2i32).num().unwrap().value, Numerics::Natural(2usize));

        assert_eq!(ast::NumNode::new(-2).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2i32).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2isize).num().unwrap().value, Numerics::Integer(-2isize));

        assert_eq!(ast::NumNode::new(-2.62).num().unwrap().value, Numerics::Real(-2.62f64));
        assert_eq!(ast::NumNode::new(2.62).num().unwrap().value, Numerics::Real(2.62f64));

        assert_eq!(ast::NumNode::new("2").num().unwrap().value, Numerics::Natural(2));
        assert_eq!(ast::NumNode::new("325").num().unwrap().value, Numerics::Natural(325));
        assert_eq!(ast::NumNode::new("0b01010110").num().unwrap().value, Numerics::Natural(0b01010110));
        assert_eq!(ast::NumNode::new("0o721").num().unwrap().value, Numerics::Natural(0o721));
        assert_eq!(ast::NumNode::new("0xfa").num().unwrap().value, Numerics::Natural(0xfa));
        assert_eq!(ast::NumNode::new("0xf").num().unwrap().value, Numerics::Natural(0xf));
        assert_eq!(ast::NumNode::new("2.672").num().unwrap().value, Numerics::Real(2.672));
        assert_eq!(ast::NumNode::new("2.672e3").num().unwrap().value, Numerics::Real(2672.0));
        assert_eq!(ast::NumNode::new("2.672e+16").num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(16f64)));
        assert_eq!(ast::NumNode::new("2.672e-10").num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(-10f64)));
        assert_eq!(ast::NumNode::new("67e-4").num().unwrap().value, Numerics::Real(0.0067));
        assert_eq!(ast::NumNode::new("67e+10").num().unwrap().value, Numerics::Natural(670000000000));
        assert_eq!(ast::NumNode::new("-2").num().unwrap().value, Numerics::Integer(-2));
        assert_eq!(ast::NumNode::new("-325").num().unwrap().value, Numerics::Integer(-325));
        assert_eq!(ast::NumNode::new("-0b01010110").num().unwrap().value, Numerics::Integer(-0b01010110));
        assert_eq!(ast::NumNode::new("-0o721").num().unwrap().value, Numerics::Integer(-0o721));
        assert_eq!(ast::NumNode::new("-0xfa").num().unwrap().value, Numerics::Integer(-250));
        assert_eq!(ast::NumNode::new("-0xf").num().unwrap().value, Numerics::Integer(-15));
        assert_eq!(ast::NumNode::new("-2.672").num().unwrap().value, Numerics::Real(-2.672));
        assert_eq!(ast::NumNode::new("-2.672e3").num().unwrap().value, Numerics::Real(-2672.0));
        assert_eq!(ast::NumNode::new("-2.672e+16").num().unwrap().value, Numerics::Real(-26720000000000000.0));
        assert_eq!(ast::NumNode::new("-2.672e-10").num().unwrap().value, Numerics::Real(-0.0000000002672));
        assert_eq!(ast::NumNode::new("-67e-4").num().unwrap().value, Numerics::Real(-0.0067));
        assert_eq!(ast::NumNode::new("-67e+10").num().unwrap().value, Numerics::Integer(-670000000000));

        let s : String = String::from("-6e12");
        let num = ast::NumNode::new(&*s);

        assert_eq!(num.num().unwrap().value, Numerics::Integer(-6000000000000));
    }
}