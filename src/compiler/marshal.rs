use std::fs::File;
use std::io::{Write, Error};

use super::element;
use super::instructions;
use super::block;

use element::{Element, Symbol};
use instructions::{Instr, Operators};

// This ain't gonna be fun.


fn mk_bin_file(name : &str, bytes : Vec<u8>) -> File {
    let mut file = File::create(name).expect("Could not create binary.");
    file.write(&bytes.as_ref());
    file
}

fn marshal_instructions(instrs : &Vec<Instr>) -> Vec<u8> {
    let mut bytes : Vec<u8> = vec![];
    for instr in instrs {
        match *instr {
            Instr::Operator(o) => bytes.push(o),
            Instr::Operand(o)  => bytes.append(&mut vec![(o >> 8) as u8, o as u8])
        };
    }
    bytes
}

pub fn make_binary(blk : &block::LocalBlock) -> String {
    let instrs = marshal_instructions(&blk.instructions);
    let file = mk_bin_file(&blk.name, instrs);

    blk.name.to_owned()
}