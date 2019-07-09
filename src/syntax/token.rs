use std::fmt;
use super::location;

pub enum TokenType {
    Ident,
    Num,
    Op,
    Sym,
    Str,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TokenType::Ident => "Identifier",
            TokenType::Num   => "Numeric",
            TokenType::Op    => "Operator",
            TokenType::Sym   => "Symbol",
            TokenType::Str   => "String"
        };
        write!(f, "{}", printable)
    }
}

pub struct Token {
    pub class  : TokenType,
    pub string : String,
    pub location : location::Loc,
}

impl Token {
    pub fn new(class : TokenType, string : &str, loc : location::Loc) -> Token {
        Token { class: class, string: String::from(string), location: loc }
    }

    pub fn to_string(&self) -> String {
        String::from(format!("[ {class}: \"{rep}\" ({l}, {c}) ]",
            class=self.class, rep=self.string,
            l=self.location.line, c=self.location.col))
    }
}

pub trait ShowStream { fn to_string(&self) -> String; }
impl ShowStream for Vec<Token> {
    fn to_string(&self) -> String {
        let lines : Vec<String> = self.into_iter().map(|t| t.to_string()).collect();
        format!("[ {} ]", lines.join("\n  "))
    }
}