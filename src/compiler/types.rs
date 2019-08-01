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
    base_type : Option<ast::StaticTypes>,
    elements : Vec<Element<'a>>,
    unions : Vec<Set<'a>>,
    intersections : Vec<Set<'a>>,
    difference : Vec<Set<'a>>,
    conditons : block::LocalBlock<'a>
}

impl<'a> Set<'a> {
    pub fn new(filename : &'a str, base_type : Option<ast::StaticTypes>) -> Self {
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
        if let Some(base) = &self.base_type {
            return match base {
                ast::StaticTypes::TNatural => is_elem!(e, Element::ENatural),
                ast::StaticTypes::TInteger => is_elem!(e, Element::EInteger),
                ast::StaticTypes::TReal    => is_elem!(e, Element::EReal),
                ast::StaticTypes::TSymbol  => is_elem!(e, Element::ESymbol),
                ast::StaticTypes::TString  => is_elem!(e, Element::EString),
                ast::StaticTypes::TFunction(o, r) => {
                    match e {
                        Element::ECode(code) => code.return_type == **r,
                        _ => false
                    }
                },

                ast::StaticTypes::TNil      => e == Element::ENil,
                _ => false
            };
        }
        false
    }
}