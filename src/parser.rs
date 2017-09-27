#![allow(missing_docs)]

use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use errors::Result;
use value::{self, Value, ValueType};


#[derive(Debug)]
pub struct Parser<B: Backend> {
    index_builder: IndexBuilder<B>,
}

impl<B: Backend> Parser<B> {
    pub fn new(index_builder: IndexBuilder<B>) -> Self {
        Self { index_builder }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Value<'s>> {
        let record = record.trim();
        let index = self.index_builder.build(record.as_bytes())?;
        self.parse_impl(record, 0, record.len(), 0, &index)
    }

    fn parse_array<'s>(
        &self,
        record: &'s str,
        begin: usize,
        end: usize,
        index: &StructuralIndex,
        level: usize,
    ) -> Result<Value<'s>> {
        let cp = match index.comma_positions(begin, end, level) {
            Some(cp) => cp,
            None => return Ok(Value::raw(&record[begin..end])),
        };

        let mut result = Vec::new();

        for i in 0..cp.len() {
            let (vsi, vei) = trimmed(
                record,
                if i == 0 { begin + 1 } else { cp[i - 1] + 1 },
                cp[i],
            );
            let value = self.parse_impl(record, vsi, vei, level + 1, index)?;

            result.push(value);
        }

        if !cp.is_empty() {
            let (vsi, mut vei) = trimmed(record, cp[cp.len() - 1] + 1, end);
            while vei > vsi && record.as_bytes()[vei - 1] == b']' {
                vei -= 1;
            }
            let (vsi, vei) = trimmed(record, vsi, vei);
            let value = self.parse_impl(record, vsi, vei, level + 1, index)?;

            result.push(value);
        }

        Ok(Value::Array(result))
    }

    fn parse_object<'s>(
        &self,
        record: &'s str,
        begin: usize,
        mut end: usize,
        index: &StructuralIndex,
        level: usize,
    ) -> Result<Value<'s>> {
        let cp = match index.colon_positions(begin, end, level) {
            Some(cp) => cp,
            None => return Ok(Value::raw(&record[begin..end])),
        };

        let mut result = Vec::new();

        for i in (0..cp.len()).rev() {
            let (field, fsi) = index.find_field(record, if i == 0 { begin } else { cp[i - 1] }, cp[i])?;

            let delim = if i == cp.len() - 1 { b'}' } else { b',' };
            let (vsi, mut vei) = trimmed(record, cp[i] + 1, end);
            while vei > cp[i] + 1 && record.as_bytes()[vei - 1] == delim {
                vei -= 1;
            }
            let (vsi, vei) = trimmed(record, vsi, vei);
            let value = self.parse_impl(record, vsi, vei, level + 1, index)?;

            result.push((field.into(), value));

            end = fsi - 1;
        }

        Ok(Value::Object(result.into_iter().rev().collect()))
    }

    #[inline]
    fn parse_impl<'s>(
        &self,
        record: &'s str,
        begin: usize,
        end: usize,
        level: usize,
        index: &StructuralIndex,
    ) -> Result<Value<'s>> {
        match value::parse(record, begin, end)? {
            ValueType::Atomic(v) => Ok(v),
            ValueType::Array => self.parse_array(record, begin, end, index, level),
            ValueType::Object => self.parse_object(record, begin, end, index, level),
        }
    }
}


fn trimmed(s: &str, mut begin: usize, mut end: usize) -> (usize, usize) {
    while begin < end && is_ws(s, begin) {
        begin += 1;
    }
    while end >= begin && is_ws(s, end - 1) {
        end -= 1;
    }
    (begin, end)
}

#[test]
fn trimmed_1() {
    let s = "[a, b, c]";
    let (b, e) = trimmed(s, 0, s.len());
    assert_eq!(&s[b..e], "[a, b, c]");
}

fn is_ws(s: &str, i: usize) -> bool {
    match s.as_bytes()[i] {
        b' ' | b'\n' | b'\t' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::index_builder::backend::FallbackBackend;

    #[test]
    fn basic_parsing() {
        let record = r#"{
            "f1": true,
            "f2": {
                "e2": "\"foo\\",
                "e1": { "c1": null }
            },
            "f3": [ true, "10", null ]
        }"#;

        let backend = FallbackBackend::default();
        let index_builder = IndexBuilder::new(backend, 4);
        let parser = Parser::new(index_builder);

        let result = parser.parse(record).unwrap();
        assert_eq!(
            result,
            object!{
                "f1" => true,
                "f2" => object!{
                    "e2" => r#"\"foo\\"#,
                    "e1" => object!{ "c1" => Value::Null, },
                },
                "f3" => array![
                    true,
                    "10",
                    Value::Null,
                ],
            }
        );
    }

    #[test]
    fn basic_parsing_2() {
        let record = r#"{
            "f1": true,
            "f2": {
                "e2": "\"foo\\",
                "e1": { "c1": null }
            },
            "f3": [ true, "10", null ]
        }"#;

        let backend = FallbackBackend::default();
        let index_builder = IndexBuilder::new(backend, 2);
        let parser = Parser::new(index_builder);

        let result = parser.parse(record).unwrap();
        assert_eq!(
            result,
            object!(
                "f1" => true,
                "f2" => object!{
                    "e2" => r#"\"foo\\"#,
                    "e1" => Value::raw(r#"{ "c1": null }"#),
                },
                "f3" => array![
                    true,
                    "10",
                    Value::Null,
                ],
            )
        );
    }
}
