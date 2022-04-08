#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Operator(String),
    Ident(String),
    IntLiteral(i32),
    Reserved(Reserved),
    ReservedOp(ReservedOp),
    Dot,
    Comma,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    SemiColon,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Reserved {
    If,
    Else,
    While,
    Var,
    Func,
    In,
}

#[derive(Debug, PartialEq, Clone)]

pub enum ReservedOp {
    Assign,
}
