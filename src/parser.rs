#![allow(missing_docs)]

use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use errors::{ErrorKind, Result};

#[derive(Debug)]
pub enum Value<'a> {
    Atomic(&'a str),
    Object(Vec<(&'a str, Value<'a>)>),
}

#[derive(Debug)]
pub struct Parser<B: Backend> {
    index_builder: IndexBuilder<B>,
}

impl<B: Backend> Parser<B> {
    pub fn new(index_builder: IndexBuilder<B>) -> Self {
        Self { index_builder }
    }

    pub fn parse<'s>(&self, record: &'s str, level: usize) -> Result<Value<'s>> {
        let index = self.index_builder.build(record.as_bytes(), level)?;
        basic_parse(record, 0, record.len(), &index, 0)
    }
}

fn basic_parse<'s>(record: &'s str, begin: usize, end: usize, index: &StructuralIndex, level: usize) -> Result<Value<'s>> {
    let mut result = Vec::new();

    let cp = index.colon_positions(begin, end, level);
    for (i, _) in cp.iter().enumerate() {
        let field = index.find_field(record, if i == 0 { begin } else { cp[i - 1] }, cp[i])?;

        let vsi = cp[i] + 1;
        let vei = if i == cp.len() - 1 { end } else { cp[i + 1] };
        let value = match index.find_value(record, vsi, vei, level)? {
            "" => return Err(ErrorKind::InvalidRecord.into()),
            s if s.starts_with("{") && s.ends_with("}") => basic_parse(record, vsi, vei, index, level + 1)?,
            s => Value::Atomic(s),
        };

        result.push((field, value));
    }

    Ok(Value::Object(result))
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use super::super::index_builder::backend::FallbackBackend;

    // #[test]
    // fn basic_parsing() {
    //     let paths = &["$.f1", "$.f2.e1", "$.f2.e1.c2"];
    //     let record = r#"{
    //         "f1": true,
    //         "f2": {
    //             "e2": "\"foo\\",
    //             "e1": { "c1": null }
    //         },
    //         "f3": false
    //     }"#;

    //     let mut queries = QueryTree::default();
    //     for &path in paths {
    //         queries.add_path(path).unwrap();
    //     }

    //     let backend = FallbackBackend::default();
    //     let index_builder = IndexBuilder::new(backend);

    //     let parser = Parser::new(queries, index_builder);

    //     let result = parser.parse(record).unwrap();
    //     assert_eq!(result.len(), 3);
    //     assert_eq!(result[0], Some("true"));
    //     assert_eq!(result[1], Some(r#"{ "c1": null }"#));
    //     assert_eq!(result[2], None);
    // }

    // #[test]
    // fn basic_parsing_failure_case() {
    //     let record = r#"{ "f1": }"#;

    //     let mut queries = QueryTree::default();
    //     queries.add_path("$.f1").unwrap();

    //     let backend = FallbackBackend::default();
    //     let index_builder = IndexBuilder::new(backend);

    //     let parser = Parser::new(queries, index_builder);

    //     assert!(parser.parse(record).is_err());
    // }
}
