#[derive(Debug, PartialEq, Clone, Hash, Eq)]
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

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Reserved {
    If,
    Else,
    While,
    Var,
    Func,
    In,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]

pub enum ReservedOp {
    Assign,
}
