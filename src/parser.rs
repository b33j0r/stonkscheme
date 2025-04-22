//! Extremely small grammar + “give‑me‑a‑span” helper.
//! Compiles cleanly on nom 8.  Grow from here.

use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::{
    character::complete::{digit1, multispace0},
    combinator::map_res,
    error::{FromExternalError, ParseError as NomErr},
    sequence::delimited,
    IResult, Parser,
};
use thiserror::Error;

use crate::ast::Symbol;
use crate::{
    ast::Expr,
    code::{Code, CodeSpan, ParserSpan, Spanned},
};

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ParseError {
    #[error("nom error: {kind:?} at {span:?}")]
    Nom { kind: nom::error::ErrorKind, span: CodeSpan },

    #[error("invalid integer `{value}` – {msg}")]
    BadInt { value: String, msg: String, span: CodeSpan },
}

impl<'a> NomErr<ParserSpan<'a>> for ParseError {
    fn from_error_kind(input: ParserSpan<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::Nom {
            kind,
            span: CodeSpan::from(input),
        }
    }
    fn append(_: ParserSpan<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
    fn or(self, _other: Self) -> Self {
        self
    }
}

impl<'a> FromExternalError<ParserSpan<'a>, ParseError> for ParseError {
    fn from_external_error(
        input: ParserSpan<'a>,
        _kind: nom::error::ErrorKind,
        e: ParseError,
    ) -> Self {
        match e {
            ParseError::BadInt { value, msg, .. } => ParseError::BadInt {
                value,
                msg,
                span: CodeSpan::from(input),
            },
            _ => e,
        }
    }
}

/// Wrap any nom parser so it returns a `Spanned<O>`.
fn spanned<'a, F, O, E>(
    mut inner: F,
) -> impl FnMut(ParserSpan<'a>) -> IResult<ParserSpan<'a>, Spanned<O>, E>
where
    F: Parser<ParserSpan<'a>, Output=O, Error=E>,
    E: NomErr<ParserSpan<'a>>,
{
    move |input: ParserSpan<'a>| {
        let start = input.location_offset();
        let code = input.extra.clone();

        let (rest, value) = inner.parse(input)?;
        let end = rest.location_offset();

        Ok((
            rest,
            Spanned {
                value,
                span: CodeSpan::new(code, start, end),
            },
        ))
    }
}

fn parse_int<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Expr, ParseError> {
    map_res(digit1, |s: ParserSpan<'a>| {
        s.fragment().parse::<i64>().map(Expr::Integer).map_err(|e| {
            ParseError::BadInt {
                value: s.fragment().to_string(),
                msg: e.to_string(),
                span: CodeSpan::from(s),
            }
        })
    }).parse(input)
}

fn parse_combination_inner(input: ParserSpan) -> IResult<ParserSpan, Expr, ParseError> {
    separated_pair(
        parse_expr,
        multispace0,
        separated_list1(multispace0, parse_expr),
    ).map(
        |(op, args)| Expr::Combination(Box::new(op.value), args.into_iter().map(|s| s.value).collect())
    ).parse(input)
}

fn parse_combination(input: ParserSpan) -> IResult<ParserSpan, Expr, ParseError> {
    delimited(tag("("), parse_combination_inner, tag(")")).parse(input)
}

pub fn parse_expr<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Spanned<Expr>, ParseError> {
    alt((
        spanned(parse_int),
        spanned(map_res(
            take_while1(
                |c: char| c.is_ascii_alphabetic()
                    || c == '_' || c == '+' || c == '-' || c == '*'
                    || c == '=' || c == '>' || c == '<' || c == '!'
                    || c == '?' || c == '/' || c == '$'),
            |s: ParserSpan<'a>| Ok(Expr::Operator(Symbol(s.fragment().to_string()))),
        )),
        spanned(parse_combination)
    )).parse(input)
}

pub fn parse_snippet(src: &str) -> Result<Spanned<Expr>, ParseError> {
    let code = Code::from_snippet(src);
    complete_expr(&code)
}

pub fn parse_file(path: &std::path::Path) -> Result<Spanned<Expr>, ParseError> {
    let code = Code::from_file(path).map_err(|_io| ParseError::Nom {
        kind: nom::error::ErrorKind::Fail,
        span: CodeSpan::new(Code::from_snippet(""), 0, 0),
    })?;
    complete_expr(&code)
}

fn complete_expr(code: &Arc<Code>) -> Result<Spanned<Expr>, ParseError> {
    let span = Code::span(code);
    let (_, spanned) = delimited(multispace0, parse_expr, multispace0).parse(span)
        .map_err(|e| match e {
            nom::Err::Error(p) | nom::Err::Failure(p) => p,
            nom::Err::Incomplete(_) => unreachable!(),
        })?;
    Ok(spanned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Symbol;

    #[test]
    fn parses_integer() {
        let sp = parse_snippet("  42 ").expect("parse");
        assert_eq!(sp.value, Expr::Integer(42));
        assert_eq!(&sp.span.code.text[sp.span.start..sp.span.end], "42");
    }

    #[test]
    fn parses_operator() {
        let sp = parse_snippet("  + ").expect("parse");
        assert_eq!(sp.value, Expr::Operator(Symbol("+".to_string())));
        assert_eq!(&sp.span.code.text[sp.span.start..sp.span.end], "+");
    }

    #[test]
    fn parses_combination() {
        let sp = parse_snippet("(define x 1)").expect("parse");
        assert_eq!(
            sp.value,
            Expr::Combination(
                Box::new(Expr::Operator(Symbol("define".to_string()))),
                vec![
                    Expr::Operator(Symbol("x".to_string())),
                    Expr::Integer(1)
                ]
            )
        );
    }
}
