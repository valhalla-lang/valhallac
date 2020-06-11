use std::{fmt, ops};
use std::collections::VecDeque;

use super::location;
use location::Loc;

/// Identifiers, node representing a name that
/// will represent a value stored.
#[derive(Clone)]
pub struct IdentNode {
    /// The name of the identifier.
    pub value : String,

    /// Type it holds.
    pub static_type : StaticTypes,

    /// Source location.
    pub location : Loc,
}

/// Different types of possible number types in the language.
/// Max size is determined by max pointer size.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Numerics {
    /// Naturals are unsigned ints.
    Natural(usize),
    /// Integers are signed.
    Integer(isize),
    /// Reals are represented as a double.
    Real(f64)
}

fn strongest_cast(left : Numerics, right : Numerics) -> StaticTypes {
    let mut cast = StaticTypes::TNatural;
    match left {
        Numerics::Real(_) => cast = StaticTypes::TReal,
        Numerics::Integer(_) => cast = StaticTypes::TInteger,
        _ => ()
    };
    if cast == StaticTypes::TReal { return cast; }
    match right {
        Numerics::Real(_) => cast = StaticTypes::TReal,
        Numerics::Integer(_) => cast = StaticTypes::TInteger,
        _ => ()
    };
    cast
}

macro_rules! new_base {
    ($arg:expr, $base:ident) => {
        match &$arg {
            Numerics::Natural(n) => *n as $base,
            Numerics::Integer(n) => *n as $base,
            Numerics::Real(n)    => *n as $base,
        };
    };
}

macro_rules! fold_on_numeric {
    ($op:tt, $left:expr, $right:expr) => {
        {
            let cast = strongest_cast($left, $right);
            match cast {
                StaticTypes::TNatural => (new_base!($left, usize) $op new_base!($right, usize)).to_numeric(),
                StaticTypes::TInteger => (new_base!($left, isize) $op new_base!($right, isize)).to_numeric(),
                StaticTypes::TReal    => (new_base!($left,   f64) $op new_base!($right,   f64)).to_numeric(),
                _ => panic!("Numeric porting non-numeric type?")
            }
        }
    };
}

impl ops::Add<Numerics> for Numerics {
    type Output = Numerics;
    fn add(self, right : Numerics) -> Numerics {
        fold_on_numeric!(+, self, right)
    }
}

impl ops::Sub<Numerics> for Numerics {
    type Output = Numerics;
    fn sub(self, right : Numerics) -> Numerics {
        if fold_on_numeric!(>, right, self) == Numerics::Natural(1) {
            if let Numerics::Natural(u) = right {
                return fold_on_numeric!(-, self, Numerics::Integer(u as isize));
            }
        }
        fold_on_numeric!(-, self, right)
    }
}

impl ops::Mul<Numerics> for Numerics {
    type Output = Numerics;
    fn mul(self, right : Numerics) -> Numerics {
        fold_on_numeric!(*, self, right)
    }
}

impl ops::Div<Numerics> for Numerics {
    type Output = Numerics;
    fn div(self, right : Numerics) -> Numerics {
        fold_on_numeric!(/, self, right)
    }
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
        let mut test_str = <&str>::clone(self).to_ascii_lowercase();

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

impl ToNumeric for bool {
    fn to_numeric(&self) -> Numerics { Numerics::Natural(if *self { 1 } else { 0 }) }
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
#[derive(Clone)]
pub struct NumNode {
    /// Holds a the numeric value.
    pub value : Numerics,

    /// Source location.
    pub location : Loc,
}


/// Node for holding strings.
#[derive(Clone)]
pub struct StrNode {
    /// Contents of the utf-8 string.
    pub value : String,

    /// Source location.
    pub location : Loc,
}

/// Symbol Node.
#[derive(Clone)]
pub struct SymNode {
    /// Value/name stored as a string and
    /// excludes the colon (:) in front.
    pub value : String,

    /// Source location.
    pub location : Loc,
}

/// Call Node has a pointer to the callee node
/// and a list of operand nodes.
#[derive(Clone)]
pub struct CallNode {
    /// Pointer to heap allocated calling node.
    pub callee : Box<Nodes>,
    /// Pointer to list of operand nodes.
    pub operands : Vec<Nodes>,

    /// What type it returns.
    pub return_type : StaticTypes,

    /// Source location.
    pub location : Loc,
}

/// Represents a block of code / compound statements
/// in order of when they will be executed.
#[derive(Clone)]
pub struct BlockNode {
    /// Pointer to list of nodes in the code block.
    pub statements : Vec<Nodes>,

    /// Source location.
    pub location : Loc,
}

#[derive(Clone)]
pub struct FileNode {
    pub filename : String,
    /// Source location.
    pub location : Loc,
}

#[derive(Clone)]
pub struct NilNode {
    /// Source location.
    pub location : Loc,
}

/// All base types, determined at compile time.
/// The order the types are presented below, is
/// generally how we reference the types in the
/// compiled bytecode numerically (e.g. TReal => 3).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StaticTypes {
    TNatural, TInteger, TReal,
    TString, TSymbol,
    TSet(Box<StaticTypes>),
    /// TFunction(boxed operand type, boxed return type)
    TFunction(Box<StaticTypes>, Box<StaticTypes>),

    TNil,
    TUnknown
}

impl StaticTypes {
    pub fn set_inner(&self) -> Option<StaticTypes> {
        if let StaticTypes::TSet(box_inner) = self {
            return Some(*box_inner.clone());
        }
        None
    }

    pub fn is_number(&self) -> bool {
        match self {
            StaticTypes::TNatural
            | StaticTypes::TInteger
            | StaticTypes::TReal => true,
            _ => false
        }
    }
}

impl fmt::Display for StaticTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ss;
        let s = match self {
            StaticTypes::TNatural => "natural",
            StaticTypes::TInteger => "integer",
            StaticTypes::TReal    => "real",
            StaticTypes::TString  => "string",
            StaticTypes::TSymbol  => "symbol",
            StaticTypes::TSet(st) => match *st.clone() {
                StaticTypes::TNatural => "Nat",
                StaticTypes::TInteger => "Int",
                StaticTypes::TReal    => "Real",
                StaticTypes::TString  => "String",
                StaticTypes::TSymbol  => "Sym",
                StaticTypes::TFunction(o, r) => {
                    ss = format!("({} \u{1f852} {})", o, r);
                    ss.as_str()
                },
                StaticTypes::TNil     => "Empty",
                StaticTypes::TUnknown => "Any",
                _ => {
                    ss = format!("Set {}", st);
                    ss.as_str()
                },
            },
            StaticTypes::TFunction(o, r) => {
                ss = format!("({} \u{21a6} {})", o, r);
                ss.as_str()
            },
            StaticTypes::TNil     => "nothing",
            StaticTypes::TUnknown => "anything",
        };
        write!(f, "{}", s)
    }
}

/// All node types.
#[derive(Clone)]
pub enum Nodes {
    Ident(IdentNode),
    Num(NumNode),
    Str(StrNode),
    Sym(SymNode),
    Call(CallNode),
    Block(BlockNode),
    File(FileNode),
    Nil(NilNode),
}


impl fmt::Display for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let yt = self.yield_type();
        let printable = match self {
            Nodes::Ident(node)  => format!("%ident{{ :value \"{}\"; :yield {} }}", node.value, yt),
            Nodes::Num(node)    => format!("%num{{ :value {}; :yield {} }}", node.value, yt),
            Nodes::Str(node)    => format!("%str{{ :value \"{}\"; :yield {} }}", node.value, yt),
            Nodes::Sym(node)    => format!("%sym{{ :value \":{}\"; :yield {} }}", node.value, yt),
            Nodes::Call(node)   => format!(
                "%call{{\n  :yield {}\n  :callee ({})\n  :operands [|\n    {}\n  |]\n}}", yt, node.callee,
                node.operands.iter().map(Nodes::to_string).collect::<Vec<String>>().join("\n    ")),
            Nodes::Block(node)  => format!("%block{{ {} }}",
                node.statements
                .iter()
                .map(Nodes::to_string)
                .collect::<Vec<String>>()
                .join("\n")),
            Nodes::File(node)   => format!("%file{{ :filename {} }}", node.filename),
            Nodes::Nil(_)       => String::from("()"),
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
    pub fn location(&self) -> Loc {
        match self {
            Nodes::Ident(n) => n.location,
            Nodes::Call(n)  => n.location,
            Nodes::Num(n)   => n.location,
            Nodes::Str(n)   => n.location,
            Nodes::Sym(n)   => n.location,
            Nodes::Nil(n) => n.location,
            Nodes::Block(n) => n.location,
            Nodes::File(n)  => n.location,
        }
    }
    /// Function that returns the statically known type
    /// of any syntactic node generated.
    pub fn yield_type(&self) -> StaticTypes {
        match self {
            Nodes::Num(num) => {
                match num.value {
                    Numerics::Natural(_) => StaticTypes::TNatural,
                    Numerics::Integer(_) => StaticTypes::TInteger,
                    Numerics::Real(_)    => StaticTypes::TReal,
                }
            },
            Nodes::Str(_) => StaticTypes::TString,
            Nodes::Sym(_) => StaticTypes::TSymbol,
            Nodes::Ident(ident) => {
                match ident.value.as_str() {
                    "Nat"  => StaticTypes::TSet(Box::new(StaticTypes::TNatural)),
                    "Int"  => StaticTypes::TSet(Box::new(StaticTypes::TInteger)),
                    "Real" => StaticTypes::TSet(Box::new(StaticTypes::TReal)),
                    "Str" | "String" => StaticTypes::TSet(Box::new(StaticTypes::TString)),
                    "Sym" | "Symbol" => StaticTypes::TSet(Box::new(StaticTypes::TSymbol)),
                    "Empty" => StaticTypes::TSet(Box::new(StaticTypes::TNil)),
                    "Any" | "Anything" => StaticTypes::TSet(Box::new(StaticTypes::TUnknown)),
                    _ => ident.static_type.to_owned()
                }
            },
            Nodes::Call(call) => {
                match &*call.callee {
                    Nodes::Ident(ident) => {
                        match ident.value.as_str() {
                            "Set" => return StaticTypes::TSet(Box::new(call.operands[0].yield_type())),
                            _ => ()
                        };
                    },
                    Nodes::Call(sub_call) => {
                        if let Nodes::Ident(ident) = &*sub_call.callee {
                            match ident.value.as_str() {
                                "->" => {
                                    return StaticTypes::TSet(
                                        Box::new(StaticTypes::TFunction(
                                            Box::new(sub_call.operands[0].yield_type()),
                                            Box::new(call.operands[0].yield_type()))));
                                },
                                _ => ()
                            }
                        }
                    }
                    _ => ()
                };
                call.return_type.to_owned()
            },
            Nodes::Block(_)
            | Nodes::File(_) => StaticTypes::TUnknown,
            Nodes::Nil(_)    => StaticTypes::TNil,
        }
    }

    pub fn change_yield(&mut self, new_yield : StaticTypes) {
        match self {
            Nodes::Ident(i) => i.static_type = new_yield,
            Nodes::Call(c)  => c.return_type = new_yield,
            _ => panic!("Cannot change static yield type of node with inherent type.")
        }
    }

    pub fn node_type(&self) -> &str {
        match self {
            Nodes::Ident(_) => "identifier",
            Nodes::Num(_)   => "numeric",
            Nodes::Str(_)   => "string literal",
            Nodes::Sym(_)   => "symbol",
            Nodes::Nil(_)   => "nothing",
            Nodes::Call(_)  => "function call",
            Nodes::Block(_) => "code block",
            _ => "ungrammatical meta node"
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        match self {
            Nodes::Str(n)   => Some(n.value.as_str()),
            Nodes::Sym(n)   => Some(n.value.as_str()),
            Nodes::Ident(n) => Some(n.value.as_str()),
            _ => None
        }
    }

    pub fn ident(&self) -> Option<&IdentNode> { unwrap_enum!(self, Nodes::Ident) }
    pub fn   num(&self) -> Option<&NumNode>   { unwrap_enum!(self, Nodes::Num)   }
    pub fn   str(&self) -> Option<&StrNode>   { unwrap_enum!(self, Nodes::Str)   }
    pub fn   sym(&self) -> Option<&SymNode>   { unwrap_enum!(self, Nodes::Sym)   }
    pub fn  call(&self) -> Option<&CallNode>  { unwrap_enum!(self, Nodes::Call)  }
    pub fn block(&self) -> Option<&BlockNode> { unwrap_enum!(self, Nodes::Block) }
    pub fn  file(&self) -> Option<&FileNode>  { unwrap_enum!(self, Nodes::File)  }
    pub fn   nil(&self) -> Option<&NilNode>   { unwrap_enum!(self, Nodes::Nil)   }

    pub fn is_ident(&self) -> bool { self.ident().is_some() }
    pub fn is_num(&self)   -> bool { self.num().is_some()   }
    pub fn is_str(&self)   -> bool { self.str().is_some()   }
    pub fn is_sym(&self)   -> bool { self.sym().is_some()   }
    pub fn is_call(&self)  -> bool { self.call().is_some()  }
    pub fn is_block(&self) -> bool { self.block().is_some() }
    pub fn is_file(&self)  -> bool { self.file().is_some()  }
    pub fn is_nil(&self)   -> bool { self.nil().is_some()   }

 

    pub fn is_atomic(&self) -> bool {
        match self {
            Nodes::Ident(_)
            | Nodes::Num(_)
            | Nodes::Str(_)
            | Nodes::Sym(_)
            | Nodes::Nil(_)  => true,
            _ => false
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            Nodes::Num(_)=> true,
            _ => false
        }
    }
}

impl IdentNode {
    pub fn new(value : &str, location : Loc) -> Nodes {
        Nodes::Ident(IdentNode {
            value: value.to_string(),
            static_type: StaticTypes::TUnknown,
            location
        })
    }
}

impl NumNode {
    pub fn new<Num : ToNumeric>(number : Num, location : Loc) -> Nodes {
        let value = number.to_numeric();
        Nodes::Num(NumNode { value, location })
    }
}

impl StrNode {
    pub fn new(value : &str, location : Loc) -> Nodes
        { Nodes::Str(StrNode { value: value.to_string(), location }) }
}

impl SymNode {
    pub fn new(value : &str, location : Loc) -> Nodes
        { Nodes::Sym(SymNode { value: value[1..].to_string(), location }) }
}

impl CallNode {
    pub fn new(callee : Nodes, operands : Vec<Nodes>, location : Loc) -> Nodes {
        Nodes::Call(CallNode {
            callee: Box::new(callee),
            operands,
            return_type: StaticTypes::TUnknown,
            location
        })
    }

    pub fn set_return_type(&mut self, new_type : StaticTypes) {
        self.return_type = new_type;
    }

    /// The base (bottom-most) callee for a call chain.
    pub fn base_call(&self) -> Nodes {
        let mut last_call : &CallNode = self;
        loop {
            if let Nodes::Call(call) = &*last_call.callee {
                last_call = call;
            } else {
                return (*last_call.callee).clone();
            }
        }
    }

    /// Collect arguments to a call.
    pub fn collect_operands(&self) -> Vec<Nodes> {
        fn make_argument_vector(call_node : &Nodes,
                                operands : VecDeque<Nodes>) -> VecDeque<Nodes> {
            let mut pushable = operands;

            if let Nodes::Call(call) = call_node {
                pushable.push_front(call.operands[0].clone());
                return make_argument_vector(&*call.callee, pushable);
            }

            pushable.push_front(call_node.clone());
            return pushable;
        }
        let q = make_argument_vector(&Nodes::Call(self.clone()), VecDeque::new());
        Vec::from(q)
    }

    /// List of callee and operands, lisp call style list.
    pub fn collect(&self) -> Vec<Nodes> {
        let mut list = vec![self.base_call()];
        list.extend_from_slice(&self.collect_operands());
        list
    }

    pub fn is_unary(&self) -> bool {
        self.callee.ident().is_some() && !self.operands.is_empty()
    }

    pub fn is_binary(&self) -> bool {
        let sub_call = self.callee.call();
        sub_call.is_some() && !self.operands.is_empty() && sub_call.unwrap().is_unary()
    }

    pub fn operand(&self) -> Option<&Nodes> {
        self.operands.last()
    }
}

impl FileNode {
    pub fn new(filename : String, location : Loc) -> Nodes
        { Nodes::File(FileNode { filename, location }) }
}

impl NilNode {
    pub fn new(location : Loc) -> Nodes { Nodes::Nil(NilNode { location }) }
}

/// Root branch of the AST.
pub struct Root {
    pub branches : Vec<Nodes>,
    pub filename : String
}

impl Root {
    pub fn new(filename : &str) -> Self {
        Root { branches: Vec::new(), filename: filename.to_owned() }
    }
}

const TAB : &str = "  ";

pub fn pretty_print(node : &Nodes, depth : usize) -> String {
    let tab = TAB.repeat(depth);
    match node {
        Nodes::Call(n) => format!(
            "{tab}%call{{\n{tab}{T}:yield {yt}\n{tab}{T}:callee (\n{calling}\n{tab}{T})\n{tab}{T}:operand [|{op}|]\n{tab}}}",
            tab=tab, T=TAB,
            yt=n.return_type,
            calling=pretty_print(&*n.callee, depth + 2),
            op=(if n.operands.is_empty() { String::from(" ") } else { format!(
                "\n{ops}\n{tab}{T}",
                ops=pretty_print(&n.operands[0], depth + 2),
                tab=tab, T=TAB) })
        ),
        // TODO: Pretty Print Blocks.
        Nodes::Block(_) => node.to_string(),
        _ => format!("{}{}", tab, node)
    }
}


impl fmt::Display for Root {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_mapped : Vec<String> = self.branches.iter().map(|n| pretty_print(n, 0)).collect();
        write!(f, "[|\n  {}\n|]", str_mapped.join("\n").split('\n').collect::<Vec<&str>>().join("\n  "))
    }
}
