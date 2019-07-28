
/// Side of associativity.
#[derive(Copy, Clone, PartialEq)]
pub enum Side {
    Left, Right, Neither
}

/// Operator information, including:
/// - The string, representing what the operator looks like.
/// - Its precedence (as an i32), the higher the int, the higher the precedence.
/// - Associativity, which can either be left, right, or no associativity.
/// - The number of arguments it takes / its arity. Either one, or two.
#[derive(Clone, Copy)]
pub struct Operator<'a> {
    pub name : &'a str,
    pub precedence : i32,
    pub associativity : Side,
    pub arity : i32,
}

impl<'a> Operator<'a> {
    pub fn new(name : &'a str, precedence : i32, associativity : Side, arity : i32) -> Self {
        Operator {
            name: name.clone(),
            precedence,
            associativity,
            arity,
        }
    }

    pub fn is_left(&self) -> bool { self.associativity == Side::Left }

    pub fn is_right(&self) -> bool { self.associativity == Side::Right }

    pub fn has_arity(&self, n : i32) -> bool { self.arity == n }

    pub fn is_unary(&self) -> bool { self.has_arity(1)  }

    pub fn is_binary(&self) -> bool { self.has_arity(2) }
}

/// Wrapper for table of known operators.
pub struct PrecedenceTable<'a> {
    pub table : Vec<Operator<'a>>
}

#[macro_export]
macro_rules! push_op {
    ($table:expr, $op:expr, $prec:expr, $assoc:path, $arity:expr) => {
        $table.table.push(Operator::new($op, $prec as i32, $assoc, $arity as i32))
    };
}

impl<'a> PrecedenceTable<'a> {
    pub fn new() -> Self {
        let op = Operator::new;
        let table = PrecedenceTable { table: vec![
            op( "::",210, Side::Left,    2),
            op( "<>",200, Side::Right,   2),
            // Function calls have precedence 190, i.e. very high.
            //  e.g.   `f x + 3` bracketed is the same as `((f x) + 3)`.
            op("not",180, Side::Right,   1),
            op(  "-",170, Side::Right,   1),
            op(  "^",160, Side::Right,   2),
            op(  "*",150, Side::Left,    2),
            op(  "/",150, Side::Left,    2),
            op("mod",150, Side::Left,    2),
            op(  "&",140, Side::Left,    2),
            op(  "|",130, Side::Left,    2),
            op(  "+",120, Side::Left,    2),
            op(  "-",120, Side::Left,    2),
            op( "\\",120, Side::Left,    2),
            op( "->",110, Side::Right,   2),
            op( ">>",100, Side::Right,   2),
            op( "<<",100, Side::Left,    2),
            op( "==", 90, Side::Neither, 2),
            op(  "is",90, Side::Neither, 2),
            op( "/=", 90, Side::Neither, 2),
            op("isn't",90,Side::Neither, 2),
            op(  "<", 90, Side::Neither, 2),
            op( "<=", 90, Side::Neither, 2),
            op(  ">", 90, Side::Neither, 2),
            op( ">=", 90, Side::Neither, 2),
            op( "<-", 80, Side::Neither, 2),
            op( "&&", 70, Side::Right,   2),
            op("and", 70, Side::Right,   2),
            op( "||", 60, Side::Right,   2),
            op( "or", 60, Side::Right,   2),
            op( "..", 50, Side::Neither, 2),
            op(  ":", 40, Side::Neither, 2),
            op( "|>", 40, Side::Right,   2),
            op(  "=", 30, Side::Right,   2),
            op( "if", 20, Side::Neither, 2),
            op("unless", 20, Side::Neither, 2),
            op(  ",", 10, Side::Right,   2),
            op( "=>",  1, Side::Neither, 2),
            op(  "(",  0, Side::Neither, 1),
            op(  ")",  0, Side::Neither, 1),
        ]};

        table
    }

    pub fn new_op(&mut self, name : &'a str, prec : i32, assoc : Side, arity : i32) -> Operator {
        let op = Operator::new(name, prec, assoc, arity);
        self.table.push(op);
        op
    }

    pub fn new_fun(&mut self, name : &'a str, max_arity : i32) -> Operator {
        self.new_op(name, 19, Side::Neither, max_arity)
    }

    pub fn lookup(&self, name : &str, arity : i32) -> Option<&Operator> {
        self.table.iter().filter(|o| o.name == name && o.arity == arity).nth(0)
    }

    pub fn exists(&self, name : &str) -> bool {
        self.table.iter().filter(|o| o.name == name).nth(0).is_some()
    }

    pub fn precedence(&self, name : &str) -> Option<i32> {
        let op = self.lookup(name, 2);
        if op.is_some() { return Some(op.unwrap().precedence) }
        return None;
    }
}