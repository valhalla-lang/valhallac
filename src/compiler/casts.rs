use super::element;
use element::Element;

use super::super::syntax;
use syntax::ast;

pub fn numerics_to_element<'a>(num : &ast::Numerics) -> Element<'a> {
    match num {
        ast::Numerics::Natural(n) => Element::ENatural(*n),
        ast::Numerics::Integer(n) => Element::EInteger(*n),
        ast::Numerics::Real(n)    => Element::EReal(*n)
    }
}

pub enum Casts {  // In order of cast strength.
    REAL,
    INT,
    NAT
}

macro_rules! conversion {
    ($arg:expr, $to:path, $base:ident) => {
        match $arg {
            Element::ENatural(n) => $to(*n as $base),
            Element::EInteger(n) => $to(*n as $base),
            Element::EReal(n)    => $to(*n as $base),
            _ => panic!("Internal error, tried to cast non-numeric to numeric.")
        };
    };
}

pub fn cast_to<'a> (cast : Casts, args : &Vec<Element>) -> Vec<Element<'a>> {
    let mut new_args : Vec<Element> = vec![];
    for arg in args {
        let new = match cast {
            Casts::REAL => conversion!(arg, Element::EReal,    f64),
            Casts::INT  => conversion!(arg, Element::EInteger, isize),
            Casts::NAT  => conversion!(arg, Element::ENatural, usize),
        };
        new_args.push(new);
    }
    new_args
}

pub fn try_cast<'a>(args : Vec<Element<'a>>) -> Option<Vec<Element<'a>>> {
    if args.iter().all(Element::is_numeric) {
        for arg in &args {
            let converted = match arg {
                Element::EReal(_)    => Some(cast_to(Casts::REAL, &args)),
                Element::EInteger(_) => Some(cast_to(Casts::INT,  &args)),
                _ => None
            };
            if let Some(v) = converted { return Some(v); }
        }
        return Some(cast_to(Casts::NAT, &args));
    }
    None
}