use std::collections::VecDeque;

use super::location;
use super::token;
use super::ast;
use super::operators;

use super::super::err;

use location::Loc;

use token::{Token, TokenType};
use ast::Nodes;

fn location_range(loc_begin : &Loc, loc_end : &Loc) -> Loc {
    let mut loc_final = loc_end.clone();

    loc_final.lines += loc_end.line - loc_begin.line;
    loc_final.line = loc_begin.line;

    // TODO: Location should record character count from the
    // beginning of the file, this way we can give a proper multi line
    // span length. :)
    if loc_final.lines == 1 {
        loc_final.span += loc_end.col - loc_begin.col;
        loc_final.col = loc_begin.col;
    }

    loc_final
}

pub fn parse(stream : VecDeque<Token>, file : &str) -> ast::Root {
    let mut environment = ParseEnvironment::new(stream, file);
    environment.start();
    environment.root
}

struct ParseEnvironment<'a> {
    pub root : ast::Root,
    pub stream : VecDeque<Token>,
    pub optable : operators::PrecedenceTable<'a>,
    pub file : &'a str,

    ignore_newline : bool,
    location: Loc,
    eof_token : Token
}

impl<'a> ParseEnvironment<'a> {
    pub fn new(stream : VecDeque<Token>, file : &'a str) -> Self {
        ParseEnvironment {
            root: ast::Root::new(file),
            eof_token: stream.iter().last().unwrap().to_owned(),
            stream,
            optable: operators::PrecedenceTable::new(),
            file,

            ignore_newline: false,
            location: location::new(1, 1, 1),
        }
    }

    pub fn start(&mut self) {
        self.root.branches.push(ast::FileNode::new(self.file.to_owned(), self.location));

        let mut current = self.stream.get(0);
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
        self.shift();
    }

    fn shift(&mut self) -> Token {
        if self.stream.is_empty() {
            self.stream.push_back(self.eof_token.clone());
        }
        let shifted = self.stream.pop_front().unwrap();
        self.location = shifted.location;
        shifted
    }

    fn skip_newlines(&mut self) {
        while !self.stream.is_empty() && self.stream[0].string == "\n" {
            self.shift();
        }
    }

    // TODO: Generate call nodes with accurate location data.
    //  Currently this is only done in `func_apply`.

    fn null_den(&mut self, token : &Token) -> Nodes {
        let loc = token.location;
        match token.class {
            TokenType::Op | TokenType::Ident => {
                let is_op = self.optable.exists(&token.string);
                if is_op {
                    let prefix = self.optable.lookup(&token.string, 1);
                    return match self.stream[0].class {
                        TokenType::RParen => {
                            ast::IdentNode::new(&token.string, loc)
                        },
                        _ => {
                            // If the operator is suffix:
                            //   e.g. (a +)  <=>  ((+) a)
                            // Otherwise it's a partial application:
                            //   e.g. (* a)  <=>  ((flip (*)) a)
                            // But, prefix operators don't get flipped:
                            //   e.g. (- a)  <=> ((-) a)  <=>  -a
                            if prefix.is_none() {
                                ast::CallNode::new(
                                    ast::CallNode::new(
                                        ast::IdentNode::new("flip", loc),
                                        vec![ast::IdentNode::new(&token.string, loc)],
                                        self.location),
                                    vec![self.expr(500)],
                                    self.location)
                            } else {
                                ast::CallNode::new(
                                    ast::IdentNode::new(&token.string, loc),
                                    vec![self.expr(500)],
                                    self.location)
                            }
                        }
                    };
                }
                ast::IdentNode::new(&token.string, loc)
            },
            TokenType::Num => ast::NumNode::new(&*token.string, loc),
            TokenType::Str => ast::StrNode::new( &token.string, loc),
            TokenType::Sym => ast::SymNode::new( &token.string, loc),
            TokenType::LParen => {
                let maybe_current = self.stream.get(0);
                if let Some(current) = maybe_current {
                    if current.class == TokenType::RParen {
                        self.shift();
                        let mut nil_loc = loc.clone();
                        nil_loc.span += 1;
                        return ast::NilNode::new(nil_loc);
                    } else if current.class == TokenType::EOF {
                        self.expect(TokenType::RParen, maybe_current);
                    }
                } else {
                    self.expect(TokenType::RParen, None);
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
            _ => issue!(ParseError, self.file, token,
                    "`{}` has no null-denotation (cannot be used as a prefix).",
                    token.class)
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
        return left;
    }

    fn func_apply(&mut self, mut left : Nodes) -> Nodes {
        // What are `first_loc` & `final_loc` for?
        //  They update location of function call nodes to span
        //  a correct number of columns (store first and last column).
        //  Also, update `lines` to specify how many lines the function
        //  call spans.
        let first_loc = left.location();

        let mut pushed = false;
        if let Nodes::Call(ref mut call) = left {
            if call.operands.is_empty() {
                let operand_node = self.expr(190);
                call.operands.push(operand_node);
                pushed = true;
            }
        }

        if pushed { return left; }

        let operand_node = self.expr(190);
        let last_loc = operand_node.location();
        let final_loc = location_range(&first_loc, &last_loc);

        ast::CallNode::new(left, vec![operand_node], final_loc)
    }

    fn left_den(&mut self, left : Nodes, op : operators::Operator) -> Nodes {
        let left_loc = left.location();
        let first_apply = ast::CallNode::new(
            ast::IdentNode::new(op.name, self.location),
            vec![left],
            self.location);
        if self.stream[0].class == TokenType::RParen {
            return first_apply;
        }

        let right = self.expr(op.precedence
            - (if op.is_right() { 1 } else { 0 }));

        let call_loc = location_range(&left_loc, &right.location());
        ast::CallNode::new(first_apply, vec![right], call_loc)
    }

    fn expect(&self, tt : TokenType, maybe_t : Option<&Token>) {
        if maybe_t.is_none() {
            issue!(ParseError, self.file, self.stream.iter().last().unwrap(),
                "Unexpected end of stream.");
        }
        let t = maybe_t.unwrap();
        if t.class != tt {
            issue!(ParseError, self.file, t,
                "Unexpected token type: `{}`, expected: `{}`.", t.class, tt);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::Numerics;

    #[test]
    fn numeric_parsing() {
        let l = location::new(1, 1, 1);
        assert_eq!(ast::NumNode::new(2, l).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2usize, l).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2u32, l).num().unwrap().value, Numerics::Natural(2usize));
        assert_eq!(ast::NumNode::new(2i32, l).num().unwrap().value, Numerics::Natural(2usize));

        assert_eq!(ast::NumNode::new(-2, l).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2i32, l).num().unwrap().value, Numerics::Integer(-2isize));
        assert_eq!(ast::NumNode::new(-2isize, l).num().unwrap().value, Numerics::Integer(-2isize));

        assert_eq!(ast::NumNode::new(-2.62, l).num().unwrap().value, Numerics::Real(-2.62f64));
        assert_eq!(ast::NumNode::new(2.62, l).num().unwrap().value, Numerics::Real(2.62f64));

        assert_eq!(ast::NumNode::new("2", l).num().unwrap().value, Numerics::Natural(2));
        assert_eq!(ast::NumNode::new("325", l).num().unwrap().value, Numerics::Natural(325));
        assert_eq!(ast::NumNode::new("0b01010110", l).num().unwrap().value, Numerics::Natural(0b01010110));
        assert_eq!(ast::NumNode::new("0o721", l).num().unwrap().value, Numerics::Natural(0o721));
        assert_eq!(ast::NumNode::new("0xfa", l).num().unwrap().value, Numerics::Natural(0xfa));
        assert_eq!(ast::NumNode::new("0xf", l).num().unwrap().value, Numerics::Natural(0xf));
        assert_eq!(ast::NumNode::new("2.672", l).num().unwrap().value, Numerics::Real(2.672));
        assert_eq!(ast::NumNode::new("2.672e3", l).num().unwrap().value, Numerics::Real(2672.0));
        assert_eq!(ast::NumNode::new("2.672e+16", l).num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(16f64)));
        assert_eq!(ast::NumNode::new("2.672e-10", l).num().unwrap().value, Numerics::Real(2.672 * 10f64.powf(-10f64)));
        assert_eq!(ast::NumNode::new("67e-4", l).num().unwrap().value, Numerics::Real(0.0067));
        assert_eq!(ast::NumNode::new("67e+10", l).num().unwrap().value, Numerics::Natural(670000000000));
        assert_eq!(ast::NumNode::new("-2", l).num().unwrap().value, Numerics::Integer(-2));
        assert_eq!(ast::NumNode::new("-325", l).num().unwrap().value, Numerics::Integer(-325));
        assert_eq!(ast::NumNode::new("-0b01010110", l).num().unwrap().value, Numerics::Integer(-0b01010110));
        assert_eq!(ast::NumNode::new("-0o721", l).num().unwrap().value, Numerics::Integer(-0o721));
        assert_eq!(ast::NumNode::new("-0xfa", l).num().unwrap().value, Numerics::Integer(-250));
        assert_eq!(ast::NumNode::new("-0xf", l).num().unwrap().value, Numerics::Integer(-15));
        assert_eq!(ast::NumNode::new("-2.672", l).num().unwrap().value, Numerics::Real(-2.672));
        assert_eq!(ast::NumNode::new("-2.672e3", l).num().unwrap().value, Numerics::Real(-2672.0));
        assert_eq!(ast::NumNode::new("-2.672e+16", l).num().unwrap().value, Numerics::Real(-26720000000000000.0));
        assert_eq!(ast::NumNode::new("-2.672e-10", l).num().unwrap().value, Numerics::Real(-0.0000000002672));
        assert_eq!(ast::NumNode::new("-67e-4", l).num().unwrap().value, Numerics::Real(-0.0067));
        assert_eq!(ast::NumNode::new("-67e+10", l).num().unwrap().value, Numerics::Integer(-670000000000));

        let s : String = String::from("-6e12");
        let num = ast::NumNode::new(&*s, l);

        assert_eq!(num.num().unwrap().value, Numerics::Integer(-6000000000000));
    }
}
