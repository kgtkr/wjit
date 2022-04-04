use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_while, take_while1},
    character::complete::{char, satisfy},
    combinator::{eof, map, map_opt, value},
    multi::many0,
    sequence::terminated,
    IResult,
};

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

fn letters(input: &str) -> IResult<&str, String> {
    let (input, head) = satisfy(|c: char| c.is_ascii_alphabetic())(input)?;
    let (input, tail) = take_while(|c: char| c.is_ascii_alphanumeric() || c == '_')(input)?;
    Ok((input, head.to_string() + tail))
}

fn symbols(input: &str) -> IResult<&str, String> {
    map(is_a("!$%&*+-/<=>?@^"), |s: &str| s.to_string())(input)
}

fn spaces(input: &str) -> IResult<&str, ()> {
    value((), take_while1(|c: char| c.is_ascii_whitespace()))(input)
}

fn digits(input: &str) -> IResult<&str, i32> {
    map_opt(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.parse::<i32>().ok()
    })(input)
}

fn line_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("#")(input)?;
    let (input, _) = many0(satisfy(|c: char| c != '\n'))(input)?;
    Ok((input, ()))
}

fn token(input: &str) -> IResult<&str, Token> {
    alt((
        map(letters, |s| match s.as_str() {
            "if" => Token::Reserved(Reserved::If),
            "else" => Token::Reserved(Reserved::Else),
            "while" => Token::Reserved(Reserved::While),
            "var" => Token::Reserved(Reserved::Var),
            "func" => Token::Reserved(Reserved::Func),
            "in" => Token::Reserved(Reserved::In),
            _ => Token::Ident(s),
        }),
        map(symbols, |s| match s.as_str() {
            "=" => Token::ReservedOp(ReservedOp::Assign),
            _ => Token::Operator(s),
        }),
        map(digits, Token::IntLiteral),
        value(Token::Dot, char('.')),
        value(Token::Comma, char(',')),
        value(Token::OpenParen, char('(')),
        value(Token::CloseParen, char(')')),
        value(Token::OpenBrace, char('{')),
        value(Token::CloseBrace, char('}')),
        value(Token::SemiColon, char(';')),
    ))(input)
}

fn tokens(input: &str) -> IResult<&str, Vec<Token>> {
    map(
        many0(alt((
            value(None, spaces),
            value(None, line_comment),
            map(token, Some),
        ))),
        |xs| xs.into_iter().filter_map(|x| x).collect::<Vec<_>>(),
    )(input)
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    terminated(tokens, eof)(input)
}
