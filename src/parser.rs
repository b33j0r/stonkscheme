//! Extremely small grammar + “give‑me‑a‑span” helper.
//! Compiles cleanly on nom 8. Grow from here.

use std::str::FromStr;
use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::multispace0;
use nom::combinator::map_res;
use nom::error::{FromExternalError, ParseError as NomErr};
use nom::multi::separated_list1;
use nom::number::complete::recognize_float;
use nom::sequence::{delimited, separated_pair};
use nom::{IResult, Parser};
use thiserror::Error;

use crate::ast::{Expr, Symbol};
use crate::code::{Code, CodeSpan, ParserSpan, Spanned};

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ParseError {
    #[error("nom error: {kind:?} at {span:?}")]
    Nom { kind: nom::error::ErrorKind, span: CodeSpan },

    #[error("invalid number `{value}` – {msg}")]
    BadInt { value: String, msg: String, span: CodeSpan },
}

impl<'a> NomErr<ParserSpan<'a>> for ParseError {
    fn from_error_kind(input: ParserSpan<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::Nom { kind, span: CodeSpan::from(input) }
    }
    fn append(_: ParserSpan<'a>, _: nom::error::ErrorKind, other: Self) -> Self { other }
    fn or(self, _other: Self) -> Self { self }
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
            other => other,
        }
    }
}

/// Wrap any nom parser so it returns a `Spanned<O>`.
fn spanned<'a, F, O, E>(mut inner: F) -> impl FnMut(ParserSpan<'a>) -> IResult<ParserSpan<'a>, Spanned<O>, E>
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
            Spanned { value, span: CodeSpan::new(code, start, end) },
        ))
    }
}

fn parse_number<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Expr, ParseError> {
    map_res(
        recognize_float,
        |span: ParserSpan<'a>| {
            let fragment = span.fragment().clone();
            let cleaned: String = fragment.chars().filter(|&c| c != '_').collect();
            if cleaned.contains('.') || cleaned.contains('e') || cleaned.contains('E') {
                cleaned.parse::<f64>()
                    .map(Expr::Float)
                    .map_err(|e| ParseError::BadInt { value: fragment.to_string(), msg: e.to_string(), span: CodeSpan::from(span) })
            } else {
                cleaned.parse::<i64>()
                    .map(Expr::Integer)
                    .map_err(|e| ParseError::BadInt { value: fragment.to_string(), msg: e.to_string(), span: CodeSpan::from(span) })
            }
        },
    )
        .parse(input)
}

fn parse_combination_inner<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Expr, ParseError> {
    separated_pair(
        parse_expr,
        multispace0,
        separated_list1(multispace0, parse_expr),
    )
        .map(|(op, args)| Expr::Combination(Box::new(op.value), args.into_iter().map(|s| s.value).collect()))
        .parse(input)
}

fn parse_combination<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Expr, ParseError> {
    delimited(tag("("), parse_combination_inner, tag(")")).parse(input)
}

pub fn parse_expr<'a>(input: ParserSpan<'a>) -> IResult<ParserSpan<'a>, Spanned<Expr>, ParseError> {
    alt((
        spanned(parse_number),
        spanned(map_res(
            take_while1(|c: char| {
                c.is_ascii_alphabetic() || c == '_' || c == '+' || c == '-' || c == '*' ||
                    c == '=' || c == '>' || c == '<' || c == '!' || c == '?' || c == '/' || c == '$'
            }),
            |span: ParserSpan<'a>| Ok(Expr::Symbol(Symbol(span.fragment().to_string()))),
        )),
        spanned(parse_combination),
    ))
        .parse(input)
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
    let (_, spanned) = delimited(multispace0, parse_expr, multispace0)
        .parse(span)
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
    fn parses_float() {
        let sp = parse_snippet("  3.14 ").expect("parse");
        assert_eq!(sp.value, Expr::Float(3.14));
        assert_eq!(&sp.span.code.text[sp.span.start..sp.span.end], "3.14");
    }

    #[test]
    fn parses_operator() {
        let sp = parse_snippet("  + ").expect("parse");
        assert_eq!(sp.value, Expr::Symbol(Symbol("+".to_string())));
        assert_eq!(&sp.span.code.text[sp.span.start..sp.span.end], "+");
    }

    #[test]
    fn parses_combination() {
        let sp = parse_snippet("(define x 1)").expect("parse");
        assert_eq!(
            sp.value,
            Expr::Combination(
                Box::new(Expr::Symbol(Symbol("define".to_string()))),
                vec![Expr::Symbol(Symbol("x".to_string())), Expr::Integer(1)]
            )
        );
    }
}

impl FromStr for Expr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sp = parse_snippet(s)?;
        Ok(sp.value)
    }
}

impl FromStr for Spanned<Expr> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_snippet(s)
    }
}
