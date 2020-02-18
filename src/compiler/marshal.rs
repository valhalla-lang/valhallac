/*!
 * NOTES:
 * - No top level bytes should be `0x00`.
 *   This includes constant specifiers, operators (operands are OK),
 *   etc.  0x00 should be reserved as a terminator for certain
 *   strings and blocks.
 *
 * Compiled Bytecode Format:
 * ```ignore
 *  | VERSION [u8; 3]
 *  | MARSHALLED CODE BLOCK:
 *  |  | source-filename [u8; x] (abs path, null terminated, utf8)
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
 *  |  | (block end: 0x00 (EOI))
 * ```
 */
use std::collections::HashMap;

use super::element;
use super::instructions;
use super::block;

use element::Element;
use instructions::Instr;

// This ain't gonna be fun.


/// Gives each type a specifier prefix to identify them.
fn constant_ident_prefix(element : &Element) -> u8 {
    return match element {
        Element::ENatural(_) => 0x01,
        Element::EInteger(_) => 0x02,
        Element::EReal(_)    => 0x03,
        Element::EString(_)  => 0x04,
        _ => panic!("No byte-ident for this constant type")
    } as u8;
}

macro_rules! num_marshal_append {
    ($num:expr, $arr:expr) => {
        {
            // Split to big-endian byte-vector.
            let mut split = $num.to_be_bytes().to_vec();

            // Ignore leading zeros.
            let mut i = 0u8;
            for byte in &split {
                if *byte != 0 { break; }
                i += 1;
            }
            split = split[i as usize..].to_vec();

            $arr.push(split.len() as u8);
            $arr.append(&mut split);
        }
    };
}
/*
 * Constant marshalling:
 *
 * [...] = one (1) byte.
 * [TSP] = Type Specifier Prefix
 *
 * For numbers:
 *  `[TPS] [NUM OF BYTES (n)] [BYTE 1] [BYTE 2] ... [BYTE n]`
 *         \_______________________________________________/
 * For strings:                       |  These are the same concept.
 *         /￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣\
 *  `[TPS] [NUM OF SIZE BYTES (n)] [SIZE BYTE 1]...[SIZE BYTE n] [CHAR 1]...[CHAR m]`
 *                                 \_____size of string (m)____/
 */
fn marshal_element(element : &Element) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    match element {
        Element::ENatural(n) => {
            bytes.push(constant_ident_prefix(element));
            num_marshal_append!(n, bytes);
        },
        Element::EInteger(i) => {
            bytes.push(constant_ident_prefix(element));
            num_marshal_append!(i, bytes);
        },
        Element::EReal(r) => {
            bytes.push(constant_ident_prefix(element));
            num_marshal_append!(r, bytes);
        },
        Element::EString(s) => {
            let s_bytes = s.as_bytes().to_vec();
            let s_bytes_len = s.len();
            bytes.push(constant_ident_prefix(element));
            num_marshal_append!(s_bytes_len, bytes);
            bytes.extend(s_bytes);
        }
        _ => panic!("I do not know how to marshal this type.")
    };
    bytes
}

fn marshal_instructions(instrs : &[Instr]) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    for instr in instrs {
        match *instr {
            Instr::Operator(o) => bytes.push(o),
            Instr::Operand(o)  => bytes.extend(vec![(o >> 8) as u8, o as u8])
        };
    }
    bytes
}

fn marshal_consts(consts : &[Element]) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    for element in consts {
        bytes.extend(marshal_element(element));
    }
    bytes
}

fn marshal_locals(locals : &HashMap<String, u16>) -> Vec<u8> {
    let mut strings : Vec<Vec<u8>> = vec![vec![0x00]; locals.len()];
    for key in locals.keys() {
        let mut string = key.as_bytes().to_vec();
        string.push(0x00);
        strings[locals[key] as usize] = string;
    }
    strings.into_iter().flatten().collect()
}

pub fn marshal_block(blk : &block::LocalBlock) -> Vec<u8> {
    let instrs = marshal_instructions(&blk.instructions);
    let consts = marshal_consts(&blk.constants);
    let locals = marshal_locals(&blk.locals_map);
    let source_name =  blk.filename.to_owned();

    let mut bytes : Vec<u8> = vec![];
    // Null-terminated file name.
    bytes.extend(source_name.as_bytes());
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

    bytes
}

pub fn generate_binary(blk : &block::LocalBlock) -> Vec<u8> {
    let mut bytes = crate::VERSION.to_vec();
    bytes.extend(marshal_block(blk));

    print!("Bytes:\n  ");
    let mut i = 1;
    for byte in &bytes {
        print!("{:02x} ", byte);
       if i % 16 == 0 { print!("\n  ") };
        i += 1;
    }
    println!();

    bytes
}
