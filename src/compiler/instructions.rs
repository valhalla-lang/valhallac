#[derive(Debug)]
pub enum Instr {
    Operator(u8),
    Operand(u16)
}

#[repr(u8)]
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
}