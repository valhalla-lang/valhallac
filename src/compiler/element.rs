use std::fmt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use snailquote::escape;

use super::block;
use super::types;

#[derive(Clone)]
pub struct Symbol {
    hash : u64,
    string : String
}

fn hash_symbol(string : &str) -> u64 {
    let mut s = DefaultHasher::new();
    string.hash(&mut s);
    s.finish()
}

impl Symbol {
    pub fn new(s : &str) -> Self {
        Symbol {
            hash: hash_symbol(s),
            string: s.to_owned()
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.string)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other : &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Clone, PartialEq)]
pub enum Element<'a> {
    ENatural(usize),
    EInteger(isize),
    EReal(f64),
    EString(&'a str),
    ESymbol(Symbol),
    ECode(Box<block::LocalBlock<'a>>),
    ESet(Box<types::Set<'a>>),
    ENil
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
            Element::ENatural(t) => format!("{: <13} (Nat) ", t),
            Element::EInteger(t) => format!("{: <13} (Int) ", t),
            Element::EReal(t)    => format!("{: <13} (Real)", if t.fract() == 0f64 { format!("{:.1}", t) } else { f64::to_string(t) }),
            Element::EString(t)  => format!("{: <13} (Str) ", format!("\"{}\"", escape(t))),
            Element::ESymbol(t)  => format!("{: <13} (Sym) ", t.to_string()),
            Element::ECode(t)    => format!("{: <13} (Code)", t.name),
            Element::ESet(t)     => format!("{: <13p} (Set) ", t),
            Element::ENil        => format!("{: <13} (Nil) ", "nil"),
        };
        write!(f, "{}", s)
    }
}
