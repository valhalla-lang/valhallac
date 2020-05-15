#![allow(non_camel_case_types)]
#![allow(clippy::pub_enum_variant_names)]

use crate::syntax::{token::Token, location::Loc};

use std::fs;
use std::fmt;
use std::io::{BufRead, BufReader};

use colored;
use colored::*;

use unindent::unindent;

pub struct LINE;
pub struct LOC;

pub enum IssueType {
    LexError, LexWarn,
    ParseError, ParseWarn,
    TypeError, TypeWarn,
    CompError, CompWarn
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            IssueType::LexError   => "Lexicographical Error".red(),
            IssueType::ParseError =>         "Grammar Error".red(),
            IssueType::TypeError  =>          "Typing Error".red(),
            IssueType::CompError  =>     "Compilation Error".red(),
            IssueType::LexWarn    => "Lexicographical Warning".yellow(),
            IssueType::ParseWarn  =>         "Grammar Warning".yellow(),
            IssueType::TypeWarn   => "         Typing Warning".yellow(),
            IssueType::CompWarn   => "    Compilation Warning".yellow(),
        };
        write!(f, "{}", printable)
    }
}

pub fn loc_issue(class : IssueType, filename : &str, loc : &Loc,  message : &str) {
    eprintln!();
    let file = fs::File::open(filename)
        .expect("Invalid filename for error message.");
    let line = BufReader::new(file).lines().nth((loc.line - 1) as usize)
        .expect(&format!("Line ({}) does not exist, file is too short.", loc.line))
        .expect("Could not get line.");

    let formatted = unindent(message).split('\n').collect::<Vec<&str>>().join("\n  ");
    eprintln!("{}{} {}", "issue".bold().red(), ":".white(), formatted.bold());
    eprint!("{}", "".clear());
    eprintln!(" ==> {class} in (`{file}`:{line}:{col}):\n{space}|\n{line_str}| {stuff}",
        class=class.to_string().bold(), file=filename, line=loc.line,
        col=loc.col, space=" ".repeat(5),
        line_str=format!("{: >4} ", loc.line.to_string().bold()), stuff=line);
    eprintln!("{space}|{: >offset$}",
        "^".repeat(loc.span as usize).yellow().bold(), space=" ".repeat(5),
        offset=((loc.col + loc.span) as usize));
}

pub fn token_issue(class : IssueType, filename : &str, token : &Token,  message : &str) {
    loc_issue(class, filename, &token.location, message);
}

pub fn line_issue(class : IssueType, filename : &str, line_n : usize,  message : &str) {
    eprintln!();
    let file = fs::File::open(filename)
        .expect("Invalid filename for error message.");
    let line = BufReader::new(file).lines().nth((line_n - 1) as usize)
        .unwrap()
        .unwrap();

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
macro_rules! warn {
    ($type:ident, $file:expr, err::LINE, $line:expr, $message:expr) => {{
            err::line_issue(err::IssueType::$type,
                $file, $line, $message);
    }};
    ($type:ident, $file:expr, err::LINE, $line:expr, $message:expr, $($form:expr),*) => {{
            err::line_issue(err::IssueType::$type,
                $file, $line, &format!($message, $($form),*));
    }};
    ($type:ident, $file:expr, err::LOC, $loc:expr, $message:expr) => {{
            err::loc_issue(err::IssueType::$type,
                $file, $loc, $message);
    }};
    ($type:ident, $file:expr, err::LOC, $loc:expr, $message:expr, $($form:expr),*) => {{
            err::loc_issue(err::IssueType::$type,
                $file, $loc, &format!($message, $($form),*));
    }};
    ($type:ident, $file:expr, $token:expr, $message:expr) => {{
            err::token_issue(err::IssueType::$type,
                $file, $token, $message);
    }};
    ($type:ident, $file:expr, $token:expr, $message:expr, $($form:expr),*) => {{
            err::token_issue(err::IssueType::$type,
                $file, $token, &format!($message, $($form),*));
    }};
}

#[macro_export]
macro_rules! issue {
    ($type:ident, $($args:tt)*) => {{
        warn!($type, $($args)*);
        std::process::exit(1)
    }};
}
