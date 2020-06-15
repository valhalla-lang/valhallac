use std::fmt;
use crate::site::Site;

#[cfg(feature="debug")]
use {
    snailquote::escape,
    unicode_width::UnicodeWidthStr,
    std::collections::VecDeque
};

/// # TODO: Use this.
/// Way of representing a level of indentation.
#[allow(dead_code)]
enum Indent {
    Tab,
    Spaces(u32),
}

/// Contains all possible types/classes of
/// lexiacal tokens.
#[derive(PartialEq, Clone)]
pub enum TokenType {
    /// Identifiers, variables, function names etc.
    Ident,
    /// Numerics, anything that directly represents a number.
    Num,
    /// Any operators, simular to idents but are lexed differently.
    Op,
    /// Symbols, they are like elements of C enums, they begin with a colon.
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
            Self::Ident  => "Identifier",
            Self::Num    => "Numeric",
            Self::Op     => "Operator",
            Self::Sym    => "Symbol",
            Self::Str    => "String",
            Self::LParen => "L-Paren",
            Self::RParen => "R-Paren",
            Self::LBrack => "L-Bracket",
            Self::RBrack => "R-Bracket",
            Self::LBrace => "L-Brace",
            Self::RBrace => "R-Brace",
            Self::LVec   => "L-Vector",
            Self::RVec   => "R-Vector",
            Self::Term   => "Terminator",
            Self::EOF    => "End-Of-File",
        };
        write!(f, "{}", printable)
    }
}

/// Token structure, an individual lexical token,
/// represented by its type/class, what it was written as
/// in the program, and its location in the code.
#[derive(Clone)]
pub struct Token {
    /// What type/class of token it is.
    pub class  : TokenType,
    /// What string the token matched with.
    pub string : String,
    /// Where the token is in the code.
    pub location : Site,
}

impl Token {
    /// Constructs a new Token structure.
    pub fn new(class : TokenType, string : &str, site : Site) -> Token {
        Token { class, string: String::from(string), location: site }
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
}

/// String representation of the token.
#[cfg(feature="debug")]
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut escaped = escape(&self.string.to_string()).into_owned();
        if !escaped.ends_with('"') {
            escaped = format!("\"{}\"", escaped);
        }

        write!(f, "[ {class}:{spaces1}{rep}{spaces2}{l}:{c} ({span}) ]",
            class=self.class, rep=escaped,
            spaces1=" ".repeat(12 - self.class.to_string().width()),
            spaces2=" ".repeat(30 - escaped.width()),
            l=self.location.location.line.unwrap(),
            c=self.location.location.column.unwrap(),
            span=self.location.location.columns.unwrap())
    }
}

/// Allows for a custom string representation for the
/// token-stream as a whole.
#[cfg(feature="debug")]
pub trait ShowStream {
    /// String representation of token-stream.
    fn to_string(&self) -> String;
}

#[cfg(feature="debug")]
impl ShowStream for VecDeque<Token> {
    fn to_string(&self) -> String {
        let lines : Vec<String> = self.iter().map(Token::to_string).collect();
        format!("[ {} ]", lines.join(",\n  "))
    }
}
