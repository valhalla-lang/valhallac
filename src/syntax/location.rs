/// Holds line, column and span of a lexical token.
#[derive(Clone)]
pub struct Loc {
    /// Line number.
    pub line : u32,
    /// Column number.
    pub col  : u32,
    /// Span/Width (in characters) of token.
    pub span : u32,
}

/// Construct new Loc structure.
pub fn new(line : u32, col : u32, span : u32) -> Loc {
    Loc { line, col, span }
}

