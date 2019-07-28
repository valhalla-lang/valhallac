use super::element;
use element::Element;

use super::instructions;
use instructions::{Instr, Operators};

use super::casts;

/// Gets the appropriate operator for the internal functions.
/// Assumes all args have equal type.
pub fn get_internal_op(ident : &str, args : Option<&Vec<Element>>) -> Option<Instr> {
    match ident {
        "+" => {
            if args.is_none() { return Some(Instr::Operator(Operators::U_ADD as u8)); }
            Some(Instr::Operator(match args.unwrap()[0] {
                Element::ENatural(_) => Operators::N_ADD,
                Element::EInteger(_) => Operators::I_ADD,
                Element::EReal(_)    => Operators::R_ADD,
                _                    => Operators::U_ADD
            } as u8))
        },
        "-" => {
            if args.is_none() { return Some(Instr::Operator(Operators::U_SUB as u8)); }
            Some(Instr::Operator(match args.unwrap()[0] {
                Element::ENatural(_) => Operators::N_SUB,
                Element::EInteger(_) => Operators::I_SUB,
                Element::EReal(_)    => Operators::R_SUB,
                _                    => Operators::U_SUB
            } as u8))
        },
        "*" => {
            if args.is_none() { return Some(Instr::Operator(Operators::U_MUL as u8)); }
            Some(Instr::Operator(match args.unwrap()[0] {
                Element::ENatural(_) => Operators::N_MUL,
                Element::EInteger(_) => Operators::I_MUL,
                Element::EReal(_)    => Operators::R_MUL,
                _                    => Operators::U_MUL
            } as u8))
        },
        "/" => {
            if args.is_none() { return Some(Instr::Operator(Operators::U_DIV as u8)); }
            Some(Instr::Operator(match args.unwrap()[0] {
                Element::ENatural(_) => Operators::N_DIV,
                Element::EInteger(_) => Operators::I_DIV,
                Element::EReal(_)    => Operators::R_DIV,
                _                    => Operators::U_DIV
            } as u8))
        }
        _ => None
    }
}