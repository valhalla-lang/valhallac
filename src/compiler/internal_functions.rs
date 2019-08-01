use super::instructions;
use instructions::{Instr, Operators};

use super::super::syntax;
use syntax::ast;

/// Gets the appropriate operator for the internal functions.
/// Assumes all args have equal type.
pub fn get_internal_op(ident : &str, args : Option<&Vec<&ast::Nodes>>) -> Option<Instr> {
    let mut first = ast::StaticTypes::TUnknown;
    let mut is_uni = args.is_none();
    if !is_uni {
        let unwrapped = args.unwrap();
        first = unwrapped[0].yield_type();
        is_uni = !unwrapped.iter().all(|e| e.yield_type() == first);
    }

    match ident {
        "+" => {
            if is_uni { return Some(Instr::Operator(Operators::U_ADD as u8)); }

            Some(Instr::Operator(match first {
                ast::StaticTypes::TNatural => Operators::N_ADD,
                ast::StaticTypes::TInteger => Operators::I_ADD,
                ast::StaticTypes::TReal    => Operators::R_ADD,
                _                          => Operators::U_ADD
            } as u8))
        },
        "-" => {
            if is_uni { return Some(Instr::Operator(Operators::U_SUB as u8)); }

            Some(Instr::Operator(match first {
                ast::StaticTypes::TNatural => Operators::N_SUB,
                ast::StaticTypes::TInteger => Operators::I_SUB,
                ast::StaticTypes::TReal    => Operators::R_SUB,
                _                          => Operators::U_SUB
            } as u8))
        },
        "*" => {
            if is_uni { return Some(Instr::Operator(Operators::U_MUL as u8)); }

            Some(Instr::Operator(match first {
                ast::StaticTypes::TNatural => Operators::N_MUL,
                ast::StaticTypes::TInteger => Operators::I_MUL,
                ast::StaticTypes::TReal    => Operators::R_MUL,
                _                          => Operators::U_MUL
            } as u8))
        },
        "/" => {
            if is_uni { return Some(Instr::Operator(Operators::U_DIV as u8)); }

            Some(Instr::Operator(match first {
                ast::StaticTypes::TNatural => Operators::N_DIV,
                ast::StaticTypes::TInteger => Operators::I_DIV,
                ast::StaticTypes::TReal    => Operators::R_DIV,
                _                          => Operators::U_DIV
            } as u8))
        }
        _ => None
    }
}