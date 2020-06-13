use std::collections::VecDeque;

use super::token;
use super::ast;
use super::operators;

use crate::{issue, site};
use site::{Site, Location};


use token::{Token, TokenType};
use ast::Nodes;

fn location_range(loc_begin : &Location, loc_end : &Location) -> Location {
    let mut loc_final = loc_end.clone();

    if let Location {
        line: Some(ref mut line),
        lines: Some(ref mut lines),
        column: Some(ref mut column),
        columns: Some(ref mut columns),
        span: Some(ref mut span),
        byte_offset: Some(ref mut byte_offset),
        ..
    } = loc_final {
        *lines += loc_end.line.unwrap() - loc_begin.line.unwrap();
        *line = loc_begin.line.unwrap();
        *span = loc_end.eos().unwrap()
            - loc_begin.byte_offset.unwrap();
        *columns = loc_end.column.unwrap() + loc_end.columns.unwrap()
            - loc_begin.column.unwrap();
        *column = loc_begin.column.unwrap();
        *byte_offset = loc_begin.byte_offset.unwrap();
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
    site : Site,
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
            site: Site::single_line(1, 1, 1, 1, 0),
        }
    }

    pub fn start(&mut self) {
        self.root.branches.push(ast::FileNode::new(
            self.file.to_owned(), self.site.clone()));

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
        self.site = shifted.location.to_owned();
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
        let loc = token.location.to_owned();
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
                                        ast::IdentNode::new("flip", loc.to_owned()),
                                        vec![ast::IdentNode::new(&token.string, loc)],
                                        self.site.to_owned()),
                                    vec![self.expr(500)],
                                    self.site.to_owned())
                            } else {
                                ast::CallNode::new(
                                    ast::IdentNode::new(&token.string, loc),
                                    vec![self.expr(500)],
                                    self.site.to_owned())
                            }
                        }
                    };
                }
                ast::IdentNode::new(&token.string, loc)
            },
            TokenType::Num => ast::NumNode::new(&*token.string, loc),
            TokenType::Str => ast::StrNode::new( &token.string,  loc),
            TokenType::Sym => ast::SymNode::new( &token.string,  loc),
            TokenType::LParen => {
                let maybe_current = self.stream.get(0);
                if let Some(current) = maybe_current {
                    if current.class == TokenType::RParen {
                        self.shift();
                        let mut nil_loc = loc.clone();

                        if let Location {
                            span: Some(ref mut span),
                            columns: Some(ref mut columns),
                            ..
                        } = nil_loc.location {
                            *span += 1;
                            *columns += 1;
                        } else {
                            panic!("All tokens should have width.");
                        }

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
            _ => {
                issue!(ParseError, token.location.with_filename(self.file),
                    "`{}` has no null-denotation.",
                    token.class)
                        .note("Cannot be used as a prefix / left-of-expression.")
                        .print();
                ast::NilNode::new(loc)
            }
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
        let first_site = left.site().clone();

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
        let last_site = operand_node.site();
        let mut final_site = first_site.clone();
        final_site.location = location_range(
            &first_site.location,
              &last_site.location);

        ast::CallNode::new(left, vec![operand_node], final_site)
    }

    fn left_den(&mut self, left : Nodes, op : operators::Operator) -> Nodes {
        let left_site = left.site();
        let first_apply = ast::CallNode::new(
            ast::IdentNode::new(op.name, self.site.to_owned()),
            vec![left],
            self.site.to_owned());

        if self.stream[0].class == TokenType::RParen {
            return first_apply;
        }

        let right = self.expr(op.precedence
            - (if op.is_right() { 1 } else { 0 }));

        let mut call_site = left_site.clone();
        call_site.location = location_range(
            &left_site.location,
            &right.location());
        ast::CallNode::new(first_apply, vec![right], call_site)
    }

    fn expect(&self, tt : TokenType, maybe_t : Option<&Token>) {
        if maybe_t.is_none() {
            fatal!(ParseError,
                self.stream.iter()
                    .last().unwrap()
                    .location
                    .with_filename(self.file),
                "Unexpected end of stream.")
                    .print();
        }
        let t = maybe_t.unwrap();
        if t.class != tt {
            fatal!(ParseError, t.location.with_filename(self.file),
                "Unexpected token type: `{}`, expected: `{}`.", t.class, tt)
                    .print();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::Numerics;

    macro_rules! num_test {
        ($num:expr, $site:expr, $res:expr) => {
            assert_eq!(
                ast::NumNode::new($num, $site.clone()).num().unwrap().value,
                $res);
        };
    }

    #[test]
    fn numeric_parsing() {
        let l = Site::new();
        num_test!(2, l, Numerics::Natural(2usize));
        num_test!(2, l, Numerics::Natural(2usize));
        num_test!(2usize, l, Numerics::Natural(2usize));
        num_test!(2u32, l, Numerics::Natural(2usize));
        num_test!(2i32, l, Numerics::Natural(2usize));

        num_test!(-2, l, Numerics::Integer(-2isize));
        num_test!(-2i32, l, Numerics::Integer(-2isize));
        num_test!(-2isize, l, Numerics::Integer(-2isize));

        num_test!(-2.62, l, Numerics::Real(-2.62f64));
        num_test!(2.62, l, Numerics::Real(2.62f64));

        num_test!("2", l, Numerics::Natural(2));
        num_test!("325", l, Numerics::Natural(325));
        num_test!("0b01010110", l, Numerics::Natural(0b01010110));
        num_test!("0o721", l, Numerics::Natural(0o721));
        num_test!("0xfa", l, Numerics::Natural(0xfa));
        num_test!("0xf", l, Numerics::Natural(0xf));
        num_test!("2.672", l, Numerics::Real(2.672));
        num_test!("2.672e3", l, Numerics::Real(2672.0));
        num_test!("2.672e+16", l, Numerics::Real(2.672 * 10f64.powf(16f64)));
        num_test!("2.672e-10", l, Numerics::Real(2.672 * 10f64.powf(-10f64)));
        num_test!("67e-4", l, Numerics::Real(0.0067));
        num_test!("67e+10", l, Numerics::Natural(670000000000));
        num_test!("-2", l, Numerics::Integer(-2));
        num_test!("-325", l, Numerics::Integer(-325));
        num_test!("-0b01010110", l, Numerics::Integer(-0b01010110));
        num_test!("-0o721", l, Numerics::Integer(-0o721));
        num_test!("-0xfa", l, Numerics::Integer(-250));
        num_test!("-0xf", l, Numerics::Integer(-15));
        num_test!("-2.672", l, Numerics::Real(-2.672));
        num_test!("-2.672e3", l, Numerics::Real(-2672.0));
        num_test!("-2.672e+16", l, Numerics::Real(-26720000000000000.0));
        num_test!("-2.672e-10", l, Numerics::Real(-0.0000000002672));
        num_test!("-67e-4", l, Numerics::Real(-0.0067));
        num_test!("-67e+10", l, Numerics::Integer(-670000000000));

        let s : String = String::from("-6e12");
        let num = ast::NumNode::new(&*s, l);

        assert_eq!(num.num().unwrap().value, Numerics::Integer(-6000000000000));
    }
}
