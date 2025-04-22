use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp(pub DateTime<Utc>);

#[derive(Debug, Clone, PartialEq)]
pub struct Duration(pub ChronoDuration);

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol(pub String);

impl Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Comment(String),
    Combination(Box<Expr>, Vec<Expr>),
    Symbol(Symbol),
    Boolean(bool),
    Float(f64),
    String(String),
    Duration(Duration),
    Timestamp(Timestamp),
    Integer(i64),
}
