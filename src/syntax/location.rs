pub struct Loc {
    pub line : u32,
    pub col  : u32,
    pub span : u32,
}

pub fn new(line : u32, col : u32, span : u32) -> Loc {
    Loc {
        line: line,
        col:  col,
        span: span
    }
}

