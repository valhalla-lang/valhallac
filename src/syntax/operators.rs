
/// Side of associativity.
#[derive(PartialEq)]
pub enum Side {
    Left, Right, Neither
}

/// Operator information, including:
/// - The string, representing what the operator looks like.
/// - Its precedence (as an i32), the higher the int, the higher the precedence.
/// - Associativity, which can either be left, right, or no associativity.
/// - The number of arguments it takes / its arity. Either one, or two.
pub struct Operator {
    pub name : String,
    pub precedence : i32,
    pub associativity : Side,
    pub arity : i32,
}

impl Operator {
    pub fn new(name : &str, precedence : i32, associativity : Side, arity : i32) -> Self {
        Operator {
            name: name.to_string(),
            precedence,
            associativity,
            arity,
        }
    }

    pub fn is_left(&self) -> bool {
        if self.associativity == Side::Left {
            return true;
        }
        false
    }

    pub fn is_right(&self) -> bool {
        if self.associativity == Side::Right {
            return true;
        }
        false
    }
}

/// Wrapper for table of known operators.
pub struct PrecedenceTable {
    pub table : Vec<Operator>
}

#[macro_export]
macro_rules! push_op {
    ($table:expr, $op:expr, $prec:expr, $assoc:path, $arity:expr) => {
        $table.table.push(Operator::new($op, $prec as i32, $assoc, $arity as i32))
    };
}

impl PrecedenceTable {
    pub fn new() -> Self {
        let op = Operator::new;
        let mut table = PrecedenceTable { table: vec![
            op( "::",21, Side::Left,    2),
            op( "<>",20, Side::Right,   2),
            // Function calls have precedence 19, i.e. very high.
            //  e.g.   `f x + 3` bracketed is the same as `(f (x) (+) (3))`.
            op("not",18, Side::Right,   1),
            op(  "-",17, Side::Right,   1),
            op(  "^",16, Side::Right,   2),
            op(  "*",15, Side::Left,    2),
            op(  "/",15, Side::Left,    2),
            op("mod",15, Side::Left,    2),
            op(  "&",14, Side::Left,    2),
            op(  "|",13, Side::Left,    2),
            op(  "+",12, Side::Left,    2),
            op(  "-",12, Side::Left,    2),
            op( "\\",12, Side::Left,    2),
            op( "->",11, Side::Right,   2),
            op( ">>",10, Side::Right,   2),
            op( "<<",10, Side::Left,    2),
            op( "==", 9, Side::Neither, 2),
            op(  "is",9, Side::Neither, 2),
            op( "/=", 9, Side::Neither, 2),
            op("isn't",9,Side::Neither, 2),
            op(  "<", 9, Side::Neither, 2),
            op( "<=", 9, Side::Neither, 2),
            op(  ">", 9, Side::Neither, 2),
            op( ">=", 9, Side::Neither, 2),
            op( "<-", 8, Side::Neither, 2),
            op( "&&", 7, Side::Right,   2),
            op("and", 7, Side::Right,   2),
            op( "||", 6, Side::Right,   2),
            op( "or", 6, Side::Right,   2),
            op( "..", 5, Side::Neither, 2),
            op(  ":", 4, Side::Neither, 2),
            op( "|>", 4, Side::Right,   2),
            op(  "=", 3, Side::Right,   2),
            op( "if", 2, Side::Neither, 2),
            op("unless", 2, Side::Neither, 2),
            op(  ",", 1, Side::Right,   2),        
            op( "=>", 0, Side::Neither, 2),
        ]};
        
        table
    }

    pub fn lookup(&self, name : &str, arity : i32) -> Option<&Operator> {
        self.table.iter().filter(|o| o.name == name && o.arity == arity).next()
    }

    pub fn exists(&self, name : &str) -> bool {
        self.table.iter().filter(|o| o.name == name).next().is_some()
    }
}