#![macro_use]
#![allow(missing_docs)]

use std::borrow::Cow;
use errors::{Error, ErrorKind, Result, ResultExt};


#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Value<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(Cow<'a, str>),
    Array(Vec<Value<'a>>),
    Object(Vec<(&'a str, Value<'a>)>),
    Raw(&'a str),
}

impl<'a> From<bool> for Value<'a> {
    fn from(val: bool) -> Value<'a> {
        Value::Boolean(val)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(val: &'a str) -> Value<'a> {
        Value::String(val.into())
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(val: String) -> Value<'a> {
        Value::String(val.into())
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    fn from(val: Cow<'a, str>) -> Value<'a> {
        Value::String(val)
    }
}



#[derive(Debug)]
pub enum ValueType<'a> {
    Atomic(Value<'a>),
    Array,
    Object,
}

/// Parse the input string and returns the instance of `Value`.
pub fn parse<'a>(s: &'a str, begin: usize, end: usize) -> Result<ValueType<'a>> {
    match &s[begin..end] {
        "null" => Ok(ValueType::Atomic(Value::Null)),
        "true" => Ok(ValueType::Atomic(Value::Boolean(true))),
        "false" => Ok(ValueType::Atomic(Value::Boolean(false))),
        s if s.starts_with("\"") => Ok(ValueType::Atomic(Value::String(s.into()))),
        s if s.starts_with("[") => Ok(ValueType::Array),
        s if s.starts_with("{") => Ok(ValueType::Object),
        s => if let Ok(n) = s.parse::<f64>() {
            Ok(ValueType::Atomic(Value::Number(n)))
        } else {
            Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "Value::from_str")
        },
    }
}

#[macro_export]
macro_rules! object {
    ($( $f:expr => $v:expr,)+ ) => {{
        Value::Object(vec![
            $(
                ($f, Value::from($v)),
            )*
        ])
    }}
}

#[macro_export]
macro_rules! array {
    ($($v:expr,)*) => {{
        Value::Array(vec![
            $(
                Value::from($v),
            )*
        ])
    }}
}
