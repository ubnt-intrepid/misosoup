#![allow(missing_docs)]

use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use errors::Result;
use value::{self, Value, ValueType};


#[derive(Debug)]
pub struct Parser<B: Backend> {
    index_builder: IndexBuilder<B>,
    max_level: usize,
}

impl<B: Backend> Parser<B> {
    pub fn new(index_builder: IndexBuilder<B>, max_level: usize) -> Self {
        Self {
            index_builder,
            max_level,
        }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Value<'s>> {
        let record = record.trim();
        let index = self.index_builder.build(record.as_bytes(), self.max_level)?;
        match value::parse(record, 0, record.len())? {
            ValueType::Atomic(v) => Ok(v),
            ValueType::Array => self.parse_array(record, 0, record.len(), &index, 0),
            ValueType::Object => self.parse_object(record, 0, record.len(), &index, 0),
        }
    }

    fn parse_array<'s>(&self, record: &'s str, begin: usize, end: usize, index: &StructuralIndex, level: usize) -> Result<Value<'s>> {
        let mut result = Vec::new();

        let cp = index.comma_positions(begin, end, level);
        for (i, _) in cp.iter().enumerate() {
            let (vsi, vei) = trimmed(
                record,
                if i == 0 { begin + 1 } else { cp[i - 1] + 1 },
                cp[i],
            );

            let value = match value::parse(record, vsi, vei)? {
                ValueType::Atomic(v) => v,
                ValueType::Array if level + 1 < self.max_level => self.parse_array(record, vsi, vei, index, level + 1)?,
                ValueType::Object if level + 1 < self.max_level => self.parse_object(record, vsi, vei, index, level + 1)?,
                _ => Value::Raw(&record[vsi..vei]),
            };

            result.push(value);
        }

        if !cp.is_empty() {
            let (vsi, mut vei) = trimmed(record, cp[cp.len() - 1] + 1, end - 1);
            while vei >= vsi && record.as_bytes()[vei - 1] == b']' {
                vei -= 1;
            }
            let (vsi, vei) = trimmed(record, vsi, vei);

            let value = match value::parse(record, vsi, vei)? {
                ValueType::Atomic(v) => v,
                ValueType::Array if level + 1 < self.max_level => self.parse_array(record, vsi, vei, index, level + 1)?,
                ValueType::Object if level + 1 < self.max_level => self.parse_object(record, vsi, vei, index, level + 1)?,
                _ => Value::Raw(&record[vsi..vei]),
            };

            result.push(value);
        }

        Ok(Value::Array(result))
    }

    fn parse_object<'s>(&self, record: &'s str, begin: usize, mut end: usize, index: &StructuralIndex, level: usize) -> Result<Value<'s>> {
        let mut result = Vec::new();

        let cp = index.colon_positions(begin, end, level);
        for i in (0..cp.len()).rev() {
            let (field, fsi) = index.find_field(record, if i == 0 { begin } else { cp[i - 1] }, cp[i])?;

            let delim = if i == cp.len() - 1 { b'}' } else { b',' };
            let (vsi, mut vei) = trimmed(record, cp[i] + 1, end);
            while vei >= cp[i] + 1 && record.as_bytes()[vei - 1] == delim {
                vei -= 1;
            }
            let (vsi, vei) = trimmed(record, vsi, vei);

            let value = match value::parse(record, vsi, vei)? {
                ValueType::Atomic(v) => v,
                ValueType::Array if level + 1 < self.max_level => self.parse_array(record, vsi, vei, index, level + 1)?,
                ValueType::Object if level + 1 < self.max_level => self.parse_object(record, vsi, vei, index, level + 1)?,
                _ => Value::Raw(&record[vsi..vei]),
            };

            result.push((field, value));

            end = fsi - 1;
        }

        Ok(Value::Object(result.into_iter().rev().collect()))
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
        let index_builder = IndexBuilder::new(backend);
        let parser = Parser::new(index_builder, 4);

        let result = parser.parse(record).unwrap();
        assert_eq!(
            result,
            object!{
                "f1" => Value::Boolean(true),
                "f2" => object!{
                    "e2" => Value::String(r#""\"foo\\""#),
                    "e1" => object!{ "c1" => Value::Null, },
                },
                "f3" => array![
                    Value::Boolean(true),
                    Value::String("\"10\""),
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
        let index_builder = IndexBuilder::new(backend);
        let parser = Parser::new(index_builder, 2);

        let result = parser.parse(record).unwrap();
        assert_eq!(
            result,
            object!(
                "f1" => Value::Boolean(true),
                "f2" => object!{
                    "e2" => Value::String(r#""\"foo\\""#),
                    "e1" => Value::Raw(r#"{ "c1": null }"#),
                },
                "f3" => array![
                    Value::Boolean(true),
                    Value::String("\"10\""),
                    Value::Null,
                ],
            )
        );
    }
}
