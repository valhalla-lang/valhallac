use std::fmt;

use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug)]
pub enum Instr {
    Operator(u8),
    Operand(u16)
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self {
            Instr::Operand(n) => format!("{: >5}\n", n),
            Instr::Operator(n) => {
                format!("({:08b}):{}", n, Operators::from_u8(*n).unwrap())
            }
        };
        write!(f, "{}", s)
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Primitive)]
pub enum Operators {
    HALT        = 0,
    PUSH_CONST  = 1,
    PUSH_LOCAL  = 2,
    PUSH_SUPER  = 3,
    POP         = 4,
    STORE_LOCAL = 5,
    DUP         = 6,
    DUP_N       = 7,
    SWAP        = 8,

    N_ADD       = 40,
    I_ADD       = 41,
    R_ADD       = 42,
    U_ADD       = 43,
    CONCAT      = 44,
    N_SUB       = 45,
    I_SUB       = 46,
    R_SUB       = 47,
    U_SUB       = 48,
    N_MUL       = 49,
    I_MUL       = 50,
    R_MUL       = 51,
    U_MUL       = 52,
    N_DIV       = 53,
    I_DIV       = 54,
    R_DIV       = 55,
    U_DIV       = 56,
}


impl fmt::Display for Operators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self {
            Operators::HALT        => "HALT",
            Operators::PUSH_CONST  => "PUSH_CONST",
            Operators::PUSH_LOCAL  => "PUSH_LOCAL",
            Operators::PUSH_SUPER  => "PUSH_SUPER",
            Operators::POP         => "POP\n",
            Operators::STORE_LOCAL => "STORE_LOCAL",
            Operators::DUP         => "DUP\n",
            Operators::DUP_N       => "DUP_N",
            Operators::SWAP        => "SWAP\n",

            Operators::N_ADD       => "N_ADD\n",
            Operators::I_ADD       => "I_ADD\n",
            Operators::R_ADD       => "R_ADD\n",
            Operators::U_ADD       => "U_ADD\n",
            Operators::CONCAT      => "CONCAT\n",

            Operators::N_SUB       => "N_SUB\n",
            Operators::I_SUB       => "I_SUB\n",
            Operators::R_SUB       => "R_SUB\n",
            Operators::U_SUB       => "U_SUB\n",

            Operators::N_MUL       => "N_MUL\n",
            Operators::I_MUL       => "I_MUL\n",
            Operators::R_MUL       => "R_MUL\n",
            Operators::U_MUL       => "U_MUL\n",

            Operators::N_DIV       => "N_DIV\n",
            Operators::I_DIV       => "I_DIV\n",
            Operators::R_DIV       => "R_DIV\n",
            Operators::U_DIV       => "U_DIV\n",

            _ => "INVALID_OPCODE\n"
        };
        write!(f, "{}", s)
    }
}
