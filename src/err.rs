use crate::syntax::token;

use std::fs;
use std::fmt;
use std::io::{BufRead, BufReader};

use colored;
use colored::*;

use unindent::unindent;

#[allow(non_camel_case_types)]
pub struct NO_TOKEN;

#[allow(clippy::pub_enum_variant_names)]
pub enum ErrorType {
    LexError,
    ParseError,
    TypeError,
    CompError,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ErrorType::LexError   => "Lexicographical Error",
            ErrorType::ParseError =>         "Grammar Error",
            ErrorType::TypeError  =>          "Typing Error",
            ErrorType::CompError  =>     "Compilation Error",
        };
        write!(f, "{}", printable)
    }
}

pub fn tissue(class : ErrorType, filename : &str, token : &token::Token,  message : &str) {
    let file = fs::File::open(filename).expect("Invalid filename for error message.");
    let line = BufReader::new(file).lines().nth((token.location.line - 1) as usize).unwrap().unwrap();

    let formatted = unindent(message).split('\n').collect::<Vec<&str>>().join("\n  ");
    eprintln!("{}{} {}", "issue".bold().red(), ":".white(), formatted.bold());
    eprint!("{}", "".clear());
    eprintln!(" ==> {class} in (`{file}`:{line}:{col}):\n{space}|\n{line_str}| {stuff}",
        class=class.to_string().bold(), file=filename, line=token.location.line,
        col=token.location.col, space=" ".repeat(5),
        line_str=format!("{: >4} ", token.location.line.to_string().bold()), stuff=line);
    eprintln!("{space}|{: >offset$}",
        "^".repeat(token.location.span as usize), space=" ".repeat(5),
        offset=((token.location.col + token.location.span) as usize));
}

pub fn lissue(class : ErrorType, filename : &str, line_n : usize,  message : &str) {
    let file = fs::File::open(filename).expect("Invalid filename for error message.");
    let line = BufReader::new(file).lines().nth((line_n - 1) as usize).unwrap().unwrap();

    let formatted = unindent(message).split('\n').collect::<Vec<&str>>().join("\n  ");
    eprintln!("{}{} {}", "issue".bold().red(), ":".white(), formatted.bold());
    eprint!("{}", "".clear());
    eprintln!(" ==> {class} in (`{file}`:{line}):\n{space}|\n{line_str}| {stuff}",
        class=class.to_string().bold(), file=filename, line=line_n,
        space=" ".repeat(5),
        line_str=format!("{: >4} ", line_n.to_string().bold()), stuff=line);
    eprintln!("     |");
}

#[macro_export]
macro_rules! issue {
    ($type:ident, $file:expr, err::NO_TOKEN, $line:expr, $message:expr) => {
        {
            err::lissue(err::ErrorType::$type,
                $file, $line, $message);
            std::process::exit(1)
        }
    };
    ($type:ident, $file:expr, err::NO_TOKEN, $line:expr,$message:expr, $($form:expr),*) => {
        {
            err::lissue(err::ErrorType::$type,
                $file, $line, &format!($message, $($form),*));
            std::process::exit(1)
        }
    };
    ($type:ident, $file:expr, $token:expr, $message:expr) => {
        {
            err::tissue(err::ErrorType::$type,
                $file, $token, $message);
            std::process::exit(1)
        }
    };
    ($type:ident, $file:expr, $token:expr, $message:expr, $($form:expr),*) => {
        {
            err::tissue(err::ErrorType::$type,
                $file, $token, &format!($message, $($form),*));
            std::process::exit(1)
        }
    };
}

