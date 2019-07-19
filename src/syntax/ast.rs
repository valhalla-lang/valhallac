use std::fmt;

/// Identifiers, node representing a name that
/// will represent a value stored.
pub struct IdentNode {
    /// The name of the identifer.
    pub value : String
}

/// Different types of possible number types in the langauge.
/// Max size is determined by max pointer size.
#[derive(PartialEq, Debug)]
pub enum Numerics {
    /// Naturals are unsigned ints.
    Natural(usize),
    /// Integers are signed.
    Integer(isize),
    /// Reals are represented as a double.
    Real(f64)
}

/// Parse a string of more than two chars with a specified radix, into an ast::Numeric.
fn parse_with_radix(neg : bool, s : &str, radix : u32) -> Numerics {
    let unsigned = usize::from_str_radix(s.get(2..).unwrap(), radix).unwrap();
    if neg {
        return Numerics::Integer(-(unsigned as isize));
    }
    return Numerics::Natural(unsigned);
}

/// Converts primitive types into ast::Numerics.
pub trait ToNumeric { fn to_numeric(&self) -> Numerics; }
impl ToNumeric for &str {
    fn to_numeric(&self) -> Numerics {
        let mut test_str = self.clone().to_ascii_lowercase();

        let is_neg = self.starts_with('-');
        if is_neg { test_str = test_str.get(1..).unwrap().to_string(); }

        return match test_str.get(0..2) {
            Some("0x") => parse_with_radix(is_neg, &test_str, 16),
            Some("0o") => parse_with_radix(is_neg, &test_str,  8),
            Some("0b") => parse_with_radix(is_neg, &test_str,  2),
            Some(_) => {
                let exp_notation : Vec<&str> = test_str.split('e').collect();
                let     mantissa : &str = exp_notation.get(0).unwrap();
                let mut exponent : &str = exp_notation.get(1).unwrap_or(&"0");
                if exponent.is_empty() { exponent = "0"; }
                let exponent : i32 = exponent.parse().unwrap();

                if mantissa.contains('.') || exponent < 0 {
                    let mut number = mantissa.parse::<f64>().unwrap() * 10f64.powi(exponent);
                    if is_neg { number *= -1f64; }
                    return Numerics::Real(number);
                }

                let number : usize = mantissa.parse().unwrap();
                if is_neg {
                    return Numerics::Integer(-(number as isize) * 10isize.pow(exponent as u32));
                }
                return Numerics::Natural(number * 10usize.pow(exponent as u32));
            }
            None => {
                if is_neg {
                    return Numerics::Integer(-test_str.parse::<isize>().unwrap());
                }
                Numerics::Natural(test_str.parse::<usize>().unwrap())
            }
        };
    }
}

impl ToNumeric for usize {
    fn to_numeric(&self) -> Numerics { Numerics::Natural(*self) }
}
impl ToNumeric for u32 {
    fn to_numeric(&self) -> Numerics { Numerics::Natural(*self as usize) }
}
impl ToNumeric for isize {
    fn to_numeric(&self) -> Numerics {
        if *self > 0 { return Numerics::Natural(*self as usize); }
        Numerics::Integer(*self)
    }
}
impl ToNumeric for i32 {
    fn to_numeric(&self) -> Numerics {
        if *self > 0 { return Numerics::Natural(*self as usize); }
        Numerics::Integer(*self as isize)
    }
}
impl ToNumeric for f64 {
    fn to_numeric(&self) -> Numerics { Numerics::Real(*self) }
}

impl fmt::Display for Numerics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            Numerics::Natural(n) => n.to_string(),
            Numerics::Integer(n) => n.to_string(),
            Numerics::Real(n)    => n.to_string(),
        };
        write!(f, "{}", printable)
    }
}

/// Node that represents a number.
pub struct NumNode {
    /// Holds a the numeric value.
    pub value : Numerics
}


/// Node for holding strings.
pub struct StrNode {
    /// Contents of the utf-8 string.
    pub value : String
}

/// Symbol Node.
pub struct SymNode {
    /// Value/name stored as a string and
    /// excludes the colon (:) in front.
    pub value : String
}

/// Call Node has a pointer to the callee node
/// and a list of operand nodes.
pub struct CallNode {
    /// Pointer to heap allocated calling node. 
    pub callee : Box<Nodes>,
    /// Pointer to list of operand nodes.
    pub operands : Vec<Nodes>
}

/// Represents a block of code / compound statements
/// in order of when they will be executed.
pub struct BlockNode {
    /// Pointer to list of nodes in the code block.
    pub statements : Vec<Nodes>
}

/// All node types.
pub enum Nodes {
    Ident(IdentNode),
    Num(NumNode),
    Str(StrNode),
    Sym(SymNode),
    Call(CallNode),
    Block(BlockNode)
}


impl fmt::Display for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            Nodes::Ident(node)  => format!("%ident{{ :value \"{}\" }}", node.value),
            Nodes::Num(node)    => format!("%num{{ :value {} }}", node.value),
            Nodes::Str(node)    => format!("%str{{ :value \"{}\" }}", node.value),
            Nodes::Sym(node)    => format!("%sym{{ :value \"{}\" }}", node.value),
            Nodes::Call(node)   => format!(
                "%call{{\n  :callee ({})\n  :operands [|\n    {}\n  |]\n}}", node.callee,
                node.operands.iter().map(Nodes::to_string).collect::<Vec<String>>().join("\n    ")),
            Nodes::Block(node)  => format!("%block{{ ... }}"),
        };
        write!(f, "{}", printable)
    }
}

macro_rules! unwrap_enum {
    ($e:expr, $m:path) => {
        match $e {
            $m(inner) => Some(&inner),
            _ => None
        }
    };
}


impl Nodes {
    pub fn ident(&self) -> Option<&IdentNode> { unwrap_enum!(self, Nodes::Ident) }
    pub fn   num(&self) -> Option<&NumNode>   { unwrap_enum!(self, Nodes::Num)   }
    pub fn   str(&self) -> Option<&StrNode>   { unwrap_enum!(self, Nodes::Str)   }
    pub fn   sym(&self) -> Option<&SymNode>   { unwrap_enum!(self, Nodes::Sym)   }
    pub fn  call(&self) -> Option<&CallNode>  { unwrap_enum!(self, Nodes::Call)  }
    pub fn block(&self) -> Option<&BlockNode> { unwrap_enum!(self, Nodes::Block) }

    pub fn is_atomic(&self) -> bool {
        match self {
            Nodes::Ident(_)  => true,
            Nodes::Num(_)    => true,
            Nodes::Str(_)    => true,
            Nodes::Sym(_)    => true,
            Nodes::Call(_)   => false,
            Nodes::Block(_)  => false,
        }
    }
}

impl IdentNode {
    pub fn new(value : &str) -> Nodes { Nodes::Ident(IdentNode { value: value.to_string() }) }
}

impl NumNode {
    pub fn new<Num : ToNumeric>(number : Num) -> Nodes {
        let value = number.to_numeric();
        Nodes::Num(NumNode { value })
    }
}

impl StrNode {
    pub fn new(value : &str) -> Nodes { Nodes::Str(StrNode { value: value.to_string() }) }
}

impl SymNode {
    pub fn new(value : &str) -> Nodes { Nodes::Sym(SymNode { value: value.to_string() }) }
}

impl CallNode {
    pub fn new(callee : Nodes, operands : Vec<Nodes>) -> Nodes {
        Nodes::Call(CallNode {
            callee: Box::new(callee),
            operands: operands,
        })
    }
}

/// Root branch of the AST.
pub struct Root {
    pub branches : Vec<Nodes>
}

impl Root {
    pub fn new() -> Self {
        Root { branches: Vec::new() }
    }
}

const TAB : &str = "  "; 

pub fn pretty_print(node : &Nodes, depth : usize) -> String {
    let tab = TAB.repeat(depth);
    let printable = match node {
            Nodes::Ident(_)  => format!("{}{}", tab, node),
            Nodes::Num(_)    => format!("{}{}", tab, node),
            Nodes::Str(_)    => format!("{}{}", tab, node),
            Nodes::Sym(_)    => format!("{}{}", tab, node),
            Nodes::Call(n)   => format!(
                "{tab}%call{{\n{tab}{T}:callee (\n{calling}\n{tab}{T})\n{tab}{T}:operands [|\n{ops}\n{tab}{T}|]\n{tab}}}",
                tab=tab, T=TAB,
                calling=pretty_print(&*n.callee, depth + 2),
                ops=n.operands.iter().map(|e| pretty_print(e, depth + 2)).collect::<Vec<String>>().join("\n")
            ),
            Nodes::Block(n)  => format!("%block{{ ... }}"),
    };
    printable
}


impl fmt::Display for Root {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_mapped : Vec<String> = self.branches.iter().map(|n| pretty_print(n, 0)).collect();
        write!(f, "[|\n  {}\n|]", str_mapped.join("\n").split("\n").collect::<Vec<&str>>().join("\n  "))
    }
}