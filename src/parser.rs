#![allow(missing_docs)]

use crate::errors::Result;
use crate::index_builder::backend::Backend;
use crate::index_builder::{IndexBuilder, StructuralIndex};
use crate::value::{self, Value, ValueType};
use std::ptr;

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
        let index = self.index_builder.build(record)?;
        self.parse_impl(&index, 0, record.len(), 0)
    }

    #[allow(unsafe_code)]
    fn parse_array<'a, 's>(
        &self,
        index: &StructuralIndex<'a, 's>,
        begin: usize,
        end: usize,
        level: usize,
    ) -> Result<Value<'s>> {
        let mut cp = Vec::new();
        if !index.comma_positions(begin, end, level, &mut cp) {
            return Ok(Value::raw(index.substr(begin, end)));
        };
        cp.push(end - 1); // dummy

        let mut result = Vec::with_capacity(cp.len());
        unsafe {
            result.set_len(cp.len());
        }

        for i in 0..cp.len() {
            let (vsi, vei) =
                index.find_array_value(if i == 0 { begin + 1 } else { cp[i - 1] + 1 }, cp[i]);
            if i == 0 && vsi == vei {
                unsafe {
                    // ensure not to call destructors of `uninitialized` elements.
                    result.set_len(0);
                }
                return Ok(Value::Array(result));
            }
            let value = self.parse_impl(index, vsi, vei, level + 1).map_err(|e| {
                unsafe {
                    result.set_len(i);
                }
                e
            })?;

            unsafe {
                ptr::write(result.get_unchecked_mut(i), value);
            }
        }

        Ok(Value::Array(result))
    }

    #[allow(unsafe_code)]
    fn parse_object<'a, 's>(
        &self,
        index: &StructuralIndex<'a, 's>,
        begin: usize,
        mut end: usize,
        level: usize,
    ) -> Result<Value<'s>> {
        let mut cp = Vec::new();
        if !index.colon_positions(begin, end, level, &mut cp) {
            return Ok(Value::raw(index.substr(begin, end)));
        }

        let mut result = Vec::with_capacity(cp.len());
        unsafe {
            result.set_len(cp.len());
        }

        let mut err = Ok(());
        for i in (0..cp.len()).rev() {
            let (field, fsi) =
                match index.find_object_field(if i == 0 { begin } else { cp[i - 1] }, cp[i]) {
                    Ok(v) => v,
                    Err(e) => {
                        err = Err((i, e));
                        break;
                    }
                };

            let (vsi, vei) = index.find_object_value(cp[i] + 1, end, i == cp.len() - 1);
            let value = match self.parse_impl(index, vsi, vei, level + 1) {
                Ok(v) => v,
                Err(e) => {
                    err = Err((i, e));
                    break;
                }
            };

            unsafe {
                ptr::write(result.get_unchecked_mut(i), (field, value));
            }

            end = fsi - 1;
        }

        if let Err((i, e)) = err {
            unsafe {
                for j in i + 1..cp.len() {
                    // call destructors of `initialized` elements.
                    ptr::drop_in_place(result.get_unchecked_mut(j));
                }
                // ensure not to call destructors of `uninitialized` elements
                result.set_len(0);
            }
            return Err(e);
        }

        Ok(Value::Object(result))
    }

    #[inline]
    fn parse_impl<'a, 's>(
        &self,
        index: &StructuralIndex<'a, 's>,
        begin: usize,
        end: usize,
        level: usize,
    ) -> Result<Value<'s>> {
        match value::parse(&index.substr(begin, end))? {
            ValueType::Atomic(v) => Ok(v),
            ValueType::Array => self.parse_array(index, begin, end, level),
            ValueType::Object => self.parse_object(index, begin, end, level),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::index_builder::backend::FallbackBackend;
    use super::*;

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
            object! {
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
