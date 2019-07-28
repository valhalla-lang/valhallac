use std::fmt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use snailquote::escape;

#[derive(Clone, Copy)]
pub struct Symbol<'a> {
    hash : u64,
    string : &'a str
}

fn hash_symbol(string : &str) -> u64 {
    let mut s = DefaultHasher::new();
    string.hash(&mut s);
    s.finish()
}

impl<'a> Symbol<'a> {
    pub fn new(string : &'a str) -> Self {
        Symbol {
            hash: hash_symbol(string),
            string
        }
    }
}

impl<'a> fmt::Display for Symbol<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.string)
    }
}

impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other : &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Clone, PartialEq)]
pub enum Element<'a> {
    ENatural(usize),
    EInteger(isize),
    EReal(f64),
    EString(String),
    ESymbol(Symbol<'a>),
}

impl<'a> Element<'a> {
    pub fn is_numeric(&self) -> bool {
        match *self {
            Element::ENatural(_)
            | Element::EInteger(_)
            | Element::EReal(_) => true,
            _ => false
        }
    }
}


impl<'a> fmt::Display for Element<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Element::ENatural(t) => format!("{: <5}  => (Nat)   ", t),
            Element::EInteger(t) => format!("{: <5}  => (Int)   ", t),
            Element::EReal(t)    => format!("{: <5}  => (Real)  ", if t.fract() == 0f64 { format!("{:.1}", t) } else { f64::to_string(t) }),
            Element::EString(t)  => format!("{: <5}  => (String)", format!("\"{}\"", escape(t))),
            Element::ESymbol(t)  => format!("{: <5}  => (Sym)   ", t.to_string()),
        };
        write!(f, "{}", s)
    }
}
