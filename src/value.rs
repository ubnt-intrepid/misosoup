#![macro_use]
#![allow(missing_docs)]

use std::borrow::Cow;
use std::fmt;
use errors::{Error, ErrorKind, Result, ResultExt};


#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EscapedStr<'a>(Cow<'a, str>);

impl<'a> EscapedStr<'a> {
    pub fn as_raw_str(&self) -> &str {
        &self.0
    }
}

impl<'a> fmt::Debug for EscapedStr<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<'a> From<&'a str> for EscapedStr<'a> {
    #[inline]
    fn from(val: &'a str) -> Self {
        EscapedStr(val.into())
    }
}

impl<'a> From<String> for EscapedStr<'a> {
    #[inline]
    fn from(val: String) -> Self {
        EscapedStr(val.into())
    }
}

impl<'a> From<Cow<'a, str>> for EscapedStr<'a> {
    #[inline]
    fn from(val: Cow<'a, str>) -> Self {
        EscapedStr(val.into())
    }
}


pub type LinearMap<K, V> = Vec<(K, V)>;

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Value<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(EscapedStr<'a>),
    Array(Vec<Value<'a>>),
    Object(LinearMap<EscapedStr<'a>, Value<'a>>),
    Raw(Cow<'a, str>),
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(ref s) => fmt::Debug::fmt(s, f),
            Value::Array(ref arr) => fmt::Debug::fmt(arr, f),
            Value::Object(ref obj) => f.debug_map()
                .entries(obj.iter().map(|&(ref k, ref v)| (k, v)))
                .finish(),
            Value::Raw(ref s) => write!(f, "Raw({:?})", s),
        }
    }
}

impl<'a> Value<'a> {
    #[inline]
    pub fn raw<S: Into<Cow<'a, str>>>(val: S) -> Self {
        Value::Raw(val.into())
    }
}

impl<'a> From<bool> for Value<'a> {
    #[inline]
    fn from(val: bool) -> Value<'a> {
        Value::Boolean(val)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    #[inline]
    fn from(val: &'a str) -> Value<'a> {
        Value::String(val.into())
    }
}

impl<'a> From<String> for Value<'a> {
    #[inline]
    fn from(val: String) -> Value<'a> {
        Value::String(val.into())
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    #[inline]
    fn from(val: Cow<'a, str>) -> Value<'a> {
        Value::String(val.into())
    }
}



#[derive(Debug)]
pub enum ValueType<'a> {
    Atomic(Value<'a>),
    Array,
    Object,
}

/// Parse the input string and returns the instance of `Value`.
#[inline]
pub fn parse<'a>(s: &'a str) -> Result<ValueType<'a>> {
    match s {
        "null" => Ok(ValueType::Atomic(Value::Null)),
        "true" => Ok(ValueType::Atomic(Value::Boolean(true))),
        "false" => Ok(ValueType::Atomic(Value::Boolean(false))),
        s if s.starts_with("\"") && s.ends_with("\"") && s.len() > 1 => {
            // FIXME: check if s is a valid UTF-8 string
            Ok(ValueType::Atomic(Value::String(s[1..s.len() - 1].into())))
        }
        s if s.starts_with("[") && s.ends_with("]") => Ok(ValueType::Array),
        s if s.starts_with("{") && s.ends_with("}") => Ok(ValueType::Object),
        s => if let Ok(n) = s.parse::<f64>() {
            Ok(ValueType::Atomic(Value::Number(n)))
        } else {
            Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| format!("Value::from_str({:?})", s))
        },
    }
}

#[macro_export]
macro_rules! object {
    ($( $f:expr => $v:expr,)+ ) => {{
        let mut h = Vec::new();
        $(
            h.push((From::from($f), From::from($v)));
        )*
        Value::Object(h)
    }}
}

#[macro_export]
macro_rules! array {
    ($($v:expr,)*) => {{
        Value::Array(vec![
            $(
                From::from($v),
            )*
        ])
    }}
}
