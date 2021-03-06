use crate::{issue, site::Site};

use super::token;
use token::{Token, TokenType};

use std::collections::VecDeque;
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

/// All chars that may constitute an ident.
const IDENT_CHARS : &str = r"\p{L}\?!'\-_";

// TODO: Parse symbols with spaces? `:"..."` syntax.
lazy_static! {
    static ref OP    : Regex = re!(r"\A([,\+\.\*\|\\/\&%\$\^\~<¬=@>\-]+|:{2,})");
    static ref IDENT : Regex = re!(&format!(r"\A([{id}][{id}\p{{N}}]*)", id=IDENT_CHARS));
    static ref SYM   : Regex = re!(r"\A(:[^\s]+)");
    static ref NUM   : Regex = re!(r"\A(\-?(?:(?:0[xX][0-9a-f]+)|(?:0[bB][01]+)|(?:0[Oo][0-7]+)|(?:(?:[0-9]+(?:\.[0-9]+)?(?:e[\+\-]?[0-9]+)?))))");
}

macro_rules! try_match {
    ($stream:expr, $partial:expr,
     $reg:expr, $token_type:expr,
     $current_char_ptr:expr, $line:expr, $col:expr) => {
        if let Some(matched) = $reg.first_match($partial) {
            let width = matched.width();
            let bytes = matched.len();
            $stream.push_back(Token::new(
                $token_type, &matched,
                Site::single_line($line, $col,
                    width, bytes, $current_char_ptr)));
            $current_char_ptr += bytes;
            $col += width;
            $stream.back()
        } else {
            None
        }
    };
}

/// Takes a piece of code (as a &str) and returns
/// the generated token-stream (as a VecDeque<Token>).
pub fn lex(string : &str, filename : &str) -> VecDeque<Token> {
    let mut token_stream : VecDeque<Token> = VecDeque::new();

    let mut current_char_ptr = 0;
    let string_size = string.bytes().count();

    let mut partial : &str;
    let mut line : usize = 1;
    let mut col  : usize = 1;

    // Step through
    while current_char_ptr < string_size {
        // Align to character boundary.
        if let Some(slice) = &string.get(current_char_ptr..) {
            partial = slice;
        } else { // Not on boundary yet.
            current_char_ptr += 1;
            continue;
        }

        let two_chars = partial.get(0..2).unwrap_or("\0\0");

        // Consume EON comment:
        if two_chars == "#!" || two_chars == "--" {
            let old_char_ptr = current_char_ptr;
            current_char_ptr += if two_chars == "--" { 2 } else { 1 };
            loop {
                let current_char = string.bytes()
                    .nth(current_char_ptr)
                    .unwrap_or(b'\0');
                if current_char == b'\n' || current_char == b'\0' {
                    break;
                }
                current_char_ptr += 1;
            }
            col += string.get(old_char_ptr..current_char_ptr)
                .expect("Comment ended or started not on char boundary.")
                .width();

            continue;
        }
        // TODO: Consume multi-line comments (`--* ... *--`).
        // TODO: Lex `with:` `let:` `in:` `where:` `do:`
        // indentation blocks.

        let vec_brack = match two_chars {
            "[|" => Some(TokenType::LVec),
            "|]" => Some(TokenType::RVec),
              _  => None
        };
        if let Some(tt) = vec_brack {
            token_stream.push_back(Token::new(
                tt, two_chars,
                Site::single_line(line, col,
                    2, 2, current_char_ptr)));
            col += 2;
            current_char_ptr += 2;
            continue;
        }

        if two_chars == ": " {
            token_stream.push_back(Token::new(
                TokenType::Op, ":",
                Site::single_line(line, col,
                    1, 2, current_char_ptr)));
            col += 2;
            current_char_ptr += 2;
            continue;
        }

        let first_char = partial.chars().nth(0)
            .expect("Empty program was trying to be lexed."); // This shouldn't happen.

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
            token_stream.push_back(Token::new(
                tt, &first_char.to_string(),
                Site::single_line(line, col,
                    1, 1, current_char_ptr)));
            if first_char == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            current_char_ptr += 1;
            continue;
        }

        if first_char == '"' {
            let mut contents = String::new();

            let mut eos = false;
            let mut i = 1;
            let old_col = col;
            let old_char_ptr = current_char_ptr;
            while !eos {  // Spaghetti
                if let Some(character) = partial.chars().nth(i) {
                    if character == '"' {
                        current_char_ptr += 1;
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
                                    if let Some(code) = partial
                                        .get((current_char_ptr + 2)
                                            ..(current_char_ptr + 4)) {
                                        i += 2;
                                        col += 2;
                                        current_char_ptr += 2;
                                        (u8::from_str_radix(code, 16)
                                            .expect("Malformed hex.") as char)
                                            .to_string()
                                    } else { String::new() }
                                }
                                c => c.to_string()
                            };
                            i += 1;
                            col += 1;
                            current_char_ptr += 1;
                            contents.push_str(&escaped);
                            continue;
                        } else {
                            eos = true;
                            // TODO Error: Unexpected EOS!
                        }
                    } else {
                        contents.push(character);
                        i += 1;
                        col += character.width().unwrap_or(2);
                        current_char_ptr += character.len_utf8();
                        continue;
                    }
                } else {
                    eos = true;
                    // TODO Error: Unexpected EOS!
                }
                i += 1;
                current_char_ptr += 1;
                col += 1;
            }
            token_stream.push_back(Token::new(
                TokenType::Str, &contents,
                Site::single_line(line, old_col,
                    col - old_col,
                    current_char_ptr - old_char_ptr,
                    old_char_ptr)));
            continue;
        }

        let matched = try_match!(token_stream, partial,
            NUM, TokenType::Num,
            current_char_ptr, line, col);
        if matched.is_some() { continue; }

        let matched = try_match!(token_stream, partial,
            OP, TokenType::Op,
            current_char_ptr, line, col);
        if matched.is_some() { continue; }

        let matched = try_match!(token_stream, partial,
            IDENT, TokenType::Ident,
            current_char_ptr, line, col);
        if matched.is_some() { continue; }

        let matched = try_match!(token_stream, partial,
            SYM, TokenType::Sym,
            current_char_ptr, line, col);
        if let Some(token) = matched {
            if two_chars == ":)" {
                issue!(LexWarn, token.location.with_filename(filename),
                    "Nice smiley-face, but are you sure you wanted to \
                     use a `Symbol' here?  Use `:\")\"` to be more explicit.")
                     .print();
            }
            continue;
        }

        current_char_ptr += 1;
        if partial.is_char_boundary(0) { col += 1 }
    }

    let mut last_location = Site::new();
    if let  Some(last_token) = token_stream.back() {
        last_location = last_token.location.to_owned();
    }

    token_stream.push_back(Token::new(
        TokenType::EOF, "\0",
        last_location));

    token_stream
}
