use chrono::{DateTime, Duration as ChronoDuration, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp(pub DateTime<Utc>);

#[derive(Debug, Clone, PartialEq)]
pub struct Duration(pub ChronoDuration);


#[derive(Debug, Clone, PartialEq)]
pub struct Price(pub f64);

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Operator(Symbol),
    Duration(Duration),
    Timestamp(Timestamp),
    Combination(Box<Expr>, Vec<Expr>),
}
