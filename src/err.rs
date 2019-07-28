use crate::syntax::token;

use std::fs;
use std::fmt;
use std::io::{BufRead, BufReader};

pub enum Types {
    LexError,
    ParseError,
    TypeError,
    CompError,
}

impl fmt::Display for Types {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Types::LexError   => "Lexicographical Error",
            Types::ParseError =>         "Grammar Error",
            Types::TypeError  =>          "Typing Error",
            Types::CompError  =>     "Compilation Error",
        };
        write!(f, "{}", printable)
    }
}

pub fn issue(class : Types, filename : &str, token : &token::Token,  message : &str) {
    let file = fs::File::open(filename).expect("Invalid filename for error message.");
    let line = BufReader::new(file).lines().nth((token.location.line - 1) as usize).unwrap().unwrap();

    eprintln!("{}", message);
    eprintln!(" ==> {class} in  (`{file}`:{line}:{col}):\n{space}|\n{line_str}| {stuff}",
        class=class, file=filename, line=token.location.line,
        col=token.location.col, space=" ".repeat(5),
        line_str=format!("{: >4} ", token.location.line), stuff=line);
    eprintln!("{space}|{: >offset$}",
        "^".repeat(token.location.span as usize), space=" ".repeat(5),
        offset=((token.location.col + token.location.span) as usize));
}

#[macro_export]
macro_rules! issue {
    ($type:path, $file:expr, $token:expr, $message:expr) => {
        {
            err::issue($type, $file, $token, $message);
            std::process::exit(1)
        }
    };
    ($type:path, $file:expr, $token:expr, $message:expr, $($form:expr),*) => {
        {
            err::issue($type, $file, $token, &format!($message, $($form),*));
            std::process::exit(1)
        }
    };
}

