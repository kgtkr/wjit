#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub funcs: Vec<Func>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub name: String,
    pub args: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    IntLiteral(i32),
    Ident(String),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    PrefixOp(PrefixOp, Box<Expr>),
    Assign(String, Box<Expr>),
    Call(String, Vec<Expr>),
    While(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Var(String, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixOp {
    Not,
    Minus,
}
