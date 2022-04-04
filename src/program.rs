use nom::{
    branch::alt,
    combinator::{eof, map},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::token::{Reserved, ReservedOp, Token};

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

fn satisfy_opt<O>(f: impl Fn(&Token) -> Option<O>) -> impl Fn(&[Token]) -> IResult<&[Token], O> {
    move |input1: &[Token]| {
        let (token, input2) =
            input1
                .split_first()
                .ok_or(nom::Err::Error(nom::error::Error::new(
                    input1,
                    nom::error::ErrorKind::Eof,
                )))?;
        match f(token) {
            Some(result) => Ok((&input2, result)),
            None => Err(nom::Err::Error(nom::error::Error::from_error_kind(
                input1,
                nom::error::ErrorKind::Fail,
            ))),
        }
    }
}

fn int_literal(input: &[Token]) -> IResult<&[Token], i32> {
    satisfy_opt(|token| match token {
        &Token::IntLiteral(value) => Some(value),
        _ => None,
    })(input)
}

fn ident(input: &[Token]) -> IResult<&[Token], String> {
    satisfy_opt(|token| match token {
        Token::Ident(value) => Some(value.clone()),
        _ => None,
    })(input)
}

fn paren_expr(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenParen => Some(()),
        _ => None,
    })(input)?;
    let (input, expr) = expr(input)?;
    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseParen => Some(()),
        _ => None,
    })(input)?;
    Ok((input, expr))
}

fn expr0(input: &[Token]) -> IResult<&[Token], Expr> {
    alt((
        map(int_literal, Expr::IntLiteral),
        paren_expr,
        map(ident, Expr::Ident),
    ))(input)
}

fn call_params(input: &[Token]) -> IResult<&[Token], Vec<Expr>> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenParen => Some(()),
        _ => None,
    })(input)?;
    let (input, params) = separated_list0(
        satisfy_opt(|token| match token {
            Token::Comma => Some(()),
            _ => None,
        }),
        expr,
    )(input)?;
    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseParen => Some(()),
        _ => None,
    })(input)?;
    Ok((input, params))
}

fn expr1(input: &[Token]) -> IResult<&[Token], Expr> {
    let input1 = input;
    let (input, expr) = expr0(input)?;

    let (input, params) = many0(call_params)(input)?;

    Ok((
        input,
        params
            .into_iter()
            .try_fold(expr, |expr, params| match expr {
                Expr::Ident(ident) => Ok(Expr::Call(ident, params)),
                _ => Err(nom::Err::Error(nom::error::Error::from_error_kind(
                    input1,
                    nom::error::ErrorKind::Fail,
                ))),
            })?,
    ))
}

fn expr2(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, prefix_ops) = many0(satisfy_opt(|token| match token {
        Token::Operator(op) => match op.as_str() {
            "-" => Some(PrefixOp::Minus),
            "!" => Some(PrefixOp::Not),
            _ => None,
        },
        _ => None,
    }))(input)?;
    let (input, expr) = expr1(input)?;
    Ok((
        input,
        prefix_ops
            .into_iter()
            .rev()
            .fold(expr, |expr, op| Expr::PrefixOp(op, Box::new(expr))),
    ))
}

fn expr3(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr2(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "*" => Some(BinaryOp::Mul),
                "/" => Some(BinaryOp::Div),
                "%" => Some(BinaryOp::Mod),
                _ => None,
            },
            _ => None,
        }),
        expr2,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr4(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr3(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "+" => Some(BinaryOp::Add),
                "-" => Some(BinaryOp::Sub),
                _ => None,
            },
            _ => None,
        }),
        expr3,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr5(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr4(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "<" => Some(BinaryOp::Lt),
                ">" => Some(BinaryOp::Gt),
                "<=" => Some(BinaryOp::Le),
                ">=" => Some(BinaryOp::Ge),
                _ => None,
            },
            _ => None,
        }),
        expr4,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr6(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr5(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "==" => Some(BinaryOp::Eq),
                "!=" => Some(BinaryOp::Ne),
                _ => None,
            },
            _ => None,
        }),
        expr5,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr7(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr6(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "&&" => Some(BinaryOp::And),
                _ => None,
            },
            _ => None,
        }),
        expr6,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr8(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, expr) = expr7(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            Token::Operator(op) => match op.as_str() {
                "||" => Some(BinaryOp::Or),
                _ => None,
            },
            _ => None,
        }),
        expr7,
    )))(input)?;
    Ok((
        input,
        binary_ops.into_iter().fold(expr, |expr1, (op, expr2)| {
            Expr::BinaryOp(Box::new(expr1), op, Box::new(expr2))
        }),
    ))
}

fn expr9(input: &[Token]) -> IResult<&[Token], Expr> {
    let input1 = input;
    let (input, expr) = expr8(input)?;
    let (input, exprs) = many0(preceded(
        satisfy_opt(|token| match token {
            Token::ReservedOp(ReservedOp::Assign) => Some(()),
            _ => None,
        }),
        expr8,
    ))(input)?;
    let (last, init) = match exprs.split_last() {
        Some((last, init)) => (last.clone(), init.clone()),
        None => return Ok((input, expr)),
    };
    Ok((
        input,
        init.into_iter()
            .rev()
            .try_fold(last, |rhs, lhs| match lhs {
                Expr::Ident(ident) => Ok(Expr::Assign(ident.clone(), Box::new(rhs))),
                _ => Err(nom::Err::Error(nom::error::Error::from_error_kind(
                    input1,
                    nom::error::ErrorKind::Fail,
                ))),
            })?,
    ))
}

fn if_(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::If) => Some(()),
        _ => None,
    })(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::Else) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr3) = expr(input)?;

    Ok((
        input,
        Expr::If(Box::new(expr1), Box::new(expr2), Box::new(expr3)),
    ))
}

fn while_(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::While) => Some(()),
        _ => None,
    })(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    Ok((input, Expr::While(Box::new(expr1), Box::new(expr2))))
}

fn block(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenBrace => Some(()),
        _ => None,
    })(input)?;

    let (input, exprs) = many0(terminated(
        expr,
        satisfy_opt(|token| match token {
            Token::SemiColon => Some(()),
            _ => None,
        }),
    ))(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseBrace => Some(()),
        _ => None,
    })(input)?;

    Ok((input, Expr::Block(exprs)))
}

fn var(input: &[Token]) -> IResult<&[Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::Var) => Some(()),
        _ => None,
    })(input)?;

    let (input, ident) = ident(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::ReservedOp(ReservedOp::Assign) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::In) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    Ok((input, Expr::Var(ident, Box::new(expr1), Box::new(expr2))))
}

fn expr(input: &[Token]) -> IResult<&[Token], Expr> {
    alt((expr9, if_, while_, block, var))(input)
}

fn func(input: &[Token]) -> IResult<&[Token], Func> {
    let (input, _) = satisfy_opt(|token| match token {
        Token::Reserved(Reserved::Func) => Some(()),
        _ => None,
    })(input)?;

    let (input, func_ident) = ident(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, params) = separated_list0(
        satisfy_opt(|token| match token {
            Token::Comma => Some(()),
            _ => None,
        }),
        ident,
    )(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        Token::CloseParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr) = expr(input)?;

    Ok((
        input,
        Func {
            name: func_ident,
            args: params,
            body: expr,
        },
    ))
}

fn program(input: &[Token]) -> IResult<&[Token], Program> {
    let (input, funcs) = many0(func)(input)?;

    Ok((input, Program { funcs }))
}

pub fn parse(input: &[Token]) -> IResult<&[Token], Program> {
    terminated(program, eof)(input)
}
