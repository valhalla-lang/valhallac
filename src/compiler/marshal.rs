/*!
 * NOTES:
 * - No top level bytes should be `0x00`.
 *   This includes constant specifiers, operators (operands are OK),
 *   etc.  0x00 should be reserved as a terminator for strings and
 *   blocks.
 *
 * Compiled Bytecode Format:
 * ```
 *  | VERSION [u8; 3]
 *  | MARSHALLED CODE BLOCK:
 *  |  | filename [u8; x] (abs path, null terminated, utf8)
 *  |  | module-name [u8; x] (null terminated, utf8)
 *  |  | stack-depth [u8; 2]
 *  |  |
 *  |  | CONSTANTS [u8; x] (block begin: 0x11 byte)
 *  |  |     (can contain other marshalled code blocks)
 *  |  | (block end: 0x00)
 *  |  | LOCAL NAMES [u8; x] (block begin: 0x12)
 *  |  |     (contains null terminated strings)
 *  |  | (block end: 0x00)
 *  |  | INSTRUCTION CODES [u8; x] (block begin: 0x13)
 *  |  |     (contains stream of operators and operands)
 *  |  | (block end: 0x00)
 * ```
!*/
use std::fs::File;
use std::io::{Write, Error};
use std::collections::HashMap;

use super::element;
use super::instructions;
use super::block;

use element::{Element, Symbol};
use instructions::{Instr, Operators};

// This ain't gonna be fun.

fn mk_bin_file(name : &str, bytes : Vec<u8>) -> File {
    let mut file = File::create(name).expect("Could not create binary.");
    file.write(&bytes.as_ref()).expect("Could not write to file.");
    file
}

fn constant_ident(element : &Element) -> u8 {
    return match element {
        Element::ENatural(_) => 0x01,
        Element::EInteger(_) => 0x02,
        Element::EReal(_)    => 0x03,
        _ => panic!("No byte-ident for this constant type")
    } as u8;
}

macro_rules! num_marshal_append {
    ($num:expr, $arr:expr) => {
        {
            let mut split = $num.to_be_bytes().to_vec();
            $arr.push(split.len() as u8);
            $arr.append(&mut split);
        }
    };
}
/*
 * Number marshaling:
 * ```
 * # [...] = one byte.
 * [NUM TYPE] [BYTE LEN] [BYTE 1] [BYTE 2] ... [BYTE n - 1] [BYTE n]
 * ```
 */
fn marshal_element(element : &Element) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    match element {
        Element::ENatural(n) => {
            bytes.push(constant_ident(element));
            num_marshal_append!(n, bytes);
        },
        Element::EInteger(i) => {
            bytes.push(constant_ident(element));
            num_marshal_append!(i, bytes);
        },
        Element::EReal(r) => {
            bytes.push(constant_ident(element));
            num_marshal_append!(r, bytes);
        },
        _ => ()
    };
    bytes
}

fn marshal_instructions(instrs : &Vec<Instr>) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    for instr in instrs {
        match *instr {
            Instr::Operator(o) => bytes.push(o),
            Instr::Operand(o)  => bytes.extend(vec![(o >> 8) as u8, o as u8])
        };
    }
    bytes
}

fn marshal_consts(consts : &Vec<Element>) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    for element in consts {
        bytes.extend(marshal_element(element));
    }
    bytes
}

fn marshal_locals(locals : &HashMap<String, u16>) -> Vec<u8> {
    let mut strings : Vec<Vec<u8>> = Vec::with_capacity(locals.len());
    strings = vec![vec![0x00]; locals.len()];
    for key in locals.keys() {
        let mut string = key.as_bytes().to_vec();
        string.push(0x00);
        strings[locals[key] as usize] = string;
    }
    strings.into_iter().flatten().collect()
}

pub fn make_binary(blk : &block::LocalBlock) -> String {
    let instrs = marshal_instructions(&blk.instructions);
    let consts = marshal_consts(&blk.constants);
    let locals = marshal_locals(&blk.locals_map);

    let mut filename =  blk.filename.to_owned();
    if filename.ends_with(".vh") {
        filename = filename[0..filename.len() - 3].to_owned();
    }
    let mut bytes : Vec<u8> = vec![];
    // Version number [u8; 3].
    bytes.extend(&crate::VERSION);
    // Null-terminated file name.
    bytes.extend(filename.as_bytes());
    bytes.push(0x00);
    // Null-terminated module name.
    bytes.extend(blk.name.as_bytes());
    bytes.push(0x00);
    // Stack depth [u8; 2].
    bytes.extend(&(blk.stack_depth as u16).to_be_bytes());
    // Constants.
    bytes.push(0x11); // Begin constants block.
    bytes.extend(consts);
    bytes.push(0x00);
    // Locals.
    bytes.push(0x12); // Begin locals block.
    bytes.extend(locals);
    bytes.push(0x00);
    // Instructions.
    bytes.push(0x13);
    bytes.extend(instrs);
    bytes.push(0x00);
    let file = mk_bin_file(&filename, bytes);

    filename
}