use super::token;
use token::{Token, TokenType};

use super::location;

use lazy_static::lazy_static;
use regex::Regex;

macro_rules! re {
    ($string:expr) => {
        Regex::new($string).unwrap()
    };
}

trait RegexExt { fn first_match(&self, string : &str) -> Option<String>; }
impl RegexExt for Regex {
    fn first_match(&self, string : &str) -> Option<String> {
        let cap = self.captures(string);
        match cap {
            Some(c) => {
                match c.get(1) {
                    Some(m) => Some(String::from(m.as_str())),
                    None => None
                }
            },
            None => None
        }
    }
}

const IDENT_CHARS : &str = r"\p{L}\?\!\'\-\_";

lazy_static! {
    static ref OP    : Regex = re!(r"\A([\+\.\*\|\\/\&%\$\^\~><=¬@\-]+)");
    static ref IDENT : Regex = re!(&format!(r"\A([{id}][{id}\p{{N}}]+)", id=IDENT_CHARS));
    static ref NUM   : Regex = re!(r"\A(\-?(?:(?:[0-9]+(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?)|(?:0x[0-9a-f]+)|(?:0b[01]+)|(?:0o[0-7]+)))");
}

macro_rules! try_match {
    ($stream:expr, $partial:expr,
     $reg:expr, $token_type:expr,
     $current_char:expr, $line:expr, $col:expr) => {
        if let Some(matched) = $reg.first_match($partial) {
            let span = matched.chars().count() as u32;
            $stream.push(Token::new(
                $token_type, &matched,
                location::new($line, $col, span)
            ));
            $current_char += matched.len();
            $col += span;
            continue;
        }
    };
}

pub fn lex(string : String) -> Vec<Token> {
    let mut token_stream : Vec<Token> = Vec::new();
    
    let mut current_char = 0;
    let string_size = string.len();

    let mut partial : &str;
    let mut line = 1;
    let mut col  = 1;

    while current_char < string_size {
        partial = &string[current_char..];

        try_match!(token_stream, partial,
            NUM, TokenType::Num,
            current_char, line, col);

        try_match!(token_stream, partial,
            OP, TokenType::Op,
            current_char, line, col);

        if partial.chars().nth(0).unwrap() == '\n' {
            line += 1;
            col = 1;
            current_char += 1;
            continue;
        }        
        current_char += 1;
        if partial.is_char_boundary(0) { col += 1 }
    }
    token_stream
}