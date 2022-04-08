use nom::{
    branch::alt,
    combinator::{eof, map},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::ast::*;
use crate::token;

fn satisfy_opt<O>(
    f: impl Fn(&token::Token) -> Option<O>,
) -> impl Fn(&[token::Token]) -> IResult<&[token::Token], O> {
    move |input1: &[token::Token]| {
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

fn int_literal(input: &[token::Token]) -> IResult<&[token::Token], i32> {
    satisfy_opt(|token| match token {
        &token::Token::IntLiteral(value) => Some(value),
        _ => None,
    })(input)
}

fn ident(input: &[token::Token]) -> IResult<&[token::Token], String> {
    satisfy_opt(|token| match token {
        token::Token::Ident(value) => Some(value.clone()),
        _ => None,
    })(input)
}

fn paren_expr(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenParen => Some(()),
        _ => None,
    })(input)?;
    let (input, expr) = expr(input)?;
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseParen => Some(()),
        _ => None,
    })(input)?;
    Ok((input, expr))
}

fn expr0(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    alt((
        map(int_literal, Expr::IntLiteral),
        paren_expr,
        map(ident, Expr::Ident),
    ))(input)
}

fn call_params(input: &[token::Token]) -> IResult<&[token::Token], Vec<Expr>> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenParen => Some(()),
        _ => None,
    })(input)?;
    let (input, params) = separated_list0(
        satisfy_opt(|token| match token {
            token::Token::Comma => Some(()),
            _ => None,
        }),
        expr,
    )(input)?;
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseParen => Some(()),
        _ => None,
    })(input)?;
    Ok((input, params))
}

fn expr1(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
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

fn expr2(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, prefix_ops) = many0(satisfy_opt(|token| match token {
        token::Token::Operator(op) => match op.as_str() {
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

fn expr3(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr2(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr4(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr3(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr5(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr4(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr6(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr5(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr7(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr6(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr8(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, expr) = expr7(input)?;
    let (input, binary_ops) = many0(tuple((
        satisfy_opt(|token| match token {
            token::Token::Operator(op) => match op.as_str() {
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

fn expr9(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let input1 = input;
    let (input, expr) = expr8(input)?;
    let (input, exprs) = many0(preceded(
        satisfy_opt(|token| match token {
            token::Token::ReservedOp(token::ReservedOp::Assign) => Some(()),
            _ => None,
        }),
        expr8,
    ))(input)?;
    let (last, init) = match exprs.split_last() {
        Some((last, init)) => (last.clone(), {
            let mut exprs = init.to_vec();
            exprs.insert(0, expr);
            exprs
        }),
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

fn if_(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::If) => Some(()),
        _ => None,
    })(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::Else) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr3) = expr(input)?;

    Ok((
        input,
        Expr::If(Box::new(expr1), Box::new(expr2), Box::new(expr3)),
    ))
}

fn while_(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::While) => Some(()),
        _ => None,
    })(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseParen => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    Ok((input, Expr::While(Box::new(expr1), Box::new(expr2))))
}

fn block(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenBrace => Some(()),
        _ => None,
    })(input)?;

    let (input, exprs) = many0(terminated(
        expr,
        satisfy_opt(|token| match token {
            token::Token::SemiColon => Some(()),
            _ => None,
        }),
    ))(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseBrace => Some(()),
        _ => None,
    })(input)?;

    Ok((input, Expr::Block(exprs)))
}

fn var(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::Var) => Some(()),
        _ => None,
    })(input)?;

    let (input, ident) = ident(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::ReservedOp(token::ReservedOp::Assign) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr1) = expr(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::In) => Some(()),
        _ => None,
    })(input)?;

    let (input, expr2) = expr(input)?;

    Ok((input, Expr::Var(ident, Box::new(expr1), Box::new(expr2))))
}

fn expr(input: &[token::Token]) -> IResult<&[token::Token], Expr> {
    alt((expr9, if_, while_, block, var))(input)
}

fn func(input: &[token::Token]) -> IResult<&[token::Token], Func> {
    let (input, _) = satisfy_opt(|token| match token {
        token::Token::Reserved(token::Reserved::Func) => Some(()),
        _ => None,
    })(input)?;

    let (input, func_ident) = ident(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::OpenParen => Some(()),
        _ => None,
    })(input)?;

    let (input, params) = separated_list0(
        satisfy_opt(|token| match token {
            token::Token::Comma => Some(()),
            _ => None,
        }),
        ident,
    )(input)?;

    let (input, _) = satisfy_opt(|token| match token {
        token::Token::CloseParen => Some(()),
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

fn module(input: &[token::Token]) -> IResult<&[token::Token], Module> {
    let (input, funcs) = many0(func)(input)?;

    Ok((input, Module { funcs }))
}

pub fn parse(input: &[token::Token]) -> IResult<&[token::Token], Module> {
    terminated(module, eof)(input)
}
