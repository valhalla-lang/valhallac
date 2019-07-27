use std::fmt;

pub struct Symbol<'a> {
    hash : u32,
    string : &'a str
}

impl<'a> fmt::Debug for Symbol<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.string)
    }
}

impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other : &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(PartialEq, Debug)]
pub enum Element<'a> {
    ENatural(usize),
    EInteger(isize),
    EReal(f64),
    EString(String),
    ESymbol(Symbol<'a>),
}