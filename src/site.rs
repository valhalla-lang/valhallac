use std::path::PathBuf;

/// Location from within source.
#[derive(Clone, Copy)]
pub struct Location {
    /// Specific line (first line).
    pub line : Option<usize>,
    /// First column on first line.
    pub column : Option<usize>,
    /// Last column on the last line.
    pub last_column: Option<usize>,

    /// Number of lines it spans.
    pub lines : Option<usize>,
    /// Number of columns it spans (according to unicode character width).
    pub columns : Option<usize>,
    /// Number of bytes the selection spans (includes line-feeds).
    pub span : Option<usize>,

    /// Amount of bytes from the beginning of the file to the first column.
    pub byte_offset : Option<usize>,
}

/// Only to be used for fake nodes.
pub const NULL_LOCATION : Location = Location {
    line: None,
    column: None,
    last_column: None,
    lines: None,
    columns: None,
    span: None,
    byte_offset: None,
};

/// Describes exactly where the source comes from.
#[derive(Clone)]
pub struct Site {
    /// Source may or may not come from a file.
    pub path : Option<PathBuf>,
    /// Source is from a REPL instance.
    pub repl : bool,

    /// The specific piece of source / AST node may come from
    ///  a specific column and line from within the file.
    pub location : Location,

    /// Is the node in a real location?
    pub fake : bool,
}

pub const FAKE_SITE : Site = Site {
    path: None,
    repl: false,
    location: NULL_LOCATION,
    fake: true,
};

impl Location {
    /// Last line in selection.
    #[inline]
    pub fn last_line(&self) -> Option<usize> {
        match self.line {
            Some(line) => if let Some(lines) = self.lines {
                Some(line + lines - 1)
            } else { None }
            None => None
        }
    }

    /// Number of characters from the last column to the beginning of file,
    /// (offset in bytes from the top of file to last column).
    #[inline]
    pub fn eos(&self) -> Option<usize> {
        match self.byte_offset {
            Some(bof) => if let Some(span) = self.span {
                Some(bof + span)
            } else { None }
            None => None
        }
    }
}

impl std::default::Default for Location {
    fn default() -> Self { NULL_LOCATION }
}

impl Site {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single_line(line : usize, col : usize,
                       cols : usize, bytes : usize, ptr : usize) -> Self {
        let mut s = Self::default();
        s.location.line = Some(line);
        s.location.column = Some(col);
        s.location.columns = Some(cols);
        s.location.span = Some(bytes);
        s.location.lines = Some(1);
        s.location.last_column = Some(col + cols);
        s.location.byte_offset = Some(ptr);
        s
    }

    pub fn with_filename(&self, name : &str) -> Self {
        let mut s = self.clone();
        s.path = Some(PathBuf::from(name));
        s
    }
}

impl std::default::Default for Site {
    fn default() -> Self { FAKE_SITE }
}
