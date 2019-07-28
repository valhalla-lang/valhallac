use std::fmt;
use super::location;

use snailquote::escape;
use unicode_width::UnicodeWidthStr;

/// Contains all possible types/classes of
/// lexiacal tokens.
#[derive(PartialEq)]
pub enum TokenType {
    /// Identifiers, variables, function names etc.
    Ident,
    /// Numerics, anything that directly represents a number.
    Num,
    /// Any operators, simular to idents but are lexed differently.
    Op,
    /// Symbols, they are like elements of enums, they begin with a colon.
    Sym,
    /// Strings, enclosed by double quotes ("...").
    Str,
    /// Left Parenthesis.
    LParen,
    /// Rigt Parenthesis.
    RParen,
    /// Left Square Bracket.
    LBrack,
    /// Right Square Bracket.
    RBrack,
    /// Left curly-brace.
    LBrace,
    /// Right curly-brace.
    RBrace,
    /// Left vector-list bracket.
    LVec,
    /// Right vector-list bracket.
    RVec,
    /// Terminator, something that ends a line.
    /// Either a semi-colon (;) or a new-line (\n).
    Term,
    /// End Of File, last token in the stream.
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TokenType::Ident  => "Identifier",
            TokenType::Num    => "Numeric",
            TokenType::Op     => "Operator",
            TokenType::Sym    => "Symbol",
            TokenType::Str    => "String",
            TokenType::LParen => "L-Paren",
            TokenType::RParen => "R-Paren",
            TokenType::LBrack => "L-Bracket",
            TokenType::RBrack => "R-Bracket",
            TokenType::LBrace => "L-Brace",
            TokenType::RBrace => "R-Brace",
            TokenType::LVec   => "L-Vector",
            TokenType::RVec   => "R-Vector",
            TokenType::Term   => "Terminator",
            TokenType::EOF    => "End-Of-File",
        };
        write!(f, "{}", printable)
    }
}

/// Token structure, an individual lexiacal token,
/// represented by its type/class, what it was written as
/// in the program, and its location in the code.
pub struct Token {
    /// What type/class of token it is.
    pub class  : TokenType,
    /// What string the token matched with.
    pub string : String,
    /// Where the token is in the code.
    pub location : location::Loc,
}

impl Token {
    /// Constructs a new Token structure.
    pub fn new(class : TokenType, string : &str, loc : location::Loc) -> Token {
        Token { class, string: String::from(string), location: loc }
    }

    /// Checks if the token represents an atomic datum.
    pub fn is_atomic(&self) -> bool {
        match self.class {
            TokenType::Ident
            | TokenType::Num
            | TokenType::Op
            | TokenType::Sym
            | TokenType::Str => true,
            _ => false,
        }
    }

    /// String representation of the token.
    pub fn to_string(&self) -> String {
        let mut escaped = escape(&self.string.to_string()).into_owned();
        if !escaped.ends_with('"') {
            escaped = format!("\"{}\"", escaped);
        }

        format!("[ {class}:{spaces1}{rep}{spaces2}({l}, {c}):{span} ]",
            class=self.class, rep=escaped,
            spaces1=" ".repeat(12 - self.class.to_string().width()),
            spaces2=" ".repeat(50 - escaped.width()),
            l=self.location.line, c=self.location.col,
            span=self.location.span)
    }
}

/// Allows for a custom string representation for the
/// token-stream as a whole.
pub trait ShowStream {
    /// String representation of token-stream.
    fn to_string(&self) -> String;
}
impl ShowStream for Vec<Token> {
    fn to_string(&self) -> String {
        let lines : Vec<String> = self.iter().map(Token::to_string).collect();
        format!("[ {} ]", lines.join(",\n  "))
    }
}
