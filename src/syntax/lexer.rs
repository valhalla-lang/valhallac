use super::token;
use token::{Token, TokenType};

use super::location;

use lazy_static::lazy_static;
use regex::Regex;

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

macro_rules! re {
    ($string:expr) => {
        Regex::new($string).unwrap()
    };
}

/// Extension allows first Regex match to be easily picked out
/// and returns Option<String> containing the string for the capture.
trait RegexExt {
    /// Gets first match in string.
    fn first_match(&self, string : &str) -> Option<String>;
}
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

/// All chars that may constitue an ident.
const IDENT_CHARS : &str = r"\p{L}\?!'\-_";

lazy_static! {
    static ref OP    : Regex = re!(r"\A([,\+\.\*\|\\/\&%\$\^\~<¬=@>\-]+|:{2,})");
    static ref IDENT : Regex = re!(&format!(r"\A([{id}][{id}\p{{N}}]*)", id=IDENT_CHARS));
    static ref SYM   : Regex = re!(&format!(r"\A(:[{id}\p{{N}}]+)", id=IDENT_CHARS));
    static ref NUM   : Regex = re!(r"\A(\-?(?:(?:0[xX][0-9a-f]+)|(?:0[bB][01]+)|(?:0[Oo][0-7]+)|(?:(?:[0-9]+(?:\.[0-9]+)?(?:e[\+\-]?[0-9]+)?))))");
}

macro_rules! try_match {
    ($stream:expr, $partial:expr,
     $reg:expr, $token_type:expr,
     $current_char:expr, $line:expr, $col:expr) => {
        if let Some(matched) = $reg.first_match($partial) {
            let span = matched.width() as u32;
            $stream.push(Token::new(
                $token_type, &matched,
                location::new($line, $col, span)));
            $current_char += matched.len();
            $col += span;
            continue;
        }
    };
}

/// Takes a piece of code (as a &str) and returns
/// the generated token-stream (as a Vec<Token>).
pub fn lex(string : &str) -> Vec<Token> {
    let mut token_stream : Vec<Token> = Vec::new();
    
    let mut current_char = 0;
    let string_size = string.bytes().count();

    let mut partial : &str;
    let mut line = 1;
    let mut col  = 1;

    while current_char < string_size {
        if let Some(slice) = &string.get(current_char..) {
            partial = slice;
        } else { // Not on boundary yet.
            current_char += 1;
            continue;
        }

        let maybe_vec = &partial.get(0..2).unwrap_or("");
        let vec_brack = match maybe_vec {
            &"[|" => Some(TokenType::LVec),
            &"|]" => Some(TokenType::RVec),
              _  => None
        };
        if let Some(tt) = vec_brack {
            token_stream.push(Token::new(
                tt, maybe_vec,
                location::new(line, col, 2)));
            col += 2;
            current_char += 2;
            continue;
        }

        if *maybe_vec == ": " {
            token_stream.push(Token::new(
                TokenType::Op, ":",
                location::new(line, col, 1)));
            col += 2;
            current_char += 2;
            continue;
        }

        let first_char = partial.chars().nth(0)
            .expect("Empty program was trying to be lexed."); // This should't happen.

        let single_char_token = match first_char {
            '(' => Some(TokenType::LParen),
            ')' => Some(TokenType::RParen),
            '[' => Some(TokenType::LBrack),
            ']' => Some(TokenType::RBrack),
            '{' => Some(TokenType::LBrace),
            '}' => Some(TokenType::RBrace),
            '\n' | ';' => Some(TokenType::Term),
             _  => None
        };

        if let Some(tt) = single_char_token {
            token_stream.push(Token::new(
                tt, &first_char.to_string(),
                location::new(line, col, 1)));
            if first_char == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            current_char += 1;
            continue;
        }

        if first_char == '"' {
            let mut contents = String::new();

            let mut eos = false;
            let mut i = 1;
            let old_col = col;
            while !eos {  // Spaghet
                if let Some(character) = partial.chars().nth(i) {
                    if character == '"' {
                        current_char += 1;
                        col += 1;
                        eos = true;
                    } else if character == '\\' {
                        if let Some(next) = partial.chars().nth(i + 1) {
                            let escaped : String = match next {
                               '\\' => String::from("\\"),
                                'r' => String::from("\r"),
                                'n' => String::from("\n"),
                                't' => String::from("\t"),
                                'b' => String::from("\x08"),
                                '0' => String::from("\0"),
                                'x' => {
                                    if let Some(code) = partial.get((current_char + 2)..(current_char + 4)) {
                                        i += 2;
                                        col += 2;
                                        current_char += 2;
                                        (u8::from_str_radix(code, 16).expect("Malformed hex.") as char).to_string()
                                    } else { String::new() }
                                }
                                c => c.to_string()
                            };
                            i += 1;
                            col += 1;
                            current_char += 1;
                            contents.push_str(&escaped);
                            continue;
                        } else {
                            eos = true;
                            // Error: Unexpected EOS!
                        }
                    } else {
                        contents.push(character);
                        i += 1;
                        col += character.width().unwrap_or(2) as u32;
                        current_char += character.len_utf8();
                        continue;
                    }
                } else {
                    eos = true;
                    // Error: Unexpected EOS!
                }
                i += 1;
                current_char += 1;
                col += 1;
            }
            token_stream.push(Token::new(
                TokenType::Str, &contents,
                location::new(line, old_col, col - old_col)));
            continue;
        }

        try_match!(token_stream, partial,
            NUM, TokenType::Num,
            current_char, line, col);

        try_match!(token_stream, partial,
            OP, TokenType::Op,
            current_char, line, col);

        try_match!(token_stream, partial,
            IDENT, TokenType::Ident,
            current_char, line, col);

        try_match!(token_stream, partial,
            SYM, TokenType::Sym,
            current_char, line, col);

        current_char += 1;
        if partial.is_char_boundary(0) { col += 1 }
    }

    token_stream.push(Token::new(
        TokenType::EOF, "\0",
        location::new(line, col, 1)));
    token_stream
}