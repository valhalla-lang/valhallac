use super::token;
use super::ast;
use super::operators;


use super::super::err;

use token::{Token, TokenType};
use ast::{Nodes, Numerics};

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

    ignore_newline : bool
}

impl<'a> ParseEnvironment<'a> {
    pub fn new(stream : Vec<Token>, file : &'a str) -> Self {
        ParseEnvironment {
            root: ast::Root::new(),
            stream: stream,
            optable: operators::PrecedenceTable::new(),
            file,

            ignore_newline: false
        }
    }

    pub fn start(&mut self) {
        let mut current = self.stream.first();
        while current.is_some() && current.unwrap().class != TokenType::EOF {
            if current.unwrap().class == TokenType::Term {
                self.stream.remove(0);
                current = self.stream.get(0);
                continue;
            }
            let e = self.expr(0);
            self.root.branches.push(e);
            current = self.stream.get(0);
        }
    }

    fn skip_newlines(&mut self) {
        while !self.stream.is_empty() && self.stream[0].string == "\n" {
            self.stream.remove(0);
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
                            ast::CallNode::new(ast::IdentNode::new(&token.string), vec![])
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
                    self.stream.remove(0);
                    return ast::EmptyNode::new();
                }


                self.ignore_newline = true;
                self.skip_newlines();
                let expr = self.expr(0);
                self.skip_newlines();
                self.ignore_newline = false;
                self.expect(TokenType::RParen, self.stream.get(0));
                self.stream.remove(0);
                expr
            }
            _ => issue!(err::Types::ParseError, self.file, token,
                    "`{}` has no null-denotation.", token.class)
        }
    }

    fn expr(&mut self, right_prec : i32) -> Nodes {
        let mut popped = self.stream.remove(0);
        while !self.stream.is_empty() && self.ignore_newline && popped.string == "\n" {
            popped = self.stream.remove(0);
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
                self.stream.remove(0);
                continue;
            }
            if next == "\0" || next == "\n" || next == ")" { break; }

            let maybe_op = self.optable.lookup(next, 2);
            if let Some(op) = maybe_op {
                self.stream.remove(0);
                let cloned = operators::Operator::new(next, op.precedence, op.associativity, 2);
                left = self.left_den(left, cloned);
            } else {  // Function call.
                left = self.func_apply(left);
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
}

#[cfg(test)]
mod test {
    use super::*;

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