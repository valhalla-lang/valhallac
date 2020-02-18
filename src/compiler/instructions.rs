use std::fmt;

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instr {
    Operator(u8),
    Operand(u16)
}

impl Instr {
    pub fn depth_delta(self, maybe_operand : Option<Instr>) -> isize {
        if let Instr::Operand(_) = self
        { panic!("An operand does not have an impact on stack depth."); }

        if let Some(instr_operand) = maybe_operand {
            if let Instr::Operand(operand) = instr_operand {
            if let Instr::Operator(code) = self {
                return match Operators::from_u8(code.to_owned()).unwrap() {
                    Operators::HALT        =>  0,
                    Operators::PUSH_CONST  =>  1,
                    Operators::PUSH_LOCAL  =>  1,
                    Operators::PUSH_SUPER  =>  1,
                    Operators::STORE_LOCAL => -1,
                    Operators::DUP_N       => operand as isize,
                    Operators::CAST        =>  0,
                    Operators::SET_LINE    =>  0,
                    _ => panic!("This type of opcode doesn't take operands.")
                };
            }}
        } else if let Instr::Operator(code) = self {
            match code {
                40..=56 => return -1,
                _ => ()
            }
            return match Operators::from_u8(code.to_owned()).unwrap() {
                Operators::POP    => -1,
                Operators::DUP    =>  1,
                Operators::SWAP   =>  0,
                Operators::CALL_1 => -1,
                Operators::CHECK_TYPE => -2,
                Operators::MAKE_FUNC  => -1,
                Operators::YIELD      => -1,
                Operators::RAW_PRINT  =>  0,
                Operators::NOP => 0,
                _ => panic!("This opcode must take an operand.")
            };
        }
        panic!("Uncovered opcode.")
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self {
            Instr::Operand(n) => format!("{: >4} (0x{:04x})\n", n, n),
            Instr::Operator(n) => {
                let op_str = &Operators::from_u8(*n).unwrap().to_string();
                if op_str.ends_with('\n') {
                    format!("(0x{:02x}):{}", n, op_str)
                } else {
                    format!("(0x{:02x}):{: <16}", n, op_str)
                }
            }
        };
        write!(f, "{}", s)
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Primitive, Clone, Copy)]
pub enum Operators {
    EOI         = 0,   // TAKES 0 OPERAND(s) (Not a proper operator)

    PUSH_CONST  = 1,   // TAKES 1 OPERAND(s)
    PUSH_LOCAL  = 2,   // TAKES 1 OPERAND(s)
    PUSH_SUPER  = 3,   // TAKES 1 OPERAND(s)
    POP         = 4,   // TAKES 0 OPERAND(s)
    STORE_LOCAL = 5,   // TAKES 1 OPERAND(s)
    DUP         = 6,   // TAKES 0 OPERAND(s)
    DUP_N       = 7,   // TAKES 1 OPERAND(s)
    SWAP        = 8,   // TAKES 0 OPERAND(s)
    CALL_1      = 9,   // TAKES 0 OPERAND(s)
    CHECK_TYPE  = 10,  // TAKES 0 OPERAND(s)
    CAST        = 11,  // TAKES 2 OPERAND(s) (2 operands, 1 out of 2 bytes for each)
    MAKE_FUNC   = 12,  // TAKES 0 OPERAND(s)
    YIELD       = 13,  // TAKES 0 OPERAND(s)
    RAW_PRINT   = 14,  // TAKES 0 OPERAND(s)

    N_ADD       = 40,  // TAKES 0 OPERAND(s)
    I_ADD       = 41,  // TAKES 0 OPERAND(s)
    R_ADD       = 42,  // TAKES 0 OPERAND(s)
    U_ADD       = 43,  // TAKES 0 OPERAND(s)
    CONCAT      = 44,  // TAKES 0 OPERAND(s)
    N_SUB       = 45,  // TAKES 0 OPERAND(s)
    I_SUB       = 46,  // TAKES 0 OPERAND(s)
    R_SUB       = 47,  // TAKES 0 OPERAND(s)
    U_SUB       = 48,  // TAKES 0 OPERAND(s)
    N_MUL       = 49,  // TAKES 0 OPERAND(s)
    I_MUL       = 50,  // TAKES 0 OPERAND(s)
    R_MUL       = 51,  // TAKES 0 OPERAND(s)
    U_MUL       = 52,  // TAKES 0 OPERAND(s)
    N_DIV       = 53,  // TAKES 0 OPERAND(s)
    I_DIV       = 54,  // TAKES 0 OPERAND(s)
    R_DIV       = 55,  // TAKES 0 OPERAND(s)
    U_DIV       = 56,  // TAKES 0 OPERAND(s)

    HALT        = 200, // TAKES 1 OPERAND(s)

    // Misc- / Meta-codes
    SET_LINE = 254,  // TAKES 1 OPERAND(s)
    NOP = 255,       // TAKES 0 OPERAND(s)
}

impl Operators {
    pub fn takes_operand(self) -> bool {
        match self {
            Self::HALT
            | Self::PUSH_CONST
            | Self::PUSH_LOCAL
            | Self::PUSH_SUPER
            | Self::STORE_LOCAL
            | Self::DUP_N
            | Self::CAST
            | Self::SET_LINE => true,
            _ => false
        }
    }
}

impl fmt::Display for Operators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self {
            Operators::EOI         => "EOI",

            Operators::HALT        => "HALT",
            Operators::PUSH_CONST  => "PUSH_CONST",
            Operators::PUSH_LOCAL  => "PUSH_LOCAL",
            Operators::PUSH_SUPER  => "PUSH_SUPER",
            Operators::POP         => "POP\n",
            Operators::STORE_LOCAL => "STORE_LOCAL",
            Operators::DUP         => "DUP\n",
            Operators::DUP_N       => "DUP_N",
            Operators::SWAP        => "SWAP\n",
            Operators::CALL_1      => "CALL_1\n",
            Operators::CHECK_TYPE  => "CHECK_TYPE\n",
            Operators::CAST        => "CAST",
            Operators::MAKE_FUNC   => "MAKE_FUNC\n",
            Operators::YIELD       => "YIELD\n",
            Operators::RAW_PRINT   => "RAW_PRINT\n",

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

            Operators::SET_LINE    => "SET_LINE",

            _ => "INVALID_OPCODE\n"
        };
        write!(f, "{}", s)
    }
}
