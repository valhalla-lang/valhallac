use super::element;
use element::Element;

use super::block;

use super::super::syntax::ast;

macro_rules! is_elem {
    ($e:expr, $elem:path) => {
        {
            match $e {
                $elem(_) => true,
                _ => false
            }
        }
    };
}

#[derive(Clone, PartialEq)]
pub struct Set<'a> {
    base_type : Option<ast::BaseTypes>,
    elements : Vec<Element<'a>>,
    unions : Vec<Set<'a>>,
    intersections : Vec<Set<'a>>,
    difference : Vec<Set<'a>>,
    conditons : block::LocalBlock<'a>
}

impl<'a> Set<'a> {
    pub fn new(filename : &'a str, base_type : Option<ast::BaseTypes>) -> Self {
        Self {
            base_type,
            elements: vec![],
            unions: vec![],
            intersections: vec![],
            difference: vec![],
            conditons : block::LocalBlock::new("<set-conditions>", filename)
        }
    }
    pub fn is_memeber(&self, e : Element) -> bool {
        if let Some(base) = self.base_type {
            return match base {
                ast::BaseTypes::TNatural => is_elem!(e, Element::ENatural),
                ast::BaseTypes::TInteger => is_elem!(e, Element::EInteger),
                ast::BaseTypes::TReal    => is_elem!(e, Element::EReal),
                ast::BaseTypes::TSym     => is_elem!(e, Element::ESymbol),
                ast::BaseTypes::TString  => is_elem!(e, Element::EString),
                ast::BaseTypes::TNil      => e == Element::ENil,
                _ => false
            };
        }
        false
    }
}